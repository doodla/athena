//! The labeled set and the scoring of Jev's shadow answers (`docs/design/defaults-loop.md` sections 10.2 to
//! 10.4; build order step 6).
//!
//! **The labeled set** is every recorded introspection that has a reviewer's `athena.class-label/1` record
//! (the failure class a person, or a stronger model that a person spot-checked, assigned; `none` when no
//! registry class fits). Each row keeps who labeled it. Introspections with no label are *unreviewed* and are
//! left out and counted. A reviewed introspection with no stored Jev answer, or whose Jev call failed, is also
//! left out and counted, so the report shows how much of the record base it rests on.
//!
//! **The score** compares Jev's stored failure-class probabilities with the label, by Brier score (the sum
//! over options of the squared gap between the probability and the one-hot label). Two references are scored
//! the same way: always predicting `none`, and a plain word-overlap matcher over the stored event text and the
//! registry's class names and descriptions. Skill is the Brier improvement over the better reference. Also
//! reported are top-1 accuracy, the false-match rate on `none` labels, whether each reviewed group lands in one
//! Jev class with no class mixing groups, and the Brier score of the is-repeat answer where the reviewer said
//! whether the event repeats.
//!
//! **The split is by time.** Rows before a cut date calibrate (here: the overlap matcher's threshold and the
//! repeat base rate); later rows test. Only the test part is scored. The default cut leaves the earliest two
//! thirds for calibration.
//!
//! **No verdict without enough cases.** With fewer than [`MIN_TEST_CASES`] test rows the report says so and
//! carries no score and no verdict. The interval on mean skill is given only when the test size reaches the
//! size section 10.4 item 1 requires; otherwise the report says no bound was computed and the per-record table
//! is the evidence.
//!
//! Only the failure-class and is-repeat answers are scored: they are the only Jev answers recorded. This module
//! reads records and never opens a file for writing; a test asserts it.

use crate::Athena;
use crate::introspect::{ClassRow, JEV_NEW_OPTION, month_files};
use anyhow::{Result, bail};
use athena_data::records::{AnyRecord, ClassLabel, IntrospectionRecord, Labeler};
use jiff::Timestamp;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The fewest test rows at which a score and a verdict are given. An **estimate**: a smaller test part cannot
/// show a skill of the size section 10.4 assumes, and `10` keeps the table readable. It is not a power
/// calculation.
pub const MIN_TEST_CASES: usize = 10;

/// The false-match rate on `none` labels that section 10.4 item 3 requires to stay under (an **estimate** set at
/// review).
pub const MAX_FALSE_MATCH_RATE: f64 = 0.10;

/// The z value of a two-sided 95% normal interval.
const Z95: f64 = 1.96;

/// Thresholds tried for the word-overlap matcher on the calibration part; ties go to the lowest.
const OVERLAP_GRID: [f64; 5] = [0.1, 0.2, 0.3, 0.4, 0.5];

/// The option that stands for "no registry class": Jev's `new`, and a reviewer's `none`.
pub const NONE_OPTION: &str = JEV_NEW_OPTION;

// ---------------------------------------------------------------------------------------------------------
// Reading
// ---------------------------------------------------------------------------------------------------------

/// The records the labeled set is built from, as read from `records/`.
#[derive(Debug, Default)]
pub struct Inputs {
    pub introspections: Vec<IntrospectionRecord>,
    pub labels: Vec<ClassLabel>,
    pub classes: Vec<ClassRow>,
}

/// Reads the introspections, class labels and class registry under `root/records/`. A missing directory reads
/// as empty. Nothing is written.
pub fn read_inputs(root: &Path) -> Result<Inputs> {
    let mut inputs = Inputs::default();
    for (path, bytes) in month_files(root, "introspections")? {
        match AnyRecord::parse(&path, &bytes)? {
            AnyRecord::Introspection(i) => inputs.introspections.push(*i),
            _ => bail!("{path}: not an athena.introspection/1 record"),
        }
    }
    for (path, bytes) in month_files(root, "class-labels")? {
        match AnyRecord::parse(&path, &bytes)? {
            AnyRecord::ClassLabel(l) => inputs.labels.push(l),
            _ => bail!("{path}: not an athena.class-label/1 record"),
        }
    }
    for (path, bytes) in month_files(root, "failure-classes")? {
        match AnyRecord::parse(&path, &bytes)? {
            AnyRecord::FailureClass(c) => inputs.classes.push(ClassRow {
                id: c.id.to_string(),
                name: c.name,
                description: c.description,
            }),
            _ => bail!("{path}: not an athena.failure-class/1 record"),
        }
    }
    inputs.classes.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(inputs)
}

// ---------------------------------------------------------------------------------------------------------
// The labeled set
// ---------------------------------------------------------------------------------------------------------

/// One labeled case: a reviewed introspection with a usable Jev answer.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Row {
    pub introspection: String,
    pub at: Timestamp,
    /// The reviewed class: a registry id, or `None` for "none".
    pub label: Option<String>,
    /// Who labeled it.
    pub labeled_by: Labeler,
    /// The reviewer's judgement that the event repeats an earlier case, when given.
    pub is_repeat: Option<bool>,
    /// The stored event text the overlap reference reads: what happened and the cause.
    pub text: String,
    /// Jev's chosen option (a registry id or `new`).
    pub jev_choice: String,
    /// Jev's probability of each option.
    pub jev_probabilities: BTreeMap<String, f64>,
    /// Jev's probability that the event is a repeat.
    pub jev_repeat: f64,
}

impl Row {
    /// The option the label stands for: the class id, or [`NONE_OPTION`].
    pub fn target(&self) -> &str {
        self.label.as_deref().unwrap_or(NONE_OPTION)
    }
}

