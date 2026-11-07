//! A scan that keeps in-process executable writes out of the workspace's test code.
//!
//! A test that makes a file executable from inside its own process holds a write handle for a moment, and a
//! fork in another test thread can copy that handle, so the first test's run of the file fails with `ETXTBSY`.
//! Stand-ins must be written with `athena_testkit::write_executable`, which does the write in a child.
//!
//! [`violations_with`] is one pure function over a path, its text and the shared wrappers of the workspace.
//! The tree scan feeds it every Rust file under `crates/*/{src,tests}` and `xtask/{src,tests}`; the
//! planted-violation tests feed it strings. [`violations`] is the same with no shared wrappers.
//!
//! What counts as test code: every file under a `tests/` directory; files named `tests.rs`, `*_tests.rs` or
//! `test_support.rs`; and, in other files under `src/`, the text from the first `#[cfg(test)]` or
//! `#[cfg(all(test` line to the end (the convention `xtask/tests/policy.rs` uses). Production code above that
//! line may set permissions freely.
//!
//! What a flagged line is, in test code:
//! - a direct call that sets a mode: `set_permissions(`, `from_mode(`, `.set_mode(`, `.mode(<arg>)`,
//!   `set_executable(`, or `Command::new("chmod")` ("chmod child");
//! - an external `chmod` built another way: a line with the word `chmod` and a command-building marker
//!   (`Command::new(`, `.arg(`, `.args(`, `"sh"`, `"bash"`, `"-c"`), or a `"chmod ..."` line right after a line
//!   with a marker ("chmod argument");
//! - a call to a wrapper: a function whose body both sets a mode and sets an execute bit (an octal literal
//!   with an odd digit not preceded by `!`, a `+x` string, or a chmod argument line). A wrapper counts when it
//!   is called in the same file, or when it is `pub` or `pub(crate)` and called in another file of the same
//!   crate ("wrapper call").
//!
//! Lines inside a `write_executable(` call are skipped, because the script text may mention `chmod`. The span
//! ends where the call's parentheses close, counted outside string, raw string and char literals and
//! comments; a call that does not close within 40 lines is skipped on its first line only, so a runaway span
//! cannot hide later violations.
//!
//! This is a text scan, not a parser; decision D163 lists what it cannot see: a wrapper that takes the mode as
//! a parameter, a name shared by two functions of one crate, and a script held in a variable.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Files that hold a flagged line on purpose, each with the reason. The guard test fails when an entry no
/// longer exists or no longer has a hit, so the list cannot go stale.
const EXEMPT: &[(&str, &str)] = &[
    (
        "crates/athena-testkit/tests/no_in_process_exec_writes.rs",
        "holds the planted violations as string literals",
    ),
    (
        "crates/athena-testkit/tests/stand_in_stress.rs",
        "its in-process variant writes the bad way on purpose, as the control",
    ),
];

/// How many lines past its opening line a `write_executable(` call may run before the scan stops trusting it.
const MAX_SPAN_LINES: usize = 40;

