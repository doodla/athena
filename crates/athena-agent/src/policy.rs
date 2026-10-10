//! The agent tool policy behind Athena's `PreToolUse` hook (design §7.5, D22; M11, M25).
//!
//! The hook runs `athena hook pre-tool-use --policy <file>` for every tool call. Its Bash policy is an
//! allow-list:
//!
//! - a command is split into segments at `&&`, `||`, `;`, `|`, and newlines; anything that cannot be
//!   classified is denied: subshells, command or variable substitution, here-documents, backgrounding,
//!   brace expansion, and environment assignments;
//! - each segment's program must be on the role's list, with an allowed first argument (cargo
//!   subcommands; git `status`, `diff`, `log`, `show`);
//! - cargo's `--ignored` and `--include-ignored` are denied to every role, so no agent starts the live
//!   tests, which make real model calls;
//! - git's `-c`, `-C`, `--git-dir`, `--work-tree`, `--exec-path`, and similar overrides are denied;
//! - every path argument, after resolving `..` and symlinks, must lie inside the snapshot, and outside its
//!   `.git`; output redirection is confined the same way;
//! - roles without write access (reader, verifier) may not write at all.
//!
//! The file tools are confined the same way. A denial wins even where a permission rule would allow the
//! call (E4), and it says why and what to do instead.

use athena_data::records::Role;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One role's policy for one invocation, written by the core and read by the hook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub role: Role,
    /// The snapshot: every path must resolve inside it.
    pub root: PathBuf,
    /// Program → allowed first arguments (`*` allows any).
    pub commands: BTreeMap<String, Vec<String>>,
    /// Whether the role may write files (only the producer).
    pub writes: bool,
}

/// The hook's answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// No objection: Claude Code's own permission rules decide.
    NoObjection,
    /// Denied, with the reason and the safe next step.
    Deny(String),
}

impl Decision {
    /// The hook's output: nothing for no objection, a deny decision otherwise (never "allow").
    pub fn hook_output(&self) -> Option<String> {
        match self {
            Decision::NoObjection => None,
            Decision::Deny(reason) => Some(
                serde_json::json!({
                    "hookSpecificOutput": {
                        "hookEventName": "PreToolUse",
                        "permissionDecision": "deny",
                        "permissionDecisionReason": format!("Athena policy: {reason}"),
                    }
                })
                .to_string(),
            ),
        }
    }
}

fn deny(reason: impl Into<String>) -> Decision {
    Decision::Deny(reason.into())
}

/// Evaluates one tool call. `cwd` is the agent's current directory, as the hook input reports it.
pub fn evaluate(policy: &Policy, tool: &str, input: &Value, cwd: Option<&Path>) -> Decision {
    let root = match policy.root.canonicalize() {
        Ok(root) => root,
        Err(_) => return deny("the snapshot directory is missing"),
    };
    let cwd = cwd.map(Path::to_path_buf).unwrap_or_else(|| root.clone());
    match tool {
        "Read" | "Grep" | "Glob" | "LS" => {
            for key in ["file_path", "path", "notebook_path"] {
                if let Some(path) = input.get(key).and_then(Value::as_str)
                    && let Err(reason) = confine(&root, &cwd, path, false)
                {
                    return deny(reason);
                }
            }
            if let Some(pattern) = input.get("pattern").and_then(Value::as_str)
                && tool == "Glob"
                && (pattern.starts_with('/') || pattern.starts_with('~') || pattern.contains(".."))
            {
                let prefix = glob_prefix(pattern);
                if let Err(reason) = confine(
                    &root,
                    &cwd,
                    if prefix.is_empty() { "." } else { prefix },
                    false,
                ) {
                    return deny(reason);
                }
                if pattern.contains("..") {
                    return deny(
                        "patterns that climb out of the working directory are not available",
                    );
                }
            }
            Decision::NoObjection
        }
        "Edit" | "Write" | "MultiEdit" | "NotebookEdit" => {
            if !policy.writes {
                return deny(format!(
                    "the {} role does not write files; report what should change instead",
                    policy.role.as_str()
                ));
            }
            for key in ["file_path", "notebook_path"] {
                if let Some(path) = input.get(key).and_then(Value::as_str)
                    && let Err(reason) = confine(&root, &cwd, path, true)
                {
                    return deny(reason);
                }
            }
            Decision::NoObjection
        }
        "Bash" => match input.get("command").and_then(Value::as_str) {
            Some(command) => match check_bash(policy, &root, &cwd, command) {
                Ok(()) => Decision::NoObjection,
                Err(reason) => deny(reason),
            },
            None => deny("a Bash call without a command"),
        },
        other => deny(format!(
            "{other} is not a tool of the {} role",
            policy.role.as_str()
        )),
    }
}

