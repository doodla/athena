//! The `athena status` views: the status of the work, the evidence snapshot's status, and the blocks they
//! are built from. Split out of `display.rs`; the helpers they share stay there.

use super::{
    gate_counts_lines, keep_rule_lines, mistakes, planning_cost_lines, producer_settings_lines, usd,
};
use athena_core::status::StatusView;
use std::fmt::Write;

/// The priority and plan state at the end of a job's line. The priority shows only when it is not normal
/// (absence means normal); the plan state is empty for a job that was not planned.
pub(super) fn plan_suffix(j: &athena_core::status::JobLine) -> String {
    let priority = if j.priority == athena_core::plan::Priority::Normal {
        String::new()
    } else {
        format!(" · priority {}", j.priority.as_str())
    };
    let plan = j
        .plan
        .as_ref()
        .map_or(String::new(), |plan| format!(" · {plan}"));
    let hold = j.hold.as_ref().map_or(String::new(), |hold| {
        if j.state == "interrupted" {
            format!(" · {hold} (held; the next drain resumes it from its kept candidate)")
        } else {
            format!(" · {hold} (still queued)")
        }
    });
    let retried = j
        .retried_as
        .as_ref()
        .map_or(String::new(), |new| format!(" · retried as {new}"));
    format!("{priority}{plan}{hold}{retried}")
}

/// The lines of work the scope guard flagged or paused, one block each: level, first id and signals (T1 to
/// T5), the numbers in words, and for a pause what lifts it. Empty text when there are none.
pub(super) fn lines_section(lines: &[athena_core::scope_guard::LineMark]) -> String {
    let mut out = String::new();
    for (i, l) in lines.iter().enumerate() {
        let label = if i == 0 { "lines" } else { "" };
        let level = if l.is_pause() { "paused" } else { "flagged" };
        let _ = writeln!(out, "{label:<11} {level} {} · {}", l.line, l.signal_names());
        for s in &l.signals {
            let _ = writeln!(out, "{:<11}   {} {}: {}", "", s.signal, s.level, s.detail);
        }
        if l.is_pause() {
            let _ = writeln!(out, "{:<11}   to lift: {}", "", l.lift);
        }
    }
    out
}

/// The open repeated causes of Hestia's error review: each with how many times and in how many jobs it
/// occurred, and the request that addresses it.
/// The `trend` line of `athena status`: the classes of work that improved and the ones that got worse.
pub(super) fn trend_section(trend: &athena_core::class_trend::ClassTrend) -> String {
    let mut out = String::new();
    for (i, line) in trend.lines().iter().enumerate() {
        if i == 0 {
            let _ = writeln!(out, "trend       {line}");
        } else {
            let _ = writeln!(out, "           {line}");
        }
    }
    out
}

pub(super) fn repeated_causes_section(causes: &[athena_core::hestia::RepeatedCause]) -> String {
    let mut out = String::new();
    for (i, c) in causes.iter().enumerate() {
        let label = if i == 0 { "repeated" } else { "" };
        let request = match &c.request {
            Some(r) => format!("request {} ({})", r.job, r.state),
            None => "no request yet".to_string(),
        };
        let _ = writeln!(
            out,
            "{label:<11} {} time(s) in {} jobs · {} · {request}",
            c.count(),
            c.jobs.len(),
            c.cause
        );
        if let Some(class) = &c.class {
            let _ = writeln!(out, "{:<11}   class: {class}", "");
        }
        let _ = writeln!(
            out,
            "{:<11}   introspections: {}",
            "",
            c.introspections.join(", ")
        );
    }
    out
}

/// The repeated causes whose stage `athena.toml` has switched off: apart from the others, because the error
/// review backfill does not queue a request for them.
pub(super) fn dormant_causes_section(causes: &[athena_core::hestia::DormantCause]) -> String {
    let mut out = String::new();
    for (i, d) in causes.iter().enumerate() {
        let label = if i == 0 { "dormant" } else { "" };
        let _ = writeln!(
            out,
            "{label:<11} {} time(s) in {} jobs · {} · {} stage is off in athena.toml, no request queued",
            d.cause.count(),
            d.cause.jobs.len(),
            d.cause.cause,
            d.stage.name()
        );
        let _ = writeln!(
            out,
            "{:<11}   introspections: {}",
            "",
            d.cause.introspections.join(", ")
        );
    }
    out
}

