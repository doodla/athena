//! Rendering of findings and plans: the shared blocks the planner, critic, producer and verifier
//! briefs print for a plan and for the findings on it.

use super::{VerifierPlan, action, list, quote};
use athena_data::records::{
    Demonstration, DependentKind, Finding, FindingSource, JobPlan, Severity,
};
use std::fmt::Write;

fn severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Blocker => "blocker",
        Severity::Major => "major",
        Severity::Minor => "minor",
        Severity::Nit => "nit",
    }
}

fn finding_source(source: FindingSource) -> &'static str {
    match source {
        FindingSource::Verifier => "verifier",
        FindingSource::Athena => "athena",
        FindingSource::Critic => "critic",
    }
}

/// Findings, one block each, quoted as data. `empty` is written when there are none.
pub(super) fn findings_list(out: &mut String, findings: &[Finding], empty: &str) {
    if findings.is_empty() {
        let _ = writeln!(out, "{empty}");
    }
    for (i, f) in findings.iter().enumerate() {
        let location = f
            .location
            .as_ref()
            .map_or(String::new(), |l| format!(" at {l}"));
        let _ = writeln!(
            out,
            "{}. [{}] ({}){location}: {}\n{}",
            i + 1,
            severity(f.severity),
            finding_source(f.source),
            f.summary.trim(),
            quote(&format!("Evidence: {}", f.evidence.trim()))
        );
    }
}

/// The plan, in full, under the heading level the caller chose: used by the revision brief, the critic's brief
/// and the producer's section. Deterministic: the same record gives the same bytes.
pub(super) fn plan_body(out: &mut String, plan: &JobPlan) {
    let _ = writeln!(out, "Approach: {}\n", plan.approach.trim());
    let _ = writeln!(out, "Decisions:");
    if plan.decisions.is_empty() {
        let _ = writeln!(out, "(none)");
    }
    for (i, d) in plan.decisions.iter().enumerate() {
        let _ = writeln!(
            out,
            "{}. {} Reason: {}",
            i + 1,
            d.choice.trim(),
            d.reason.trim()
        );
        for r in &d.rejected {
            let _ = writeln!(
                out,
                "   - Rejected: {} Why: {}",
                r.option.trim(),
                r.why.trim()
            );
        }
    }
    let _ = writeln!(out, "\nFiles:");
    if plan.files.is_empty() {
        let _ = writeln!(out, "(none)");
    }
    for f in &plan.files {
        let _ = writeln!(
            out,
            "- {} ({}, {}{}): {}",
            f.path,
            action(f.action),
            if f.exists { "exists" } else { "new" },
            if f.governance {
                ", a governance file"
            } else {
                ""
            },
            f.purpose.trim()
        );
    }
    let _ = writeln!(out, "\nSteps:");
    for (i, s) in plan.steps.iter().enumerate() {
        let _ = writeln!(out, "{}. {}", i + 1, s.trim());
    }
    let _ = writeln!(out, "\nRisks:");
    if plan.risks.is_empty() {
        let _ = writeln!(out, "(none)");
    }
    for r in &plan.risks {
        let _ = writeln!(
            out,
            "- {} Mitigation: {}",
            r.risk.trim(),
            r.mitigation.trim()
        );
    }
    // Printed only when present, so a plan recorded before the field existed renders as it always did.
    if !plan.dependents.is_empty() {
        let _ = writeln!(out, "\nDependents:");
        for d in &plan.dependents {
            let kind = match d.kind {
                DependentKind::Code => "code",
                DependentKind::Rule => "rule",
                DependentKind::HostSetting => "host setting",
            };
            let mut line = format!("- {}: {} ({kind})", d.name.trim(), d.dependent.trim());
            for (label, text) in [("Test", &d.test), ("Reason", &d.reason)] {
                if let Some(t) = text.as_deref().filter(|t| !t.trim().is_empty()) {
                    let _ = write!(line, " {label}: {}", t.trim());
                }
            }
            let _ = writeln!(out, "{line}");
        }
    }
    let _ = writeln!(out, "\nDemonstrations:");
    demonstration_lines(out, &plan.demonstrations);
    let _ = writeln!(out, "\nOpen questions:");
    list(out, &plan.open_questions, "(none)");
}

pub(super) fn demonstration_lines(out: &mut String, demonstrations: &[Demonstration]) {
    for d in demonstrations {
        let _ = writeln!(
            out,
            "- Criterion {}: {} Run: `{}`. Observes: {}",
            d.criterion,
            d.method.trim(),
            d.command.trim(),
            d.observes.trim()
        );
        if let Some(f) = &d.fails_without {
            let _ = writeln!(out, "  Fails without the behavior: {}", f.trim());
        }
        if let Some(s) = &d.seam {
            let _ = writeln!(out, "  Test seam: {}", s.trim());
        }
    }
}

/// The producer's "Job plan (accepted, round r)" section, rendered from the accepted plan. Deterministic,
/// so its SHA-256 in the plan record rebuilds the producer's exact prompt (review F6).
pub fn render_plan_section(plan: &JobPlan) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "## Job plan (accepted, round {})\n\nA planner wrote this plan and a critic accepted it. Follow it. You may depart \
         from it only with a reason, recorded in open_issues. The verifier runs the demonstrations below.\n",
        plan.round
    );
    plan_body(&mut out, plan);
    out
}

/// The producer's prompt for a planned job: the committed producer brief, then the rendered plan section.
/// The send-back is built from this text, not from the brief alone (review F6).
pub fn producer_prompt_with_plan(brief: &str, section: &str) -> String {
    format!("{brief}\n\n{section}")
}

/// The verifier's section for an accepted plan: the demonstrations and the planned files, nothing of the
/// planner's reasoning (review F5).
pub(super) fn verifier_plan_section(out: &mut String, plan: &VerifierPlan) {
    let _ = writeln!(
        out,
        "\n## Planned demonstrations (round {})\n\nA critic accepted these demonstrations before the change was made. Judge the request and \
         the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a \
         planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.\n",
        plan.round
    );
    demonstration_lines(out, &plan.demonstrations);
    let _ = writeln!(out, "\nPlanned files:\n");
    for (path, a) in &plan.files {
        let _ = writeln!(out, "- {path} ({})", action(*a));
    }
    out.push('\n');
}