/// How many records were left out of the labeled set, and why. Every introspection is counted once, so
/// `labeled + unreviewed + no_jev_answer + jev_error == introspections`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Exclusions {
    /// Introspections read.
    pub introspections: usize,
    /// Rows in the labeled set.
    pub labeled: usize,
    /// No class-label record names the introspection.
    pub unreviewed: usize,
    /// Reviewed, but the introspection holds no Jev answer (Jev was not asked, or the record predates it).
    pub no_jev_answer: usize,
    /// Reviewed, but the Jev call failed and holds an error instead of answers.
    pub jev_error: usize,
    /// Class labels that name no recorded introspection (not a row; reported so a typo is visible).
    pub labels_without_introspection: usize,
}

/// The labeled set, oldest first, with what was left out.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct LabeledSet {
    pub rows: Vec<Row>,
    pub excluded: Exclusions,
}

/// The label that counts for each introspection id: where several exist, the newest by time, then id.
pub fn newest_labels(labels: &[ClassLabel]) -> BTreeMap<String, &ClassLabel> {
    let mut newest: BTreeMap<String, &ClassLabel> = BTreeMap::new();
    for label in labels {
        let key = label.introspection.to_string();
        let better = newest
            .get(&key)
            .is_none_or(|old| (label.at, &label.id) > (old.at, &old.id));
        if better {
            newest.insert(key, label);
        }
    }
    newest
}

/// The introspections no class-label record names, in the order given. This is the one definition of
/// "unreviewed": [`build`] counts it and `athena status` shows it.
pub fn unreviewed<'a>(
    introspections: &'a [IntrospectionRecord],
    labels: &[ClassLabel],
) -> Vec<&'a IntrospectionRecord> {
    let newest = newest_labels(labels);
    introspections
        .iter()
        .filter(|i| !newest.contains_key(&i.id.to_string()))
        .collect()
}

/// Builds the labeled set. Where an introspection has several labels the newest by time, then id, counts.
pub fn build(inputs: &Inputs) -> LabeledSet {
    let newest = newest_labels(&inputs.labels);
    let known: BTreeSet<String> = inputs
        .introspections
        .iter()
        .map(|i| i.id.to_string())
        .collect();
    let mut set = LabeledSet::default();
    set.excluded.introspections = inputs.introspections.len();
    set.excluded.labels_without_introspection =
        newest.keys().filter(|k| !known.contains(*k)).count();
    set.excluded.unreviewed = unreviewed(&inputs.introspections, &inputs.labels).len();
    for i in &inputs.introspections {
        let Some(label) = newest.get(&i.id.to_string()) else {
            continue;
        };
        let Some(jev) = &i.jev else {
            set.excluded.no_jev_answer += 1;
            continue;
        };
        if jev.error.is_some() {
            set.excluded.jev_error += 1;
            continue;
        }
        let (Some(class), Some(repeat)) = (&jev.class, jev.repeat_probability) else {
            set.excluded.no_jev_answer += 1;
            continue;
        };
        set.rows.push(Row {
            introspection: i.id.to_string(),
            at: i.at,
            label: label.class.as_ref().map(ToString::to_string),
            labeled_by: label.labeled_by,
            is_repeat: label.is_repeat,
            text: format!(
                "{} {}",
                i.what_happened.as_deref().unwrap_or(""),
                i.cause.as_deref().unwrap_or("")
            ),
            jev_choice: class.choice.clone(),
            jev_probabilities: class.probabilities.clone(),
            jev_repeat: repeat,
        });
    }
    set.rows
        .sort_by(|a, b| (a.at, &a.introspection).cmp(&(b.at, &b.introspection)));
    set.excluded.labeled = set.rows.len();
    set
}

// ---------------------------------------------------------------------------------------------------------
// Scoring
// ---------------------------------------------------------------------------------------------------------

/// The Brier score of `probabilities` against the one-hot label `target`: the sum over the options (those
/// given, and the target) of the squared gap. 0 is perfect; 2 is certain and wrong.
pub fn brier(probabilities: &BTreeMap<String, f64>, target: &str) -> f64 {
    let mut options: BTreeSet<&str> = probabilities.keys().map(String::as_str).collect();
    options.insert(target);
    options
        .into_iter()
        .map(|option| {
            let p = probabilities.get(option).copied().unwrap_or(0.0);
            let y = if option == target { 1.0 } else { 0.0 };
            (p - y) * (p - y)
        })
        .sum()
}

/// The Brier score of a forecast that puts probability 1 on `predicted`.
fn brier_hard(predicted: &str, target: &str) -> f64 {
    if predicted == target { 0.0 } else { 2.0 }
}

fn words(text: &str) -> BTreeSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() >= 4)
        .map(str::to_lowercase)
        .collect()
}

/// The word-overlap reference: the registry class sharing the largest fraction of its words (class name and
/// description) with the event text, or [`NONE_OPTION`] when that fraction does not exceed `threshold`.
struct Overlap {
    classes: Vec<(String, BTreeSet<String>)>,
}

impl Overlap {
    fn new(classes: &[ClassRow]) -> Self {
        Overlap {
            classes: classes
                .iter()
                .map(|c| {
                    (
                        c.id.clone(),
                        words(&format!("{} {}", c.name, c.description)),
                    )
                })
                .collect(),
        }
    }