/// `athena status --snapshot <dir>`: the evidence snapshot's base, history, job counts and throughput. The
/// throughput block is the one `athena status` prints, from the same text function.
pub fn snapshot_status(v: &athena_core::snapshot::SnapshotView) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "snapshot    {}", v.dir);
    let _ = writeln!(out, "base        {}", &v.base[..v.base.len().min(12)]);
    let commit = |c: &athena_core::snapshot::HistoryEntry| {
        format!(
            "{} {} {}",
            &c.hash[..c.hash.len().min(12)],
            c.committed_at,
            c.subject
        )
    };
    match (&v.history.newest, &v.history.oldest) {
        (Some(newest), Some(oldest)) => {
            let _ = writeln!(
                out,
                "history     {} first-parent commit(s)\n            newest {}\n            oldest {}",
                v.history.commits,
                commit(newest),
                commit(oldest)
            );
        }
        _ => {
            let _ = writeln!(out, "history     no commits");
        }
    }
    let jobs: Vec<String> = v
        .jobs_by_state
        .iter()
        .map(|(state, n)| format!("{n} {state}"))
        .collect();
    let _ = writeln!(
        out,
        "jobs        {}",
        if jobs.is_empty() {
            "none".to_string()
        } else {
            jobs.join(" · ")
        }
    );
    let _ = writeln!(out, "landings    {} landing record(s)", v.landing_records);
    if !v.invalid_records.is_empty() {
        let _ = writeln!(
            out,
            "invalid     {} file(s) under records/ are not acceptable records: {}",
            v.invalid_records.len(),
            v.invalid_records.join(", ")
        );
    }
    if let Some(throughput) = &v.throughput {
        let _ = writeln!(out, "{}", throughput.text());
    }
    out
}

