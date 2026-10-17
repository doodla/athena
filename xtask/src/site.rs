//! The explainer site: `cargo run -p xtask -- site [--out <dir>]`.
//!
//! Renders one self-contained HTML file, `index.html`, from the JSON files of
//! `studies/firstmate-work-handling` (including `intro.json`, the newcomer introduction). Every sentence on the page comes from those files, is HTML-escaped,
//! and carries the id of the claim in `claims.json` that supports it. A citation of a claim that does not
//! exist, an element that cites no claim, or a missing file is an error, so a page is never built from
//! partial data. The design is in `docs/design/explainer-site.md`.

use anyhow::{Context, Result, anyhow, bail};
use serde_json::Value;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fmt::Write as _;
use std::path::{Component, Path, PathBuf};

/// The study the page explains, relative to the repository root.
pub const STUDY_DIR: &str = "studies/firstmate-work-handling";
/// The default output directory, relative to the cargo target directory.
pub const DEFAULT_OUT: &str = "site";

/// The cargo target directory: `CARGO_TARGET_DIR` when set and not empty (made absolute against `cwd`),
/// else `target` under the repository root.
pub fn target_dir(root: &Path, env: Option<OsString>, cwd: &Path) -> PathBuf {
    match env.filter(|v| !v.is_empty()) {
        Some(dir) => cwd.join(dir),
        None => root.join("target"),
    }
}

/// The directory the page goes to when `--out` is not given: `site/` in the cargo target directory.
/// Building there never writes into the repository work tree, so a verifier's snapshot stays unchanged.
pub fn default_out(root: &Path, env: Option<OsString>, cwd: &Path) -> PathBuf {
    target_dir(root, env, cwd).join(DEFAULT_OUT)
}

/// Makes `path` absolute and free of `.` and `..`, following symlinks in the part that exists.
fn resolve(path: &Path, cwd: &Path) -> PathBuf {
    let mut abs = PathBuf::new();
    for part in cwd.join(path).components() {
        match part {
            Component::ParentDir => {
                abs.pop();
            }
            Component::CurDir => {}
            other => abs.push(other.as_os_str()),
        }
    }
    let mut tail = Vec::new();
    let mut head = abs.as_path();
    loop {
        if let Ok(real) = head.canonicalize() {
            return tail.iter().rev().fold(real, |p, part| p.join(part));
        }
        match (head.parent(), head.file_name()) {
            (Some(parent), Some(name)) => {
                tail.push(name.to_owned());
                head = parent;
            }
            _ => return abs,
        }
    }
}

/// Refuses an output directory inside the repository work tree, unless it is inside the cargo target
/// directory (which is gitignored or outside the tree). Writing anywhere else in the tree would change
/// the files a verifier snapshots.
pub fn check_out(root: &Path, out: &Path, target: &Path, cwd: &Path) -> Result<()> {
    let root = resolve(root, cwd);
    let out = resolve(out, cwd);
    let target = resolve(target, cwd);
    if out.starts_with(&root) && !out.starts_with(&target) {
        bail!(
            "refusing to write the page to {}: it is inside the repository work tree ({}), and building \
             must not change the tree. Use a directory under the cargo target directory ({}) or outside the repository",
            out.display(),
            root.display(),
            target.display()
        );
    }
    Ok(())
}

/// The parsed study files.
#[derive(Debug)]
pub struct Study {
    pin: Value,
    claims: Value,
    model: Value,
    rationales: Value,
    findings: Value,
    intro: Value,
}

/// Reads the study files from `dir`. A missing or malformed file is an error that names the file.
pub fn load(dir: &Path) -> Result<Study> {
    let read = |name: &str| -> Result<Value> {
        let path = dir.join(name);
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("study data is missing: cannot read {}", path.display()))?;
        serde_json::from_str(&text).with_context(|| format!("{} is not valid JSON", path.display()))
    };
    Ok(Study {
        pin: read("pin.json")?,
        claims: read("claims.json")?,
        model: read("model.json")?,
        rationales: read("rationales.json")?,
        findings: read("findings.json")?,
        intro: read("intro.json")?,
    })
}

/// Escapes text for HTML element content and attribute values.
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
    out
}

fn str_of<'a>(value: &'a Value, key: &str, what: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("{what} has no text field `{key}`"))
}

fn array_of<'a>(value: &'a Value, key: &str, what: &str) -> Result<&'a Vec<Value>> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{what} has no list `{key}`"))
}

/// The ids of the claims in `claims.json`, checked to be well formed.
fn claim_ids(claims: &Value) -> Result<BTreeSet<String>> {
    let list = claims
        .as_array()
        .ok_or_else(|| anyhow!("claims.json is not a list"))?;
    list.iter()
        .map(|c| str_of(c, "id", "a claim").map(String::from))
        .collect()
}

/// Links to the claims an element cites. At least one claim is required, and each must exist.
fn cites(value: &Value, what: &str, known: &BTreeSet<String>) -> Result<String> {
    let ids = array_of(value, "claims", what)?;
    if ids.is_empty() {
        bail!("{what} cites no claim");
    }
    let mut out = String::from("<span class=\"cites\">");
    for id in ids {
        let id = id
            .as_str()
            .ok_or_else(|| anyhow!("{what} has a claim id that is not text"))?;
        if !known.contains(id) {
            bail!("{what} cites {id}, which is not in claims.json");
        }
        let id = escape(id);
        let _ = write!(out, "<a href=\"#claim-{id}\">[{id}]</a> ");
    }
    out.truncate(out.trim_end().len());
    out.push_str("</span>");
    Ok(out)
}

/// Athena's own repository, linked from the next-steps section.
const ATHENA_URL: &str = "https://github.com/doodla/athena";

