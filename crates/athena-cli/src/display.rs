//! What people see: the reading, the result of a run, the state of the work, and a job's trace
//! (design §3.2, §3.4, §3.6). Agents use `--json` for the same views.

use athena_core::reading::Proposal;
use athena_core::run::Finished;
use athena_core::status::{JobTrace, StatusView};
use athena_data::records::{
    CriterionKind, DeliverableAction, Forecast, OutcomeState, Quantiles, Reading, Score,
};
use std::fmt::Write;

fn pct(p: f64) -> String {
    format!("{:.0}%", p * 100.0)
}

fn usd(x: f64) -> String {
    if x < 0.01 {
        format!("${x:.4}")
    } else {
        format!("${x:.2}")
    }
}

/// A duration in seconds, written for people.
pub fn duration(seconds: f64) -> String {
    let s = seconds.round() as u64;
    match s {
        0..60 => format!("{s}s"),
        60..3600 => {
            if s.is_multiple_of(60) {
                format!("{}m", s / 60)
            } else {
                format!("{}m{:02}s", s / 60, s % 60)
            }
        }
        _ => format!("{}h{:02}m", s / 3600, (s % 3600) / 60),
    }
}

fn cost_quantiles(q: &Quantiles) -> String {
    format!("{} (80%: {}–{})", usd(q.p50()), usd(q.p10()), usd(q.p90()))
}

fn time_quantiles(q: &Quantiles) -> String {
    format!(
        "{} (80%: {}–{})",
        duration(q.p50()),
        duration(q.p10()),
        duration(q.p90())
    )
}

fn action(a: DeliverableAction) -> &'static str {
    match a {
        DeliverableAction::Create => "create",
        DeliverableAction::Modify => "modify",
        DeliverableAction::Delete => "delete",
    }
}

fn short_model(model: &str) -> &str {
    model
        .strip_prefix("claude-")
        .and_then(|m| m.split('-').next())
        .unwrap_or(model)
}

fn forecast_lines(out: &mut String, label: &str, f: &Forecast, models: Option<&str>) {
    let _ = write!(
        out,
        "  {label:<9} cost {} · time {}",
        cost_quantiles(&f.cost_usd),
        time_quantiles(&f.wall_time_s)
    );
    if let Some(models) = models {
        let _ = write!(out, " · models {models}");
    }
    let _ = writeln!(
        out,
        "\n            reading accepted {} · passes verification {} · you intervene after dispatch {}",
        pct(f.p_reading_accepted),
        pct(f.p_verification_passes),
        pct(f.p_user_intervenes)
    );
}

/// The reading, as `ask` and `correct` show it.
pub fn reading(job: &str, r: &Reading, interactive: bool) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "athena · job {job} · reading {}\n", r.n);
    let _ = writeln!(out, "I read this as: {}\n", r.summary.trim());
    let _ = writeln!(
        out,
        "  class {} · capability {} · producer {} · verifier {}",
        r.class_of_work,
        r.capability.as_deref().unwrap_or("none"),
        r.producer_model,
        r.verifier_model
    );
    if r.capability.is_none() {
        let _ = writeln!(
            out,
            "  declined  {}",
            r.decline_reason.as_deref().unwrap_or("")
        );
    }
    if !r.defaults_applied.is_empty() {
        let defaults: Vec<String> = r
            .defaults_applied
            .iter()
            .map(|d| format!("{} ({})", d.id, d.how.trim()))
            .collect();
        let _ = writeln!(out, "  defaults  {}", defaults.join("; "));
    }
    if !r.assumptions.is_empty() {
        let _ = writeln!(out, "  assumes   {}", r.assumptions.join("; "));
    }
    if !r.deliverables.is_empty() {
        let items: Vec<String> = r
            .deliverables
            .iter()
            .map(|d| format!("{} ({}) {}", d.path, action(d.action), pct(d.p)))
            .collect();
        let _ = writeln!(out, "  produces  {}", items.join(" · "));
    }
    for (i, c) in r.acceptance_criteria.iter().enumerate() {
        let kind = match c.kind {
            CriterionKind::Content => "[content] ",
            CriterionKind::Behavior => "[behavior]",
        };
        let label = if i == 0 { "done when" } else { "" };
        let _ = writeln!(out, "  {label:<9} {}. {kind} {}", i + 1, c.text);
    }
    if !r.out_of_scope.is_empty() {
        let _ = writeln!(out, "  not       {}", r.out_of_scope.join("; "));
    }
    let models = format!(
        "{} (producer), {} (verifier)",
        short_model(&r.producer_model),
        short_model(&r.verifier_model)
    );
    forecast_lines(&mut out, "predicts", &r.forecasts.reader, Some(&models));
    forecast_lines(&mut out, "reference", &r.forecasts.reference, None);
    if let Some(d) = &r.departures {
        let _ = writeln!(out, "  departs   {}", d.trim());
    }
    if let Some(q) = &r.question {
        let _ = writeln!(
            out,
            "  question  {} — Enter runs with the assumption above",
            q.trim()
        );
    }
    out.push('\n');
    if interactive {
        if r.capability.is_some() {
            let _ = writeln!(
                out,
                "Enter runs it · type a one-line correction · q leaves it proposed"
            );
        } else {
            let _ = writeln!(
                out,
                "Enter or q leaves it declined · type a one-line correction to read it again"
            );
        }
    }
    out
}