/// A reason is reported for each flagged line.
type Hit = (usize, &'static str);

/// Names of the shared wrappers of each crate, keyed by crate id (see [`crate_id`]).
type Shared = BTreeMap<String, BTreeSet<String>>;

fn is_test_file(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    rel.starts_with("tests/")
        || rel.contains("/tests/")
        || name == "tests.rs"
        || name == "test_support.rs"
        || name.ends_with("_tests.rs")
}

/// The 0-based index of the first line that counts as test code, or `None` when none does.
fn test_region_start(rel: &str, text: &str) -> Option<usize> {
    if is_test_file(rel) {
        return Some(0);
    }
    if !(rel.starts_with("src/") || rel.contains("/src/")) {
        return None;
    }
    text.lines().position(|l| {
        let l = l.trim_start();
        l.starts_with("#[cfg(test)]") || l.starts_with("#[cfg(all(test")
    })
}

/// A `.mode(` call that takes an argument (it sets a mode); `.mode()` with none only reads one.
fn sets_mode(line: &str) -> bool {
    let mut rest = line;
    while let Some(i) = rest.find(".mode(") {
        rest = &rest[i + ".mode(".len()..];
        if !rest.starts_with(')') {
            return true;
        }
    }
    false
}

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// The end (exclusive) of a plain or byte string literal that opens at `i`.
fn plain_string_end(b: &[u8], i: usize) -> usize {
    let mut j = i + 1;
    while j < b.len() {
        match b[j] {
            b'\\' => j += 2,
            b'"' => return j + 1,
            _ => j += 1,
        }
    }
    b.len()
}

/// The end (exclusive) of a raw string literal (`r"..."`, `r#"..."#`, `br#"..."#`) whose `r` is at `i`, or
/// `None` when the `r` does not open one.
fn raw_string_end(b: &[u8], i: usize) -> Option<usize> {
    let boundary = i == 0
        || !is_ident(b[i - 1])
        || (matches!(b[i - 1], b'b' | b'c') && (i == 1 || !is_ident(b[i - 2])));
    if !boundary {
        return None;
    }
    let mut j = i + 1;
    let mut hashes = 0;
    while b.get(j) == Some(&b'#') {
        hashes += 1;
        j += 1;
    }
    if b.get(j) != Some(&b'"') {
        return None;
    }
    for k in j + 1..b.len() {
        if b[k] == b'"'
            && b.get(k + 1..k + 1 + hashes)
                .is_some_and(|s| s.iter().all(|&c| c == b'#'))
        {
            return Some(k + 1 + hashes);
        }
    }
    Some(b.len())
}

/// The end (exclusive) of a char literal whose quote is at `i`, or `None` when the quote opens a lifetime.
fn char_literal_end(text: &str, i: usize) -> Option<usize> {
    let b = text.as_bytes();
    if *b.get(i + 1)? == b'\\' {
        let mut j = i + 3;
        while j < b.len() && b[j] != b'\'' {
            j += 1;
        }
        return Some((j + 1).min(b.len()));
    }
    let c = text[i + 1..].chars().next()?;
    let close = i + 1 + c.len_utf8();
    (c != '\'' && b.get(close) == Some(&b'\'')).then_some(close + 1)
}

/// For each byte of `text`, whether it is code: outside string, raw string, byte string and char literals
/// and comments. Brace and parenthesis counting uses it so a `'{'` or a `$(` inside a script does not count.
fn code_mask(text: &str) -> Vec<bool> {
    let b = text.as_bytes();
    let mut code = vec![true; b.len()];
    let mut i = 0;
    while i < b.len() {
        let end = if b[i] == b'/' && b.get(i + 1) == Some(&b'/') {
            text[i..].find('\n').map_or(b.len(), |n| i + n)
        } else if b[i] == b'/' && b.get(i + 1) == Some(&b'*') {
            text[i + 2..].find("*/").map_or(b.len(), |n| i + n + 4)
        } else if b[i] == b'"' {
            plain_string_end(b, i)
        } else if b[i] == b'r' {
            raw_string_end(b, i).unwrap_or(i)
        } else if b[i] == b'\'' {
            char_literal_end(text, i).unwrap_or(i)
        } else {
            i
        };
        if end > i {
            code[i..end.min(b.len())].fill(false);
            i = end;
        } else {
            i += 1;
        }
    }
    code
}

/// The byte offset at which each line of `text` starts.
fn line_starts(text: &str) -> Vec<usize> {
    let mut at = 0;
    text.split_inclusive('\n')
        .map(|piece| {
            let start = at;
            at += piece.len();
            start
        })
        .collect()
}

/// The 0-based index of the line that holds byte `pos`.
fn line_of(starts: &[usize], pos: usize) -> usize {
    starts.partition_point(|&s| s <= pos).saturating_sub(1)
}

/// For each line, whether it lies inside a `write_executable(` call. The span runs from the opening line to
/// the line where the call's parentheses close, counted in code only (see [`code_mask`]). A call that does
/// not close within [`MAX_SPAN_LINES`] lines marks only its opening line.
fn helper_spans(text: &str, code: &[bool], starts: &[usize]) -> Vec<bool> {
    const CALL: &str = "write_executable(";
    let b = text.as_bytes();
    let mut spans = vec![false; starts.len()];
    for (at, _) in text.match_indices(CALL) {
        if !code[at] || (at > 0 && is_ident(b[at - 1])) {
            continue;
        }
        let open_line = line_of(starts, at);
        let mut line = open_line;
        let mut depth = 0i32;
        let mut close_line = None;
        for (k, &c) in b.iter().enumerate().skip(at + CALL.len() - 1) {
            if c == b'\n' {
                line += 1;
                if line - open_line > MAX_SPAN_LINES {
                    break;
                }
            }
            if !code[k] {
                continue;
            }
            match c {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        close_line = Some(line);
                        break;
                    }
                }
                _ => {}
            }
        }
        for mark in &mut spans[open_line..=close_line.unwrap_or(open_line)] {
            *mark = true;
        }
    }
    spans
}