/// The web address of a GitHub repository given its clone URL (`https://github.com/<owner>/<name>[.git]`),
/// or `None` when the repository is not on GitHub.
fn github_base(repo: &str) -> Option<String> {
    let rest = repo.strip_prefix("https://github.com/")?;
    let rest = rest.trim_end_matches('/');
    let rest = rest.strip_suffix(".git").unwrap_or(rest);
    let mut parts = rest.split('/');
    let (owner, name) = (parts.next()?, parts.next()?);
    if owner.is_empty() || name.is_empty() || parts.next().is_some() {
        return None;
    }
    Some(format!("https://github.com/{owner}/{name}"))
}

/// A link to a file and lines of the pinned commit on GitHub. Characters that would end the path early are
/// percent-encoded. A start line of 0 means the claim names no line, so no anchor is added.
fn source_url(base: &str, sha: &str, path: &str, start: u64, end: u64) -> String {
    let path: String = path
        .chars()
        .map(|c| match c {
            '%' => "%25".to_string(),
            ' ' => "%20".to_string(),
            '#' => "%23".to_string(),
            '?' => "%3F".to_string(),
            other => other.to_string(),
        })
        .collect();
    let anchor = if start == 0 {
        String::new()
    } else if end <= start {
        format!("#L{start}")
    } else {
        format!("#L{start}-L{end}")
    };
    format!("{base}/blob/{sha}/{path}{anchor}")
}

/// A claim's text starts with a bracketed tag such as `[mechanism]`; this splits it from the sentence.
fn split_tag(text: &str) -> (Option<&str>, &str) {
    if let Some(rest) = text.strip_prefix('[')
        && let Some((tag, body)) = rest.split_once(']')
    {
        return (Some(tag), body.trim_start());
    }
    (None, text)
}