    fn predict(&self, text: &str, threshold: f64) -> String {
        let text = words(text);
        let mut best: Option<(&str, f64)> = None;
        for (id, class_words) in &self.classes {
            if class_words.is_empty() {
                continue;
            }
            let shared = class_words.intersection(&text).count() as f64;
            let score = shared / class_words.len() as f64;
            if best.is_none_or(|(_, b)| score > b) {
                best = Some((id, score));
            }
        }
        match best {
            Some((id, score)) if score > threshold => id.to_string(),
            _ => NONE_OPTION.to_string(),
        }
    }

    /// The grid threshold with the fewest misses on the calibration rows (the lowest on a tie).
    fn calibrate(&self, calibration: &[&Row]) -> f64 {
        let misses = |t: f64| {
            calibration
                .iter()
                .filter(|r| self.predict(&r.text, t) != r.target())
                .count()
        };
        OVERLAP_GRID
            .into_iter()
            .min_by_key(|t| misses(*t))
            .expect("the grid is not empty")
    }
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn sample_sd(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let m = mean(values);
    (values.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / (values.len() - 1) as f64).sqrt()
}

/// A normal-approximation 95% interval on the mean.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Interval {
    pub low: f64,
    pub high: f64,
}

/// The 95% interval on the mean of the per-record improvements `diffs`, given only when the sample is large
/// enough for a lower bound above zero: `n > (1.96 * sd / mean)^2` with a positive mean (section 10.4 item 1).
pub fn interval(diffs: &[f64]) -> Option<Interval> {
    let needed = needed_n(diffs)?;
    let n = diffs.len() as f64;
    if n <= needed {
        return None;
    }
    let m = mean(diffs);
    let half = Z95 * sample_sd(diffs) / n.sqrt();
    Some(Interval {
        low: m - half,
        high: m + half,
    })
}

/// The test size `(1.96 * sd / mean)^2` that a bound needs; none when there are fewer than two records or the
/// mean is not above zero.
pub fn needed_n(diffs: &[f64]) -> Option<f64> {
    if diffs.len() < 2 {
        return None;
    }
    let m = mean(diffs);
    (m > 0.0).then(|| (Z95 * sample_sd(diffs) / m).powi(2))
}

/// One test row's scores.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RowScore {
    pub introspection: String,
    pub at: Timestamp,
    pub label: Option<String>,
    pub labeled_by: Labeler,
    pub jev_choice: String,
    pub jev_brier: f64,
    pub always_none_brier: f64,
    pub overlap_brier: f64,
}

/// The is-repeat score on the test rows where the reviewer said whether the event repeats.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RepeatScore {
    pub n: usize,
    pub jev_brier: f64,
    pub always_false_brier: f64,
    /// The calibration part's base rate as a constant forecast; none when no calibration row says.
    pub base_rate_brier: Option<f64>,
    pub better_reference: String,
    /// Reference Brier minus Jev Brier; positive is better than the reference.
    pub skill: f64,
}

/// The score of the class answers on the test part.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Score {
    /// The test size.
    pub n: usize,
    pub jev_brier: f64,
    pub always_none_brier: f64,
    pub overlap_brier: f64,
    /// The overlap matcher's threshold, set on the calibration part.
    pub overlap_threshold: f64,
    /// `always none` or `word overlap`, whichever has the lower mean Brier on the test part.
    pub better_reference: String,
    /// Mean per-record Brier improvement over the better reference.
    pub skill: f64,
    pub interval: Option<Interval>,
    /// The test size a lower bound would need, when the mean is above zero.
    pub needed_n: Option<f64>,
    /// Set when no interval is given: why, and what to read instead.
    pub no_bound_note: Option<String>,
    pub top1_accuracy: f64,
    /// Labels that are `none`, in the test part.
    pub none_labels: usize,
    /// Of those, the share Jev matched to a registry class; none when there are no `none` labels.
    pub false_match_rate: Option<f64>,
    pub repeat: Option<RepeatScore>,
    pub per_record: Vec<RowScore>,
}

/// Where a section 10.4 item stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ItemStatus {
    Met,
    NotMet,
    /// Computed, but the sample cannot show it either way.
    NotEstablished,
    /// This build does not compute it.
    NotEvaluated,
}

impl ItemStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            ItemStatus::Met => "met",
            ItemStatus::NotMet => "not met",
            ItemStatus::NotEstablished => "not established",
            ItemStatus::NotEvaluated => "not evaluated",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VerdictItem {
    /// The item's number in section 10.4.
    pub item: u8,
    pub name: String,
    pub status: ItemStatus,
    pub detail: String,
}

/// The overall reading of the items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Overall {
    /// An item that was computed is not met.
    NotMet,
    /// No computed item fails, but at least one cannot be established from this sample.
    NotEstablished,
    /// Every item this build computes is met. The others are not evaluated, so this is not a promotion.
    ComputedItemsMet,
}

/// The promotion verdict of section 10.4, over the items computable from these records.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Verdict {
    pub overall: Overall,
    pub summary: String,
    pub items: Vec<VerdictItem>,
}

/// The report: the labeled set's size, the split, what was left out, and either the score and verdict or the
/// reason there are none.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Report {
    pub excluded: Exclusions,
    /// Rows before the cut.
    pub calibration: usize,
    /// Rows from the cut on; the only rows scored.
    pub test: usize,
    pub cut: Option<Timestamp>,
    pub min_test_cases: usize,
    /// Set when there are too few labeled cases; then `score` and `verdict` are absent.
    pub insufficient: Option<String>,
    pub score: Option<Score>,
    pub verdict: Option<Verdict>,
    /// The rows of the labeled set, with who labeled each.
    pub rows: Vec<Row>,
}

