//! The automation register (OLY-B-A9-01): each automation is registered with its cost, its measured savings and
//! the toil or repeat cause it came from, and the ones that cost more than they save are listed for retirement.
//!
//! The rule is a pure function over a list of entries. It is not wired into the drain or any gate.

/// One registered automation. Cost and savings are in minutes over the same period, so they compare directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Automation {
    pub name: String,
    pub owner: String,
    /// Minutes to build and run it over the period.
    pub cost_minutes: u64,
    /// Measured minutes it saved over the same period; `None` when no savings were registered.
    pub savings_minutes: Option<u64>,
    /// The toil or repeat cause it takes on.
    pub source: String,
}

/// The outcome of reviewing a register.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Review<'a> {
    /// Entries whose cost is greater than their measured savings.
    pub retire: Vec<&'a Automation>,
    /// Entries with no registered savings; they cannot be judged and must be completed.
    pub unregistered: Vec<&'a Automation>,
}

/// Why an entry cannot be registered.
pub fn registration_problem(entry: &Automation) -> Option<String> {
    let blank = |s: &str| s.trim().is_empty();
    if blank(&entry.name) {
        return Some("the automation has no name".into());
    }
    if blank(&entry.owner) {
        return Some(format!("{}: no owner", entry.name));
    }
    if blank(&entry.source) {
        return Some(format!("{}: no toil or repeat cause named", entry.name));
    }
    if entry.savings_minutes.is_none() {
        return Some(format!("{}: no savings registered", entry.name));
    }
    None
}

/// Adds `entry` to `register`, refusing one that lacks a name, an owner, a source or savings.
pub fn register(register: &mut Vec<Automation>, entry: Automation) -> Result<(), String> {
    match registration_problem(&entry) {
        Some(problem) => Err(problem),
        None => {
            register.push(entry);
            Ok(())
        }
    }
}

/// Lists the entries to retire (cost above savings) and the entries with no savings registered.
pub fn review(entries: &[Automation]) -> Review<'_> {
    let mut out = Review::default();
    for entry in entries {
        match entry.savings_minutes {
            None => out.unregistered.push(entry),
            Some(saved) if entry.cost_minutes > saved => out.retire.push(entry),
            Some(_) => {}
        }
    }
    out
}
