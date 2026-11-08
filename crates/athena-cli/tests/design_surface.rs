//! The design-surface inventory at this repository's HEAD agrees with what the binary and `athena.toml` show.

use athena_core::design_surface::inventory_at;
use athena_git::Git;
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

fn repository() -> Git {
    Git::at(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
}

fn head(git: &Git) -> String {
    git.head()
        .unwrap()
        .expect("the repository has a commit to inventory")
}

/// The commands `athena --help` lists, from its `Commands:` section.
fn help_commands() -> BTreeSet<String> {
    let out = Command::new(env!("CARGO_BIN_EXE_athena"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    text.lines()
        .skip_while(|l| l.trim_end() != "Commands:")
        .skip(1)
        .take_while(|l| l.starts_with("  "))
        .filter_map(|l| l.split_whitespace().next())
        .filter(|name| *name != "help")
        .map(String::from)
        .collect()
}

/// Every table path `athena.toml` uses, such as `models.names`.
fn toml_tables(prefix: &str, table: &toml::Table, out: &mut BTreeSet<String>) {
    for (key, value) in table {
        let path = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        let tables: Vec<&toml::Table> = match value {
            toml::Value::Table(t) => vec![t],
            toml::Value::Array(a) => a.iter().filter_map(|v| v.as_table()).collect(),
            _ => continue,
        };
        out.insert(path.clone());
        for t in tables {
            toml_tables(&path, t, out);
        }
    }
}

#[test]
fn the_inventory_at_head_lists_every_command_in_help_and_every_section_athena_toml_uses() {
    let git = repository();
    let head = head(&git);
    let inventory = inventory_at(&git, &head).unwrap();

    let listed = help_commands();
    assert!(listed.len() > 10, "help lists the commands: {listed:?}");
    for name in &listed {
        assert!(
            inventory.commands.contains_key(name),
            "`athena --help` prints `{name}` but the inventory does not list it"
        );
    }

    let text = String::from_utf8(git.show_file(&head, "athena.toml").unwrap().unwrap()).unwrap();
    let table: toml::Table = text.parse().unwrap();
    let mut used = BTreeSet::new();
    toml_tables("", &table, &mut used);
    assert!(
        used.contains("budget"),
        "athena.toml has sections: {used:?}"
    );
    for path in &used {
        // `[capabilities.change]` names a capability, which the schema holds as a map.
        let section = path.split('.').next().unwrap();
        let known = inventory.config_keys.get(section);
        assert_eq!(
            known.map(String::as_str),
            Some("section"),
            "athena.toml uses [{path}] but the inventory has no section `{section}`"
        );
        if section != "capabilities" {
            assert!(
                inventory.config_keys.contains_key(path),
                "athena.toml uses [{path}] but the inventory has no such section"
            );
        }
    }
}

#[test]
fn the_inventory_at_head_is_identical_when_computed_twice() {
    let git = repository();
    let head = head(&git);
    let one = inventory_at(&git, &head).unwrap();
    let two = inventory_at(&git, &head).unwrap();
    assert!(!one.is_empty());
    assert_eq!(one, two);
    assert_eq!(
        serde_json::to_vec(&one).unwrap(),
        serde_json::to_vec(&two).unwrap()
    );
}