/// Renders the page. Every factual statement cites a claim id; the pinned commit is stated up front.
pub fn render(study: &Study) -> Result<String> {
    let known = claim_ids(&study.claims)?;
    let claims = array_of_value(&study.claims, "claims.json")?;
    let sha = str_of(&study.pin, "sha", "pin.json")?;
    let repo = str_of(&study.pin, "repository_url", "pin.json")?;
    let system = str_of(&study.pin, "system", "pin.json")?;
    let study_id = str_of(&study.pin, "study_id", "pin.json")?;

    let github = github_base(repo);
    let repo_link = match (&github, repo.starts_with("https://")) {
        (Some(base), _) => format!("<a href=\"{}\">{}</a>", escape(base), escape(repo)),
        (None, true) => format!("<a href=\"{0}\">{0}</a>", escape(repo)),
        (None, false) => format!("<code>{}</code>", escape(repo)),
    };
    let about = study
        .intro
        .get("about")
        .ok_or_else(|| anyhow!("intro.json has no `about`"))?;

    let mut body = String::new();
    let _ = write!(
        body,
        "<header><h1>Athena explains how {sys} handles work</h1>\
         <p class=\"lede\">{athena}</p>\
         <p class=\"lede\">{study_text}</p>\
         <p class=\"lede\">This page is that study of {sys}. Its study id, <code>{id}</code>, is the name of \
         the folder under <code>studies/</code> that holds the data this page is generated from. \
         Each statement below ends with the id of the claim that supports it; the claims, with links to \
         their sources, are listed at the end.</p>\
         <p class=\"pin\">Pinned {sys} commit: <code>{sha}</code> in {repo_link}</p></header>",
        sys = escape(system),
        athena = escape(str_of(about, "athena", "intro.json about")?),
        study_text = escape(str_of(about, "study", "intro.json about")?),
        id = escape(study_id),
        sha = escape(sha),
    );

    // Start here: what firstmate is and one piece of work end to end.
    body.push_str(
        "<section><h2>Start here</h2>\
         <p>The three parts below say what firstmate is, follow one piece of work, and say what happens \
         when something fails or has no limit. A glossary of the words used follows the main sections.</p>\
         <h3>Words used first</h3>",
    );
    leadin(&mut body, &study.intro, "terms", &known)?;
    body.push_str("<dl class=\"deciders\">");
    for t in array_of(&study.intro, "start_terms", "intro.json")? {
        let term = str_of(t, "term", "a start term")?;
        let what = format!("start term {term}");
        let _ = write!(
            body,
            "<dt>{}</dt><dd>{} {}</dd>",
            escape(term),
            escape(str_of(t, "text", &what)?),
            cites(t, &what, &known)?
        );
    }
    body.push_str("</dl><h3>What firstmate is</h3><ul>");
    for item in array_of(&study.intro, "what", "intro.json")? {
        intro_item(&mut body, item, "an intro sentence", &known)?;
    }
    body.push_str("</ul><h3>One piece of work, start to finish</h3><ol>");
    for item in array_of(&study.intro, "walkthrough", "intro.json")? {
        intro_item(&mut body, item, "a walk-through step", &known)?;
    }
    body.push_str("</ol><h3>When things go wrong, and what is not limited</h3><ul>");
    for item in array_of(&study.intro, "limits", "intro.json")? {
        intro_item(&mut body, item, "a failure or limit sentence", &known)?;
    }
    body.push_str("</ul></section>");

    // States
    body.push_str("<section><h2>States of a piece of work</h2>");
    leadin(&mut body, &study.intro, "states", &known)?;
    body.push_str("<ul>");
    for state in array_of(&study.model, "states", "model.json")? {
        let id = str_of(state, "id", "a state")?;
        let kind = str_of(state, "kind", &format!("state {id}"))?;
        let _ = write!(
            body,
            "<li><strong>{}</strong> <span class=\"tag\">{}</span> {}</li>",
            escape(id),
            escape(kind),
            cites(state, &format!("state {id}"), &known)?
        );
    }
    body.push_str("</ul></section>");

    // Counters
    let counters = array_of(&study.model, "counters", "model.json")?;
    if !counters.is_empty() {
        body.push_str("<section><h2>Counters</h2>");
        leadin(&mut body, &study.intro, "counters", &known)?;
        body.push_str("<ul>");
        for counter in counters {
            let id = str_of(counter, "id", "a counter")?;
            let changed: Vec<&str> = array_of(counter, "changed_by", &format!("counter {id}"))?
                .iter()
                .filter_map(Value::as_str)
                .collect();
            let _ = write!(
                body,
                "<li><strong>{}</strong>, changed by transitions {} {}</li>",
                escape(id),
                escape(&changed.join(", ")),
                cites(counter, &format!("counter {id}"), &known)?
            );
        }
        body.push_str("</ul></section>");
    }

    // Transitions
    body.push_str("<section><h2>Transitions</h2>");
    leadin(&mut body, &study.intro, "transitions", &known)?;
    body.push_str("<dl class=\"deciders\">");
    for d in array_of(&study.intro, "deciders", "intro.json")? {
        let term = str_of(d, "term", "a decider")?;
        let what = format!("decider {term}");
        let _ = write!(
            body,
            "<dt>{}</dt><dd>{} {}</dd>",
            escape(term),
            escape(str_of(d, "text", &what)?),
            cites(d, &what, &known)?
        );
    }
    body.push_str("</dl><div class=\"cards\">");
    let summaries = array_of(&study.intro, "summaries", "intro.json")?;
    for t in array_of(&study.model, "transitions", "model.json")? {
        let id = str_of(t, "id", "a transition")?;
        let what = format!("transition {id}");
        let summary = summaries
            .iter()
            .find(|s| s.get("id").and_then(Value::as_str) == Some(id))
            .ok_or_else(|| anyhow!("{what} has no summary in intro.json"))?;
        let summary_what = format!("summary of {what}");
        let _ = write!(
            body,
            "<article class=\"card\"><h3>{} <span class=\"arrow\">{} &rarr; {}</span></h3>\
             <p class=\"summary\">{} {}</p><dl>",
            escape(id),
            escape(str_of(t, "from", &what)?),
            escape(str_of(t, "to", &what)?),
            escape(str_of(summary, "text", &summary_what)?),
            cites(summary, &summary_what, &known)?
        );
        let _ = write!(
            body,
            "<dt>Trigger</dt><dd>{} (decided by {})</dd>",
            escape(str_of(t, "trigger", &what)?),
            escape(str_of(t, "trigger_evaluated_by", &what)?)
        );
        // A guard of "none" is a placeholder: nothing checks it, so no checker is named.
        let guard = str_of(t, "guard", &what)?;
        if guard == "none" {
            body.push_str("<dt>Guard</dt><dd>No guard applies</dd>");
        } else {
            let _ = write!(
                body,
                "<dt>Guard</dt><dd>{} (checked by {})</dd>",
                escape(guard),
                escape(str_of(t, "guard_evaluated_by", &what)?)
            );
        }
        if let Some(failure) = t.get("failure").and_then(Value::as_str) {
            let _ = write!(body, "<dt>Failure</dt><dd>{}</dd>", escape(failure));
        }
        let _ = write!(body, "</dl><p>{}</p></article>", cites(t, &what, &known)?);
    }
    body.push_str("</div></section>");

    // Rationales
    body.push_str("<section><h2>Why it is built this way</h2>");
    leadin(&mut body, &study.intro, "why", &known)?;
    body.push_str("<div class=\"cards\">");
    let titles = array_of(&study.intro, "rationale_titles", "intro.json")?;
    for r in array_of_value(&study.rationales, "rationales.json")? {
        let id = str_of(r, "id", "a rationale")?;
        let what = format!("rationale {id}");
        let title = titles
            .iter()
            .find(|t| t.get("id").and_then(Value::as_str) == Some(id))
            .ok_or_else(|| anyhow!("{what} has no title in intro.json"))?;
        let _ = write!(
            body,
            "<article class=\"card\"><h3>{} <span class=\"arrow\">{}</span></h3><p>{}</p>",
            escape(id),
            escape(str_of(title, "title", &format!("title of {what}"))?),
            escape(str_of(r, "decision", &what)?)
        );
        let alternatives = array_of(r, "alternatives", &what)?;
        if !alternatives.is_empty() {
            body.push_str("<p>Alternatives named:</p><ul>");
            for a in alternatives.iter().filter_map(Value::as_str) {
                let _ = write!(body, "<li>{}</li>", escape(a));
            }
            body.push_str("</ul>");
        }
        let _ = write!(
            body,
            "<p>Trade-off: {}</p><p>{}</p></article>",
            escape(str_of(r, "tradeoff", &what)?),
            cites(r, &what, &known)?
        );
    }
    body.push_str("</div></section>");

    // Glossary: after the walk-through and the sections that use its words.
    body.push_str("<section><h2>Glossary</h2><dl class=\"glossary\">");
    let mut entries: Vec<&Value> = array_of(&study.intro, "glossary", "intro.json")?
        .iter()
        .collect();
    entries.sort_by_cached_key(|e| e.get("term").and_then(Value::as_str).map(str::to_lowercase));
    for entry in entries {
        let term = str_of(entry, "term", "a glossary entry")?;
        let what = format!("glossary entry {term}");
        let definition = escape(str_of(entry, "definition", &what)?);
        // Terms about the page's own structure make no claim about firstmate and need no citation.
        let cite = if entry.get("about").and_then(Value::as_str) == Some("page") {
            String::from("<span class=\"tag\">about this page</span>")
        } else {
            cites(entry, &what, &known)?
        };
        let _ = write!(
            body,
            "<dt id=\"term-{}\">{}</dt><dd>{} {}</dd>",
            escape(&term.replace(' ', "-")),
            escape(term),
            definition,
            cite
        );
    }
    body.push_str("</dl></section>");

    // Findings: records of the study itself, not claims about the studied system.
    body.push_str(
        "<section><h2>Limits of this study</h2><p>These are the study's own findings, identified \
         by finding id; they say nothing about firstmate beyond what the claims above support.</p><ul>",
    );
    for f in array_of_value(&study.findings, "findings.json")? {
        let id = str_of(f, "id", "a finding")?;
        let _ = write!(
            body,
            "<li><strong>{}</strong> <span class=\"tag\">{}</span> {}</li>",
            escape(id),
            escape(str_of(f, "kind", &format!("finding {id}"))?),
            escape(str_of(f, "detail", &format!("finding {id}"))?)
        );
    }
    body.push_str("</ul></section>");

    // Next steps: where to read more about Athena and firstmate.
    let firstmate_url = github.clone().unwrap_or_else(|| repo.to_string());
    let firstmate_link = if firstmate_url.starts_with("https://") {
        format!("<a href=\"{0}\">{0}</a>", escape(&firstmate_url))
    } else {
        format!("<code>{}</code>", escape(&firstmate_url))
    };
    let _ = write!(
        body,
        "<section><h2>Next steps</h2><ul>\
         <li>Learn about Athena, and how to install and try it: <a href=\"{a}\">{a}</a>.</li>\
         <li>Learn about {sys}, the project studied here, and see how to use it: {firstmate_link}.</li>\
         <li>Check a statement: follow a bracketed claim id to the claim, then its source link to the \
         file and lines in {sys} at the pinned commit.</li></ul></section>",
        a = ATHENA_URL,
        sys = escape(system),
    );

    // Claims and their sources
    body.push_str("<section><h2>Claims and sources</h2><ol class=\"claims\">");
    // Claims are listed in numeric order of their id (c9 before c10), whatever the order in the file.
    let mut ordered: Vec<&Value> = claims.iter().collect();
    ordered.sort_by_cached_key(|c| claim_key(c.get("id").and_then(Value::as_str).unwrap_or("")));
    for c in ordered {
        let id = str_of(c, "id", "a claim")?;
        let what = format!("claim {id}");
        let (tag, sentence) = split_tag(str_of(c, "text", &what)?);
        let _ = write!(body, "<li id=\"claim-{}\">", escape(id));
        if let Some(tag) = tag {
            let _ = write!(body, "<span class=\"tag\">{}</span> ", escape(tag));
        }
        let _ = write!(
            body,
            "{} <span class=\"cites\">[{}]</span> <span class=\"mark\">{}</span><ul class=\"sources\">",
            escape(sentence),
            escape(id),
            escape(str_of(c, "mark", &what)?)
        );
        // A claim lists each source location once, even when several quotes come from the same lines.
        let mut seen = BTreeSet::new();
        for s in array_of(c, "sources", &what)? {
            let path = str_of(s, "path", &what)?;
            let start = s.get("start_line").and_then(Value::as_u64).unwrap_or(0);
            let end = s.get("end_line").and_then(Value::as_u64).unwrap_or(start);
            let text = format!("{path}:{start}-{end}");
            if !seen.insert(text.clone()) {
                continue;
            }
            match &github {
                Some(base) => {
                    let _ = write!(
                        body,
                        "<li><a href=\"{}\"><code>{}</code></a></li>",
                        escape(&source_url(base, sha, path, start, end)),
                        escape(&text)
                    );
                }
                None => {
                    let _ = write!(body, "<li><code>{}</code></li>", escape(&text));
                }
            }
        }
        body.push_str("</ul></li>");
    }
    body.push_str("</ol></section>");

    let _ = write!(
        body,
        "<footer>Generated from <code>{}</code>: {} claims at commit <code>{}</code>.</footer>",
        escape(STUDY_DIR),
        known.len(),
        escape(sha)
    );

    Ok(format!(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>Athena explains how {sys} handles work</title><style>{CSS}</style></head><body><main>{body}</main></body></html>\n",
        sys = escape(system)
    ))
}

