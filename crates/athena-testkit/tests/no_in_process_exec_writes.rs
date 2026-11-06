//! The rule that test stand-in programs are written through `athena_testkit::write_executable`, enforced by
//! scanning the workspace's test code.
//!
//! A test that writes a script in its own process and then marks it executable (`std::fs::set_permissions`,
//! `Permissions::from_mode`, `PermissionsExt::set_mode`) can later fail with `ETXTBSY` when another test thread
//! forks while the file is still open for writing. See the crate documentation of `athena_testkit` for the
//! mechanism. The helper writes through a child process instead, so this scan fails on any test code that
//! changes a file's permission bits in-process.
//!
//! What is scanned: every Rust file under `crates/*/src`, `crates/*/tests`, `xtask/src` and `xtask/tests`,
//! parsed with `syn`. Test code is:
//!
//! - every file under a `tests/` directory;
//! - an item with `#[cfg(test)]` (or a `cfg` that names `test` without `not`, such as the
//!   `any(test, feature = "test-support")` of shared fixtures) and everything inside it;
//! - a function with `#[test]`;
//! - a file declared by `mod name;` under such an item.
//!
//! Production code is not scanned: the shipped crates set permission bits on purpose (installing tool files,
//! snapshots), and that stays. The scan skips the helper crate's `src/` (the helper is the one allowed
//! mechanism) and `crates/athena-testkit/tests/stand_in_stress.rs`, whose comparison variant writes in-process
//! on purpose. Both skips are listed in [`SKIPPED`].
//!
//! The names flagged are in [`FLAGGED`]. A flagged name inside a macro call's tokens counts too.

use proc_macro2::{TokenStream, TokenTree};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use syn::visit::Visit;

/// The function and method names that change permission bits in-process.
const FLAGGED: [&str; 3] = ["set_permissions", "from_mode", "set_mode"];

/// Paths, relative to the workspace root, that the scan skips: a file, or a directory with a trailing `/`.
const SKIPPED: [&str; 2] = [
    "crates/athena-testkit/src/",
    "crates/athena-testkit/tests/stand_in_stress.rs",
];

/// One flagged name.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Hit {
    line: usize,
    name: String,
    /// Whether the name is inside test code of its file (a file under `tests/` is all test code).
    in_test: bool,
}

/// What one source file holds that the scan needs.
#[derive(Debug, Default)]
struct FileScan {
    hits: Vec<Hit>,
    /// Out-of-line modules (`mod name;`) and whether each is declared in test code.
    modules: Vec<(String, bool)>,
}

/// Whether `attrs` mark the item as test code: `#[test]`, or a `cfg` that names `test` and not `not`.
fn marks_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("test") {
            return true;
        }
        if !attr.path().is_ident("cfg") {
            return false;
        }
        let syn::Meta::List(list) = &attr.meta else {
            return false;
        };
        let text = list.tokens.to_string();
        let words: Vec<&str> = text
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .collect();
        words.contains(&"test") && !words.contains(&"not")
    })
}

struct Scanner {
    depth: usize,
    scan: FileScan,
}

impl Scanner {
    fn record(&mut self, name: &str, line: usize) {
        if FLAGGED.contains(&name) {
            self.scan.hits.push(Hit {
                line,
                name: name.to_string(),
                in_test: self.depth > 0,
            });
        }
    }

    fn tokens(&mut self, stream: TokenStream) {
        for tree in stream {
            match tree {
                TokenTree::Group(group) => self.tokens(group.stream()),
                TokenTree::Ident(ident) => {
                    self.record(&ident.to_string(), ident.span().start().line);
                }
                _ => {}
            }
        }
    }

    /// Visits `walk` with the test depth raised when `attrs` mark test code.
    fn within(&mut self, attrs: &[syn::Attribute], walk: impl FnOnce(&mut Self)) {
        let test = marks_test(attrs);
        if test {
            self.depth += 1;
        }
        walk(self);
        if test {
            self.depth -= 1;
        }
    }
}

impl<'ast> Visit<'ast> for Scanner {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        self.within(&node.attrs, |this| {
            if node.content.is_none() {
                this.scan
                    .modules
                    .push((node.ident.to_string(), this.depth > 0));
            }
            syn::visit::visit_item_mod(this, node);
        });
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.within(&node.attrs, |this| syn::visit::visit_item_fn(this, node));
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        self.within(&node.attrs, |this| syn::visit::visit_item_impl(this, node));
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.within(&node.attrs, |this| {
            syn::visit::visit_impl_item_fn(this, node);
        });
    }

    fn visit_item_const(&mut self, node: &'ast syn::ItemConst) {
        self.within(&node.attrs, |this| {
            syn::visit::visit_item_const(this, node);
        });
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        self.within(&node.attrs, |this| syn::visit::visit_item_use(this, node));
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        self.record(&node.method.to_string(), node.method.span().start().line);
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        if let Some(last) = node.path.segments.last() {
            self.record(&last.ident.to_string(), last.ident.span().start().line);
        }
        syn::visit::visit_expr_path(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        self.tokens(node.tokens.clone());
        syn::visit::visit_macro(self, node);
    }
}