/// `athena status`.
pub fn status(v: &StatusView) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "athena {} ({}{}) · driver {}",
        v.athena.version,
        &v.athena.source_commit[..v.athena.source_commit.len().min(12)],
        if v.athena.dirty { ", dirty" } else { "" },
        v.driver
    );
    if let Some(line) = &v.stale_build {
        let _ = writeln!(out, "{line}");
    }
    let _ = writeln!(
        out,
        "repository  {} · {} @ {}",
        v.repository,
        v.branch.as_deref().unwrap_or("(detached)"),
        &v.head[..12]
    );
    if let Some(n) = &v.nested {
        let _ = writeln!(out, "nested      inside job {n}");
    }
    let _ = writeln!(
        out,
        "kill switch {}",
        if v.kill_switch_on {
            "ON (athena start turns it off)"
        } else {
            "off"
        }
    );
    if let Some(focus) = &v.focus {
        let _ = writeln!(out, "focus       {focus}");
    }
    let _ = writeln!(out, "bootstrap   {}", v.bootstrap.note);
    if let Some(report) = v.reconcile.as_ref().filter(|r| !r.is_quiet()) {
        for line in report.lines() {
            let _ = writeln!(out, "{line}");
        }
    }
    if v.jobs_by_state.is_empty() {
        let _ = writeln!(out, "jobs        none yet");
    } else {
        let counts: Vec<String> = v
            .jobs_by_state
            .iter()
            .map(|(s, n)| format!("{s} {n}"))
            .collect();
        let _ = writeln!(out, "jobs        {}", counts.join(" · "));
        for (i, j) in v.recent.iter().enumerate() {
            let label = if i == 0 { "recent" } else { "" };
            let _ = writeln!(
                out,
                "{label:<11} {} {:<11} {} · {}{}",
                j.id,
                j.state,
                usd(j.cost_usd),
                j.title,
                plan_suffix(j)
            );
        }
    }
    if let Some(p) = &v.planning {
        let _ = writeln!(
            out,
            "planning    first round: {} accepted · {} stopped",
            p.accepted, p.stopped
        );
    }
    if let Some(p) = &v.planning_cost {
        out.push_str(&planning_cost_lines(p));
    }
    if let Some(k) = &v.keep_rule {
        out.push_str(&keep_rule_lines(k));
    }
    if !v.producer_settings.is_empty() {
        out.push_str(&producer_settings_lines(&v.producer_settings));
    }
    if let Some(durations) = &v.check_durations {
        let _ = writeln!(out, "{}", durations.text());
    }
    if !v.gate_counts.is_empty() {
        out.push_str(&gate_counts_lines(&v.gate_counts));
    }
    if let Some(throughput) = &v.throughput {
        let _ = writeln!(out, "{}", throughput.text());
    }
    let _ = writeln!(out, "{}", v.build_measurement);
    if let Some(trend) = &v.class_trend {
        out.push_str(&trend_section(trend));
    }
    for (i, j) in v.open.iter().enumerate() {
        let label = if i == 0 { "open" } else { "" };
        let _ = writeln!(
            out,
            "{label:<11} {} {} · {}{}",
            j.id,
            j.state,
            j.title,
            plan_suffix(j)
        );
    }
    out.push_str(&lines_section(&v.lines));
    out.push_str(&repeated_causes_section(&v.repeated_causes));
    out.push_str(&dormant_causes_section(&v.dormant_causes));
    for (i, n) in v.open_interventions.iter().enumerate() {
        let label = if i == 0 { "follow-up" } else { "" };
        let _ = writeln!(
            out,
            "{label:<11} {} [{}] {} · {}",
            n.id, n.category, n.what, n.next
        );
    }
    let total: usize = v.interventions_by_category.values().sum();
    if total > 0 {
        let cats: Vec<String> = v
            .interventions_by_category
            .iter()
            .map(|(k, n)| format!("{k} {n}"))
            .collect();
        let classes: Vec<String> = v
            .interventions_by_class
            .iter()
            .map(|(k, n)| format!("{k} {n}"))
            .collect();
        let _ = writeln!(
            out,
            "interventions {total}: {} · by class: {}",
            cats.join(", "),
            classes.join(", ")
        );
    } else {
        let _ = writeln!(out, "interventions none");
    }
    if v.mistakes.total > 0 {
        out.push_str(&mistakes(&v.mistakes));
    }
    let _ = writeln!(out, "restarts    {}", v.restarts);
    for (i, h) in v.forecasts.iter().enumerate() {
        let label = if i == 0 { "forecasts" } else { "" };
        let fmt = |x: Option<f64>| x.map_or("-".into(), |v| format!("{v:.3}"));
        let _ = writeln!(
            out,
            "{label:<11} {}: {} scored · mean event Brier reader {} · reference {}",
            h.class,
            h.scored_jobs,
            fmt(h.reader_mean_brier),
            fmt(h.reference_mean_brier)
        );
    }
    for (i, p) in v.pending.iter().enumerate() {
        let label = if i == 0 { "pending" } else { "" };
        let state = match &p.resolved_by {
            Some(r) => format!("resolved by {r}"),
            None => format!("unresolved: {}", p.basis),
        };
        let _ = writeln!(out, "{label:<11} {}#{} — {state}", p.record, p.field);
    }
    let _ = writeln!(
        out,
        "unreviewed  {} introspection(s) with no class label{}",
        v.unreviewed_introspections,
        if v.unreviewed_introspections > 0 {
            " (athena review <id>)"
        } else {
            ""
        }
    );
    for (i, p) in v.proposals.iter().enumerate() {
        let label = if i == 0 { "proposals" } else { "" };
        let _ = writeln!(
            out,
            "{label:<11} {} [{}] version {} -> {} · class {}: {}",
            p.id, p.kind, p.version_from, p.version_to, p.failure_class, p.statement
        );
        let _ = writeln!(out, "{:<11}   applies to: {}", "", p.applies_to.join(", "));
        for source in &p.sources {
            let _ = writeln!(out, "{:<11}   source: {source}", "");
        }
    }
    for (i, w) in v.work_proposals.iter().enumerate() {
        let label = if i == 0 { "steward" } else { "" };
        let _ = writeln!(out, "{label:<11} {} [{}] {}", w.id, w.rung, w.title);
        let _ = writeln!(out, "{:<11}   source: {}", "", w.source);
        for evidence in &w.evidence {
            let _ = writeln!(out, "{:<11}   evidence: {evidence}", "");
        }
    }
    if let Some(a) = &v.audience {
        let _ = writeln!(
            out,
            "audience    {}/5 from a {} reading {} ({})",
            a.rating, a.audience, a.target, a.at
        );
        for finding in &a.findings {
            let _ = writeln!(out, "{:<11}   finding: {finding}", "");
        }
    }
    if let Some(a) = &v.audit {
        let _ = writeln!(out, "audit       job {}: {} ({})", a.job, a.summary, a.at);
        for finding in &a.findings {
            let _ = writeln!(out, "{:<11}   finding: {finding}", "");
        }
    }
    if let Some(health) = &v.code_health {
        out.push_str(&code_health_section(health));
    }
    if !v.invalid_records.is_empty() {
        let _ = writeln!(
            out,
            "invalid    {} (run athena check)",
            v.invalid_records.join(", ")
        );
    }
    for (i, n) in v.next.iter().enumerate() {
        let label = if i == 0 { "next" } else { "" };
        let _ = writeln!(out, "{label:<11} {n}");
    }
    out
}