/// Splits `rows` (oldest first) at `cut`; without one, the earliest two thirds calibrate. Returns the cut used.
pub fn split(rows: &[Row], cut: Option<Timestamp>) -> (Option<Timestamp>, Vec<&Row>, Vec<&Row>) {
    let cut = cut.or_else(|| rows.get(rows.len() * 2 / 3).map(|r| r.at));
    let Some(cut) = cut else {
        return (None, Vec::new(), Vec::new());
    };
    let (calibration, test) = rows.iter().partition(|r| r.at < cut);
    (Some(cut), calibration, test)
}

/// Scores the labeled set and reaches the verdict. `cut` overrides the default time split.
pub fn report(set: &LabeledSet, classes: &[ClassRow], cut: Option<Timestamp>) -> Report {
    let (cut, calibration, test) = split(&set.rows, cut);
    let mut report = Report {
        excluded: set.excluded.clone(),
        calibration: calibration.len(),
        test: test.len(),
        cut,
        min_test_cases: MIN_TEST_CASES,
        insufficient: None,
        score: None,
        verdict: None,
        rows: set.rows.clone(),
    };
    if test.len() < MIN_TEST_CASES {
        report.insufficient = Some(format!(
            "too few labeled cases: {} in the test part ({} labeled in all, {} in the calibration part); at least {MIN_TEST_CASES} test cases are needed. No score and no verdict.",
            test.len(),
            set.rows.len(),
            calibration.len(),
        ));
        return report;
    }
    let score = score(&calibration, &test, classes);
    report.verdict = Some(verdict(&score, &test));
    report.score = Some(score);
    report
}

fn score(calibration: &[&Row], test: &[&Row], classes: &[ClassRow]) -> Score {
    let overlap = Overlap::new(classes);
    let threshold = overlap.calibrate(calibration);
    let per_record: Vec<RowScore> = test
        .iter()
        .map(|r| RowScore {
            introspection: r.introspection.clone(),
            at: r.at,
            label: r.label.clone(),
            labeled_by: r.labeled_by,
            jev_choice: r.jev_choice.clone(),
            jev_brier: brier(&r.jev_probabilities, r.target()),
            always_none_brier: brier_hard(NONE_OPTION, r.target()),
            overlap_brier: brier_hard(&overlap.predict(&r.text, threshold), r.target()),
        })
        .collect();
    let column = |f: fn(&RowScore) -> f64| per_record.iter().map(f).collect::<Vec<f64>>();
    let jev = column(|r| r.jev_brier);
    let none = column(|r| r.always_none_brier);
    let word = column(|r| r.overlap_brier);
    let (better_reference, reference) = if mean(&word) < mean(&none) {
        ("word overlap", &word)
    } else {
        ("always none", &none)
    };
    let diffs: Vec<f64> = reference.iter().zip(&jev).map(|(r, j)| r - j).collect();
    let skill = mean(&diffs);
    let interval = interval(&diffs);
    let needed = needed_n(&diffs);
    let no_bound_note = interval.is_none().then(|| match needed {
        Some(needed) => format!(
            "no bound was computed: a lower bound needs more than {needed:.1} test cases at this spread and the test part has {}. Judge the skill from the raw Brier scores and the per-record table.",
            test.len()
        ),
        None => "no bound was computed: the mean improvement is not above zero. Judge the skill from the raw Brier scores and the per-record table.".to_string(),
    });
    let none_rows: Vec<&&Row> = test.iter().filter(|r| r.label.is_none()).collect();
    let false_match_rate = (!none_rows.is_empty()).then(|| {
        none_rows
            .iter()
            .filter(|r| r.jev_choice != NONE_OPTION)
            .count() as f64
            / none_rows.len() as f64
    });
    Score {
        n: test.len(),
        jev_brier: mean(&jev),
        always_none_brier: mean(&none),
        overlap_brier: mean(&word),
        overlap_threshold: threshold,
        better_reference: better_reference.to_string(),
        skill,
        interval,
        needed_n: needed,
        no_bound_note,
        top1_accuracy: test.iter().filter(|r| r.jev_choice == r.target()).count() as f64
            / test.len() as f64,
        none_labels: none_rows.len(),
        false_match_rate,
        repeat: repeat_score(calibration, test),
        per_record,
    }
}

fn repeat_score(calibration: &[&Row], test: &[&Row]) -> Option<RepeatScore> {
    let rated: Vec<(f64, f64)> = test
        .iter()
        .filter_map(|r| r.is_repeat.map(|y| (r.jev_repeat, f64::from(u8::from(y)))))
        .collect();
    if rated.is_empty() {
        return None;
    }
    let jev: Vec<f64> = rated.iter().map(|(p, y)| (p - y) * (p - y)).collect();
    let never: Vec<f64> = rated.iter().map(|(_, y)| y * y).collect();
    let calibration_labels: Vec<f64> = calibration
        .iter()
        .filter_map(|r| r.is_repeat.map(|y| f64::from(u8::from(y))))
        .collect();
    let base: Option<Vec<f64>> = (!calibration_labels.is_empty()).then(|| {
        let rate = mean(&calibration_labels);
        rated.iter().map(|(_, y)| (rate - y) * (rate - y)).collect()
    });
    let (name, reference) = match &base {
        Some(base) if mean(base) < mean(&never) => ("base rate", mean(base)),
        _ => ("always false", mean(&never)),
    };
    Some(RepeatScore {
        n: rated.len(),
        jev_brier: mean(&jev),
        always_false_brier: mean(&never),
        base_rate_brier: base.as_deref().map(mean),
        better_reference: name.to_string(),
        skill: reference - mean(&jev),
    })
}

