//! `athena review`: a person's verdict on the class of one recorded introspection
//! (`docs/design/defaults-loop.md` section 10.2).
//!
//! The command shows the introspection (what happened, class, cause, prevention and Jev's shadow answer) and, in
//! the same call, commits an `athena.class-label/1` record with the verdict: the recorded class is right, the
//! right class is another registry class, or it is a new class (written as an `athena.failure-class/1` record in
//! the same commit). A review is a record: the introspection is never edited. Reviewing an introspection again
//! adds a newer label, which replaces the earlier one under the rule in [`crate::labeled`].
//!
//! Nothing is written unless every check passes: the introspection id must name exactly one introspection, a
//! named class must be in the registry, exactly one verdict must be given, and a note or description must be a
//! single line.

use crate::Athena;
use crate::commits::{NewFile, message};
use crate::labeled::newest_labels;
use crate::store::Records;
use anyhow::{Result, bail};
use athena_data::RecordId;
use athena_data::records::{
    AnyRecord, ClassLabel, ClassStatus, FailureClass, IntrospectionJev, IntrospectionRecord,
    Labeler,
};
use serde::Serialize;

/// What the person decided about the class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// The class the introspection recorded is right.
    Right,
    /// The right class is this other registry class.
    Class(RecordId),
    /// No registry class fits: add this one.
    NewClass { name: String, description: String },
}

impl Verdict {
    /// The verdict from the command's flags. Exactly one of `right`, `class` and `new_class` is required;
    /// `description` goes with `new_class` and with nothing else.
    pub fn from_flags(
        right: bool,
        class: Option<&str>,
        new_class: Option<&str>,
        description: Option<&str>,
    ) -> Result<Verdict> {
        let given =
            usize::from(right) + usize::from(class.is_some()) + usize::from(new_class.is_some());
        if given == 0 {
            bail!(
                "say what the class is: --right, --class <class-id>, or --new-class <name> --description \"...\""
            );
        }
        if given > 1 {
            bail!("give exactly one of --right, --class and --new-class");
        }
        if description.is_some() && new_class.is_none() {
            bail!("--description goes with --new-class");
        }
        if right {
            return Ok(Verdict::Right);
        }
        if let Some(class) = class {
            let id = RecordId::parse(class)
                .map_err(|e| anyhow::anyhow!("--class {class:?} is not a class id: {e}"))?;
            return Ok(Verdict::Class(id));
        }
        let name = one_line("the class name", new_class.unwrap_or_default())?;
        let Some(description) = description else {
            bail!("--new-class needs --description \"<one line>\"");
        };
        let description = one_line("the class description", description)?;
        Ok(Verdict::NewClass { name, description })
    }
}

/// `text` trimmed, which must be one non-empty line.
fn one_line(what: &str, text: &str) -> Result<String> {
    let text = text.trim();
    if text.is_empty() {
        bail!("{what} is empty");
    }
    if text.contains(['\n', '\r']) {
        bail!("{what} must be a single line");
    }
    Ok(text.to_string())
}

/// One option of Jev's class answer.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JevOption {
    /// A registry class id, or `new`.
    pub option: String,
    /// The class name, for a registry id.
    pub name: Option<String>,
    pub probability: f64,
}

/// Jev's shadow answer as the review shows it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JevShown {
    /// False when the introspection holds no Jev record.
    pub asked: bool,
    /// Why the call produced no answers.
    pub error: Option<String>,
    /// The option Jev chose and how sure it said it was.
    pub choice: Option<String>,
    pub confidence: Option<f64>,
    /// Every option, most probable first.
    pub options: Vec<JevOption>,
    pub repeat_probability: Option<f64>,
}

/// The introspection as the review shows it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Shown {
    pub id: String,
    pub trigger: String,
    pub job: Option<String>,
    pub what_happened: Option<String>,
    /// The recorded class, as `name (id)`.
    pub class: Option<String>,
    pub cause: Option<String>,
    pub prevention_kind: Option<String>,
    pub prevention: Option<String>,
    pub jev: JevShown,
}