/// A token of a shell command.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word {
        text: String,
        glob: bool,
        tilde: bool,
    },
    Separator,
    Redirect {
        op: String,
        target: Option<String>,
    },
}

/// Splits a command into words, separators, and redirections, refusing anything it cannot classify.
fn tokenize(command: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = command.chars().collect();
    let mut tokens = Vec::new();
    let mut word = String::new();
    let mut in_word = false;
    let mut glob = false;
    let mut tilde = false;
    let mut i = 0;
    macro_rules! end_word {
        () => {
            if in_word {
                tokens.push(Token::Word {
                    text: std::mem::take(&mut word),
                    glob,
                    tilde,
                });
                in_word = false;
                glob = false;
                tilde = false;
            }
        };
    }
    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' => end_word!(),
            '\n' => {
                end_word!();
                tokens.push(Token::Separator);
            }
            '\'' => {
                in_word = true;
                i += 1;
                while i < chars.len() && chars[i] != '\'' {
                    word.push(chars[i]);
                    i += 1;
                }
                if i >= chars.len() {
                    return Err("an unterminated quote".into());
                }
            }
            '"' => {
                in_word = true;
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    match chars[i] {
                        '$' | '`' => {
                            return Err("expansion inside double quotes; use single quotes".into());
                        }
                        '\\' if i + 1 < chars.len()
                            && matches!(chars[i + 1], '"' | '\\' | '$' | '`') =>
                        {
                            word.push(chars[i + 1]);
                            i += 1;
                        }
                        other => word.push(other),
                    }
                    i += 1;
                }
                if i >= chars.len() {
                    return Err("an unterminated quote".into());
                }
            }
            '\\' => {
                if i + 1 < chars.len() {
                    if chars[i + 1] != '\n' {
                        in_word = true;
                        word.push(chars[i + 1]);
                    }
                    i += 1;
                }
            }
            '$' | '`' => return Err("command or variable substitution".into()),
            '(' | ')' => return Err("subshells and grouping".into()),
            '{' | '}' => return Err("brace expansion and grouping".into()),
            '#' if !in_word => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }
            ';' => {
                end_word!();
                tokens.push(Token::Separator);
            }
            '&' => {
                if chars.get(i + 1) == Some(&'&') {
                    end_word!();
                    tokens.push(Token::Separator);
                    i += 1;
                } else if chars.get(i + 1) == Some(&'>') {
                    end_word!();
                    i += 2;
                    let (target, next) = redirect_target(&chars, i)?;
                    tokens.push(Token::Redirect {
                        op: "&>".into(),
                        target: Some(target),
                    });
                    i = next;
                    continue;
                } else {
                    return Err("backgrounding".into());
                }
            }
            '|' => {
                end_word!();
                match chars.get(i + 1) {
                    Some('|') => i += 1,
                    Some('&') => return Err("|& pipes".into()),
                    _ => {}
                }
                tokens.push(Token::Separator);
            }
            '<' | '>' => {
                // A word of digits right before `<` or `>` is the file descriptor it redirects.
                let fd = if in_word && !word.is_empty() && word.chars().all(|d| d.is_ascii_digit())
                {
                    in_word = false;
                    std::mem::take(&mut word)
                } else {
                    end_word!();
                    String::new()
                };
                let mut op = format!("{fd}{c}");
                i += 1;
                match chars.get(i) {
                    Some('<') if c == '<' => return Err("here-documents".into()),
                    Some('>') if c == '>' => {
                        op.push('>');
                        i += 1;
                    }
                    Some('&') => {
                        op.push('&');
                        i += 1;
                        let start = i;
                        while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '-') {
                            i += 1;
                        }
                        if i == start {
                            return Err("an unusual redirection".into());
                        }
                        tokens.push(Token::Redirect { op, target: None });
                        continue;
                    }
                    Some('|') => {
                        op.push('|');
                        i += 1;
                    }
                    _ => {}
                }
                let (target, next) = redirect_target(&chars, i)?;
                tokens.push(Token::Redirect {
                    op,
                    target: Some(target),
                });
                i = next;
                continue;
            }
            '*' | '?' | '[' => {
                in_word = true;
                glob = true;
                word.push(c);
            }
            '~' if !in_word => {
                in_word = true;
                tilde = true;
                word.push(c);
            }
            other => {
                in_word = true;
                word.push(other);
            }
        }
        i += 1;
    }
    if in_word {
        tokens.push(Token::Word {
            text: word,
            glob,
            tilde,
        });
    }
    Ok(tokens)
}