/// Whether each reviewed group lands in one Jev class, and whether any Jev class mixes groups. Returns the
/// groups split over several classes and the Jev classes that hold several groups.
fn grouping(test: &[&Row]) -> (Vec<String>, Vec<String>) {
    let mut by_group: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut by_class: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for r in test {
        by_group
            .entry(r.target())
            .or_default()
            .insert(&r.jev_choice);
        by_class
            .entry(&r.jev_choice)
            .or_default()
            .insert(r.target());
    }
    let many = |m: BTreeMap<&str, BTreeSet<&str>>| {
        m.into_iter()
            .filter(|(_, v)| v.len() > 1)
            .map(|(k, _)| k.to_string())
            .collect::<Vec<_>>()
    };
    (many(by_group), many(by_class))
}

fn item(item: u8, name: &str, status: ItemStatus, detail: String) -> VerdictItem {
    VerdictItem {
        item,
        name: name.to_string(),
        status,
        detail,
    }
}

fn verdict(score: &Score, test: &[&Row]) -> Verdict {
    let skill = match (&score.interval, score.skill > 0.0) {
        (Some(i), _) => item(
            1,
            "matching skill above zero on the test part",
            ItemStatus::Met,
            format!(
                "skill {:.3} over {} (95% interval {:.3} to {:.3}, n = {})",
                score.skill, score.better_reference, i.low, i.high, score.n
            ),
        ),
        (None, true) => item(
            1,
            "matching skill above zero on the test part",
            ItemStatus::NotEstablished,
            format!(
                "skill {:.3} over {} but no bound was computed (n = {}); the user judges it from the raw scores",
                score.skill, score.better_reference, score.n
            ),
        ),
        (None, false) => item(
            1,
            "matching skill above zero on the test part",
            ItemStatus::NotMet,
            format!(
                "skill {:.3} over {} is not above zero (n = {})",
                score.skill, score.better_reference, score.n
            ),
        ),
    };
    let (split_groups, mixed_classes) = grouping(test);
    const GROUPING: &str = "each reviewed group lands in one class, none mixed (reviewed groups only; R1 to R4 and the cases of section 3.6 are not looked up)";
    let grouping_item = if split_groups.is_empty() && mixed_classes.is_empty() {
        item(
            2,
            GROUPING,
            ItemStatus::Met,
            format!(
                "{} reviewed groups, none split, none mixed",
                group_count(test)
            ),
        )
    } else {
        item(
            2,
            GROUPING,
            ItemStatus::NotMet,
            format!(
                "groups split over several Jev classes: [{}]; Jev classes that mix groups: [{}]",
                split_groups.join(", "),
                mixed_classes.join(", ")
            ),
        )
    };
    let false_match = match score.false_match_rate {
        None => item(
            3,
            "false-match rate on `none` labels under 10% (estimate)",
            ItemStatus::NotEstablished,
            "the test part has no `none` labels".to_string(),
        ),
        Some(rate) => item(
            3,
            "false-match rate on `none` labels under 10% (estimate)",
            if rate < MAX_FALSE_MATCH_RATE {
                ItemStatus::Met
            } else {
                ItemStatus::NotMet
            },
            format!(
                "{:.1}% of {} `none` labels",
                rate * 100.0,
                score.none_labels
            ),
        ),
    };
    let mut items = vec![skill, grouping_item, false_match];
    for (number, name, why) in [
        (
            4,
            "no cited record id is invented, and every draft parses",
            "needs shadow proposals, which this build does not make",
        ),
        (
            5,
            "drafts and causes rated faithful (90%, estimate)",
            "needs prose ratings, which are not built",
        ),
        (
            6,
            "set membership and signal match, promoted separately",
            "those answers are not recorded or scored here",
        ),
        (
            7,
            "the user reviews this report and agrees",
            "the user's decision",
        ),
    ] {
        items.push(item(
            number,
            name,
            ItemStatus::NotEvaluated,
            why.to_string(),
        ));
    }
    let computed = &items[..3];
    let (overall, summary) = if computed.iter().any(|i| i.status == ItemStatus::NotMet) {
        (
            Overall::NotMet,
            "not promotable: a computed item is not met",
        )
    } else if computed
        .iter()
        .any(|i| i.status == ItemStatus::NotEstablished)
    {
        (
            Overall::NotEstablished,
            "not established: no computed item fails, but this sample cannot show one of them",
        )
    } else {
        (
            Overall::ComputedItemsMet,
            "the computed items (1 to 3) are met; items 4 to 7 are not evaluated, so this is not a promotion",
        )
    };
    Verdict {
        overall,
        summary: summary.to_string(),
        items,
    }
}

fn group_count(test: &[&Row]) -> usize {
    test.iter()
        .map(|r| r.target())
        .collect::<BTreeSet<_>>()
        .len()
}

// ---------------------------------------------------------------------------------------------------------
// The command
// ---------------------------------------------------------------------------------------------------------

