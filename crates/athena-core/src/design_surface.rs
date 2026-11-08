//! The design surface: a deterministic inventory of what Athena offers, and the difference between two.
//!
//! [`inventory_at`] lists, at one commit and from the files in that commit's tree (never the working tree):
//!
//! - the commands and their flags, read from the clap definitions in `crates/athena-cli/src/main.rs`;
//! - the `athena.toml` sections and keys, read from the config structs in `crates/athena-data/src/config.rs`;
//! - the record schemas and their versions, read from the `AnyRecord` serde renames in
//!   `crates/athena-data/src/records.rs`;
//! - the roles and their versions, read from the `RoleDefinition` values in `crates/athena-data/src/roles.rs`;
//! - the workspace crates, read from the workspace `Cargo.toml`.
//!
//! No model is called. The same commit always gives the same inventory, with every kind sorted by name.
//! [`diff`] compares two inventories into added, removed and changed items per kind, with totals.
//! [`Athena::record_design_surface`] commits an inventory as an `athena.design-surface/1` record, so a later
//! review can compare against it with [`diff`].
//!
//! A section that is added or removed counts once: the keys under it are folded into its item (its detail
//! says how many), so removing one `athena.toml` section is one removed item and not one per key.

use crate::Athena;
use crate::commits::{NewFile, message};
use anyhow::{Context, Result};
use athena_data::RecordId;
use athena_data::records::{AnyRecord, DesignSurface, SurfaceInventory};
use athena_git::Git;
use jiff::Timestamp;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use syn::visit::Visit;

/// Where the clap definitions live.
pub const CLI_SOURCE: &str = "crates/athena-cli/src/main.rs";
/// Where the `athena.toml` schema lives.
pub const CONFIG_SOURCE: &str = "crates/athena-data/src/config.rs";
/// Where the record envelope lives.
pub const RECORDS_SOURCE: &str = "crates/athena-data/src/records.rs";
/// Where the role definitions live.
pub const ROLES_SOURCE: &str = "crates/athena-data/src/roles.rs";
/// The workspace manifest.
pub const WORKSPACE_MANIFEST: &str = "Cargo.toml";

/// The deepest nesting of commands or config sections that is followed.
const MAX_DEPTH: usize = 8;

// ---------------------------------------------------------------------------------------------------------
// The inventory

/// The design surface at `commit`. A source file that is absent at that commit contributes nothing; one
/// that does not parse is an error.
pub fn inventory_at(git: &Git, commit: &str) -> Result<SurfaceInventory> {
    let mut inventory = SurfaceInventory::default();
    if let Some(text) = source(git, commit, CLI_SOURCE)? {
        read_cli(&text, &mut inventory).with_context(|| format!("reading {CLI_SOURCE}"))?;
    }
    if let Some(text) = source(git, commit, CONFIG_SOURCE)? {
        read_config(&text, &mut inventory).with_context(|| format!("reading {CONFIG_SOURCE}"))?;
    }
    if let Some(text) = source(git, commit, RECORDS_SOURCE)? {
        read_records(&text, &mut inventory).with_context(|| format!("reading {RECORDS_SOURCE}"))?;
    }
    if let Some(text) = source(git, commit, ROLES_SOURCE)? {
        read_roles(&text, &mut inventory).with_context(|| format!("reading {ROLES_SOURCE}"))?;
    }
    read_crates(git, commit, &mut inventory).context("reading the workspace crates")?;
    Ok(inventory)
}

fn source(git: &Git, commit: &str, path: &str) -> Result<Option<String>> {
    Ok(git
        .show_file(commit, path)?
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned()))
}

// --- syn helpers -----------------------------------------------------------------------------------------

/// The items of an attribute such as `#[arg(long, value_name = "N")]`: each name, with its value if it has
/// one. Items whose syntax is not understood are left out.
fn attr_items(attr: &syn::Attribute) -> Vec<(String, Option<syn::Expr>)> {
    let mut items = Vec::new();
    let _ = attr.parse_nested_meta(|meta| {
        let name = meta
            .path
            .get_ident()
            .map(|i| i.to_string())
            .unwrap_or_default();
        if meta.input.peek(syn::Token![=]) {
            let value: syn::Expr = meta.value()?.parse()?;
            items.push((name, Some(value)));
        } else {
            if meta.input.peek(syn::token::Paren) {
                let inner;
                syn::parenthesized!(inner in meta.input);
                let _: proc_macro2::TokenStream = inner.parse()?;
            }
            items.push((name, None));
        }
        Ok(())
    });
    items
}