/// Reads the target word of a redirection, starting at `i`.
fn redirect_target(chars: &[char], mut i: usize) -> Result<(String, usize), String> {
    while i < chars.len() && matches!(chars[i], ' ' | '\t') {
        i += 1;
    }
    let rest: String = chars[i..].iter().collect();
    let mut tokens = tokenize_word_prefix(&rest)?;
    if tokens.0.is_empty() {
        return Err("a redirection without a target".into());
    }
    let target = std::mem::take(&mut tokens.0);
    Ok((target, i + tokens.1))
}

/// Reads one word (with quotes) from the start of `text`; returns it and the number of chars consumed.
fn tokenize_word_prefix(text: &str) -> Result<(String, usize), String> {
    let chars: Vec<char> = text.chars().collect();
    let mut end = 0;
    let mut quote: Option<char> = None;
    while end < chars.len() {
        let c = chars[end];
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None if c == '\'' || c == '"' => quote = Some(c),
            None if matches!(c, ' ' | '\t' | '\n' | ';' | '&' | '|' | '<' | '>') => break,
            None => {}
        }
        end += 1;
    }
    if quote.is_some() {
        return Err("an unterminated quote".into());
    }
    let raw: String = chars[..end].iter().collect();
    match tokenize(&raw)?.into_iter().next() {
        Some(Token::Word { text, glob, tilde }) => {
            if glob || tilde {
                return Err("a redirection target with a pattern or ~".into());
            }
            Ok((text, end))
        }
        _ => Ok((String::new(), end)),
    }
}

/// Resolves `path` against `cwd` and checks it lies inside `root` (and, for writes, outside `.git`).
fn confine(root: &Path, cwd: &Path, path: &str, write: bool) -> Result<PathBuf, String> {
    if path.starts_with('~') {
        return Err(format!(
            "{path}: paths outside your working directory are not available"
        ));
    }
    let joined = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        cwd.join(path)
    };
    let lexical = crate::env::normalize(&joined);
    // Resolve symlinks in the longest existing prefix, then re-append the rest.
    let mut existing = lexical.clone();
    let mut rest = Vec::new();
    while !existing.exists() {
        match (
            existing.file_name().map(|n| n.to_os_string()),
            existing.parent().map(Path::to_path_buf),
        ) {
            (Some(name), Some(parent)) => {
                rest.push(name);
                existing = parent;
            }
            _ => break,
        }
    }
    let mut resolved = existing.canonicalize().unwrap_or(existing);
    for part in rest.iter().rev() {
        resolved.push(part);
    }
    if !resolved.starts_with(root) {
        return Err(format!(
            "{path} resolves outside your working directory; work only inside it (records/, private files, and history are absent by design)"
        ));
    }
    let git_dir = root.join(".git");
    if resolved.starts_with(&git_dir) {
        return Err(format!(
            "{path} is inside the snapshot's git directory; Athena owns git state"
        ));
    }
    if write && resolved == root {
        return Err("the working directory itself cannot be changed".into());
    }
    Ok(resolved)
}