/// Parses `source` and lists the flagged names in it and the out-of-line modules it declares.
fn scan_source(source: &str) -> Result<FileScan, syn::Error> {
    let file = syn::parse_file(source)?;
    let mut scanner = Scanner {
        depth: 0,
        scan: FileScan::default(),
    };
    scanner.within(&file.attrs, |this| this.visit_file(&file));
    Ok(scanner.scan)
}

/// A violation: where a test changes permission bits in-process.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Violation {
    file: PathBuf,
    line: usize,
    name: String,
}

/// Every `.rs` file under `dir`, recursively, sorted.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The directories of `root` that hold Rust files the scan reads.
fn scan_roots(root: &Path) -> Vec<PathBuf> {
    let mut roots = vec![root.join("xtask/src"), root.join("xtask/tests")];
    if let Ok(crates) = std::fs::read_dir(root.join("crates")) {
        let mut crates: Vec<_> = crates.flatten().map(|e| e.path()).collect();
        crates.sort();
        for krate in crates {
            roots.push(krate.join("src"));
            roots.push(krate.join("tests"));
        }
    }
    roots
}

/// The file an out-of-line `mod name;` in `file` names, if it exists. A crate root (`lib.rs`, `main.rs`, a
/// file in a `bin/` or `tests/` directory) and a `mod.rs` look beside themselves; any other file looks in a
/// directory named for itself.
fn module_file(file: &Path, name: &str) -> Option<PathBuf> {
    let parent = file.parent()?;
    let stem = file.file_stem()?.to_string_lossy();
    let in_root_dir = parent
        .file_name()
        .is_some_and(|d| d == "bin" || d == "tests");
    let dir = if matches!(stem.as_ref(), "lib" | "main" | "mod") || in_root_dir {
        parent.to_path_buf()
    } else {
        parent.join(stem.as_ref())
    };
    [
        dir.join(format!("{name}.rs")),
        dir.join(name).join("mod.rs"),
    ]
    .into_iter()
    .find(|p| p.is_file())
}

/// Scans the workspace at `root` and returns the violations plus the number of files read and the set of files
/// found to be test-only.
fn scan_workspace(root: &Path) -> (Vec<Violation>, usize, BTreeSet<PathBuf>) {
    let mut files = Vec::new();
    for dir in scan_roots(root) {
        rust_files(&dir, &mut files);
    }
    files.retain(|file| {
        let relative = file.strip_prefix(root).unwrap_or(file).to_string_lossy();
        !SKIPPED.iter().any(|skip| {
            if skip.ends_with('/') {
                relative.starts_with(skip)
            } else {
                relative == *skip
            }
        })
    });

    let mut scans = BTreeMap::new();
    for file in &files {
        let source = std::fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("reading {}: {e}", file.display()));
        let scan =
            scan_source(&source).unwrap_or_else(|e| panic!("parsing {}: {e}", file.display()));
        scans.insert(file.clone(), scan);
    }

    // Files that are test code as a whole: under a tests/ directory, or declared by `mod x;` in test code,
    // repeated until no new file is found.
    let mut test_files: BTreeSet<PathBuf> = files
        .iter()
        .filter(|file| {
            file.strip_prefix(root)
                .is_ok_and(|relative| relative.components().any(|c| c.as_os_str() == "tests"))
        })
        .cloned()
        .collect();
    loop {
        let before = test_files.len();
        for (file, scan) in &scans {
            for (name, declared_in_test) in &scan.modules {
                if (*declared_in_test || test_files.contains(file))
                    && let Some(child) = module_file(file, name)
                {
                    test_files.insert(child);
                }
            }
        }
        if test_files.len() == before {
            break;
        }
    }

    let mut violations = Vec::new();
    for (file, scan) in &scans {
        let whole = test_files.contains(file);
        for hit in &scan.hits {
            if whole || hit.in_test {
                violations.push(Violation {
                    file: file.strip_prefix(root).unwrap_or(file).to_path_buf(),
                    line: hit.line,
                    name: hit.name.clone(),
                });
            }
        }
    }
    (violations, files.len(), test_files)
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root exists")
}