/// The items of every attribute of `attrs` whose path is one of `paths`.
fn items_of(attrs: &[syn::Attribute], paths: &[&str]) -> Vec<(String, Option<syn::Expr>)> {
    attrs
        .iter()
        .filter(|a| paths.iter().any(|p| a.path().is_ident(p)))
        .flat_map(attr_items)
        .collect()
}

fn find_item<'a>(
    items: &'a [(String, Option<syn::Expr>)],
    name: &str,
) -> Option<&'a Option<syn::Expr>> {
    items.iter().find(|(n, _)| n == name).map(|(_, v)| v)
}

fn str_value(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Lit(l) => match &l.lit {
            syn::Lit::Str(s) => Some(s.value()),
            _ => None,
        },
        _ => None,
    }
}

fn derives(attrs: &[syn::Attribute]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for attr in attrs.iter().filter(|a| a.path().is_ident("derive")) {
        let _ = attr.parse_nested_meta(|meta| {
            if let Some(last) = meta.path.segments.last() {
                out.insert(last.ident.to_string());
            }
            Ok(())
        });
    }
    out
}

/// `ScoreJev` and `score_jev` both become `score-jev`, as clap spells them.
fn kebab(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c == '_' {
            out.push('-');
        } else if c.is_uppercase() {
            if i > 0 {
                out.push('-');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// A type as written, with spaces only after commas: `Option<Vec<String>>`.
fn type_text(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(p) => p
            .path
            .segments
            .iter()
            .map(|segment| {
                let mut text = segment.ident.to_string();
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    let inner: Vec<String> = args
                        .args
                        .iter()
                        .map(|arg| match arg {
                            syn::GenericArgument::Type(t) => type_text(t),
                            _ => "_".to_string(),
                        })
                        .collect();
                    text.push_str(&format!("<{}>", inner.join(", ")));
                }
                text
            })
            .collect::<Vec<_>>()
            .join("::"),
        syn::Type::Reference(r) => format!("&{}", type_text(&r.elem)),
        syn::Type::Tuple(t) => format!(
            "({})",
            t.elems.iter().map(type_text).collect::<Vec<_>>().join(", ")
        ),
        syn::Type::Slice(s) => format!("[{}]", type_text(&s.elem)),
        syn::Type::Array(a) => format!("[{}]", type_text(&a.elem)),
        _ => "_".to_string(),
    }
}

/// The name of the type a field holds, looking through `Option`, `Vec`, `Box`, `Arc` and the value of a map.
fn type_ident(ty: &syn::Type) -> Option<String> {
    let syn::Type::Path(p) = ty else {
        return None;
    };
    let last = p.path.segments.last()?;
    let name = last.ident.to_string();
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return Some(name);
    };
    let mut types = args.args.iter().filter_map(|a| match a {
        syn::GenericArgument::Type(t) => Some(t),
        _ => None,
    });
    match name.as_str() {
        "Option" | "Vec" | "Box" | "Arc" => types.next().and_then(type_ident),
        "BTreeMap" | "HashMap" => types.nth(1).and_then(type_ident),
        _ => Some(name),
    }
}

fn join(prefix: &str, name: &str, separator: &str) -> String {
    if prefix.is_empty() {
        name.to_string()
    } else {
        format!("{prefix}{separator}{name}")
    }
}

fn named_fields(fields: &syn::Fields) -> Vec<&syn::Field> {
    match fields {
        syn::Fields::Named(named) => named.named.iter().collect(),
        _ => Vec::new(),
    }
}

// --- commands and flags ----------------------------------------------------------------------------------

fn read_cli(text: &str, inventory: &mut SurfaceInventory) -> Result<()> {
    let file = syn::parse_file(text)?;
    let mut enums = HashMap::new();
    let mut roots = Vec::new();
    for item in &file.items {
        match item {
            syn::Item::Enum(e) => {
                enums.insert(e.ident.to_string(), e);
            }
            syn::Item::Struct(s) if derives(&s.attrs).contains("Parser") => roots.push(s),
            _ => {}
        }
    }
    for root in roots {
        for field in named_fields(&root.fields) {
            walk_field(field, "", &enums, inventory, 0);
        }
    }
    Ok(())
}