/// The literal part of a word before its first pattern character.
fn glob_prefix(word: &str) -> &str {
    let end = word.find(['*', '?', '[']).unwrap_or(word.len());
    &word[..end]
}

const CARGO_GLOBAL_FLAGS: [&str; 8] = [
    "-q",
    "--quiet",
    "-v",
    "-vv",
    "--verbose",
    "--locked",
    "--offline",
    "--frozen",
];
const GIT_DENIED_ARGS: [&str; 5] = [
    "--output",
    "--ext-diff",
    "--no-index",
    "--exec-path",
    "--git-dir",
];

fn check_bash(policy: &Policy, root: &Path, cwd: &Path, command: &str) -> Result<(), String> {
    let tokens = tokenize(command).map_err(|what| {
        format!("{what} cannot be checked, so it is denied; run one plain command at a time")
    })?;
    let mut segments: Vec<Vec<Token>> = vec![Vec::new()];
    for token in tokens {
        match token {
            Token::Separator => segments.push(Vec::new()),
            other => segments
                .last_mut()
                .expect("one segment at least")
                .push(other),
        }
    }
    let mut ran_any = false;
    for segment in segments.iter().filter(|s| !s.is_empty()) {
        check_segment(policy, root, cwd, segment)?;
        ran_any = true;
    }
    if !ran_any {
        return Err("an empty command".into());
    }
    Ok(())
}

fn check_segment(
    policy: &Policy,
    root: &Path,
    cwd: &Path,
    segment: &[Token],
) -> Result<(), String> {
    let mut words: Vec<(&str, bool)> = Vec::new();
    for token in segment {
        match token {
            Token::Word { text, glob, tilde } => {
                if *tilde {
                    return Err(format!(
                        "{text}: paths outside your working directory are not available"
                    ));
                }
                words.push((text.as_str(), *glob));
            }
            Token::Redirect { op, target } => {
                check_redirect(policy, root, cwd, op, target.as_deref())?
            }
            Token::Separator => {}
        }
    }
    let Some(&(program, _)) = words.first() else {
        return Err("a redirection without a command".into());
    };
    if let Some((name, _)) = program.split_once('=')
        && !name.is_empty()
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(
            "environment assignments are not allowed; the environment is set by Athena".into(),
        );
    }
    if program.contains('/') {
        return Err(format!(
            "{program}: run programs by name, from the allowed list"
        ));
    }
    let allowed = policy.commands.get(program).ok_or_else(|| {
        format!(
            "`{program}` is not on the {} role's list ({}); use the Read, Grep, and Glob tools to inspect files",
            policy.role.as_str(),
            policy.commands.keys().cloned().collect::<Vec<_>>().join(", ")
        )
    })?;
    let args = &words[1..];
    if args
        .iter()
        .any(|(a, _)| *a == "--ignored" || *a == "--include-ignored")
    {
        return Err("--ignored and --include-ignored would start the live tests, which call real models; they are for people, not agents".into());
    }
    match program {
        "cargo" => check_cargo(policy, root, cwd, allowed, args),
        "git" => check_git(allowed, args),
        _ => {
            let first = args
                .iter()
                .find(|(a, _)| !a.starts_with('-'))
                .map(|(a, _)| *a);
            if !allowed.iter().any(|a| a == "*")
                && !first.is_some_and(|f| allowed.iter().any(|a| a == f))
            {
                return Err(format!(
                    "{program} {}: not an allowed use of {program}",
                    first.unwrap_or("")
                ));
            }
            let writes = matches!(program, "rm" | "mv" | "chmod" | "cp" | "touch" | "mkdir");
            if writes && !policy.writes {
                return Err(format!(
                    "the {} role does not write files",
                    policy.role.as_str()
                ));
            }
            let mut skip_mode = program == "chmod";
            for (arg, glob) in args {
                if arg.starts_with('-') {
                    continue;
                }
                if skip_mode {
                    skip_mode = false;
                    continue;
                }
                let path = if *glob { glob_prefix(arg) } else { arg };
                let path = if path.is_empty() { "." } else { path };
                confine(root, cwd, path, writes)?;
            }
            Ok(())
        }
    }
}