/// The code-health block of `athena status`: a heading, the limits, the table with aligned columns (a file's
/// lines, longest function, landings in the last week, citations, its limits, and whether it is over), and the
/// test suite's run time.
pub(super) fn code_health_section(health: &athena_core::code_health::CodeHealth) -> String {
    use athena_core::code_health::{suite_line, table_text, thresholds_line};
    let mut out = format!("code health {}\n", thresholds_line());
    for line in table_text(health).lines() {
        let _ = writeln!(out, "  {line}");
    }
    let over = health.over().len();
    let _ = writeln!(
        out,
        "  {over} file(s) over a limit; {}",
        suite_line(health.suite_seconds)
    );
    out
}

/// One introspection, as a block of labeled lines in `athena status <job>`.
pub(super) fn introspection(out: &mut String, i: &athena_core::status::IntrospectionView) {
    let none = "-".to_string();
    let _ = writeln!(
        out,
        "introspection {} · {} · {}{}",
        i.id,
        i.trigger,
        i.status,
        i.intervention
            .as_ref()
            .map_or(String::new(), |id| format!(" · intervention {id}"))
    );
    if let Some(why) = &i.failure {
        let _ = writeln!(out, "  failed        {why}");
    }
    let field = |label: &str, value: &Option<String>, out: &mut String| {
        let _ = writeln!(out, "  {label:<13} {}", value.as_ref().unwrap_or(&none));
    };
    field("what happened", &i.what_happened, out);
    let class = i.failure_class.as_ref().map(|c| {
        if i.new_class {
            format!("{c} · new, proposed")
        } else {
            c.clone()
        }
    });
    field("class", &class, out);
    let cases = if i.earlier_cases.is_empty() {
        "none".to_string()
    } else {
        i.earlier_cases.join(", ")
    };
    let _ = writeln!(out, "  earlier cases {cases}");
    if !i.decisions_matched.is_empty() {
        let _ = writeln!(out, "  decisions     {}", i.decisions_matched.join(", "));
    }
    field("cause", &i.cause, out);
    let prevention = i
        .prevention
        .as_ref()
        .map(|p| format!("{}: {p}", i.prevention_kind.as_deref().unwrap_or("?")));
    field("prevention", &prevention, out);
    field("next request", &i.proposed_request, out);
    for flag in &i.flags {
        let _ = writeln!(out, "  flag          {flag}");
    }
    for id in &i.dropped_ids {
        let _ = writeln!(
            out,
            "  dropped id    {id} (not on the list passed to the model)"
        );
    }
    if let Some(jev) = &i.jev {
        let line = match (&jev.error, &jev.class, jev.repeat_probability) {
            (Some(why), _, _) => format!("not answered: {why}"),
            (None, Some(class), Some(repeat)) => format!(
                "class {} (confidence {:.2}) · repeat {repeat:.2}",
                class.choice, class.confidence
            ),
            _ => "no answers".to_string(),
        };
        let _ = writeln!(
            out,
            "  jev (shadow)  {line}; recorded only, decides nothing"
        );
    }
    let _ = writeln!(
        out,
        "  written by    a language model, not Jev{}",
        i.cost_usd.map_or(String::new(), |c| format!(" · ${c:.4}"))
    );
}