/// What a non-interactive `ask` or `correct` prints after the reading.
pub fn follow_up(p: &Proposal) -> String {
    let id = p.job.as_str();
    let mut out = format!("committed  {} (proposal)\n", &p.commit[..12]);
    match &p.reading {
        Some(r) if r.capability.is_some() => {
            let _ = writeln!(out, "next       athena go {id}");
            let _ = writeln!(
                out,
                "           athena correct {id} \"<one-line correction>\""
            );
            let _ = writeln!(out, "           athena close {id} --why \"...\"");
        }
        Some(_) => {
            let _ = writeln!(
                out,
                "next       athena correct {id} \"<one-line correction>\"   # or: athena close {id} --why \"...\""
            );
        }
        None => {
            let _ = writeln!(
                out,
                "the reader failed: {}",
                p.reader_error.as_deref().unwrap_or("unknown")
            );
            let _ = writeln!(
                out,
                "next       athena correct {id} \"<line>\"   # reads again · or: athena close {id} --why \"...\""
            );
        }
    }
    out
}

fn inside(q: &Quantiles, actual: f64) -> &'static str {
    if q.p10() <= actual && actual <= q.p90() {
        "inside the 80% interval"
    } else {
        "outside the 80% interval"
    }
}

/// The result block of `go`.
pub fn finished(f: &Finished) -> String {
    let mut out = String::new();
    let reader = &f.reading.forecasts.reader;
    if let Some(totals) = &f.totals {
        let cost = totals.cost_execution.best_lower();
        let _ = write!(
            out,
            "\ncost {} (predicted {}; {})",
            usd(cost),
            usd(reader.cost_usd.p50()),
            inside(&reader.cost_usd, cost)
        );
        if let Some(t) = totals.wall_time_execution.as_ref().and_then(|q| q.value()) {
            let _ = write!(
                out,
                " · time {} (predicted {}; {})",
                duration(t),
                duration(reader.wall_time_s.p50()),
                inside(&reader.wall_time_s, t)
            );
        }
        let _ = writeln!(
            out,
            "\nall-in cost {} including the reading",
            usd(totals.cost_all.best_lower())
        );
    }
    if let Some(score) = &f.score {
        let _ = writeln!(out, "{}", score_line(score));
    }
    let _ = writeln!(out, "outcome {} · {}", f.state.as_str(), f.reason);
    let _ = writeln!(out, "trace  athena status {}", f.job.as_str());
    if f.state == OutcomeState::Rejected
        && let Some(v) = &f.verification
    {
        for finding in v.findings.iter().take(5) {
            let _ = writeln!(
                out,
                "  {:?}: {} ({})",
                finding.severity, finding.summary, finding.evidence
            )
            .map(|_| ());
        }
        let _ = writeln!(
            out,
            "next   ask again with the findings in mind, or `athena status {}`",
            f.job.as_str()
        );
    }
    out
}