fn check_cargo(
    policy: &Policy,
    root: &Path,
    cwd: &Path,
    allowed: &[String],
    args: &[(&str, bool)],
) -> Result<(), String> {
    let mut i = 0;
    while i < args.len() && args[i].0.starts_with('-') {
        if !CARGO_GLOBAL_FLAGS.contains(&args[i].0) {
            return Err(format!(
                "cargo {}: global options other than {} are not allowed",
                args[i].0,
                CARGO_GLOBAL_FLAGS.join(", ")
            ));
        }
        i += 1;
    }
    let Some(&(sub, _)) = args.get(i) else {
        return Err("cargo without a subcommand".into());
    };
    if sub.starts_with('+') {
        return Err("toolchain overrides are not allowed; the toolchain is pinned".into());
    }
    if !allowed.iter().any(|a| a == "*" || a == sub) {
        return Err(format!(
            "cargo {sub} is not allowed here; allowed: cargo {}",
            allowed.join(", ")
        ));
    }
    let mut rest = args[i + 1..].iter();
    while let Some((arg, _)) = rest.next() {
        let (name, inline) = match arg.split_once('=') {
            Some((n, v)) if n.starts_with("--") => (n, Some(v)),
            _ => (*arg, None),
        };
        match name {
            "--config" | "-Z" | "--target-dir" | "--out-dir" | "--artifact-dir" => {
                return Err(format!(
                    "cargo {name} is not allowed; Athena sets the build configuration and directories"
                ));
            }
            "--manifest-path" => {
                let value = inline
                    .map(str::to_string)
                    .or_else(|| rest.next().map(|(v, _)| v.to_string()));
                confine(root, cwd, &value.unwrap_or_default(), false)?;
            }
            "--fix" | "--allow-dirty" | "--allow-staged" if !policy.writes => {
                return Err(format!(
                    "cargo {name} writes files; the {} role does not write",
                    policy.role.as_str()
                ));
            }
            _ if name.starts_with("-Z") => {
                return Err("unstable cargo options are not allowed".into());
            }
            _ => {}
        }
    }
    Ok(())
}

fn check_git(allowed: &[String], args: &[(&str, bool)]) -> Result<(), String> {
    let mut i = 0;
    while i < args.len() && args[i].0.starts_with('-') {
        match args[i].0 {
            "--no-pager" | "-P" => i += 1,
            other => {
                return Err(format!(
                    "git {other}: options before the subcommand (such as -c, -C, --git-dir, --work-tree) are not allowed"
                ));
            }
        }
    }
    let Some(&(sub, _)) = args.get(i) else {
        return Err("git without a subcommand".into());
    };
    if !allowed.iter().any(|a| a == sub) {
        return Err(format!(
            "git {sub} is not allowed; Athena owns git state. Allowed: git {}",
            allowed.join(", ")
        ));
    }
    for (arg, _) in &args[i + 1..] {
        let name = arg.split_once('=').map(|(n, _)| n).unwrap_or(arg);
        if GIT_DENIED_ARGS.contains(&name) || name.starts_with("-O") {
            return Err(format!(
                "git {sub} {name} is not allowed (it reads or writes outside the repository)"
            ));
        }
    }
    Ok(())
}

fn check_redirect(
    policy: &Policy,
    root: &Path,
    cwd: &Path,
    op: &str,
    target: Option<&str>,
) -> Result<(), String> {
    let Some(target) = target else { return Ok(()) };
    if target == "/dev/null" {
        return Ok(());
    }
    let input = op.ends_with('<');
    if !input && !policy.writes {
        return Err(format!(
            "the {} role does not write files, so output redirection is not available",
            policy.role.as_str()
        ));
    }
    confine(root, cwd, target, !input).map(|_| ())
}

