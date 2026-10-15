//! Roles and the requirements ladder (`docs/design/roles-and-ladder.md`).
//!
//! A role is a recorded definition: a name, a goal, a stable prompt, how often it runs (counted in finished
//! tasks), and the highest rung of the ladder it may change by itself. The ladder orders what Athena's
//! requirements are made of, from the owner's goals down to the code. A role may act on its own only on the
//! lower rungs; above its limit it queues a proposal with evidence.

use serde::{Deserialize, Serialize};

/// The five rungs of the requirements ladder, highest first. The derived order puts the top rung first, so
/// `OwnerGoals < Implementation`: a smaller rung is a higher one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Ladder {
    /// What the owner wants Athena to be for.
    OwnerGoals,
    /// What `BRIEF.md` requires.
    BriefRequirements,
    /// The recorded design decisions.
    DesignDecisions,
    /// The user's defaults and the repository's conventions.
    DefaultsConventions,
    /// The code, tests and docs.
    Implementation,
}

impl Ladder {
    /// Every rung, highest first.
    pub const ALL: [Ladder; 5] = [
        Ladder::OwnerGoals,
        Ladder::BriefRequirements,
        Ladder::DesignDecisions,
        Ladder::DefaultsConventions,
        Ladder::Implementation,
    ];

    /// The highest rung any role may change by itself, whatever its own limit says.
    pub const HIGHEST_SELF_CHANGE: Ladder = Ladder::DesignDecisions;

    pub fn as_str(self) -> &'static str {
        match self {
            Ladder::OwnerGoals => "owner-goals",
            Ladder::BriefRequirements => "brief-requirements",
            Ladder::DesignDecisions => "design-decisions",
            Ladder::DefaultsConventions => "defaults-conventions",
            Ladder::Implementation => "implementation",
        }
    }

    pub fn parse(text: &str) -> Option<Ladder> {
        Ladder::ALL.into_iter().find(|r| r.as_str() == text)
    }
}

/// Whether a role whose highest self-change rung is `limit` may change `rung` itself. It may when the rung is
/// at or below the limit and is not one of the top two rungs; otherwise the role queues a proposal.
pub fn may_change(limit: Ladder, rung: Ladder) -> bool {
    rung >= Ladder::HIGHEST_SELF_CHANGE && rung >= limit
}

/// The schema tag of a role definition.
pub const ROLE_SCHEMA: &str = "athena.role/1";

/// `athena.role/1`: a role, as recorded. A change to a role is a new record with the next `version`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoleDefinition {
    /// Lowercase letters, digits and hyphens.
    pub name: String,
    pub version: u32,
    pub at: jiff::Timestamp,
    /// One line: what the role is for.
    pub goal: String,
    /// The role's standing instruction. Short and stable.
    pub prompt: String,
    /// The role runs after this many finished tasks since its last run.
    pub cadence_tasks: u32,
    /// The highest rung the role may change itself; above it, the role queues a proposal.
    pub highest_rung: Ladder,
}

impl RoleDefinition {
    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty()
            || !self
                .name
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err("a role's name is lowercase letters, digits and hyphens".into());
        }
        if self.version == 0 {
            return Err("a role's version starts at 1".into());
        }
        if self.goal.trim().is_empty() || self.prompt.trim().is_empty() {
            return Err("a role has a goal and a prompt".into());
        }
        if self.cadence_tasks == 0 {
            return Err("a role's cadence is at least one finished task".into());
        }
        Ok(())
    }

    /// Whether this role may change `rung` itself.
    pub fn may_change(&self, rung: Ladder) -> bool {
        may_change(self.highest_rung, rung)
    }

    /// Where the record lives.
    pub fn path(&self) -> String {
        format!("records/roles/{}-{}.json", self.name, self.version)
    }
}

/// The steward's name.
pub const STEWARD: &str = "steward";

/// The steward's prompt. It stays fixed so that a change to it is a new recorded version.
pub const STEWARD_PROMPT: &str = "You are Athena's steward. You only read: Athena's code, its records, \
its git log, and GitHub runs, pull requests and deploys. Report a work item for each repeated failure and \
for each source you could not read. Cite sources and evidence by reference; copy no data. Name the rung of \
the requirements ladder each item concerns; Athena queues a request for a rung at or below your limit and \
only a proposal above it.";

/// The one steward definition: it runs every five finished tasks, and may change the lowest two rungs.
/// Version 2 is the first that calls a model.
pub fn steward(at: jiff::Timestamp) -> RoleDefinition {
    RoleDefinition {
        name: STEWARD.into(),
        version: 2,
        at,
        goal: "Keep Athena's work steered: read what Athena has done, notice repeated failures and blind spots, and queue work for them."
            .into(),
        prompt: STEWARD_PROMPT.into(),
        cadence_tasks: 5,
        highest_rung: Ladder::DefaultsConventions,
    }
}