/// One line of scores.
pub fn score_line(score: &Score) -> String {
    let events = |forecaster: &str| -> Option<f64> {
        let values: Vec<f64> = score
            .events
            .iter()
            .filter_map(|e| e.brier.get(forecaster).copied())
            .collect();
        (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
    };
    let mut parts = Vec::new();
    if let (Some(reader), Some(reference)) = (events("reader"), events("reference")) {
        parts.push(format!(
            "events Brier {reader:.3} (reference {reference:.3})"
        ));
    }
    if let Some(d) = &score.deliverables {
        parts.push(format!("deliverables {:.3}", d.sum));
    }
    for q in &score.quantities {
        if let Some(s) = q.forecasts.get("reader") {
            let name = if q.quantity == "cost_usd" {
                "cost"
            } else {
                "time"
            };
            parts.push(format!("{name} WIS {:.2}", s.wis));
        }
    }
    if parts.is_empty() {
        "score  (nothing resolved)".into()
    } else {
        format!("score  {}", parts.join(" · "))
    }
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
    let _ = writeln!(out, "bootstrap   {}", v.bootstrap.note);
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
                "{label:<11} {} {:<11} {} · {}",
                j.id,
                j.state,
                usd(j.cost_usd),
                j.title
            );
        }
    }
    for (i, j) in v.open.iter().enumerate() {
        let label = if i == 0 { "open" } else { "" };
        let _ = writeln!(out, "{label:<11} {} {} · {}", j.id, j.state, j.title);
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
    if !v.invalid_records.is_empty() {
        let _ = writeln!(
            out,
            "invalid     {} (run athena check)",
            v.invalid_records.join(", ")
        );
    }
    for (i, n) in v.next.iter().enumerate() {
        let label = if i == 0 { "next" } else { "" };
        let _ = writeln!(out, "{label:<11} {n}");
    }
    out
}

/// `athena status <job>`.
pub fn trace(t: &JobTrace) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "job {} · {} · records in {}/",
        t.id, t.state, t.directory
    );
    if let Some(req) = &t.request {
        let _ = writeln!(out, "\nrequest     {}", req.trim());
    }
    for r in &t.readings {
        let accepted = if t.accepted_reading == Some(r.n) {
            " (accepted)"
        } else {
            ""
        };
        let _ = writeln!(
            out,
            "reading {}{accepted}  {} · class {} · {}",
            r.n,
            r.title,
            r.class_of_work,
            r.summary.trim()
        );
        let f = &r.forecasts.reader;
        let _ = writeln!(
            out,
            "  predicted  cost {} · time {} · accepted {} · passes {} · intervene {}",
            cost_quantiles(&f.cost_usd),
            time_quantiles(&f.wall_time_s),
            pct(f.p_reading_accepted),
            pct(f.p_verification_passes),
            pct(f.p_user_intervenes)
        );
    }
    for c in &t.corrections {
        let _ = writeln!(out, "correction  {c}");
    }
    if let Some(owner) = &t.owner {
        let _ = writeln!(out, "owner       {owner}");
    }
    if let Some(v) = &t.verifier {
        let _ = writeln!(out, "verifier    {v}");
    }
    for i in &t.invocations {
        let _ = writeln!(
            out,
            "invocation  {:02} {:<8} {} · {} · {} · {} · {} commands{}",
            i.seq,
            i.role,
            i.model,
            i.outcome,
            i.cost,
            i.wall_time_s.map(duration).unwrap_or_else(|| "?".into()),
            i.commands_run,
            if i.denials > 0 {
                format!(" · {} denials", i.denials)
            } else {
                String::new()
            }
        );
        let _ = writeln!(out, "            tokens {}", i.tokens);
    }
    if let Some(c) = &t.candidate {
        let paths: Vec<String> = c["changes"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|e| e["path"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let _ = writeln!(
            out,
            "candidate   {} · {}",
            paths.join(", "),
            c["fingerprint"].as_str().unwrap_or("-")
        );
    }
    if let Some(v) = &t.verification {
        let _ = writeln!(
            out,
            "verification final {} · agent {}",
            v["final_verdict"].as_str().unwrap_or("?"),
            v["agent_verdict"].as_str().unwrap_or("-")
        );
        if let Some(reasons) = v["reasons"].as_array().filter(|r| !r.is_empty()) {
            let reasons: Vec<&str> = reasons.iter().filter_map(|r| r.as_str()).collect();
            let _ = writeln!(out, "            {}", reasons.join("; "));
        }
    }
    for outcome in [&t.outcome, &t.later_outcome].into_iter().flatten() {
        let _ = writeln!(
            out,
            "outcome     {} · {}",
            outcome["state"].as_str().unwrap_or("?"),
            outcome["reason"].as_str().unwrap_or("")
        );
    }
    if let Some(score) = &t.score
        && let Ok(score) = serde_json::from_value::<Score>(score.clone())
    {
        let _ = writeln!(
            out,
            "{}",
            score_line(&score).replacen("score  ", "score       ", 1)
        );
    }
    for i in &t.interventions {
        let _ = writeln!(out, "intervention {i}");
    }
    for c in &t.commits {
        let _ = writeln!(out, "commit      {c}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_read_like_people_write_them() {
        assert_eq!(duration(42.0), "42s");
        assert_eq!(duration(240.0), "4m");
        assert_eq!(duration(365.0), "6m05s");
        assert_eq!(duration(3900.0), "1h05m");
    }
}