/// Sort key for a claim id: its number when it is `c<number>`, so `c9` sorts before `c10`; other ids follow, by text.
fn claim_key(id: &str) -> (u64, String) {
    match id.strip_prefix('c').and_then(|n| n.parse::<u64>().ok()) {
        Some(n) => (n, String::new()),
        None => (u64::MAX, id.to_string()),
    }
}

/// A plain-language lead-in under a section heading, from `leadins.<key>` in `intro.json`.
fn leadin(body: &mut String, intro: &Value, key: &str, known: &BTreeSet<String>) -> Result<()> {
    let what = format!("the {key} lead-in");
    let item = intro
        .get("leadins")
        .and_then(|l| l.get(key))
        .ok_or_else(|| anyhow!("intro.json has no lead-in `{key}`"))?;
    let text = escape(str_of(item, "text", &what)?);
    let _ = write!(
        body,
        "<p class=\"leadin\">{} {}</p>",
        text,
        cites(item, &what, known)?
    );
    Ok(())
}

/// One sentence of the intro as a list item. It must cite at least one claim.
fn intro_item(body: &mut String, item: &Value, what: &str, known: &BTreeSet<String>) -> Result<()> {
    let text = escape(str_of(item, "text", what)?);
    let _ = write!(body, "<li>{} {}</li>", text, cites(item, what, known)?);
    Ok(())
}

fn array_of_value<'a>(value: &'a Value, file: &str) -> Result<&'a Vec<Value>> {
    value
        .as_array()
        .ok_or_else(|| anyhow!("{file} is not a list"))
}