/// The audience's name.
pub const AUDIENCE: &str = "audience";

/// The audience's prompt. It stays fixed so that a change to it is a new recorded version.
pub const AUDIENCE_PROMPT: &str = "You are Athena's audience. You only read: the page you are given, as the \
reader named in your brief would meet it for the first time. Say whether that reader gets what they came \
for. Give one rating from 1 (cannot use it) to 5 (got what they came for). Give findings, and cite the \
evidence for every one: a quoted passage or the heading where you saw it. Copy no data beyond that. Change \
nothing.";

/// The one audience definition: it rates the live explainer page. It reads and records only, so it may change
/// nothing above the implementation rung, and in practice changes nothing at all. The cadence is how many
/// finished tasks should pass between runs; `athena audience` runs it on request.
pub fn audience(at: jiff::Timestamp) -> RoleDefinition {
    RoleDefinition {
        name: AUDIENCE.into(),
        version: 1,
        at,
        goal: "Read a shipped outcome as its audience would, rate how well it serves them, and cite the evidence for each finding."
            .into(),
        prompt: AUDIENCE_PROMPT.into(),
        cadence_tasks: 20,
        highest_rung: Ladder::Implementation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at() -> jiff::Timestamp {
        "2026-10-14T00:00:00Z".parse().unwrap()
    }

    #[test]
    fn the_audience_is_valid_and_asks_for_a_rating_with_evidence() {
        let role = audience(at());
        assert_eq!(role.validate(), Ok(()));
        assert_eq!(role.name, "audience");
        assert_eq!(role.path(), "records/roles/audience-1.json");
        assert!(role.cadence_tasks > 0);
        assert!(!role.goal.is_empty());
        for needle in ["1", "5", "rating", "evidence", "for the first time"] {
            assert!(role.prompt.contains(needle), "{needle}");
        }
        assert!(
            role.prompt.len() < 600,
            "the prompt is short: {}",
            role.prompt.len()
        );
    }

    #[test]
    fn the_ladder_is_ordered_highest_first() {
        assert_eq!(Ladder::ALL.len(), 5);
        assert!(Ladder::ALL.windows(2).all(|w| w[0] < w[1]));
        for rung in Ladder::ALL {
            assert_eq!(Ladder::parse(rung.as_str()), Some(rung));
        }
    }

    #[test]
    fn nobody_changes_the_top_two_rungs_whatever_the_limit() {
        for limit in Ladder::ALL {
            assert!(!may_change(limit, Ladder::OwnerGoals));
            assert!(!may_change(limit, Ladder::BriefRequirements));
        }
    }

    #[test]
    fn a_role_changes_only_rungs_at_or_below_its_limit() {
        assert!(may_change(Ladder::OwnerGoals, Ladder::DesignDecisions));
        assert!(may_change(Ladder::DesignDecisions, Ladder::DesignDecisions));
        assert!(!may_change(
            Ladder::DefaultsConventions,
            Ladder::DesignDecisions
        ));
        assert!(may_change(
            Ladder::DefaultsConventions,
            Ladder::DefaultsConventions
        ));
        assert!(may_change(
            Ladder::DefaultsConventions,
            Ladder::Implementation
        ));
        assert!(!may_change(
            Ladder::Implementation,
            Ladder::DefaultsConventions
        ));
    }

    #[test]
    fn the_steward_is_valid_and_limited_to_the_lowest_two_rungs() {
        let role = steward(at());
        assert_eq!(role.validate(), Ok(()));
        assert_eq!(role.cadence_tasks, 5);
        assert_eq!(role.path(), "records/roles/steward-2.json");
        assert!(!role.may_change(Ladder::DesignDecisions));
        assert!(role.may_change(Ladder::DefaultsConventions));
        assert!(role.may_change(Ladder::Implementation));
        assert!(
            role.prompt.len() < 500,
            "the prompt is short: {}",
            role.prompt.len()
        );
    }

    #[test]
    fn a_bad_role_is_refused() {
        let good = steward(at());
        for bad in [
            RoleDefinition {
                name: "Steward".into(),
                ..good.clone()
            },
            RoleDefinition {
                version: 0,
                ..good.clone()
            },
            RoleDefinition {
                prompt: " ".into(),
                ..good.clone()
            },
            RoleDefinition {
                cadence_tasks: 0,
                ..good
            },
        ] {
            assert!(bad.validate().is_err(), "{bad:?}");
        }
    }
}