fn is_subcommand(field: &syn::Field) -> bool {
    let items = items_of(&field.attrs, &["command", "clap"]);
    find_item(&items, "subcommand").is_some()
}

fn walk_field(
    field: &syn::Field,
    command: &str,
    enums: &HashMap<String, &syn::ItemEnum>,
    inventory: &mut SurfaceInventory,
    depth: usize,
) {
    if is_subcommand(field) {
        if let Some(name) = type_ident(&field.ty) {
            walk_enum(&name, command, enums, inventory, depth + 1);
        }
    } else {
        add_argument(field, command, inventory);
    }
}

fn walk_enum(
    name: &str,
    prefix: &str,
    enums: &HashMap<String, &syn::ItemEnum>,
    inventory: &mut SurfaceInventory,
    depth: usize,
) {
    let Some(e) = enums.get(name) else {
        return;
    };
    if depth > MAX_DEPTH {
        return;
    }
    for variant in &e.variants {
        let items = items_of(&variant.attrs, &["command", "clap"]);
        let own_name = find_item(&items, "name")
            .and_then(|v| v.as_ref())
            .and_then(str_value)
            .unwrap_or_else(|| kebab(&variant.ident.to_string()));
        let command = join(prefix, &own_name, " ");
        let visibility = if find_item(&items, "hide").is_some() {
            "hidden"
        } else {
            "visible"
        };
        inventory
            .commands
            .insert(command.clone(), visibility.to_string());
        for field in named_fields(&variant.fields) {
            walk_field(field, &command, enums, inventory, depth);
        }
    }
}

/// A flag (`--yes`, `-y`) or a positional argument (`<job>`) of `command`, with its type.
fn add_argument(field: &syn::Field, command: &str, inventory: &mut SurfaceInventory) {
    let Some(ident) = &field.ident else {
        return;
    };
    let own = ident.to_string().trim_start_matches("r#").to_string();
    let items = items_of(&field.attrs, &["arg", "clap"]);
    let name = if let Some(long) = find_item(&items, "long") {
        let spelled = long
            .as_ref()
            .and_then(str_value)
            .unwrap_or_else(|| kebab(&own));
        format!("--{spelled}")
    } else if let Some(short) = find_item(&items, "short") {
        let letter = match short {
            Some(syn::Expr::Lit(l)) => match &l.lit {
                syn::Lit::Char(c) => Some(c.value()),
                _ => None,
            },
            _ => None,
        }
        .or_else(|| own.chars().next())
        .unwrap_or('?');
        format!("-{letter}")
    } else {
        format!("<{}>", kebab(&own))
    };
    let required = matches!(
        find_item(&items, "required"),
        Some(None)
            | Some(Some(syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Bool(syn::LitBool { value: true, .. }),
                ..
            })))
    );
    let mut detail = type_text(&field.ty);
    if required {
        detail.push_str(" required");
    }
    inventory.flags.insert(join(command, &name, " "), detail);
}

// --- config keys -----------------------------------------------------------------------------------------

fn read_config(text: &str, inventory: &mut SurfaceInventory) -> Result<()> {
    let file = syn::parse_file(text)?;
    let structs: HashMap<String, &syn::ItemStruct> = file
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Struct(s) => Some((s.ident.to_string(), s)),
            _ => None,
        })
        .collect();
    walk_struct("Config", "", &structs, inventory, 0);
    Ok(())
}

fn walk_struct(
    name: &str,
    prefix: &str,
    structs: &HashMap<String, &syn::ItemStruct>,
    inventory: &mut SurfaceInventory,
    depth: usize,
) {
    let Some(s) = structs.get(name) else {
        return;
    };
    if depth > MAX_DEPTH {
        return;
    }
    for field in named_fields(&s.fields) {
        let serde = items_of(&field.attrs, &["serde"]);
        if find_item(&serde, "skip").is_some() || find_item(&serde, "flatten").is_some() {
            continue;
        }
        let key = find_item(&serde, "rename")
            .and_then(|v| v.as_ref())
            .and_then(str_value)
            .or_else(|| field.ident.as_ref().map(|i| i.to_string()));
        let Some(key) = key else {
            continue;
        };
        let full = join(prefix, &key, ".");
        match type_ident(&field.ty).filter(|t| structs.contains_key(t)) {
            Some(inner) => {
                inventory.config_keys.insert(full.clone(), "section".into());
                walk_struct(&inner, &full, structs, inventory, depth + 1);
            }
            None => {
                inventory.config_keys.insert(full, type_text(&field.ty));
            }
        }
    }
}