/// The reason a single line sets a mode from inside the process, if it does.
fn primitive(line: &str) -> Option<&'static str> {
    if line.contains("set_executable(") {
        Some("set_executable wrapper")
    } else if line.contains("set_permissions(") {
        Some("set_permissions")
    } else if line.contains("from_mode(") {
        Some("Permissions::from_mode")
    } else if line.contains(".set_mode(") {
        Some("set_mode")
    } else if sets_mode(line) {
        Some("OpenOptions mode")
    } else if line.contains("Command::new(") && line.contains("chmod\"") {
        Some("chmod child")
    } else {
        None
    }
}

/// Text that marks a line as building a command: running a program, a shell, or a shell's `-c` flag.
const MARKERS: [&str; 6] = [
    "Command::new(",
    ".arg(",
    ".args(",
    "\"sh\"",
    "\"bash\"",
    "\"-c\"",
];

fn has_marker(line: &str) -> bool {
    MARKERS.iter().any(|m| line.contains(m))
}

/// Whether `line` hands `chmod` to a program: the word and a marker on the line, or a `"chmod..."` line
/// right after a line with a marker (a split `Command::new(` or `.args([`). `prev` is the line before.
fn chmod_arg(line: &str, prev: &str) -> bool {
    line.contains("chmod")
        && (has_marker(line) || (line.trim_start().starts_with("\"chmod") && has_marker(prev)))
}

/// Positions in `body` where `pat` starts in code.
fn code_hits<'a>(
    body: &'a str,
    mask: &'a [bool],
    pat: &'a str,
) -> impl Iterator<Item = usize> + 'a {
    body.match_indices(pat)
        .map(|(i, _)| i)
        .filter(move |&i| mask[i])
}

/// Whether the code of `body` holds an octal literal with an execute bit (an odd digit) that no `!` negates.
fn exec_literal(body: &str, mask: &[bool]) -> bool {
    let b = body.as_bytes();
    code_hits(body, mask, "0o").any(|i| {
        let before = i.checked_sub(1).map(|p| b[p]);
        before.is_none_or(|p| !is_ident(p) && p != b'!')
            && b[i + 2..]
                .iter()
                .take_while(|c| matches!(c, b'0'..=b'7' | b'_'))
                .any(|c| matches!(c, b'1' | b'3' | b'5' | b'7'))
    })
}