#[test]
fn no_test_code_in_the_workspace_writes_an_executable_in_process() {
    let root = workspace_root();
    let (violations, scanned, test_files) = scan_workspace(&root);

    // Guard against a scan that quietly finds nothing to read.
    assert!(scanned > 50, "only {scanned} Rust files were scanned");
    for expected in [
        "crates/athena-core/src/test_support.rs",
        "crates/athena-cli/tests/reconcile.rs",
    ] {
        assert!(
            test_files.contains(&root.join(expected)),
            "{expected} was not recognised as test code; the scan's module resolution is broken"
        );
    }

    let report: Vec<String> = violations
        .iter()
        .map(|v| format!("{}:{} uses {}", v.file.display(), v.line, v.name))
        .collect();
    assert!(
        violations.is_empty(),
        "test code changes file permissions in-process; write stand-in programs with \
         athena_testkit::write_executable instead (it avoids ETXTBSY, \"Text file busy\"):\n{}",
        report.join("\n")
    );
}

const PLANTED: &str = r##"
#[cfg(test)]
mod tests {
    #[test]
    fn writes_a_stand_in() {
        std::fs::write(&path, "#!/bin/sh\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}
"##;

fn flagged_names(source: &str) -> Vec<(String, bool)> {
    scan_source(source)
        .unwrap()
        .hits
        .into_iter()
        .map(|h| (h.name, h.in_test))
        .collect()
}

#[test]
fn a_planted_in_process_executable_write_in_a_test_module_is_flagged() {
    assert_eq!(
        flagged_names(PLANTED),
        [
            ("set_permissions".to_string(), true),
            ("from_mode".to_string(), true)
        ]
    );
}

#[test]
fn the_same_calls_in_production_code_are_seen_but_not_as_test_code() {
    let production = "fn install(path: &std::path::Path) { \
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap(); }";
    assert_eq!(
        flagged_names(production),
        [
            ("set_permissions".to_string(), false),
            ("from_mode".to_string(), false)
        ]
    );
    let not_test = "#[cfg(not(test))] fn install(p: &Path) { std::fs::set_permissions(p, x); }";
    assert_eq!(
        flagged_names(not_test),
        [("set_permissions".to_string(), false)]
    );
}

#[test]
fn a_test_function_a_method_call_and_a_macro_argument_are_flagged() {
    let bare_test = "#[test] fn t() { perms.set_mode(0o755); }";
    assert_eq!(flagged_names(bare_test), [("set_mode".to_string(), true)]);
    let in_macro = "#[cfg(test)] mod t { fn f() { assert!(set_permissions(&p, q).is_ok()); } }";
    assert_eq!(
        flagged_names(in_macro),
        [("set_permissions".to_string(), true)]
    );
    let shared_fixture =
        "#[cfg(any(test, feature = \"test-support\"))] mod t { fn f() { set_permissions(a, b); } }";
    assert_eq!(
        flagged_names(shared_fixture),
        [("set_permissions".to_string(), true)]
    );
}

#[test]
fn out_of_line_modules_are_listed_with_whether_test_code_declares_them() {
    let source = "#[cfg(test)] mod support; mod real;";
    let scan = scan_source(source).unwrap();
    assert_eq!(
        scan.modules,
        [("support".to_string(), true), ("real".to_string(), false)]
    );
}

#[test]
fn a_planted_violation_in_a_scratch_workspace_is_reported_with_file_and_line() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let write = |relative: &str, text: &str| {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    // Inline test module in a source file; production use beside it is fine.
    write(
        "crates/a/src/lib.rs",
        &format!(
            "pub fn prod(p: &std::path::Path) {{ std::fs::set_permissions(p, x); }}\n{PLANTED}"
        ),
    );
    // A whole file under tests/, and a module file declared by `#[cfg(test)] mod fixture;`.
    write(
        "crates/a/tests/it.rs",
        "fn f() { set_permissions(a, b); }\n",
    );
    write("crates/b/src/lib.rs", "#[cfg(test)]\nmod fixture;\n");
    write(
        "crates/b/src/fixture.rs",
        "fn f() { Permissions::from_mode(1); }\n",
    );
    // The helper crate's own sources are skipped.
    write(
        "crates/athena-testkit/src/lib.rs",
        "#[cfg(test)] mod t { fn f() { set_permissions(a, b); } }\n",
    );

    let (violations, _, _) = scan_workspace(root);
    let found: Vec<(String, usize, String)> = violations
        .iter()
        .map(|v| (v.file.display().to_string(), v.line, v.name.clone()))
        .collect();
    assert_eq!(
        found,
        [
            (
                "crates/a/src/lib.rs".to_string(),
                8,
                "set_permissions".to_string()
            ),
            (
                "crates/a/src/lib.rs".to_string(),
                8,
                "from_mode".to_string()
            ),
            (
                "crates/a/tests/it.rs".to_string(),
                1,
                "set_permissions".to_string()
            ),
            (
                "crates/b/src/fixture.rs".to_string(),
                1,
                "from_mode".to_string()
            ),
        ]
    );
}