// --- record schemas --------------------------------------------------------------------------------------

fn read_records(text: &str, inventory: &mut SurfaceInventory) -> Result<()> {
    let file = syn::parse_file(text)?;
    let mut versions: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
    for item in &file.items {
        let syn::Item::Enum(e) = item else {
            continue;
        };
        if e.ident != "AnyRecord" {
            continue;
        }
        for variant in &e.variants {
            let serde = items_of(&variant.attrs, &["serde"]);
            let Some(tag) = find_item(&serde, "rename")
                .and_then(|v| v.as_ref())
                .and_then(str_value)
            else {
                continue;
            };
            if let Some((name, version)) = tag.rsplit_once('/')
                && let Ok(version) = version.parse::<u32>()
            {
                versions
                    .entry(name.to_string())
                    .or_default()
                    .insert(version);
            }
        }
    }
    for (name, set) in versions {
        let list: Vec<String> = set.iter().map(u32::to_string).collect();
        inventory.record_schemas.insert(name, list.join(","));
    }
    Ok(())
}

// --- roles -----------------------------------------------------------------------------------------------

/// Collects the `name` and `version` fields of every `RoleDefinition { .. }` literal.
struct RoleFinder<'a> {
    consts: &'a HashMap<String, String>,
    found: Vec<(Option<String>, Option<String>)>,
}

impl RoleFinder<'_> {
    /// The text of a name expression: a literal, a string constant, or either followed by `.into()`.
    fn name_of(&self, expr: &syn::Expr) -> Option<String> {
        match expr {
            syn::Expr::MethodCall(call) => self.name_of(&call.receiver),
            syn::Expr::Path(p) => p
                .path
                .get_ident()
                .and_then(|i| self.consts.get(&i.to_string()).cloned()),
            other => str_value(other),
        }
    }
}

impl<'ast> Visit<'ast> for RoleFinder<'_> {
    fn visit_expr_struct(&mut self, node: &'ast syn::ExprStruct) {
        if node
            .path
            .segments
            .last()
            .is_some_and(|s| s.ident == "RoleDefinition")
        {
            let field = |wanted: &str| {
                node.fields.iter().find_map(|f| match &f.member {
                    syn::Member::Named(n) if n == wanted => Some(&f.expr),
                    _ => None,
                })
            };
            let name = field("name").and_then(|e| self.name_of(e));
            let version = field("version").and_then(|e| match e {
                syn::Expr::Lit(l) => match &l.lit {
                    syn::Lit::Int(i) => Some(i.base10_digits().to_string()),
                    _ => None,
                },
                _ => None,
            });
            self.found.push((name, version));
        }
        syn::visit::visit_expr_struct(self, node);
    }
}

fn read_roles(text: &str, inventory: &mut SurfaceInventory) -> Result<()> {
    let file = syn::parse_file(text)?;
    let consts: HashMap<String, String> = file
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Const(c) => str_value(&c.expr).map(|v| (c.ident.to_string(), v)),
            _ => None,
        })
        .collect();
    let mut finder = RoleFinder {
        consts: &consts,
        found: Vec::new(),
    };
    for item in &file.items {
        if let syn::Item::Fn(f) = item
            && let syn::ReturnType::Type(_, ty) = &f.sig.output
            && type_ident(ty).as_deref() == Some("RoleDefinition")
        {
            finder.visit_block(&f.block);
        }
    }
    for (name, version) in finder.found {
        let name = name.unwrap_or_else(|| "?".to_string());
        inventory
            .roles
            .insert(name, version.unwrap_or_else(|| "?".to_string()));
    }
    Ok(())
}

// --- workspace crates ------------------------------------------------------------------------------------