/// Whether a function body (with its code mask) is a stand-in wrapper: it sets a mode and it sets an
/// execute bit. Requiring the execute bit keeps a private-file mode (`0o600`) or a write-bit toggle
/// (`| 0o200`, `& !0o222`) from counting.
fn is_wrapper_body(body: &str, mask: &[bool]) -> bool {
    let sets = ["set_permissions(", "from_mode(", ".set_mode("]
        .iter()
        .any(|p| code_hits(body, mask, p).next().is_some())
        || code_hits(body, mask, ".mode(").any(|i| !body[i + ".mode(".len()..].starts_with(')'));
    if !sets {
        return false;
    }
    let mut prev = "";
    let chmod_line = body.lines().any(|l| {
        let hit = chmod_arg(l, prev);
        prev = l;
        hit
    });
    body.contains("+x") || chmod_line || exec_literal(body, mask)
}

/// A function in a scanned file that makes a file executable.
struct Wrapper {
    name: String,
    /// `pub` or `pub(crate)`: callable from other files of the crate.
    public: bool,
}

/// The byte range of the body of the function whose name ends at `from`, braces included, or `None` for a
/// declaration with no body.
fn body_span(b: &[u8], code: &[bool], from: usize) -> Option<(usize, usize)> {
    let mut parens = 0i32;
    let mut open = None;
    for (k, &c) in b.iter().enumerate().skip(from) {
        if !code[k] {
            continue;
        }
        match c {
            b'(' | b'[' => parens += 1,
            b')' | b']' => parens -= 1,
            b';' if parens == 0 => return None,
            b'{' if parens == 0 => {
                open = Some(k);
                break;
            }
            _ => {}
        }
    }
    let open = open?;
    let mut depth = 0i32;
    for (k, &c) in b.iter().enumerate().skip(open) {
        if !code[k] {
            continue;
        }
        match c {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((open, k));
                }
            }
            _ => {}
        }
    }
    None
}

/// Every function of `text` that is a wrapper (see [`is_wrapper_body`]). Headers are `fn name(` or
/// `fn name<` after optional `pub`, `pub(crate)`, `const`, `unsafe` and `async`.
fn wrappers(text: &str, code: &[bool], starts: &[usize]) -> Vec<Wrapper> {
    let b = text.as_bytes();
    let mut found = Vec::new();
    for (i, piece) in text.split_inclusive('\n').enumerate() {
        let trimmed = piece.trim_start();
        let at = starts[i] + piece.len() - trimmed.len();
        if !code.get(at).copied().unwrap_or(false) {
            continue;
        }
        let mut rest = trimmed;
        let mut public = false;
        if let Some(r) = rest
            .strip_prefix("pub(crate) ")
            .or_else(|| rest.strip_prefix("pub "))
        {
            public = true;
            rest = r;
        }
        while let Some(r) = ["const ", "unsafe ", "async "]
            .iter()
            .find_map(|q| rest.strip_prefix(q))
        {
            rest = r;
        }
        let Some(rest) = rest.strip_prefix("fn ") else {
            continue;
        };
        let name: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        let after = &rest[name.len()..];
        if name.is_empty() || !(after.starts_with('(') || after.starts_with('<')) {
            continue;
        }
        let name_end = at + trimmed.len() - rest.len() + name.len();
        let Some((open, close)) = body_span(b, code, name_end) else {
            continue;
        };
        if is_wrapper_body(&text[open..=close], &code[open..=close]) {
            found.push(Wrapper { name, public });
        }
    }
    found
}

/// The crate a repository path belongs to: the component after `crates/`, or `xtask`.
fn crate_id(rel: &str) -> &str {
    if let Some(rest) = rel.strip_prefix("crates/") {
        rest.split('/').next().unwrap_or("")
    } else if rel.starts_with("xtask/") {
        "xtask"
    } else {
        ""
    }
}