/// Builds the site: reads the study under `root` and writes `index.html` into `out`.
pub fn build(root: &Path, out: &Path) -> Result<()> {
    let html = render(&load(&root.join(STUDY_DIR))?)?;
    std::fs::create_dir_all(out).with_context(|| format!("cannot create {}", out.display()))?;
    let index = out.join("index.html");
    std::fs::write(&index, html).with_context(|| format!("cannot write {}", index.display()))?;
    println!("wrote {}", index.display());
    Ok(())
}

const CSS: &str = "\
:root{color-scheme:light dark;--bg:#fff;--fg:#1b1f24;--muted:#59636e;--card:#f6f8fa;--line:#d0d7de;--accent:#0b5cad}\
@media (prefers-color-scheme:dark){:root{--bg:#0d1117;--fg:#e6edf3;--muted:#9198a1;--card:#161b22;--line:#30363d;--accent:#58a6ff}}\
*{box-sizing:border-box}\
body{margin:0;background:var(--bg);color:var(--fg);font:16px/1.55 system-ui,-apple-system,Segoe UI,sans-serif}\
main{max-width:64rem;margin:0 auto;padding:1rem}\
h1{font-size:1.9rem;line-height:1.2}h2{margin-top:2.2rem;border-bottom:1px solid var(--line);padding-bottom:.3rem}\
a{color:var(--accent)}code{font-family:ui-monospace,monospace;font-size:.9em;word-break:break-all}\
.lede,.pin,footer{color:var(--muted)}\
.cards{display:grid;grid-template-columns:repeat(auto-fit,minmax(18rem,1fr));gap:1rem}\
.card{background:var(--card);border:1px solid var(--line);border-radius:.5rem;padding:.8rem 1rem}\
.card h3{margin:.2rem 0}.arrow{color:var(--muted);font-weight:normal}\
dt{font-weight:600}dd{margin:0 0 .5rem}\
.tag{display:inline-block;border:1px solid var(--line);border-radius:1rem;padding:0 .5rem;font-size:.8rem;color:var(--muted)}\
.cites a,.cites{font-size:.75rem;color:var(--muted)}.mark{color:var(--muted);font-size:.8rem}\
.leadin{font-size:1.05rem}.summary{font-weight:600;margin:.3rem 0}\
.deciders{margin:1rem 0}\
.sources{color:var(--muted);font-size:.85rem}\
@media (max-width:40rem){h1{font-size:1.5rem}main{padding:.6rem}}";

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, name: &str, json: &str) {
        std::fs::write(dir.join(name), json).unwrap();
    }

    /// A small study. `claim_text` is the text of claim c1; `cited` the claim a transition cites.
    fn study_dir(claim_text: &str, cited: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "pin.json",
            r#"{"study_id":"s","system":"firstmate","repository_url":"https://example.org/f.git","sha":"abc123def"}"#,
        );
        let claims = serde_json::json!([{
            "id": "c1", "mark": "read", "text": claim_text,
            "sources": [{"path": "a.md", "start_line": 3, "end_line": 4}]
        }]);
        write(dir.path(), "claims.json", &claims.to_string());
        let model = serde_json::json!({
            "states": [{"id": "queued", "kind": "initial", "claims": ["c1"]}],
            "counters": [{"id": "places", "start": 0, "changed_by": ["t1"], "claims": ["c1"]}],
            "transitions": [{
                "id": "t1", "from": "queued", "to": "queued",
                "trigger": "x", "trigger_evaluated_by": "agent",
                "guard": "y", "guard_evaluated_by": "code",
                "failure": null, "claims": [cited]
            }]
        });
        write(dir.path(), "model.json", &model.to_string());
        write(
            dir.path(),
            "rationales.json",
            r#"[{"id":"ra1","decision":"d","alternatives":["alt"],"tradeoff":"t","claims":["c1"]}]"#,
        );
        write(
            dir.path(),
            "findings.json",
            r#"[{"id":"f1","kind":"model-limit","detail":"limit"}]"#,
        );
        write(
            dir.path(),
            "intro.json",
            r#"{"about":{"athena":"Athena is a system.","study":"A study is a reading."},
                "what":[{"text":"Firstmate does a thing.","claims":["c1"]}],
                "walkthrough":[{"text":"First step.","claims":["c1"]}],
                "limits":[{"text":"A limit.","claims":["c1"]}],
                "start_terms":[{"term":"captain","text":"The person.","claims":["c1"]}],
                "rationale_titles":[{"id":"ra1","title":"A short title"}],
                "leadins":{
                  "terms":{"text":"Terms lead.","claims":["c1"]},
                  "states":{"text":"States lead.","claims":["c1"]},
                  "counters":{"text":"Counters lead.","claims":["c1"]},
                  "transitions":{"text":"Transitions lead.","claims":["c1"]},
                  "why":{"text":"Why lead.","claims":["c1"]}},
                "deciders":[
                  {"term":"agent","text":"Agent decides.","claims":["c1"]},
                  {"term":"code","text":"Code decides.","claims":["c1"]}],
                "summaries":[{"id":"t1","text":"Queued stays queued.","claims":["c1"]}],
                "glossary":[
                  {"term":"worker","definition":"Does the task.","claims":["c1"]},
                  {"term":"guard","about":"page","definition":"A condition."}]}"#,
        );
        dir
    }

    #[test]
    fn the_intro_comes_before_the_states_and_the_glossary_after_them() {
        let dir = study_dir("[mechanism] A thing happens.", "c1");
        let html = render(&load(dir.path()).unwrap()).unwrap();
        let at = |needle: &str| html.find(needle).unwrap_or_else(|| panic!("{needle}"));
        let start = at("<h2>Start here</h2>");
        assert!(start < at("<h2>States of a piece of work</h2>"));
        assert!(
            at("<h3>One piece of work, start to finish</h3>")
                < at("<h2>States of a piece of work</h2>")
        );
        assert!(at("<h2>Why it is built this way</h2>") < at("<h2>Glossary</h2>"));
        assert!(at("<h2>Glossary</h2>") < at("<h2>Next steps</h2>"));
        assert!(html.contains("<h3>What firstmate is</h3>"));
        assert!(html.contains("<h3>One piece of work, start to finish</h3>"));
        assert!(html.contains("<li>Firstmate does a thing. <span class=\"cites\">"));
        assert!(html.contains(
            "<dt id=\"term-worker\">worker</dt><dd>Does the task. <span class=\"cites\">"
        ));
        assert!(html.contains("<dt id=\"term-guard\">guard</dt><dd>A condition. <span class=\"tag\">about this page</span>"));
    }

    #[test]
    fn the_page_names_athena_before_firstmate_and_has_next_steps() {
        let dir = study_dir("[mechanism] A thing happens.", "c1");
        let html = render(&load(dir.path()).unwrap()).unwrap();
        assert!(html.contains("<title>Athena explains how firstmate handles work</title>"));
        let h1 = html.find("<h1>Athena explains").unwrap();
        let athena = html.find("Athena is a system.").unwrap();
        let study = html.find("A study is a reading.").unwrap();
        let first = html.find("Firstmate does a thing.").unwrap();
        assert!(h1 < athena && athena < study && study < first);
        assert!(html.contains("<h2>Next steps</h2>"));
        assert!(html.contains("<a href=\"https://github.com/doodla/athena\">"));
    }

    #[test]
    fn a_github_pin_gives_linked_repository_and_sources_and_others_do_not() {
        let dir = study_dir("[mechanism] A thing happens.", "c1");
        let plain = render(&load(dir.path()).unwrap()).unwrap();
        assert!(!plain.contains("/blob/"));
        assert!(plain.contains("<code>a.md:3-4</code>"));
        write(
            dir.path(),
            "pin.json",
            r#"{"study_id":"s","system":"firstmate","repository_url":"https://github.com/o/f.git","sha":"abc123def"}"#,
        );
        let html = render(&load(dir.path()).unwrap()).unwrap();
        assert!(
            html.contains("in <a href=\"https://github.com/o/f\">https://github.com/o/f.git</a>")
        );
        assert!(html.contains(
            "<a href=\"https://github.com/o/f/blob/abc123def/a.md#L3-L4\"><code>a.md:3-4</code></a>"
        ));
        assert!(html.contains("<li>Learn about firstmate, the project studied here, and see how to use it: <a href=\"https://github.com/o/f\">"));
        assert_eq!(
            source_url("https://github.com/o/f", "s", "a b#c.md", 5, 5),
            "https://github.com/o/f/blob/s/a%20b%23c.md#L5"
        );
        assert_eq!(github_base("https://example.org/o/f.git"), None);
        assert_eq!(github_base("https://github.com/o"), None);
    }

    #[test]
    fn an_uncited_intro_element_is_an_error() {
        for (path, bad) in [
            ("/what/0/claims", serde_json::json!([])),
            ("/walkthrough/0/claims", serde_json::json!(["c99"])),
            ("/limits/0/claims", serde_json::json!([])),
            ("/glossary/0/claims", serde_json::json!([])),
            ("/start_terms/0/claims", serde_json::json!([])),
            ("/leadins/terms/claims", serde_json::json!([])),
            ("/leadins/states/claims", serde_json::json!([])),
            ("/leadins/counters/claims", serde_json::json!(["c99"])),
            ("/leadins/transitions/claims", serde_json::json!([])),
            ("/leadins/why/claims", serde_json::json!([])),
            ("/deciders/0/claims", serde_json::json!([])),
            ("/summaries/0/claims", serde_json::json!([])),
        ] {
            let dir = study_dir("[mechanism] A thing happens.", "c1");
            let mut intro = read_intro(dir.path());
            *intro.pointer_mut(path).unwrap() = bad;
            write(dir.path(), "intro.json", &intro.to_string());
            assert!(render(&load(dir.path()).unwrap()).is_err(), "{path}");
        }
    }

    fn read_intro(dir: &Path) -> Value {
        serde_json::from_str(&std::fs::read_to_string(dir.join("intro.json")).unwrap()).unwrap()
    }

    #[test]
    fn each_section_has_a_lead_in_and_each_transition_a_summary() {
        let dir = study_dir("[mechanism] A thing happens.", "c1");
        let html = render(&load(dir.path()).unwrap()).unwrap();
        for (heading, lead) in [
            ("States of a piece of work", "States lead."),
            ("Counters", "Counters lead."),
            ("Transitions", "Transitions lead."),
            ("Why it is built this way", "Why lead."),
        ] {
            let at = html
                .find(&format!("<h2>{heading}</h2>"))
                .unwrap_or_else(|| panic!("{heading}"));
            let rest = &html[at..];
            assert!(rest.contains(&format!(
                "<p class=\"leadin\">{lead} <span class=\"cites\">"
            )));
        }
        assert!(html.contains("<dt>agent</dt><dd>Agent decides. <span class=\"cites\">"));
        assert!(html.contains("<dt>code</dt><dd>Code decides. <span class=\"cites\">"));
        assert!(html.contains("<p class=\"summary\">Queued stays queued. <span class=\"cites\">"));
    }

    #[test]
    fn a_transition_without_a_summary_is_an_error() {
        let dir = study_dir("[mechanism] A thing happens.", "c1");
        let mut intro = read_intro(dir.path());
        intro["summaries"] = serde_json::json!([]);
        write(dir.path(), "intro.json", &intro.to_string());
        let err = render(&load(dir.path()).unwrap()).unwrap_err();
        assert!(format!("{err:#}").contains("t1"));
    }

    #[test]
    fn the_real_study_gives_every_transition_a_summary_and_both_deciders() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let html = render(&load(&root.join(STUDY_DIR)).unwrap()).unwrap();
        assert_eq!(html.matches("<p class=\"summary\">").count(), 11);
        assert_eq!(html.matches("<p class=\"leadin\">").count(), 5);
        assert!(html.contains("<dt>agent</dt>") && html.contains("<dt>code</dt>"));
    }

    #[test]
    fn the_real_study_renders_with_an_intro_and_glossary() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let html = render(&load(&root.join(STUDY_DIR)).unwrap()).unwrap();
        for term in [
            "state",
            "counter",
            "transition",
            "trigger",
            "guard",
            "claim",
            "pin",
        ] {
            assert!(html.contains(&format!("<dt id=\"term-{term}\">")), "{term}");
        }
    }

    /// The ids of the claims listed at the end of the page, in page order.
    fn listed_claim_ids(html: &str) -> Vec<String> {
        html.split("<li id=\"claim-")
            .skip(1)
            .map(|s| s.split('"').next().unwrap().to_string())
            .collect()
    }

    /// The glossary terms on the page, in page order.
    fn glossary_terms(html: &str) -> Vec<String> {
        html.split("<dt id=\"term-")
            .skip(1)
            .map(|s| {
                let s = s.split_once('>').unwrap().1;
                s.split("</dt>").next().unwrap().to_string()
            })
            .collect()
    }

    #[test]
    fn the_glossary_is_sorted_alphabetically_whatever_the_data_order() {
        let dir = study_dir("[mechanism] A thing happens.", "c1");
        let html = render(&load(dir.path()).unwrap()).unwrap();
        assert_eq!(glossary_terms(&html), ["guard", "worker"]);
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let real = render(&load(&root.join(STUDY_DIR)).unwrap()).unwrap();
        let terms = glossary_terms(&real);
        let mut sorted = terms.clone();
        sorted.sort_by_key(|t| t.to_lowercase());
        assert_eq!(terms, sorted);
        for term in ["green", "red", "home"] {
            assert!(terms.iter().any(|t| t == term), "{term}");
        }
    }

    #[test]
    fn claims_are_listed_in_numeric_order_and_no_claim_repeats_a_source() {
        let dir = study_dir("[mechanism] A thing happens.", "c1");
        let src = |l: u64| serde_json::json!({"path": "a.md", "start_line": l, "end_line": l});
        let claim = |id: &str, sources: Vec<Value>| serde_json::json!({"id": id, "mark": "read", "text": "[mechanism] X.", "sources": sources});
        let claims = serde_json::json!([
            claim("c10", vec![]),
            claim("c1", vec![src(3), src(3), src(4)]),
            claim("c9", vec![])
        ]);
        write(dir.path(), "claims.json", &claims.to_string());
        let html = render(&load(dir.path()).unwrap()).unwrap();
        assert_eq!(listed_claim_ids(&html), ["c1", "c9", "c10"]);
        assert_eq!(html.matches("<code>a.md:3-3</code>").count(), 1);
        assert_eq!(html.matches("<code>a.md:4-4</code>").count(), 1);
    }

    #[test]
    fn the_real_study_orders_claims_and_repeats_no_source() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let html = render(&load(&root.join(STUDY_DIR)).unwrap()).unwrap();
        let ids = listed_claim_ids(&html);
        let numbers: Vec<u64> = ids.iter().map(|i| i[1..].parse().unwrap()).collect();
        assert!(numbers.windows(2).all(|w| w[0] < w[1]), "{ids:?}");
        let claims = html.split("<li id=\"claim-").skip(1);
        for claim in claims {
            let claim = claim.split("</ol>").next().unwrap();
            let mut seen = BTreeSet::new();
            for code in claim.split("<code>").skip(1) {
                let code = code.split("</code>").next().unwrap();
                assert!(seen.insert(code.to_string()), "repeated source {code}");
            }
        }
    }

    #[test]
    fn the_real_page_titles_each_design_choice_and_defines_the_start_here_terms() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let html = render(&load(&root.join(STUDY_DIR)).unwrap()).unwrap();
        for id in ["ra1", "ra2", "ra3", "ra4", "ra5", "ra6"] {
            let at = html
                .find(&format!("<h3>{id} <span class=\"arrow\">"))
                .unwrap();
            assert!(!html[at..].starts_with(&format!("<h3>{id} <span class=\"arrow\"></span>")));
        }
        let start = html.find("<h2>Start here</h2>").unwrap();
        let first_use = html.find("<h3>What firstmate is</h3>").unwrap();
        for term in ["captain", "yolo", "green", "red", "local-only landing"] {
            let at = html[start..first_use]
                .find(&format!("<dt>{term}</dt>"))
                .unwrap_or_else(|| panic!("{term}"));
            assert!(at > 0);
        }
        assert!(html.contains("is the name of the folder under <code>studies/</code>"));
    }

    #[test]
    fn a_rationale_without_a_title_is_an_error() {
        let dir = study_dir("[mechanism] A thing happens.", "c1");
        let mut intro = read_intro(dir.path());
        intro["rationale_titles"] = serde_json::json!([]);
        write(dir.path(), "intro.json", &intro.to_string());
        let err = render(&load(dir.path()).unwrap()).unwrap_err();
        assert!(format!("{err:#}").contains("ra1"));
    }

    #[test]
    fn a_placeholder_guard_is_shown_as_no_guard_and_a_real_guard_names_its_checker() {
        let dir = study_dir("[mechanism] A thing happens.", "c1");
        let html = render(&load(dir.path()).unwrap()).unwrap();
        assert!(html.contains("<dt>Guard</dt><dd>y (checked by code)</dd>"));
        let mut model: Value =
            serde_json::from_str(&std::fs::read_to_string(dir.path().join("model.json")).unwrap())
                .unwrap();
        model["transitions"][0]["guard"] = serde_json::json!("none");
        model["transitions"][0]["guard_evaluated_by"] = serde_json::json!("agent");
        write(dir.path(), "model.json", &model.to_string());
        let html = render(&load(dir.path()).unwrap()).unwrap();
        assert!(html.contains("<dt>Guard</dt><dd>No guard applies</dd>"));
        assert!(!html.contains("checked by agent"));
    }

    #[test]
    fn the_real_study_shows_failures_and_limits_and_no_placeholder_guard() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let html = render(&load(&root.join(STUDY_DIR)).unwrap()).unwrap();
        assert!(html.contains("<h3>When things go wrong, and what is not limited</h3>"));
        assert!(!html.contains("guard_evaluated_by"));
        assert_eq!(html.matches("<dd>No guard applies</dd>").count(), 3);
        assert!(!html.contains("none (checked by"));
    }

    #[test]
    fn text_is_html_escaped() {
        assert_eq!(
            escape("<a href=\"x\">&'</a>"),
            "&lt;a href=&quot;x&quot;&gt;&amp;&#39;&lt;/a&gt;"
        );
        let dir = study_dir("[mechanism] <script>alert(1)</script> & more", "c1");
        let html = render(&load(dir.path()).unwrap()).unwrap();
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt; &amp; more"));
    }

    #[test]
    fn the_page_cites_claim_ids_and_the_pinned_commit() {
        let dir = study_dir("[mechanism] A thing happens.", "c1");
        let html = render(&load(dir.path()).unwrap()).unwrap();
        assert!(html.contains("href=\"#claim-c1\">[c1]</a>"));
        assert!(html.contains("id=\"claim-c1\""));
        assert!(html.contains("Pinned firstmate commit: <code>abc123def</code>"));
        assert!(html.contains("name=\"viewport\""));
        assert!(html.contains("prefers-color-scheme:dark"));
        assert!(!html.contains("<link") && !html.contains("<script"));
    }

    #[test]
    fn a_citation_of_an_unknown_claim_is_an_error() {
        let dir = study_dir("[mechanism] A thing happens.", "c99");
        let err = render(&load(dir.path()).unwrap()).unwrap_err();
        assert!(format!("{err:#}").contains("c99"));
    }

    #[test]
    fn missing_study_data_is_an_error_naming_the_file() {
        let dir = study_dir("[mechanism] A thing happens.", "c1");
        std::fs::remove_file(dir.path().join("claims.json")).unwrap();
        let err = load(dir.path()).unwrap_err();
        assert!(format!("{err:#}").contains("claims.json"));
        let empty = tempfile::tempdir().unwrap();
        assert!(build(empty.path(), &empty.path().join("out")).is_err());
        assert!(!empty.path().join("out/index.html").exists());
    }

    #[test]
    fn the_default_output_is_in_the_target_directory_not_the_work_tree() {
        let root = tempfile::tempdir().unwrap();
        let cwd = root.path();
        let out = default_out(root.path(), None, cwd);
        assert_eq!(out, root.path().join("target/site"));
        check_out(root.path(), &out, &target_dir(root.path(), None, cwd), cwd).unwrap();
        // With CARGO_TARGET_DIR set outside the tree, the page goes there.
        let elsewhere = tempfile::tempdir().unwrap();
        let env = Some(elsewhere.path().as_os_str().to_owned());
        let out = default_out(root.path(), env.clone(), cwd);
        assert_eq!(out, elsewhere.path().join("site"));
        assert!(!out.starts_with(root.path()));
        check_out(root.path(), &out, &target_dir(root.path(), env, cwd), cwd).unwrap();
        // An empty value counts as unset.
        assert_eq!(
            default_out(root.path(), Some("".into()), cwd),
            root.path().join("target/site")
        );
    }

    #[test]
    fn an_output_inside_the_repository_root_is_refused_with_a_clear_error() {
        let root = tempfile::tempdir().unwrap();
        let cwd = root.path();
        let target = target_dir(root.path(), None, cwd);
        for bad in ["site", "docs/out", "./site/../x", "target/../site", "."] {
            let err = check_out(root.path(), Path::new(bad), &target, cwd).unwrap_err();
            let text = format!("{err:#}");
            assert!(
                text.contains("inside the repository work tree"),
                "{bad}: {text}"
            );
        }
        // The target directory and places outside the tree are fine.
        for good in ["target/site", "target/x/y"] {
            check_out(root.path(), Path::new(good), &target, cwd).unwrap();
        }
        let outside = tempfile::tempdir().unwrap();
        check_out(root.path(), &outside.path().join("site"), &target, cwd).unwrap();
    }

    #[test]
    fn build_writes_index_html_at_the_output_root() {
        let root = tempfile::tempdir().unwrap();
        let study = root.path().join(STUDY_DIR);
        std::fs::create_dir_all(&study).unwrap();
        let src = study_dir("[mechanism] A thing happens.", "c1");
        for name in [
            "pin.json",
            "claims.json",
            "model.json",
            "rationales.json",
            "findings.json",
            "intro.json",
        ] {
            std::fs::copy(src.path().join(name), study.join(name)).unwrap();
        }
        let out = root.path().join("site");
        build(root.path(), &out).unwrap();
        assert!(
            std::fs::read_to_string(out.join("index.html"))
                .unwrap()
                .starts_with("<!doctype html>")
        );
    }
}