fn read_crates(git: &Git, commit: &str, inventory: &mut SurfaceInventory) -> Result<()> {
    let Some(text) = source(git, commit, WORKSPACE_MANIFEST)? else {
        return Ok(());
    };
    let manifest: toml_edit::DocumentMut =
        text.parse().context("parsing the workspace Cargo.toml")?;
    let members: Vec<String> = manifest
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default();
    for member in members {
        for dir in expand_member(git, commit, &member)? {
            let name = match source(git, commit, &format!("{dir}/Cargo.toml"))? {
                Some(text) => text
                    .parse::<toml_edit::DocumentMut>()
                    .ok()
                    .and_then(|d| {
                        d.get("package")
                            .and_then(|p| p.get("name"))
                            .and_then(|n| n.as_str())
                            .map(String::from)
                    })
                    .unwrap_or_else(|| dir.clone()),
                None => dir.clone(),
            };
            inventory.crates.insert(name, dir);
        }
    }
    Ok(())
}

/// A workspace member as the directories it names: itself, or for `dir/*` each directory under `dir` that
/// holds a `Cargo.toml`.
fn expand_member(git: &Git, commit: &str, member: &str) -> Result<Vec<String>> {
    let Some(parent) = member.strip_suffix("/*") else {
        return Ok(vec![member.to_string()]);
    };
    let mut dirs = BTreeSet::new();
    for entry in git.ls_tree_under(commit, parent)? {
        if let Some(rest) = entry.path.strip_prefix(&format!("{parent}/"))
            && let Some(dir) = rest.strip_suffix("/Cargo.toml")
            && !dir.contains('/')
        {
            dirs.insert(format!("{parent}/{dir}"));
        }
    }
    Ok(dirs.into_iter().collect())
}

// ---------------------------------------------------------------------------------------------------------
// The diff

/// An item that exists on one side only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entry {
    pub name: String,
    pub detail: String,
}

/// An item on both sides whose detail differs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Change {
    pub name: String,
    pub from: String,
    pub to: String,
}

/// What differs within one kind, each list sorted by name.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct KindDiff {
    pub added: Vec<Entry>,
    pub removed: Vec<Entry>,
    pub changed: Vec<Change>,
}

impl KindDiff {
    /// Added, removed and changed items together.
    pub fn total(&self) -> usize {
        self.added.len() + self.removed.len() + self.changed.len()
    }
}

/// The counts over all kinds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Totals {
    pub added: usize,
    pub removed: usize,
    pub changed: usize,
    pub total: usize,
}

/// The difference between two inventories, per kind.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct SurfaceDiff {
    pub commands: KindDiff,
    pub flags: KindDiff,
    pub config_keys: KindDiff,
    pub record_schemas: KindDiff,
    pub roles: KindDiff,
    pub crates: KindDiff,
}