/// Whether `line` (which starts at byte `line_at` of the text) calls a function named in `names`: the whole
/// identifier, not a method (`x.name(`) and not a definition (`fn name(`), followed by `(` or `::<`.
fn calls_wrapper(line: &str, line_at: usize, code: &[bool], names: &BTreeSet<&str>) -> bool {
    let b = line.as_bytes();
    names.iter().any(|name| {
        line.match_indices(name).any(|(p, _)| {
            let after = &line[p + name.len()..];
            code[line_at + p]
                && (p == 0 || !(is_ident(b[p - 1]) || b[p - 1] == b'.'))
                && (after.starts_with('(') || after.starts_with("::<"))
                && !line[..p].trim_end().ends_with("fn")
        })
    })
}

/// Lines of test code that make a file executable from inside the process or through an external `chmod`,
/// as (1-based line, reason). `shared` holds the `pub` wrappers of each crate's other files.
fn violations_with(rel: &str, text: &str, shared: &Shared) -> Vec<Hit> {
    let Some(start) = test_region_start(rel, text) else {
        return Vec::new();
    };
    let code = code_mask(text);
    let starts = line_starts(text);
    let spans = helper_spans(text, &code, &starts);
    let own = wrappers(text, &code, &starts);
    let mut names: BTreeSet<&str> = own.iter().map(|w| w.name.as_str()).collect();
    if let Some(set) = shared.get(crate_id(rel)) {
        names.extend(set.iter().map(String::as_str));
    }
    let mut hits = Vec::new();
    let mut prev = "";
    for (i, piece) in text.split_inclusive('\n').enumerate() {
        let line = piece.trim_end_matches(['\r', '\n']);
        let before = std::mem::replace(&mut prev, line);
        if i < start || line.trim_start().starts_with("//") || spans[i] {
            continue;
        }
        let reason = primitive(line)
            .or_else(|| chmod_arg(line, before).then_some("chmod argument"))
            .or_else(|| calls_wrapper(line, starts[i], &code, &names).then_some("wrapper call"));
        if let Some(reason) = reason {
            hits.push((i + 1, reason));
        }
    }
    hits
}

/// [`violations_with`] with no wrappers shared from other files.
fn violations(rel: &str, text: &str) -> Vec<Hit> {
    violations_with(rel, text, &Shared::new())
}

/// The `pub` and `pub(crate)` wrappers of each crate, from every scanned file except the helper's own source.
fn shared_wrappers(files: &[(String, String)]) -> Shared {
    let mut shared = Shared::new();
    for (rel, text) in files {
        if rel.starts_with("crates/athena-testkit/src/") {
            continue;
        }
        let code = code_mask(text);
        let starts = line_starts(text);
        for w in wrappers(text, &code, &starts)
            .into_iter()
            .filter(|w| w.public)
        {
            shared
                .entry(crate_id(rel).to_string())
                .or_default()
                .insert(w.name);
        }
    }
    shared
}