/// What `athena review` did.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Review {
    pub shown: Shown,
    /// The label that was committed.
    pub label: ClassLabel,
    /// The assigned class, as `name (id)`.
    pub assigned: String,
    /// The class added by this review, for a new class.
    pub new_class: Option<FailureClass>,
    /// The label this one replaces, when the introspection was already reviewed.
    pub replaces: Option<ClassLabel>,
    pub commit: String,
}

fn class_text(classes: &[FailureClass], id: &RecordId) -> String {
    let name = classes
        .iter()
        .find(|c| &c.id == id)
        .map_or("unknown class", |c| c.name.as_str());
    format!("{name} ({id})")
}

fn jev_shown(classes: &[FailureClass], jev: Option<&IntrospectionJev>) -> JevShown {
    let Some(jev) = jev else {
        return JevShown {
            asked: false,
            error: None,
            choice: None,
            confidence: None,
            options: vec![],
            repeat_probability: None,
        };
    };
    let mut options: Vec<JevOption> = jev
        .class
        .as_ref()
        .map(|c| {
            c.probabilities
                .iter()
                .map(|(option, p)| JevOption {
                    option: option.clone(),
                    name: classes
                        .iter()
                        .find(|k| k.id.as_str() == option)
                        .map(|k| k.name.clone()),
                    probability: *p,
                })
                .collect()
        })
        .unwrap_or_default();
    options.sort_by(|a, b| b.probability.total_cmp(&a.probability));
    JevShown {
        asked: true,
        error: jev.error.clone(),
        choice: jev.class.as_ref().map(|c| c.choice.clone()),
        confidence: jev.class.as_ref().map(|c| c.confidence),
        options,
        repeat_probability: jev.repeat_probability,
    }
}

fn shown(classes: &[FailureClass], i: &IntrospectionRecord) -> Shown {
    Shown {
        id: i.id.to_string(),
        trigger: i.trigger.as_str().into(),
        job: i.job.as_ref().map(ToString::to_string),
        what_happened: i.what_happened.clone(),
        class: i.failure_class.as_ref().map(|c| class_text(classes, c)),
        cause: i.cause.clone(),
        prevention_kind: i.prevention_kind.map(|k| k.as_str().into()),
        prevention: i.prevention.clone(),
        jev: jev_shown(classes, i.jev.as_ref()),
    }
}