impl Athena {
    /// `athena score-jev`: reads the records in the working tree, builds the labeled set and scores Jev's
    /// recorded shadow answers against it. Read-only: nothing is written, committed or asked of Jev.
    pub fn score_jev(&self, cut: Option<Timestamp>) -> Result<Report> {
        let inputs = read_inputs(self.git.dir())?;
        let set = build(&inputs);
        Ok(report(&set, &inputs.classes, cut))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::introspect::ask_jev;
    use crate::jev::{Answer, Evaluation, FakeJev, JevError, Usage};
    use athena_data::records::{
        ClassStatus, FailureClass, IntrospectionJev, IntrospectionStatus, IntrospectionTrigger,
        PreventionKind,
    };
    use athena_data::{JobId, RecordId};

    const A: &str = "20261001-000000-c001";
    const B: &str = "20261001-000000-c002";

    fn ts(text: &str) -> Timestamp {
        text.parse().unwrap()
    }

    /// The time of the `i`th case: one minute apart from midnight on the 1st.
    fn at(i: u32) -> Timestamp {
        ts(&format!("2026-10-01T{:02}:{:02}:00Z", i / 60, i % 60))
    }

    fn rid(i: u32, n: u32) -> RecordId {
        RecordId::from_parts(at(i), n)
    }

    fn classes() -> Vec<ClassRow> {
        vec![
            ClassRow {
                id: A.into(),
                name: "Field with no use".into(),
                description: "A new field has no reader.".into(),
            },
            ClassRow {
                id: B.into(),
                name: "Missing test".into(),
                description: "A change lands without a test.".into(),
            },
        ]
    }

    /// Jev's stored answer, made by asking the fake the real questions: probability `p` on `says` and the rest
    /// shared by the other options.
    fn jev_says(says: &str, p: f64, repeat: f64) -> IntrospectionJev {
        let options = [A, B, JEV_NEW_OPTION];
        let rest = (1.0 - p) / 2.0;
        let probabilities = options
            .iter()
            .map(|o| (o.to_string(), if *o == says { p } else { rest }))
            .collect();
        let evaluation = Evaluation {
            answers: BTreeMap::from([
                (
                    crate::introspect::JEV_CLASS_KEY.to_string(),
                    Answer::Choice {
                        choice: says.to_string(),
                        probabilities,
                        confidence: 0.9,
                    },
                ),
                (
                    crate::introspect::JEV_REPEAT_KEY.to_string(),
                    Answer::Boolean {
                        probability: repeat,
                    },
                ),
            ]),
            usage: Usage {
                input_tokens: 10,
                output_tokens: 2,
            },
            cost_usd: Some(0.00002),
        };
        ask_jev(&FakeJev::new().then(Ok(evaluation)), "state", &classes())
    }

    fn jev_error() -> IntrospectionJev {
        ask_jev(
            &FakeJev::new().then(Err(JevError::Transport("down".into()))),
            "state",
            &classes(),
        )
    }

    fn introspection(i: u32, text: &str, jev: Option<IntrospectionJev>) -> IntrospectionRecord {
        IntrospectionRecord {
            id: rid(i, 1),
            at: at(i),
            trigger: IntrospectionTrigger::Correction,
            job: Some(JobId::parse(&format!("20261001-000000-j{i:03}")).unwrap()),
            intervention: Some(rid(i, 501)),
            class_of_work: "code".into(),
            category: "corrected-output".into(),
            what_happened: Some(text.into()),
            failure_class: Some(RecordId::parse(A).unwrap()),
            new_class: false,
            earlier_cases: vec![],
            decisions_matched: vec![],
            cause: Some("Nothing ties it down.".into()),
            prevention_kind: Some(PreventionKind::Default),
            prevention: Some("Add it.".into()),
            proposed_request: Some("Add it.".into()),
            evidence: None,
            status: IntrospectionStatus::Recorded,
            failure: None,
            flags: vec![],
            dropped_ids: vec![],
            planning_stopped: false,
            from_jev: false,
            jev,
        }
    }

    fn label(i: u32, class: Option<&str>, by: Labeler, repeat: Option<bool>) -> ClassLabel {
        ClassLabel {
            id: rid(i, 900),
            at: at(i),
            introspection: rid(i, 1),
            class: class.map(|c| RecordId::parse(c).unwrap()),
            labeled_by: by,
            is_repeat: repeat,
            note: None,
        }
    }

    /// A reviewed case `i` labeled `class` (None is `none`) that Jev answers with `says`.
    fn case(inputs: &mut Inputs, i: u32, class: Option<&str>, says: &str) {
        case_with(inputs, i, class, says, 0.9);
    }

    /// As [`case`], with Jev's probability `p` on its choice.
    fn case_with(inputs: &mut Inputs, i: u32, class: Option<&str>, says: &str, p: f64) {
        inputs.introspections.push(introspection(
            i,
            "Something unrelated happened.",
            Some(jev_says(says, p, 0.2)),
        ));
        inputs
            .labels
            .push(label(i, class, Labeler::Person, Some(false)));
    }

    fn registry() -> Inputs {
        Inputs {
            classes: classes(),
            ..Inputs::default()
        }
    }

    /// 20 calibration cases, then `test` cases (6 of A that Jev gets right, 4 `none` that it matches to B).
    fn full_inputs() -> Inputs {
        let mut inputs = registry();
        for i in 0..20 {
            case(&mut inputs, i, Some(A), A);
        }
        for i in 100..106 {
            case(&mut inputs, i, Some(A), A);
        }
        for i in 106..110 {
            case(&mut inputs, i, None, B);
        }
        inputs
    }

    fn cut() -> Option<Timestamp> {
        Some(at(100))
    }

    #[test]
    fn enough_labeled_cases_give_a_score_a_sample_size_a_note_and_a_verdict() {
        let inputs = full_inputs();
        let set = build(&inputs);
        let report = report(&set, &inputs.classes, cut());
        assert_eq!((report.calibration, report.test), (20, 10));
        assert!(report.insufficient.is_none());
        let score = report.score.expect("a score");
        assert_eq!(score.n, 10);
        // Jev: 0.9 on the choice leaves 0.05 on each other option. Right (the six A cases): 0.01 + 2 * 0.0025 =
        // 0.015. Wrong (the four `none` cases, where it says B): 0.81 + 0.0025 + 0.9025 = 1.715.
        assert!((score.jev_brier - (6.0 * 0.015 + 4.0 * 1.715) / 10.0).abs() < 1e-9);
        // Always-none misses the six A cases (2 each); overlap on text with no class words does the same.
        assert!((score.always_none_brier - 1.2).abs() < 1e-9);
        assert!((score.overlap_brier - 1.2).abs() < 1e-9);
        assert_eq!(score.better_reference, "always none");
        assert!((score.skill - (1.2 - score.jev_brier)).abs() < 1e-9);
        assert!(score.skill > 0.0);
        // Mean 0.8-ish with a spread near 2: a bound would need more than 10 cases, so none is computed.
        assert!(score.interval.is_none());
        assert!(
            score
                .no_bound_note
                .as_deref()
                .unwrap()
                .contains("no bound was computed")
        );
        assert_eq!(score.top1_accuracy, 0.6);
        assert_eq!((score.none_labels, score.false_match_rate), (4, Some(1.0)));
        assert_eq!(score.per_record.len(), 10);
        let repeat = score.repeat.expect("a repeat score");
        assert_eq!(repeat.n, 10);
        assert!((repeat.jev_brier - 0.04).abs() < 1e-9);
        // Always false is perfect on rows said not to repeat, so Jev has no skill there.
        assert!(repeat.skill < 0.0);

        let verdict = report.verdict.expect("a verdict");
        assert_eq!(verdict.overall, Overall::NotMet);
        let status = |n: u8| verdict.items.iter().find(|i| i.item == n).unwrap().status;
        assert_eq!(status(1), ItemStatus::NotEstablished);
        // Every reviewed group goes to one Jev class (the `none` cases all to B), none mixed.
        assert_eq!(status(2), ItemStatus::Met);
        assert_eq!(status(3), ItemStatus::NotMet);
        for n in 4..=7 {
            assert_eq!(status(n), ItemStatus::NotEvaluated);
        }
        // The labeler stays on each row.
        assert!(report.rows.iter().all(|r| r.labeled_by == Labeler::Person));
    }

    #[test]
    fn a_large_enough_test_part_gives_an_interval_and_met_items() {
        let mut inputs = registry();
        for i in 0..5 {
            case(&mut inputs, i, Some(A), A);
        }
        // Thirty test cases, all labeled A or B and answered correctly, with a varying confidence.
        for i in 100..115 {
            case_with(
                &mut inputs,
                i,
                Some(A),
                A,
                if i % 2 == 0 { 0.9 } else { 0.6 },
            );
        }
        for i in 115..130 {
            case_with(
                &mut inputs,
                i,
                Some(B),
                B,
                if i % 2 == 0 { 0.9 } else { 0.6 },
            );
        }
        let set = build(&inputs);
        let report = report(&set, &inputs.classes, cut());
        let score = report.score.unwrap();
        let interval = score.interval.expect("an interval");
        assert!(interval.low > 0.0 && interval.low < score.skill && score.skill < interval.high);
        assert!(score.no_bound_note.is_none());
        assert_eq!(score.false_match_rate, None);
        let verdict = report.verdict.unwrap();
        let status = |n: u8| verdict.items.iter().find(|i| i.item == n).unwrap().status;
        assert_eq!(status(1), ItemStatus::Met);
        assert_eq!(status(2), ItemStatus::Met);
        // No `none` labels in the test part: item 3 cannot be shown.
        assert_eq!(status(3), ItemStatus::NotEstablished);
        assert_eq!(verdict.overall, Overall::NotEstablished);
    }

    #[test]
    fn too_few_labeled_cases_give_a_message_and_no_score_or_verdict() {
        let mut inputs = registry();
        for i in 0..3 {
            case(&mut inputs, i, Some(A), A);
        }
        let set = build(&inputs);
        for cut in [None, Some(at(1))] {
            let report = report(&set, &inputs.classes, cut);
            let message = report.insufficient.as_deref().expect("a message");
            assert!(message.contains("too few labeled cases"), "{message}");
            assert!(message.contains("No score and no verdict"));
            assert!(report.score.is_none() && report.verdict.is_none());
        }
        // Nothing at all reads the same way.
        let empty = report(&build(&registry()), &[], None);
        assert!(
            empty
                .insufficient
                .as_deref()
                .unwrap()
                .contains("too few labeled cases")
        );
        assert_eq!((empty.cut, empty.test), (None, 0));
    }

    #[test]
    fn unreviewed_introspections_and_those_without_a_usable_jev_answer_are_excluded_and_counted() {
        let mut inputs = registry();
        case(&mut inputs, 1, Some(A), A);
        // Unreviewed: a Jev answer but no label.
        inputs
            .introspections
            .push(introspection(2, "x", Some(jev_says(A, 0.9, 0.1))));
        // Reviewed, but Jev was not asked.
        inputs.introspections.push(introspection(3, "x", None));
        inputs.labels.push(label(3, Some(A), Labeler::Person, None));
        // Reviewed, but the Jev call failed.
        inputs
            .introspections
            .push(introspection(4, "x", Some(jev_error())));
        inputs
            .labels
            .push(label(4, None, Labeler::StrongerModel, None));
        // A label that names no introspection.
        inputs.labels.push(label(5, Some(B), Labeler::Person, None));
        let set = build(&inputs);
        assert_eq!(set.rows.len(), 1);
        assert_eq!(
            set.excluded,
            Exclusions {
                introspections: 4,
                labeled: 1,
                unreviewed: 1,
                no_jev_answer: 1,
                jev_error: 1,
                labels_without_introspection: 1,
            }
        );
        let e = &set.excluded;
        assert_eq!(
            e.labeled + e.unreviewed + e.no_jev_answer + e.jev_error,
            e.introspections
        );
        // The counts survive in a report with too few cases.
        assert_eq!(report(&set, &inputs.classes, None).excluded, set.excluded);
    }

    #[test]
    fn the_labeler_is_kept_and_the_newest_label_counts() {
        let mut inputs = registry();
        inputs
            .introspections
            .push(introspection(1, "x", Some(jev_says(A, 0.9, 0.1))));
        inputs
            .labels
            .push(label(1, Some(B), Labeler::StrongerModel, None));
        let mut later = label(1, Some(A), Labeler::Person, None);
        later.at = at(50);
        later.id = rid(50, 1);
        inputs.labels.push(later);
        let set = build(&inputs);
        let [row] = set.rows.as_slice() else {
            panic!("one row")
        };
        assert_eq!(row.label.as_deref(), Some(A));
        assert_eq!(row.labeled_by, Labeler::Person);
    }

    #[test]
    fn unreviewed_lists_introspections_with_no_label_and_agrees_with_build() {
        let mut inputs = registry();
        case(&mut inputs, 1, Some(A), A);
        inputs
            .introspections
            .push(introspection(2, "x", Some(jev_says(A, 0.9, 0.1))));
        let open = unreviewed(&inputs.introspections, &inputs.labels);
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].id, rid(2, 1));
        assert_eq!(build(&inputs).excluded.unreviewed, 1);
        inputs.labels.push(label(2, None, Labeler::Person, None));
        assert!(unreviewed(&inputs.introspections, &inputs.labels).is_empty());
    }

    #[test]
    fn brier_is_zero_when_right_and_two_when_certain_and_wrong() {
        let p = |a: f64, b: f64| BTreeMap::from([("a".to_string(), a), ("b".to_string(), b)]);
        assert_eq!(brier(&p(1.0, 0.0), "a"), 0.0);
        assert_eq!(brier(&p(1.0, 0.0), "b"), 2.0);
        assert!((brier(&p(0.5, 0.5), "a") - 0.5).abs() < 1e-12);
        // A label outside the options counts as an option Jev gave probability 0.
        assert_eq!(brier(&p(1.0, 0.0), "c"), 2.0);
    }

    #[test]
    fn the_interval_needs_the_sample_size_of_section_10_4() {
        // Mean 1, sd 1: needs n > 3.84.
        let diffs =
            |n: usize| -> Vec<f64> { (0..n).map(|i| if i % 2 == 0 { 0.0 } else { 2.0 }).collect() };
        assert!(interval(&diffs(3)).is_none());
        let bound = interval(&diffs(40)).expect("enough cases");
        assert!(bound.low > 0.0 && bound.low < 1.0 && bound.high > 1.0);
        // A mean at or below zero never gets a bound.
        assert!(interval(&[-1.0, -2.0, -1.5]).is_none());
        assert!(needed_n(&[1.0]).is_none());
    }

    #[test]
    fn the_default_split_leaves_the_earliest_two_thirds_for_calibration() {
        let mut inputs = registry();
        for i in 0..9 {
            case(&mut inputs, i, Some(A), A);
        }
        let set = build(&inputs);
        let (cut, calibration, test) = split(&set.rows, None);
        assert_eq!(cut, Some(at(6)));
        assert_eq!((calibration.len(), test.len()), (6, 3));
        assert!(calibration.iter().all(|r| r.at < at(6)));
    }

    #[test]
    fn the_overlap_reference_reads_class_words_and_is_set_on_the_calibration_part() {
        let overlap = Overlap::new(&classes());
        assert_eq!(overlap.predict("the field has no reader", 0.1), A);
        assert_eq!(overlap.predict("nothing in common", 0.1), NONE_OPTION);
        // A high threshold turns a partial match into `none`.
        assert_eq!(overlap.predict("a field appeared", 0.5), NONE_OPTION);
        let mut inputs = registry();
        inputs.introspections.push(introspection(
            1,
            "A change lands without a test",
            Some(jev_says(B, 0.9, 0.1)),
        ));
        inputs.labels.push(label(1, Some(B), Labeler::Person, None));
        let set = build(&inputs);
        let rows: Vec<&Row> = set.rows.iter().collect();
        assert!(overlap.calibrate(&rows) <= 0.5);
        assert_eq!(overlap.calibrate(&[]), OVERLAP_GRID[0]);
    }

    #[test]
    fn the_scoring_path_never_opens_a_file_for_writing() {
        let source = include_str!("labeled.rs");
        let code = &source[..source.find("#[cfg(test)]").unwrap()];
        for forbidden in [
            "File::create",
            "OpenOptions",
            "fs::write",
            "fs::rename",
            "fs::copy",
            "fs::remove",
            "fs::create_dir",
            "write_all",
            "set_len",
            "commit_records",
        ] {
            assert!(!code.contains(forbidden), "the module uses {forbidden}");
        }
    }

    #[test]
    fn a_failure_class_record_in_the_fixture_is_a_valid_registry_entry() {
        // The registry rows come from `athena.failure-class/1` records; check the reader maps their fields.
        let dir = tempfile::tempdir().unwrap();
        let class = AnyRecord::FailureClass(FailureClass {
            id: RecordId::parse(A).unwrap(),
            at: at(0),
            name: "Field with no use".into(),
            description: "A new field has no reader.".into(),
            status: ClassStatus::Proposed,
            proposed_by: rid(0, 1),
        });
        let path = dir.path().join(class.canonical_path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, class.to_json()).unwrap();
        let inputs = read_inputs(dir.path()).unwrap();
        assert_eq!(inputs.classes, classes()[..1]);
        assert!(
            read_inputs(&dir.path().join("none"))
                .unwrap()
                .classes
                .is_empty()
        );
    }
}