/// The permission allow rules matching a command list, for `--allowedTools`.
pub fn allowed_tool_rules(commands: &BTreeMap<String, Vec<String>>) -> Vec<String> {
    let mut rules = Vec::new();
    for (program, subs) in commands {
        if subs.iter().any(|s| s == "*") {
            rules.push(format!("Bash({program}:*)"));
        } else {
            for sub in subs {
                rules.push(format!("Bash({program} {sub}:*)"));
            }
        }
    }
    rules
}

/// Human-readable lines for the brief: what Bash may run.
pub fn describe_commands(commands: &BTreeMap<String, Vec<String>>) -> Vec<String> {
    commands
        .iter()
        .map(|(program, subs)| {
            if subs.iter().any(|s| s == "*") {
                format!("{program} (paths inside your working directory)")
            } else {
                format!("{program} {}", subs.join(" | "))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn policy(role: Role, root: &Path) -> Policy {
        let mut commands = BTreeMap::from([
            (
                "cargo".to_string(),
                vec![
                    "build".into(),
                    "test".into(),
                    "clippy".into(),
                    "fmt".into(),
                    "run".into(),
                ],
            ),
            (
                "git".to_string(),
                vec!["status".into(), "diff".into(), "log".into(), "show".into()],
            ),
            ("ls".to_string(), vec!["*".into()]),
        ]);
        let writes = role == Role::Producer;
        if writes {
            for p in ["rm", "mv", "chmod"] {
                commands.insert(p.to_string(), vec!["*".into()]);
            }
        }
        Policy {
            role,
            root: root.to_path_buf(),
            commands,
            writes,
        }
    }

    fn bash(policy: &Policy, command: &str) -> Decision {
        evaluate(policy, "Bash", &json!({ "command": command }), None)
    }

    fn snapshot() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        std::fs::write(dir.path().join("src/lib.rs"), "").unwrap();
        dir
    }

    #[test]
    fn listed_git_and_cargo_commands_are_allowed() {
        // M11 (allows): listed git and cargo commands.
        let dir = snapshot();
        let p = policy(Role::Verifier, dir.path());
        for command in [
            "cargo test --locked",
            "cargo build -q && cargo test -p athena-core",
            "git status",
            "git diff HEAD~1 -- src/lib.rs",
            "git --no-pager log --oneline -5",
            "cargo test 2>&1",
            "ls -la src",
            "cargo run -p athena-cli -- status --json",
            "cargo test > /dev/null",
        ] {
            assert_eq!(bash(&p, command), Decision::NoObjection, "{command}");
        }
    }

    #[test]
    fn escapes_to_the_users_repository_are_denied() {
        // M11 (blocks): `git -c core.hooksPath=… status`, `git --git-dir=<user repo> log`, `git update-ref`,
        // and `GIT_DIR=… git log` are denied.
        let dir = snapshot();
        let p = policy(Role::Producer, dir.path());
        for command in [
            "git -c core.hooksPath=/tmp status",
            "git --git-dir=/srv/repo/.git log",
            "git -C /srv/repo log",
            "git update-ref refs/heads/main HEAD",
            "GIT_DIR=/srv/repo/.git git log",
            "git commit -m x",
            "git diff --no-index /etc/passwd src/lib.rs",
            "git log --output=/tmp/x",
            "cat /etc/passwd",
            "ls /",
            "ls ../..",
            "rm -rf ../other",
            "rm -rf .git",
            "mv src/lib.rs /tmp/lib.rs",
            "cargo test > /tmp/out.txt",
            "cargo --config 'build.rustflags=[]' build",
            "cargo +nightly build",
            "cargo build --target-dir /tmp/t",
            "cargo build --manifest-path /srv/repo/Cargo.toml",
            "/usr/bin/git status",
            "ls ~",
        ] {
            assert!(
                matches!(bash(&p, command), Decision::Deny(_)),
                "{command} must be denied"
            );
        }
    }

    #[test]
    fn unclassifiable_commands_are_denied() {
        let dir = snapshot();
        let p = policy(Role::Producer, dir.path());
        for command in [
            "cargo test $(echo x)",
            "cargo test `echo x`",
            "(cargo test)",
            "cargo test &",
            "echo $HOME",
            "cat <<EOF\nx\nEOF",
            "cargo test |& cat",
            "ls {a,b}",
            "cargo test \"$X\"",
            "cargo test | tail -5",
            "cd src && cargo test",
        ] {
            assert!(
                matches!(bash(&p, command), Decision::Deny(_)),
                "{command} must be denied"
            );
        }
    }

    #[test]
    fn live_tests_are_denied_to_every_role() {
        // M25 (blocks): the hook denies `cargo test -- --ignored`.
        let dir = snapshot();
        for role in [Role::Producer, Role::Verifier] {
            let p = policy(role, dir.path());
            for command in [
                "cargo test -- --ignored",
                "cargo test -- --include-ignored",
                "cargo test --workspace -- --ignored live",
            ] {
                assert!(matches!(bash(&p, command), Decision::Deny(_)), "{command}");
            }
        }
    }

    #[test]
    fn the_producer_may_change_files_inside_its_snapshot_only() {
        // M11 (allows): edits, and rm, mv, and chmod inside the snapshot (producer only).
        let dir = snapshot();
        let producer = policy(Role::Producer, dir.path());
        for command in [
            "rm src/lib.rs",
            "rm -f src/*.rs",
            "mv src/lib.rs src/main.rs",
            "chmod +x src/lib.rs",
            "chmod 755 src",
        ] {
            assert_eq!(bash(&producer, command), Decision::NoObjection, "{command}");
        }
        let edit = json!({ "file_path": dir.path().join("src/new.rs").to_str().unwrap() });
        assert_eq!(
            evaluate(&producer, "Write", &edit, None),
            Decision::NoObjection
        );
        let outside = json!({ "file_path": "/tmp/elsewhere.rs" });
        assert!(matches!(
            evaluate(&producer, "Write", &outside, None),
            Decision::Deny(_)
        ));

        let verifier = policy(Role::Verifier, dir.path());
        assert!(matches!(
            bash(&verifier, "rm src/lib.rs"),
            Decision::Deny(_)
        ));
        assert!(matches!(
            evaluate(&verifier, "Edit", &edit, None),
            Decision::Deny(_)
        ));
        assert!(matches!(
            bash(&verifier, "cargo clippy --fix"),
            Decision::Deny(_)
        ));
    }

    #[test]
    fn symlinks_out_of_the_snapshot_are_resolved() {
        let dir = snapshot();
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), dir.path().join("escape")).unwrap();
        let p = policy(Role::Producer, dir.path());
        assert!(matches!(bash(&p, "rm escape/file"), Decision::Deny(_)));
        let read = json!({ "file_path": dir.path().join("escape/secret").to_str().unwrap() });
        assert!(matches!(
            evaluate(&p, "Read", &read, None),
            Decision::Deny(_)
        ));
    }

    #[test]
    fn denials_are_hook_json_and_no_objection_is_silent() {
        assert_eq!(Decision::NoObjection.hook_output(), None);
        let out = deny("because").hook_output().unwrap();
        let value: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(value["hookSpecificOutput"]["permissionDecision"], "deny");
        assert!(
            value["hookSpecificOutput"]["permissionDecisionReason"]
                .as_str()
                .unwrap()
                .contains("because")
        );
    }

    #[test]
    fn other_tools_are_denied() {
        let dir = snapshot();
        let p = policy(Role::Producer, dir.path());
        assert!(matches!(
            evaluate(&p, "WebFetch", &json!({}), None),
            Decision::Deny(_)
        ));
        assert!(matches!(
            evaluate(&p, "Agent", &json!({}), None),
            Decision::Deny(_)
        ));
    }

    #[test]
    fn permission_rules_follow_the_lists() {
        let rules = allowed_tool_rules(&BTreeMap::from([
            ("cargo".to_string(), vec!["test".to_string()]),
            ("ls".to_string(), vec!["*".to_string()]),
        ]));
        assert_eq!(
            rules,
            vec!["Bash(cargo test:*)".to_string(), "Bash(ls:*)".to_string()]
        );
    }
}