impl SurfaceDiff {
    /// Every kind with its name, in a fixed order.
    pub fn kinds(&self) -> [(&'static str, &KindDiff); 6] {
        [
            ("commands", &self.commands),
            ("flags", &self.flags),
            ("config keys", &self.config_keys),
            ("record schemas", &self.record_schemas),
            ("roles", &self.roles),
            ("crates", &self.crates),
        ]
    }

    pub fn totals(&self) -> Totals {
        let mut totals = Totals::default();
        for (_, kind) in self.kinds() {
            totals.added += kind.added.len();
            totals.removed += kind.removed.len();
            totals.changed += kind.changed.len();
        }
        totals.total = totals.added + totals.removed + totals.changed;
        totals
    }

    /// True when the two inventories are the same.
    pub fn is_empty(&self) -> bool {
        self.totals().total == 0
    }
}

fn diff_kind(old: &BTreeMap<String, String>, new: &BTreeMap<String, String>) -> KindDiff {
    let mut out = KindDiff::default();
    for (name, detail) in new {
        match old.get(name) {
            None => out.added.push(Entry {
                name: name.clone(),
                detail: detail.clone(),
            }),
            Some(before) if before != detail => out.changed.push(Change {
                name: name.clone(),
                from: before.clone(),
                to: detail.clone(),
            }),
            Some(_) => {}
        }
    }
    for (name, detail) in old {
        if !new.contains_key(name) {
            out.removed.push(Entry {
                name: name.clone(),
                detail: detail.clone(),
            });
        }
    }
    out
}

/// Folds the keys under an added or removed section into the section's own entry.
fn fold_sections(entries: Vec<Entry>) -> Vec<Entry> {
    let sections: BTreeSet<String> = entries
        .iter()
        .filter(|e| e.detail == "section")
        .map(|e| e.name.clone())
        .collect();
    let covered = |name: &str| {
        name.match_indices('.')
            .any(|(at, _)| sections.contains(&name[..at]))
    };
    let mut kept: Vec<Entry> = entries
        .iter()
        .filter(|e| !covered(&e.name))
        .cloned()
        .collect();
    for entry in &mut kept {
        if entry.detail == "section" {
            let inside = entries
                .iter()
                .filter(|e| e.name.starts_with(&format!("{}.", entry.name)))
                .count();
            if inside > 0 {
                entry.detail = format!("section ({inside} keys)");
            }
        }
    }
    kept
}

/// What changed from `old` to `new`: added, removed and changed items per kind, and (through
/// [`SurfaceDiff::totals`]) the totals.
pub fn diff(old: &SurfaceInventory, new: &SurfaceInventory) -> SurfaceDiff {
    let mut config_keys = diff_kind(&old.config_keys, &new.config_keys);
    config_keys.added = fold_sections(config_keys.added);
    config_keys.removed = fold_sections(config_keys.removed);
    SurfaceDiff {
        commands: diff_kind(&old.commands, &new.commands),
        flags: diff_kind(&old.flags, &new.flags),
        config_keys,
        record_schemas: diff_kind(&old.record_schemas, &new.record_schemas),
        roles: diff_kind(&old.roles, &new.roles),
        crates: diff_kind(&old.crates, &new.crates),
    }
}

// ---------------------------------------------------------------------------------------------------------
// The record

impl Athena {
    /// Computes the design surface at `commit` and commits it as an `athena.design-surface/1` record.
    /// Nothing else is changed. Compare against an earlier one with [`diff`].
    pub fn record_design_surface(&self, commit: &str, now: Timestamp) -> Result<DesignSurface> {
        let inventory = inventory_at(&self.git, commit)?;
        let known = crate::store::Records::load(&self.git, &self.head()?)?;
        let mut id = RecordId::generate(now);
        while known.has_record_id(&id) {
            id = RecordId::generate(now);
        }
        let commit_id = self
            .git
            .rev_parse(commit)?
            .with_context(|| format!("{commit} is not a commit"))?;
        let surface = DesignSurface {
            id,
            at: now,
            commit: commit_id,
            inventory,
        };
        let record = AnyRecord::DesignSurface(Box::new(surface.clone()));
        self.commit_records(
            &[NewFile::record(&record)],
            &message(
                &format!("Design surface at {}", &surface.commit[..12]),
                "The commands, flags, config keys, record schemas, roles and crates at that commit, so a later review can compare against it.",
                &[("Athena-Record", record.canonical_path())],
            ),
        )?;
        Ok(surface)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code_health::tests::set_up;
    use std::fs;

    const MAIN: &str = r#"
#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Ask.
    Ask {
        #[arg(required = true, num_args = 1..)]
        request: Vec<String>,
        #[arg(long)]
        yes: bool,
    },
    ScoreJev {
        #[arg(long, value_name = "T")]
        cut: Option<String>,
    },
    Hestia {
        #[command(subcommand)]
        action: HestiaAction,
    },
    #[command(hide = true)]
    Hook,
}

#[derive(Subcommand, Debug)]
enum HestiaAction {
    Sweep {
        #[arg(long = "as-json", short = 'j')]
        json: bool,
    },
}
"#;

    const CONFIG: &str = r#"
#[serde(deny_unknown_fields)]
pub struct Config {
    pub athena: AthenaSection,
    pub budget: Budget,
    pub names: BTreeMap<String, String>,
    pub parallel: Option<Parallel>,
}
pub struct AthenaSection { pub schema: u32 }
pub struct Budget { pub reader_usd: f64, pub job_usd: f64 }
pub struct Parallel { pub max_concurrency: u32 }
"#;

    const RECORDS: &str = r#"
pub enum AnyRecord {
    #[serde(rename = "athena.role/1")]
    Role(Box<Role>),
    #[serde(rename = "athena.candidate/1")]
    Candidate(Box<Candidate>),
    #[serde(rename = "athena.candidate/2")]
    Candidate2(Box<Candidate>),
}
"#;

    const ROLES: &str = r#"
pub const STEWARD: &str = "steward";
pub fn steward(at: Timestamp) -> RoleDefinition {
    RoleDefinition { name: STEWARD.into(), version: 2, at }
}
pub fn audience(at: Timestamp) -> RoleDefinition {
    RoleDefinition { name: "audience".into(), version: 1, at }
}
"#;

    const MANIFEST: &str = "[workspace]\nmembers = [\"crates/a\", \"crates/b\"]\n";