/// The one introspection `reference` names: a full id, or a suffix or prefix of one that no other shares.
fn find<'a>(records: &'a Records, reference: &str) -> Result<&'a IntrospectionRecord> {
    let reference = reference.trim();
    if reference.is_empty() {
        bail!("name an introspection id");
    }
    if let Some(exact) = records
        .introspections
        .iter()
        .find(|i| i.id.as_str() == reference)
    {
        return Ok(exact);
    }
    let matches: Vec<&IntrospectionRecord> = records
        .introspections
        .iter()
        .filter(|i| i.id.as_str().starts_with(reference) || i.id.as_str().ends_with(reference))
        .collect();
    match matches.as_slice() {
        [one] => Ok(one),
        [] => bail!("no introspection {reference:?}; `athena status` lists the unreviewed ones"),
        many => bail!(
            "{reference:?} names {} introspections ({}); give more of the id",
            many.len(),
            many.iter()
                .map(|i| i.id.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn fresh_id(records: &Records, taken: &[RecordId]) -> RecordId {
    loop {
        let id = RecordId::generate(crate::now());
        if !records.has_record_id(&id)
            && !records.class_labels.iter().any(|l| l.id == id)
            && !taken.contains(&id)
        {
            return id;
        }
    }
}

impl Athena {
    /// `athena review`: shows the introspection `reference` names and commits the person's verdict on its class
    /// in one step. Errors write nothing.
    pub fn review(&self, reference: &str, verdict: &Verdict, note: Option<&str>) -> Result<Review> {
        let note = note.map(|n| one_line("the note", n)).transpose()?;
        let head = self.head()?;
        let records = Records::load_current(&self.git, &head)?;
        let introspection = find(&records, reference)?;
        let mut new_class = None;
        let class = match verdict {
            Verdict::Right => match &introspection.failure_class {
                Some(class) => class.clone(),
                None => bail!(
                    "introspection {} records no class, so --right has nothing to confirm; use --class <class-id> or --new-class <name> --description \"...\"",
                    introspection.id
                ),
            },
            Verdict::Class(id) => {
                if !records.classes.iter().any(|c| &c.id == id) {
                    bail!("no class {id} in the registry; use --new-class to add one");
                }
                id.clone()
            }
            Verdict::NewClass { name, description } => {
                if let Some(same) = records
                    .classes
                    .iter()
                    .find(|c| c.name.to_lowercase() == name.to_lowercase())
                {
                    bail!(
                        "the registry already has a class named {:?} ({}); use --class {}",
                        same.name,
                        same.id,
                        same.id
                    );
                }
                let id = fresh_id(&records, &[]);
                let class = FailureClass {
                    id: id.clone(),
                    at: crate::now(),
                    name: name.clone(),
                    description: description.clone(),
                    status: ClassStatus::Proposed,
                    proposed_by: introspection.id.clone(),
                };
                new_class = Some(class);
                id
            }
        };
        let label = ClassLabel {
            id: fresh_id(&records, std::slice::from_ref(&class)),
            at: crate::now(),
            introspection: introspection.id.clone(),
            class: Some(class.clone()),
            labeled_by: Labeler::Person,
            is_repeat: None,
            note,
        };
        let replaces = newest_labels(&records.class_labels)
            .get(introspection.id.as_str())
            .map(|l| (*l).clone());
        let mut files = Vec::new();
        let mut trailers = Vec::new();
        if let Some(class) = &new_class {
            let record = AnyRecord::FailureClass(class.clone());
            trailers.push(("Athena-Record", record.canonical_path()));
            files.push(NewFile::record(&record));
        }
        let record = AnyRecord::ClassLabel(label.clone());
        trailers.push(("Athena-Record", record.canonical_path()));
        files.push(NewFile::record(&record));
        let commit = self.commit_records(
            &files,
            &message(
                &format!("Review the class of introspection {}", introspection.id),
                "A person's verdict on the failure class of one introspection. It is a record: the introspection is not edited.",
                &trailers,
            ),
        )?;
        let mut classes = records.classes.clone();
        classes.extend(new_class.clone());
        Ok(Review {
            shown: shown(&records.classes, introspection),
            assigned: class_text(&classes, &class),
            label,
            new_class,
            replaces,
            commit,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLASS: &str = "20261001-000000-c001";

    #[test]
    fn exactly_one_verdict_flag_is_required() {
        let right = Verdict::from_flags(true, None, None, None).unwrap();
        assert_eq!(right, Verdict::Right);
        let class = Verdict::from_flags(false, Some(CLASS), None, None).unwrap();
        assert_eq!(class, Verdict::Class(RecordId::parse(CLASS).unwrap()));
        let new =
            Verdict::from_flags(false, None, Some(" Stale cache "), Some("It lingers.")).unwrap();
        assert_eq!(
            new,
            Verdict::NewClass {
                name: "Stale cache".into(),
                description: "It lingers.".into()
            }
        );
        let error = |v: Result<Verdict>| v.unwrap_err().to_string();
        assert!(error(Verdict::from_flags(false, None, None, None)).contains("say what"));
        assert!(error(Verdict::from_flags(true, Some(CLASS), None, None)).contains("exactly one"));
        assert!(
            error(Verdict::from_flags(
                false,
                Some(CLASS),
                Some("x"),
                Some("y")
            ))
            .contains("exactly one")
        );
        assert!(error(Verdict::from_flags(false, None, Some("x"), None)).contains("--description"));
        assert!(error(Verdict::from_flags(true, None, None, Some("y"))).contains("--new-class"));
        assert!(
            error(Verdict::from_flags(false, None, Some("a\nb"), Some("y")))
                .contains("single line")
        );
    }

    #[test]
    fn a_note_or_description_is_one_non_empty_line() {
        assert_eq!(one_line("the note", "  fine ").unwrap(), "fine");
        assert!(one_line("the note", "two\nlines").is_err());
        assert!(one_line("the note", "two\rlines").is_err());
        assert!(one_line("the note", "   ").is_err());
    }
}