/// Every hit in the scanned files, as (path, hit), exempt files included.
fn scan_tree(files: &[(String, String)]) -> Vec<(&str, Hit)> {
    let shared = shared_wrappers(files);
    files
        .iter()
        .flat_map(|(rel, text)| {
            violations_with(rel, text, &shared)
                .into_iter()
                .map(move |hit| (rel.as_str(), hit))
        })
        .collect()
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Every scanned file as (path relative to the repository root, text).
fn tree() -> Vec<(String, String)> {
    let root = root();
    let mut dirs = vec![root.join("xtask/src"), root.join("xtask/tests")];
    for entry in std::fs::read_dir(root.join("crates")).unwrap() {
        let krate = entry.unwrap().path();
        dirs.push(krate.join("src"));
        dirs.push(krate.join("tests"));
    }
    let mut files = Vec::new();
    for dir in dirs {
        rust_files(&dir, &mut files);
    }
    files.sort();
    files
        .into_iter()
        .map(|f| {
            let rel = f
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let text = std::fs::read_to_string(&f).unwrap();
            (rel, text)
        })
        .collect()
}

#[test]
fn no_in_process_exec_writes() {
    let files = tree();
    assert!(files.len() > 100, "only {} files scanned", files.len());
    for must in ["xtask/src/main.rs", "crates/athena-cli/tests/slot_exec.rs"] {
        assert!(files.iter().any(|(r, _)| r == must), "{must} not scanned");
    }
    let mut found = Vec::new();
    for (rel, (line, reason)) in scan_tree(&files) {
        if EXEMPT.iter().any(|(p, _)| *p == rel) {
            continue;
        }
        found.push(format!("{rel}:{line}: {reason}"));
    }
    assert!(
        found.is_empty(),
        "in-process executable writes in test code; use athena_testkit::write_executable:\n{}",
        found.join("\n")
    );
}

#[test]
fn every_exemption_still_exists_and_still_has_a_hit() {
    let files = tree();
    let hits = scan_tree(&files);
    for (path, reason) in EXEMPT {
        assert!(
            files.iter().any(|(r, _)| r == path),
            "exempt file {path} is gone; drop it from EXEMPT"
        );
        assert!(
            hits.iter().any(|(r, _)| r == path),
            "{path} ({reason}) has no flagged line now; drop it from EXEMPT"
        );
    }
}

#[test]
fn a_set_executable_wrapper_call_is_flagged() {
    let text = "#[test]\nfn t() {\n    set_executable(&fake).unwrap();\n}\n";
    assert_eq!(
        violations("crates/x/tests/t.rs", text),
        vec![(3, "set_executable wrapper")]
    );
}

#[test]
fn set_permissions_with_0o755_is_flagged() {
    let text = "fn t() {\n    std::fs::set_permissions(&p, perms(0o755)).unwrap();\n}\n";
    let hits = violations("crates/x/tests/t.rs", text);
    assert_eq!(hits, vec![(2, "set_permissions")]);
}

#[test]
fn from_mode_is_flagged() {
    let text = "let p = std::fs::Permissions::from_mode(0o755);\n";
    assert_eq!(
        violations("crates/x/tests/t.rs", text),
        vec![(1, "Permissions::from_mode")]
    );
}

#[test]
fn set_mode_and_open_options_mode_are_flagged() {
    let text = "perms.set_mode(0o755);\nOpenOptions::new().mode(0o755).open(p);\n";
    assert_eq!(
        violations("crates/x/tests/t.rs", text),
        vec![(1, "set_mode"), (2, "OpenOptions mode")]
    );
}

#[test]
fn a_chmod_child_is_flagged() {
    let text = "Command::new(\"chmod\").arg(\"755\").arg(&p).status().unwrap();\n";
    assert_eq!(
        violations("crates/x/tests/t.rs", text),
        vec![(1, "chmod child")]
    );
}

#[test]
fn a_violation_after_the_cfg_test_marker_in_src_is_flagged() {
    let text = "fn prod() {}\n#[cfg(test)]\nmod t {\n    fn f() { set_executable(&p); }\n}\n";
    assert_eq!(
        violations("crates/x/src/lib.rs", text),
        vec![(4, "set_executable wrapper")]
    );
    let all = "#[cfg(all(test, unix))]\nmod t { fn f() { std::fs::set_permissions(a, b); } }\n";
    assert_eq!(violations("crates/x/src/lib.rs", all).len(), 1);
}

#[test]
fn a_test_support_file_is_scanned_whole() {
    let text = "fn f() { std::fs::set_permissions(a, b); }\n";
    assert_eq!(violations("crates/x/src/test_support.rs", text).len(), 1);
    assert_eq!(violations("crates/x/src/foo_tests.rs", text).len(), 1);
}

#[test]
fn production_code_comments_reads_and_the_helper_are_not_flagged() {
    let production = "fn set_executable(p: &Path) {\n    std::fs::set_permissions(p, perms).unwrap();\n}\n#[cfg(test)]\nmod t {}\n";
    assert!(violations("crates/x/src/lib.rs", production).is_empty());
    let no_marker = "fn f() { std::fs::set_permissions(a, b); }\n";
    assert!(violations("crates/x/src/lib.rs", no_marker).is_empty());
    let clean = "// set_permissions(a, b) is what not to do\n\
        let m = std::fs::metadata(&p).unwrap().permissions().mode();\n\
        athena_testkit::write_executable(&p, \"#!/bin/sh\\n\");\n\
        Command::new(\"sh\").arg(\"-c\").status().unwrap();\n";
    assert!(violations("crates/x/tests/t.rs", clean).is_empty());
}

#[test]
fn a_pub_wrapper_above_the_test_marker_is_flagged_where_a_test_calls_it() {
    let text = r#"pub fn mark_runnable(p: &Path) {
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o755)).unwrap();
}
#[cfg(test)]
mod t {
    fn f() {
        super::mark_runnable(&p);
    }
}
"#;
    assert_eq!(
        violations("crates/x/src/lib.rs", text),
        vec![(7, "wrapper call")]
    );
}