    fn base_files() -> Vec<(&'static str, String)> {
        vec![
            (CLI_SOURCE, MAIN.to_string()),
            (CONFIG_SOURCE, CONFIG.to_string()),
            (RECORDS_SOURCE, RECORDS.to_string()),
            (ROLES_SOURCE, ROLES.to_string()),
            (WORKSPACE_MANIFEST, MANIFEST.to_string()),
            (
                "crates/a/Cargo.toml",
                "[package]\nname = \"alpha\"\n".into(),
            ),
            ("crates/b/Cargo.toml", "[package]\nname = \"beta\"\n".into()),
        ]
    }

    fn commit_files(git: &Git, files: &[(&str, String)]) -> String {
        for (path, text) in files {
            let full = git.dir().join(path);
            fs::create_dir_all(full.parent().unwrap()).unwrap();
            fs::write(full, text).unwrap();
        }
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "change"]).unwrap();
        git.head().unwrap().unwrap()
    }

    fn scratch() -> (tempfile::TempDir, Git) {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::isolated(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        (dir, git)
    }

    #[test]
    fn the_inventory_lists_commands_flags_keys_schemas_roles_and_crates() {
        let (_dir, git) = scratch();
        let head = commit_files(&git, &base_files());
        let inv = inventory_at(&git, &head).unwrap();
        let keys = |m: &BTreeMap<String, String>| m.keys().cloned().collect::<Vec<_>>();
        assert_eq!(
            keys(&inv.commands),
            ["ask", "hestia", "hestia sweep", "hook", "score-jev"]
        );
        assert_eq!(inv.commands["hook"], "hidden");
        assert_eq!(inv.commands["ask"], "visible");
        assert_eq!(
            keys(&inv.flags),
            [
                "ask --yes",
                "ask <request>",
                "hestia sweep --as-json",
                "score-jev --cut"
            ]
        );
        assert_eq!(inv.flags["ask <request>"], "Vec<String> required");
        assert_eq!(inv.flags["score-jev --cut"], "Option<String>");
        assert_eq!(inv.config_keys["athena"], "section");
        assert_eq!(inv.config_keys["athena.schema"], "u32");
        assert_eq!(inv.config_keys["budget.job_usd"], "f64");
        assert_eq!(inv.config_keys["names"], "BTreeMap<String, String>");
        assert_eq!(inv.config_keys["parallel.max_concurrency"], "u32");
        assert_eq!(inv.record_schemas["athena.candidate"], "1,2");
        assert_eq!(inv.record_schemas["athena.role"], "1");
        assert_eq!(inv.roles["steward"], "2");
        assert_eq!(inv.roles["audience"], "1");
        assert_eq!(inv.crates["alpha"], "crates/a");
        assert_eq!(inv.crates["beta"], "crates/b");
    }

    #[test]
    fn the_inventory_is_read_from_the_commit_and_not_the_working_tree() {
        let (_dir, git) = scratch();
        let first = commit_files(&git, &base_files());
        let before = inventory_at(&git, &first).unwrap();
        fs::write(git.dir().join(CLI_SOURCE), "enum Command {}").unwrap();
        assert_eq!(inventory_at(&git, &first).unwrap(), before);
    }

    #[test]
    fn the_inventory_is_identical_when_computed_twice_at_one_commit() {
        let (_dir, git) = scratch();
        let head = commit_files(&git, &base_files());
        let one = inventory_at(&git, &head).unwrap();
        let two = inventory_at(&git, &head).unwrap();
        assert!(!one.is_empty());
        assert_eq!(one, two);
        assert_eq!(
            serde_json::to_vec(&one).unwrap(),
            serde_json::to_vec(&two).unwrap()
        );
    }

    #[test]
    fn a_command_added_a_section_removed_and_a_schema_moved_are_exactly_three_changes() {
        let (_dir, git) = scratch();
        let first = commit_files(&git, &base_files());
        let next = commit_files(
            &git,
            &[
                (CLI_SOURCE, MAIN.replace("    Hook,\n", "    Hook,\n    Status,\n")),
                (
                    CONFIG_SOURCE,
                    CONFIG
                        .replace("    pub budget: Budget,\n", "")
                        .replace("pub struct Budget { pub reader_usd: f64, pub job_usd: f64 }\n", ""),
                ),
                (
                    RECORDS_SOURCE,
                    RECORDS
                        .replace("athena.candidate/1", "athena.candidate/2")
                        .replace(
                            "    #[serde(rename = \"athena.candidate/2\")]\n    Candidate2(Box<Candidate>),\n",
                            "",
                        ),
                ),
            ],
        );
        let changes = diff(
            &inventory_at(&git, &first).unwrap(),
            &inventory_at(&git, &next).unwrap(),
        );
        let entry = |name: &str, detail: &str| Entry {
            name: name.into(),
            detail: detail.into(),
        };
        assert_eq!(changes.commands.added, [entry("status", "visible")]);
        assert_eq!(
            changes.config_keys.removed,
            [entry("budget", "section (2 keys)")]
        );
        assert_eq!(
            changes.record_schemas.changed,
            [Change {
                name: "athena.candidate".into(),
                from: "1,2".into(),
                to: "2".into()
            }]
        );
        let expected = SurfaceDiff {
            commands: KindDiff {
                added: vec![entry("status", "visible")],
                ..KindDiff::default()
            },
            config_keys: KindDiff {
                removed: vec![entry("budget", "section (2 keys)")],
                ..KindDiff::default()
            },
            record_schemas: changes.record_schemas.clone(),
            ..SurfaceDiff::default()
        };
        assert_eq!(changes, expected);
        assert_eq!(
            changes.totals(),
            Totals {
                added: 1,
                removed: 1,
                changed: 1,
                total: 3
            }
        );
    }

    #[test]
    fn flags_roles_and_crates_show_as_added_removed_and_changed() {
        let (_dir, git) = scratch();
        let first = commit_files(&git, &base_files());
        let next = commit_files(
            &git,
            &[
                (CLI_SOURCE, MAIN.replace("yes: bool", "yes: u8")),
                (ROLES_SOURCE, ROLES.replace("version: 2", "version: 3")),
                (WORKSPACE_MANIFEST, MANIFEST.replace(", \"crates/b\"", "")),
            ],
        );
        let changes = diff(
            &inventory_at(&git, &first).unwrap(),
            &inventory_at(&git, &next).unwrap(),
        );
        assert_eq!(changes.flags.changed.len(), 1);
        assert_eq!(changes.flags.changed[0].name, "ask --yes");
        assert_eq!(changes.roles.changed[0].to, "3");
        assert_eq!(changes.crates.removed.len(), 1);
        assert_eq!(changes.crates.removed[0].name, "beta");
        assert_eq!(changes.totals().total, 3);
        assert!(diff(&SurfaceInventory::default(), &SurfaceInventory::default()).is_empty());
    }

    #[test]
    fn a_source_that_does_not_parse_is_an_error_and_an_absent_one_is_empty() {
        let (_dir, git) = scratch();
        let broken = commit_files(&git, &[(CLI_SOURCE, "fn (".to_string())]);
        assert!(inventory_at(&git, &broken).is_err());
        let (_dir, git) = scratch();
        let bare = commit_files(&git, &[("README.md", "x".to_string())]);
        assert!(inventory_at(&git, &bare).unwrap().is_empty());
    }

    #[test]
    fn a_design_surface_is_recorded_and_read_back_to_compare_against() {
        let (_dir, _state, athena) = set_up();
        let head = commit_files(&athena.git, &base_files());
        let now = crate::now();
        let surface = athena.record_design_surface(&head, now).unwrap();
        assert_eq!(surface.commit, head);
        assert_eq!(surface.inventory, inventory_at(&athena.git, &head).unwrap());
        let records = crate::store::Records::load(&athena.git, &athena.head().unwrap()).unwrap();
        assert!(records.invalid.is_empty(), "{:?}", records.invalid);
        assert_eq!(records.design_surfaces.len(), 1);
        assert_eq!(records.design_surfaces[0], surface);
        // Later, compare the newest recorded surface with the surface at a new commit.
        let moved = commit_files(
            &athena.git,
            &[(
                CLI_SOURCE,
                MAIN.replace("    Hook,\n", "    Hook,\n    Status,\n"),
            )],
        );
        let latest = DesignSurface::latest(&records.design_surfaces).unwrap();
        let changes = diff(
            &latest.inventory,
            &inventory_at(&athena.git, &moved).unwrap(),
        );
        assert_eq!(changes.totals().total, 1);
        assert_eq!(changes.commands.added[0].name, "status");
    }
}