#[test]
fn a_test_only_wrapper_and_its_call_are_flagged() {
    let text = r#"fn mark(p: &Path) {
    std::fs::set_permissions(p, perms(0o755)).unwrap();
}
fn t() {
    mark(&p);
}
"#;
    assert_eq!(
        violations("crates/x/tests/t.rs", text),
        vec![(2, "set_permissions"), (5, "wrapper call")]
    );
}

#[test]
fn a_generic_wrapper_call_is_flagged() {
    let text = r#"fn mark<P: AsRef<Path>>(p: P) {
    std::fs::set_permissions(p.as_ref(), Permissions::from_mode(0o755)).unwrap();
}
fn t() {
    mark(&p);
}
"#;
    assert_eq!(
        violations("crates/x/tests/t.rs", text),
        vec![(2, "set_permissions"), (5, "wrapper call")]
    );
}

#[test]
fn a_chmod_inside_a_test_fn_is_flagged_at_its_own_line() {
    // The fn has no mode call, so it is not a wrapper and its call at line 5 is not flagged: the chmod
    // line is where the scan refuses it.
    let text = r#"fn mark(p: &Path) {
    Command::new("sh").arg("-c").arg("chmod +x t").status().unwrap();
}
fn t() {
    mark(&p);
}
"#;
    assert_eq!(
        violations("crates/x/tests/t.rs", text),
        vec![(2, "chmod argument")]
    );
}

#[test]
fn a_pub_wrapper_in_another_file_of_the_same_crate_is_flagged() {
    let lib = r#"pub fn install(p: &Path) {
    std::fs::set_permissions(p, Permissions::from_mode(0o755)).unwrap();
}
"#;
    let caller = "fn t() {\n    install(&p);\n}\n";
    let files = vec![("crates/x/src/lib.rs".to_string(), lib.to_string())];
    let shared = shared_wrappers(&files);
    assert_eq!(
        violations_with("crates/x/tests/t.rs", caller, &shared),
        vec![(2, "wrapper call")]
    );
    // Without the shared set the call is invisible, which is why the tree scan builds it.
    assert!(violations("crates/x/tests/t.rs", caller).is_empty());
    // The same fn when private cannot be called from another file.
    let private = vec![(
        "crates/x/src/lib.rs".to_string(),
        lib.replace("pub fn", "fn"),
    )];
    let shared = shared_wrappers(&private);
    assert!(violations_with("crates/x/tests/t.rs", caller, &shared).is_empty());
    // A pub fn of another crate with the same name is a different function.
    let shared = shared_wrappers(&files);
    assert!(violations_with("crates/y/tests/t.rs", caller, &shared).is_empty());
}

#[test]
fn a_one_letter_wrapper_flags_only_its_own_call() {
    let text = r#"fn t(p: &Path) {
    std::fs::set_permissions(p, perms(0o755)).unwrap();
}
fn u() {
    let s = format(1);
    x.t(2);
    t(&p);
}
"#;
    assert_eq!(
        violations("crates/x/tests/t.rs", text),
        vec![(2, "set_permissions"), (7, "wrapper call")]
    );
}

#[test]
fn an_external_chmod_outside_the_helper_is_flagged() {
    let cases: [(&str, usize); 5] = [
        (
            "Command::new(\"sh\").arg(\"-c\").arg(\"chmod +x t\").status().unwrap();\n",
            1,
        ),
        ("cmd.arg(\"chmod\").arg(\"+x\").arg(&p);\n", 1),
        ("cmd.args([\"-c\", \"chmod +x t\"]);\n", 1),
        ("let c = Command::new(\n    \"chmod\",\n);\n", 2),
        ("cmd.args([\n    \"-c\",\n    \"chmod +x t\",\n]);\n", 3),
    ];
    for (text, line) in cases {
        assert_eq!(
            violations("crates/x/tests/t.rs", text),
            vec![(line, "chmod argument")],
            "{text}"
        );
    }
}

#[test]
fn a_multi_line_helper_call_with_a_raw_string_does_not_hide_a_later_violation() {
    let text = r##"athena_testkit::write_executable(
    &p,
    r#"#!/bin/sh
echo $(date
chmod +x x
"#,
);
std::fs::set_permissions(&p, perms).unwrap();
"##;
    assert_eq!(
        violations("crates/x/tests/t.rs", text),
        vec![(8, "set_permissions")]
    );
}

#[test]
fn an_unclosed_helper_call_fails_closed() {
    let text = format!(
        "athena_testkit::write_executable(&p, \"x\"\n{}std::fs::set_permissions(&p, q).unwrap();\n",
        "let _ = 1;\n".repeat(45)
    );
    assert_eq!(
        violations("crates/x/tests/t.rs", &text),
        vec![(47, "set_permissions")]
    );
}

#[test]
fn helper_calls_and_look_alike_text_are_not_flagged() {
    let one_line = "athena_testkit::write_executable(&p, \"#!/bin/sh\\nchmod +x x\\n\");\n";
    let multi_line = "athena_testkit::write_executable(\n    &p,\n    \"#!/bin/sh\\nchmod +x \\\"$1\\\" && cmd.arg(1)\\n\",\n);\n";
    let shell_c = "Command::new(\"sh\").arg(\"-c\").arg(\"git -c user.name=x commit\").status();\n";
    let policy = "let inputs = [\n    \"chmod +x src/lib.rs\",\n];\n";
    for text in [one_line, multi_line, shell_c, policy] {
        assert!(violations("crates/x/tests/t.rs", text).is_empty(), "{text}");
    }
}

#[test]
fn a_write_bit_toggle_and_a_private_mode_are_not_wrappers() {
    let text = r#"fn lock(p: &Path, on: bool) {
    let mut perms = std::fs::metadata(p).unwrap().permissions();
    perms.set_mode(if on { perms.mode() | 0o200 } else { perms.mode() & !0o222 });
    std::fs::set_permissions(p, perms).unwrap();
}
fn write_private(p: &Path) {
    OpenOptions::new().write(true).mode(0o600).open(p).unwrap();
}
#[cfg(test)]
mod t {
    fn f() {
        super::lock(&p, true);
        write_private(&p);
    }
}
"#;
    assert!(violations("crates/x/src/lib.rs", text).is_empty());
}

#[test]
fn a_brace_char_literal_does_not_stretch_a_function_body() {
    // If the `'{'` counted as an open brace, `first` would swallow `mark` and look like a wrapper.
    let text = r#"fn first() -> char {
    '{'
}
fn mark(p: &Path) {
    std::fs::set_permissions(p, from_mode(0o755)).unwrap();
}
#[cfg(test)]
mod t {
    fn f() {
        first();
    }
}
"#;
    assert!(violations("crates/x/src/lib.rs", text).is_empty());
}
