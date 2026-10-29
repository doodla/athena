//! What people see: the reading, the result of a run, the state of the work, and a job's trace
//! (design §3.2, §3.4, §3.6). Agents use `--json` for the same views.

use athena_core::reading::Proposal;
use athena_core::run::Finished;
use athena_core::status::{
    JobTrace, KeepRule, KeepVerdict, LaterHarm, PlanningCost, PlanningGroup, ProducerSettingStats,
    StatusView,
};
use athena_data::records::{
    CriterionKind, DeliverableAction, Forecast, OutcomeState, Quantiles, Reading, Score,
};
use std::fmt::Write;

/// The result of `athena check --pr-head`: one line per offending commit (its sha and why), then a summary.
pub fn pr_head_report(report: &athena_core::pr_check::PrReport) -> String {
    let mut out = String::new();
    for o in &report.offences {
        let _ = writeln!(out, "  - {}: {}", o.sha, o.reason);
    }
    let verdict = if report.passed() { "pass" } else { "FAIL" };
    let _ = writeln!(
        out,
        "pr-head {verdict}: {} commit(s) between {} and {}, {} offending",
        report.commits.len(),
        report.base,
        report.head,
        report.offences.len()
    );
    out
}

/// The result of `athena check --attribution`: one line per finding naming the commit, a notice per pending
/// commit, then a summary (docs/design/records-history.md §5.2, §6, §10; D111).
pub fn attribution_report(report: &athena_gate::attribution::AttributionReport) -> String {
    let mut out = String::new();
    for p in &report.pending {
        let _ = writeln!(
            out,
            "  - notice: {} is pending reconciliation (landing-intent written {})",
            p.commit, p.intent_at
        );
    }
    for f in &report.findings {
        match &f.commit {
            Some(commit) => {
                let _ = writeln!(out, "  - {} {commit}: {}", f.kind.label(), f.detail);
            }
            None => {
                let _ = writeln!(out, "  - {}: {}", f.kind.label(), f.detail);
            }
        }
    }
    let verdict = if report.passed() { "pass" } else { "FAIL" };
    let _ = writeln!(
        out,
        "attribution {verdict}: {} commit(s) judged, {} attributed, {} pending, {} finding(s)",
        report.checked,
        report.attributed.len(),
        report.pending.len(),
        report.findings.len()
    );
    out
}

/// The same report as JSON, for `athena check --attribution --json`.
pub fn attribution_json(report: &athena_gate::attribution::AttributionReport) -> serde_json::Value {
    serde_json::json!({
        "base": report.base,
        "checked": report.checked,
        "passed": report.passed(),
        "attributed": report.attributed,
        "pending": report.pending.iter().map(|p| serde_json::json!({
            "commit": p.commit,
            "intent_at": p.intent_at.to_string(),
        })).collect::<Vec<_>>(),
        "findings": report.findings.iter().map(|f| serde_json::json!({
            "kind": f.kind.label(),
            "commit": f.commit,
            "detail": f.detail,
        })).collect::<Vec<_>>(),
    })
}

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

/// Planning rounds and cost per landed job, before and after the first resumed planning. Each figure is a mean
/// over the retry chain behind a landed job, with the landed job's own beside it; all measured from records.
fn planning_cost_lines(p: &PlanningCost) -> String {
    let group = |g: &PlanningGroup| {
        if g.jobs == 0 {
            "no landed jobs".to_string()
        } else {
            format!(
                "{} landed · {:.1} rounds, {} per job (own {:.1}, {})",
                g.jobs,
                g.mean_rounds,
                usd(g.mean_cost_usd),
                g.own_mean_rounds,
                usd(g.own_mean_cost_usd)
            )
        }
    };
    let mut out = String::new();
    let _ = writeln!(
        out,
        "planning    per landed job, measured from records (the whole retry chain, planner and critic)"
    );
    let _ = writeln!(out, "            before: {}", group(&p.before));
    match &p.first_resumed {
        Some((job, at)) => {
            let _ = writeln!(
                out,
                "            after:  {} · since the first resumed planning ({job}, {at})",
                group(&p.after)
            );
        }
        None => {
            let _ = writeln!(out, "            after:  no planning has resumed yet");
        }
    }
    out
}

/// First-pass verification rate, landed share and cost per landed job for each producer model and effort. A
/// figure with nothing to divide by reads "n/a", never a rate or a cost.
fn producer_settings_lines(settings: &[ProducerSettingStats]) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "producers   by model and effort, measured from records (first-pass verification, landed share, cost per landed job)"
    );
    for s in settings {
        let first_pass = s.first_pass_rate.map_or("n/a".to_string(), |r| {
            format!("{} ({} of {})", pct(r), s.first_pass, s.verified)
        });
        let cost = s.cost_per_landed_usd.map_or("n/a".to_string(), usd);
        let _ = writeln!(
            out,
            "            {} · {}: {} jobs · first-pass {first_pass} · landed {} ({} of {}) · cost per landed job {cost}",
            s.model,
            s.effort,
            s.jobs,
            pct(s.landed_share),
            s.landed,
            s.jobs
        );
    }
    out
}

/// The landings later cited as the cause of a failure, a revert or a fix job, one line each for planned and
/// unplanned jobs, beside the landed share of the same group (job-planning.md section 9, D143).
fn later_harm_lines(h: &LaterHarm) -> String {
    let mut out = String::from(
        "            landings later cited as the cause of a failure, revert or fix job:\n",
    );
    for (name, g) in [("planned", &h.planned), ("unplanned", &h.unplanned)] {
        let share = g.share.map_or("n/a".to_string(), pct);
        let _ = writeln!(
            out,
            "              {name:<9} {} of {} landings cited ({} failure · {} revert · {} fix) · {} of {} jobs landed ({share})",
            g.cited, g.landed, g.failures, g.reverts, g.fixes, g.landed, g.jobs
        );
    }
    out
}

/// The job-planning keep rule (job-planning.md section 9, D110): the measures, and a BREACH line with the
/// numbers when planning is below the baseline or is to be turned off.
fn keep_rule_lines(k: &KeepRule) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "keep rule   {} of {} planned jobs landed ({}) · baseline {}",
        k.landed,
        k.planned,
        pct(k.share),
        pct(k.baseline_share)
    );
    let cost = k.cost_per_landed_usd.map_or("n/a".to_string(), usd);
    let rate = k.first_round_accept_rate.map_or("n/a".to_string(), |r| {
        format!(
            "{} ({} of {})",
            pct(r),
            k.first_round_accepted,
            k.first_round_planned
        )
    });
    let _ = writeln!(
        out,
        "            cost per landed job {cost} · first-round accept rate {rate}"
    );
    out.push_str(&later_harm_lines(&k.later_harm));
    let numbers = format!(
        "{} of {} planned jobs landed ({}), baseline {}, cost per landed job {cost}",
        k.landed,
        k.planned,
        pct(k.share),
        pct(k.baseline_share)
    );
    match k.verdict {
        KeepVerdict::Breach => {
            let _ = writeln!(
                out,
                "            BREACH: planning is below the keep rule: {numbers}"
            );
        }
        KeepVerdict::TurnOff => {
            let _ = writeln!(
                out,
                "            BREACH: the keep rule says turn planning off: {numbers}"
            );
        }
        KeepVerdict::Keep | KeepVerdict::Inconclusive => {}
    }
    out
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

/// What `ask --later` prints.
/// What `athena audit` prints: the summary, each finding with its evidence, and the recorded invocation, so
/// the owner sees that the auditor ran and what it cost.
pub fn audit(a: &athena_core::auditor::Audited) -> String {
    let mut out = format!("audit of job {}: {}\n", a.audit.target, a.audit.summary);
    if a.audit.findings.is_empty() {
        out.push_str("  no findings\n");
    }
    for f in &a.audit.findings {
        let _ = writeln!(out, "  - {} (evidence: {})", f.text, f.evidence);
    }
    let _ = writeln!(
        out,
        "auditor invocation recorded: {} ({}, {:?}, cost ${:.4})",
        a.invocation_path(),
        a.invocation.model,
        a.invocation.outcome,
        a.invocation.cost.best_lower()
    );
    let _ = writeln!(
        out,
        "audit recorded: records/audits/{}/{}.json",
        a.audit.id.month(),
        a.audit.id
    );
    out
}

/// The mistake report: how many, which classes and causes recur, and the prevention proposed for each cause.
/// Counts are apart from the intervention categories. Empty-handed it says `mistakes none`.
pub fn mistakes(m: &athena_core::status::MistakeReport) -> String {
    if m.total == 0 {
        return "mistakes none\n".into();
    }
    let join = |map: &std::collections::BTreeMap<String, usize>| {
        map.iter()
            .map(|(k, n)| format!("{k} {n}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut out = format!("mistakes {}: by class: {}\n", m.total, join(&m.by_class));
    let _ = writeln!(out, "mistakes by cause: {}", join(&m.by_cause));
    for p in &m.proposals {
        let _ = writeln!(
            out,
            "prevention ({}, {} mistake(s)): {} · for the cause: {} · from introspection {}",
            p.kind,
            p.mistakes,
            p.prevention,
            p.cause,
            p.introspections.join(", ")
        );
    }
    out
}

/// What `athena audit` prints: the mistake report first, so it shows even when no job audit was produced, then
/// the job audit when there is one.
pub fn audit_report(
    m: &athena_core::status::MistakeReport,
    audited: Option<&athena_core::auditor::Audited>,
) -> String {
    let mut out = mistakes(m);
    match audited {
        Some(a) => out.push_str(&audit(a)),
        None => out.push_str("no job audit produced\n"),
    }
    out
}

pub fn queued(job: &str, commit: &str) -> String {
    format!(
        "queued     {job} · {} (nothing read, no model called)\nnext       athena go {job}   # reads it, then asks to run it\n",
        &commit[..commit.len().min(12)]
    )
}

/// What `land --pr` prints.
pub fn land_pr(report: &athena_core::gh::PrReport) -> String {
    use athena_core::gh::Checks;
    use athena_core::pr::LinkState;
    let link = &report.link;
    let mut out = format!(
        "pull request  #{} on {}/{}\n",
        link.pr_number, link.host, link.repository
    );
    let merged = link.merged_commit.as_deref().unwrap_or("?");
    let line = match (link.state, report.checks) {
        (LinkState::Landed, _) => {
            format!("landed        merged as {merged}; the merged content is the verified content\n")
        }
        (LinkState::MergedUnverified, _) => format!(
            "merged-unverified  merged as {merged}, but its content is not the verified content; the job is not landed. Re-verify it on the new tip as a new job\n"
        ),
        (LinkState::Open, Some(Checks::Green)) if report.merge_requested => {
            "open          merge requested, not yet seen merged\n".to_string()
        }
        (LinkState::Open, Some(Checks::Green)) => format!(
            "open          checks are green; merge it on the host, then run `athena land {} --pr` again to land it\n",
            link.job
        ),
        (LinkState::Open, Some(Checks::Red)) => {
            // The failing test is named whether or not the failed job was re-run.
            let test = match &report.failing_test {
                Some(name) => format!("test {name} failed"),
                None => "a check failed (the failing test could not be named)".to_string(),
            };
            let rerun = if report.reruns > 0 {
                ", and it failed again after the failed job was re-run once"
            } else {
                ""
            };
            format!("open          {test}{rerun}; nothing was merged\n")
        }
        (LinkState::Open, _) => {
            "open          checks are not finished; nothing was merged. Run the command again later\n"
                .to_string()
        }
    };
    out.push_str(&line);
    for warning in &report.warnings {
        out.push_str(&format!("warning       {warning}\n"));
    }
    if report.reruns > 0 && report.checks != Some(Checks::Red) {
        let after = if report.checks == Some(Checks::Green) {
            "and then passed"
        } else {
            "and has not finished"
        };
        out.push_str(&format!(
            "re-run        the failed job was re-run once {after}\n"
        ));
    }
    out
}

/// What `athena plan` prints: the order, the parallel groups, each planned item's goal, rung, acceptance, open
/// questions, dependencies and touched paths, then the items held back with the reason. With nothing open it
/// says so.
pub fn plan(report: &crate::drain::PlanReport) -> String {
    let mut out = String::new();
    if report.entries.is_empty() {
        out.push_str("plan        nothing is open: no queued request is waiting to be planned\n");
        return out;
    }
    let list = |items: &[String]| {
        if items.is_empty() {
            "none".to_string()
        } else {
            items.join("; ")
        }
    };
    let p = &report.plan;
    let _ = writeln!(
        out,
        "plan        {} open, {} planned, {} held back",
        report.entries.len(),
        p.order.len(),
        p.refused.len()
    );
    let _ = writeln!(out, "order       {}", list(&p.order).replace("; ", " → "));
    for (n, group) in p.groups.iter().enumerate() {
        let _ = writeln!(
            out,
            "group {}     {}{}",
            n + 1,
            group.join(", "),
            if group.len() > 1 {
                " (could run in parallel)"
            } else {
                ""
            }
        );
    }
    if !p.cyclic.is_empty() {
        let _ = writeln!(
            out,
            "cyclic      {} wait on each other",
            p.cyclic.join(", ")
        );
    }
    let entry_lines = |out: &mut String, e: &athena_core::plan::PlanEntry| {
        let _ = writeln!(out, "item        {} · {}", e.id, e.title);
        let _ = writeln!(
            out,
            "  goal        {}",
            if e.goal.is_empty() { "none" } else { &e.goal }
        );
        let _ = writeln!(
            out,
            "  rung        {}",
            e.rung.map_or("none", athena_data::roles::Ladder::as_str)
        );
        let _ = writeln!(out, "  priority    {}", e.priority.as_str());
        let _ = writeln!(out, "  acceptance  {}", list(&e.acceptance));
        let _ = writeln!(out, "  questions   {}", list(&e.open_questions));
        let _ = writeln!(out, "  depends on  {}", list(&e.depends_on));
        let _ = writeln!(out, "  touches     {}", list(&e.touches));
    };
    for id in &p.order {
        if let Some(e) = report.entries.iter().find(|e| &e.id == id) {
            entry_lines(&mut out, e);
        }
    }
    for r in &p.refused {
        if let Some(e) = report.entries.iter().find(|e| e.id == r.id) {
            entry_lines(&mut out, e);
        }
        let _ = writeln!(
            out,
            "flagged     {} · {} lacks {}: {}",
            r.id,
            r.title,
            r.missing.join(", "),
            r.reason
        );
    }
    for note in &report.notes {
        let _ = writeln!(out, "note        {note}");
    }
    let _ = writeln!(
        out,
        "read        ${:.2} spent reading unread requests; nothing was run",
        report.spent_usd
    );
    out
}

/// What `athena drain` prints: why it stopped, each job and whether it landed, the money, the rating against
/// the target and whether it was taken after a landing, and what is still queued or waiting for a person.
pub fn drain(report: &crate::drain::Report) -> String {
    let mut out = match (report.stop, &report.failed_item) {
        (crate::drain::Stop::RepeatedFailure, Some(item)) => {
            format!("stopped     repeated failure: item {item} failed twice\n")
        }
        (stop, _) => format!("stopped     {}: {}\n", stop.label(), stop.meaning()),
    };
    if let Some(output) = &report.rebuild_output {
        out.push_str("build output:\n");
        out.push_str(output);
        if !output.ends_with('\n') {
            out.push('\n');
        }
    }
    if report.jobs.is_empty() {
        out.push_str("jobs        none run\n");
    }
    for j in &report.jobs {
        let _ = write!(
            out,
            "job         {} (item {}) {} · {} · {}",
            j.job,
            j.item,
            j.outcome,
            usd(j.cost_usd),
            j.landing
        );
        if let Some(reason) = &j.reason {
            let _ = write!(out, " · {reason}");
        }
        out.push('\n');
    }
    let _ = writeln!(
        out,
        "spent       {} of {}",
        usd(report.spent_usd),
        usd(report.budget_usd)
    );
    let _ = writeln!(out, "landed      {}", report.landed);
    match report.rating {
        Some(r) if r.after_landing => {
            let _ = writeln!(
                out,
                "rating      {}/5 against a target of {}, taken after a landing: {}",
                r.value,
                report.target_rating,
                if r.met_target {
                    "target met"
                } else if report.held_back.is_some() {
                    "target not met: held back by the source check"
                } else if r.value >= report.target_rating && !report.outcome_changed {
                    "target not met: no landing changed the outcome"
                } else {
                    "below the target"
                }
            );
        }
        Some(r) => {
            let _ = writeln!(
                out,
                "rating      {}/5 against a target of {}, an earlier rating, not taken after a landing in this run: it says nothing about this run",
                r.value, report.target_rating
            );
        }
        None => {
            let _ = writeln!(
                out,
                "rating      none (target {}); no rating was taken after a landing",
                report.target_rating
            );
        }
    }
    if let Some(why) = &report.held_back {
        let _ = writeln!(
            out,
            "held back   the rating reached the target, but the check of the page against the pinned source failed, so drain kept running: {why}"
        );
    }
    if report.page_unchanged {
        out.push_str("page        unchanged: nothing landed in this run\n");
    } else if !report.outcome_changed {
        out.push_str(
            "outcome     no landing changed the outcome: none touched studies/, the site generator or the publish workflow, so the page is as it was and a rating cannot meet the target\n",
        );
    }
    for (n, group) in report.plan_groups.iter().enumerate() {
        let _ = writeln!(
            out,
            "plan        group {}: {}{}",
            n + 1,
            group.join(", "),
            if group.len() > 1 {
                " (could run in parallel)"
            } else {
                ""
            }
        );
    }
    for r in &report.refused {
        let _ = writeln!(out, "refused     {} · {}: {}", r.id, r.title, r.reason);
    }
    for note in &report.notes {
        let _ = writeln!(out, "note        {note}");
    }
    if report.queued.is_empty() {
        out.push_str("queued      nothing\n");
    }
    for i in &report.queued {
        let _ = writeln!(out, "queued      {} · {}", i.job, i.title);
    }
    for w in &report.waiting {
        let _ = writeln!(
            out,
            "waiting     {} · {} (rung {}); needs a person: above its role's limit or from an unknown role, so `athena approve-work` or `athena reject-work`",
            w.id, w.title, w.rung
        );
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
    if f.state == OutcomeState::Stale {
        let _ = writeln!(
            out,
            "next   nothing landed and the branch is as you left it; ask again to redo the work on top of it (`athena status {}` shows the request)",
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

/// The priority and plan state at the end of a job's line. The priority shows only when it is not normal
/// (absence means normal); the plan state is empty for a job that was not planned.
fn plan_suffix(j: &athena_core::status::JobLine) -> String {
    let priority = if j.priority == athena_core::plan::Priority::Normal {
        String::new()
    } else {
        format!(" · priority {}", j.priority.as_str())
    };
    let plan = j
        .plan
        .as_ref()
        .map_or(String::new(), |plan| format!(" · {plan}"));
    let hold = j
        .hold
        .as_ref()
        .map_or(String::new(), |hold| format!(" · {hold} (still queued)"));
    format!("{priority}{plan}{hold}")
}

/// The lines of work the scope guard flagged or paused, one block each: level, first id and signals (T1 to
/// T5), the numbers in words, and for a pause what lifts it. Empty text when there are none.
fn lines_section(lines: &[athena_core::scope_guard::LineMark]) -> String {
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
fn repeated_causes_section(causes: &[athena_core::hestia::RepeatedCause]) -> String {
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
fn dormant_causes_section(causes: &[athena_core::hestia::DormantCause]) -> String {
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
    if let Some(throughput) = &v.throughput {
        let _ = writeln!(out, "{}", throughput.text());
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
fn code_health_section(health: &athena_core::code_health::CodeHealth) -> String {
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
fn introspection(out: &mut String, i: &athena_core::status::IntrospectionView) {
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

/// `athena review`: the introspection that was shown and the verdict that was recorded.
pub fn review(r: &athena_core::review::Review) -> String {
    let s = &r.shown;
    let dash = "-";
    let mut out = String::new();
    let _ = writeln!(
        out,
        "introspection {} · {}{}",
        s.id,
        s.trigger,
        s.job
            .as_ref()
            .map_or(String::new(), |j| format!(" · job {j}"))
    );
    let _ = writeln!(
        out,
        "  what happened {}",
        s.what_happened.as_deref().unwrap_or(dash)
    );
    let _ = writeln!(
        out,
        "  class         {}",
        s.class.as_deref().unwrap_or(dash)
    );
    let _ = writeln!(
        out,
        "  cause         {}",
        s.cause.as_deref().unwrap_or(dash)
    );
    let prevention = s
        .prevention
        .as_ref()
        .map(|p| format!("{}: {p}", s.prevention_kind.as_deref().unwrap_or("?")));
    let _ = writeln!(
        out,
        "  prevention    {}",
        prevention.as_deref().unwrap_or(dash)
    );
    let jev = &s.jev;
    if !jev.asked {
        let _ = writeln!(out, "  jev (shadow)  Jev was not asked");
    } else if let Some(why) = &jev.error {
        let _ = writeln!(out, "  jev (shadow)  Jev failed: {why}");
    } else if let Some(choice) = &jev.choice {
        let _ = writeln!(
            out,
            "  jev (shadow)  chose {choice} (confidence {:.2}){}",
            jev.confidence.unwrap_or(0.0),
            jev.repeat_probability
                .map_or(String::new(), |p| format!(" · repeat {p:.2}")),
        );
        for o in &jev.options {
            let _ = writeln!(
                out,
                "                {:.2}  {}{}",
                o.probability,
                o.option,
                o.name.as_ref().map_or(String::new(), |n| format!(" ({n})"))
            );
        }
    } else {
        let _ = writeln!(out, "  jev (shadow)  no answers");
    }
    if let Some(old) = &r.replaces {
        let _ = writeln!(
            out,
            "this introspection was already reviewed (label {}); this review replaces it",
            old.id
        );
    }
    if let Some(class) = &r.new_class {
        let _ = writeln!(
            out,
            "added class {} ({}), proposed by this introspection",
            class.name, class.id
        );
    }
    let _ = writeln!(
        out,
        "recorded: class {} (label {}, by {}){}",
        r.assigned,
        r.label.id,
        r.label.labeled_by.as_str(),
        r.label
            .note
            .as_ref()
            .map_or(String::new(), |n| format!(" · note: {n}"))
    );
    out
}

/// `athena score-jev`: the labeled set, what was left out, and either the score and the promotion verdict or
/// the statement that there are too few labeled cases.
pub fn jev_score(r: &athena_core::labeled::Report) -> String {
    let mut out = String::new();
    let e = &r.excluded;
    let _ = writeln!(out, "Jev shadow answers against reviewed labels");
    let _ = writeln!(
        out,
        "  introspections read      {}\n  labeled (scored set)     {}\n  left out: unreviewed     {}\n  left out: no Jev answer  {}\n  left out: Jev call failed {}",
        e.introspections, e.labeled, e.unreviewed, e.no_jev_answer, e.jev_error
    );
    if e.labels_without_introspection > 0 {
        let _ = writeln!(
            out,
            "  labels naming no recorded introspection  {}",
            e.labels_without_introspection
        );
    }
    let s = &r.sources;
    let kinds = if s.evidence_by_kind.is_empty() {
        "none".to_string()
    } else {
        s.evidence_by_kind
            .iter()
            .map(|(kind, n)| format!("{kind} {n}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let _ = writeln!(
        out,
        "  labels derived from evidence  {} (by kind: {kinds}); labeled by a person or stronger model  {}; evidence labels overridden by a review  {}",
        s.evidence, s.reviewed, s.evidence_overridden
    );
    if let Some(d) = &r.derivation {
        let _ = writeln!(
            out,
            "  derivation just run      {} new labels written ({} already derived, {} reviewed and skipped, {} with no evidence left unlabeled, {} with conflicting evidence left unlabeled) of {} recorded introspections that name a class{}",
            d.derived,
            d.already_derived,
            d.reviewed,
            d.no_evidence,
            d.conflicting,
            d.recorded,
            d.commit
                .as_ref()
                .map_or(String::new(), |c| format!("; commit {c}"))
        );
    }
    let _ = writeln!(
        out,
        "  split by time            {} calibration, {} test (cut {})",
        r.calibration,
        r.test,
        r.cut.map_or("none".to_string(), |c| c.to_string())
    );
    if let Some(message) = &r.insufficient {
        let _ = writeln!(out, "\n{message}");
        return out;
    }
    let (Some(s), Some(v)) = (&r.score, &r.verdict) else {
        return out;
    };
    let _ = writeln!(out, "\nClass answers, test part (n = {})", s.n);
    let _ = writeln!(
        out,
        "  Brier, Jev               {:.3}\n  Brier, always none       {:.3}\n  Brier, word overlap      {:.3} (threshold {:.1}, set on the calibration part)\n  better reference         {}\n  skill over it            {:.3}",
        s.jev_brier,
        s.always_none_brier,
        s.overlap_brier,
        s.overlap_threshold,
        s.better_reference,
        s.skill
    );
    match (&s.interval, &s.no_bound_note) {
        (Some(i), _) => {
            let _ = writeln!(
                out,
                "  95% interval on skill    {:.3} to {:.3}",
                i.low, i.high
            );
        }
        (None, Some(note)) => {
            let _ = writeln!(out, "  {note}");
        }
        (None, None) => {}
    }
    let _ = writeln!(out, "  top-1 accuracy           {}", pct(s.top1_accuracy));
    let _ = writeln!(
        out,
        "  false-match rate         {}",
        s.false_match_rate.map_or(
            "not defined (no `none` labels in the test part)".to_string(),
            |x| format!("{} of {} `none` labels", pct(x), s.none_labels)
        )
    );
    match &s.repeat {
        Some(p) => {
            let _ = writeln!(
                out,
                "  is-repeat answer         Brier {:.3} (always false {:.3}{}), skill over {} {:.3}, n = {}",
                p.jev_brier,
                p.always_false_brier,
                p.base_rate_brier
                    .map_or(String::new(), |b| format!(", base rate {b:.3}")),
                p.better_reference,
                p.skill,
                p.n
            );
        }
        None => {
            let _ = writeln!(
                out,
                "  is-repeat answer         not scored (no test label says whether the event repeats)"
            );
        }
    }
    let _ = writeln!(out, "\nPromotion (section 10.4): {}", v.summary);
    for item in &v.items {
        let _ = writeln!(
            out,
            "  {}. {} - {}: {}",
            item.item,
            item.name,
            item.status.as_str(),
            item.detail
        );
    }
    let _ = writeln!(out, "\nPer record (test part)");
    for row in &s.per_record {
        let _ = writeln!(
            out,
            "  {}  label {}  by {}  Jev says {}  Brier Jev {:.3}, none {:.3}, overlap {:.3}",
            row.introspection,
            row.label.as_deref().unwrap_or("none"),
            row.labeled_by.as_str(),
            row.jev_choice,
            row.jev_brier,
            row.always_none_brier,
            row.overlap_brier
        );
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
    if let Some(plan) = &t.plan {
        let _ = writeln!(out, "planning    {plan}");
    }
    let rounds = t
        .plans
        .iter()
        .map(|p| p.round)
        .chain(t.critiques.iter().map(|c| c.round))
        .collect::<std::collections::BTreeSet<u32>>();
    for round in rounds {
        let plan = t.plans.iter().find(|p| p.round == round);
        let critique = t.critiques.iter().find(|c| c.round == round);
        let plan_text = plan.map_or("no plan", |p| p.approach.trim());
        let verdict = critique.map_or("no critique".to_string(), |c| {
            let mut text = format!("{:?}", c.final_verdict).to_lowercase();
            if let Some(agent) = c.agent_verdict
                && agent != c.final_verdict
            {
                text.push_str(&format!(
                    " (critic said {})",
                    format!("{agent:?}").to_lowercase()
                ));
            }
            text
        });
        let _ = writeln!(out, "plan round {round}  {verdict} · {plan_text}");
        if let Some(c) = critique {
            if let Some(m) = &c.request_reading_mismatch {
                let _ = writeln!(out, "            misread the request: {m}");
            }
            for f in &c.findings {
                let _ = writeln!(
                    out,
                    "            {:?}: {} ({})",
                    f.severity, f.summary, f.evidence
                );
            }
        }
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
    for i in &t.introspections {
        introspection(&mut out, i);
    }
    for c in &t.commits {
        let _ = writeln!(out, "commit      {c}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_core::gh::{Checks, PrReport};
    use athena_core::pr::{LinkState, PrLink};

    fn report(checks: Checks, reruns: u32, test: Option<&str>) -> PrReport {
        PrReport {
            link: PrLink {
                pr_number: 7,
                host: "github.com".into(),
                repository: "example/project".into(),
                head: "abc".into(),
                job: "j".into(),
                state: LinkState::Open,
                merged_commit: None,
            },
            checks: Some(checks),
            merge_requested: false,
            reruns,
            failing_test: test.map(str::to_string),
            warnings: Vec::new(),
        }
    }

    #[test]
    fn repeated_causes_are_listed_with_their_counts_and_the_request_that_addresses_them() {
        use athena_core::hestia::{AddressingRequest, RepeatedCause};
        let cause = |request| RepeatedCause {
            key: "class:20261027-090001-1abc".into(),
            class: Some("Reading ignores the earlier finding (20261027-090001-1abc)".into()),
            cause: "The reading stage does not load the earlier verdict.".into(),
            introspections: vec!["20261027-090010-1abc".into(), "20261027-090020-1abc".into()],
            jobs: vec!["20261027-080000-aaaa".into(), "20261027-080100-bbbb".into()],
            request,
        };
        let text = repeated_causes_section(&[
            cause(Some(AddressingRequest {
                job: "20261027-090030-cccc".into(),
                state: "queued".into(),
                landed: false,
            })),
            cause(None),
        ]);
        assert!(
            text.starts_with("repeated    2 time(s) in 2 jobs"),
            "{text}"
        );
        assert!(
            text.contains("request 20261027-090030-cccc (queued)"),
            "{text}"
        );
        assert!(text.contains("no request yet"), "{text}");
        assert!(
            text.contains("introspections: 20261027-090010-1abc, 20261027-090020-1abc"),
            "{text}"
        );
        assert_eq!(repeated_causes_section(&[]), "");
    }

    #[test]
    fn dormant_causes_are_listed_apart_with_their_stage() {
        use athena_core::hestia::{DormantCause, RepeatedCause, Stage};
        let dormant = DormantCause {
            stage: Stage::Planner,
            cause: RepeatedCause {
                key: "cause:the planner drops acceptance".into(),
                class: None,
                cause: "The planner drops acceptance.".into(),
                introspections: vec!["20261027-090010-1abc".into(), "20261027-090020-1abc".into()],
                jobs: vec!["20261027-080000-aaaa".into(), "20261027-080100-bbbb".into()],
                request: None,
            },
        };
        let text = dormant_causes_section(&[dormant]);
        assert!(
            text.starts_with("dormant     2 time(s) in 2 jobs"),
            "{text}"
        );
        assert!(
            text.contains("planner stage is off in athena.toml, no request queued"),
            "{text}"
        );
        assert!(
            text.contains("introspections: 20261027-090010-1abc"),
            "{text}"
        );
        assert_eq!(dormant_causes_section(&[]), "");
    }

    #[test]
    fn planning_cost_lines_show_both_groups_the_cutover_and_the_measured_numbers() {
        let group = |jobs, rounds, cost, own_rounds, own_cost| PlanningGroup {
            jobs,
            mean_rounds: rounds,
            mean_cost_usd: cost,
            own_mean_rounds: own_rounds,
            own_mean_cost_usd: own_cost,
        };
        let text = planning_cost_lines(&PlanningCost {
            before: group(4, 5.5, 1.25, 3.0, 0.75),
            after: group(1, 2.0, 0.4, 2.0, 0.4),
            first_resumed: Some(("20261026-150803-c7d4".into(), "2026-10-26T15:08:03Z".into())),
        });
        assert!(text.contains("measured from records"), "{text}");
        assert!(
            text.contains("before: 4 landed · 5.5 rounds, $1.25 per job (own 3.0, $0.75)"),
            "{text}"
        );
        assert!(
            text.contains("after:  1 landed · 2.0 rounds, $0.40 per job (own 2.0, $0.40)"),
            "{text}"
        );
        assert!(
            text.contains(
                "since the first resumed planning (20261026-150803-c7d4, 2026-10-26T15:08:03Z)"
            ),
            "{text}"
        );

        let none = planning_cost_lines(&PlanningCost {
            before: group(2, 3.0, 0.5, 3.0, 0.5),
            after: PlanningGroup::default(),
            first_resumed: None,
        });
        assert!(
            none.contains("after:  no planning has resumed yet"),
            "{none}"
        );
        assert!(!none.contains("since the first"), "{none}");
    }

    #[test]
    fn producer_settings_lines_list_the_three_measures_per_model_and_effort() {
        let stats = |effort: &str, landed: usize, cost: Option<f64>| ProducerSettingStats {
            model: "claude-sonnet-5-5".into(),
            effort: effort.into(),
            jobs: 4,
            verified: 4,
            first_pass: 3,
            first_pass_rate: Some(0.75),
            landed,
            landed_share: landed as f64 / 4.0,
            total_cost_usd: 6.0,
            cost_per_landed_usd: cost,
        };
        let text =
            producer_settings_lines(&[stats("medium", 2, Some(3.0)), stats("high", 0, None)]);
        assert!(
            text.contains("claude-sonnet-5-5 · medium: 4 jobs · first-pass 75% (3 of 4) · landed 50% (2 of 4) · cost per landed job $3.00"),
            "{text}"
        );
        // Nothing landed: no cost per landed job, and a landed share of zero rather than a success.
        let high = text.lines().find(|l| l.contains("· high:")).expect(&text);
        assert!(
            high.contains("landed 0% (0 of 4) · cost per landed job n/a"),
            "{text}"
        );
        let mut unverified = stats("low", 0, None);
        unverified.verified = 0;
        unverified.first_pass = 0;
        unverified.first_pass_rate = None;
        let text = producer_settings_lines(&[unverified]);
        assert!(text.contains("first-pass n/a"), "{text}");
    }

    fn keep_rule(landed: usize, planned: usize, verdict: KeepVerdict) -> KeepRule {
        KeepRule {
            planned,
            landed,
            share: landed as f64 / planned as f64,
            baseline_share: 0.64,
            total_cost_usd: 100.0,
            cost_per_landed_usd: Some(100.0 / landed as f64),
            first_round_accepted: 20,
            first_round_planned: 40,
            first_round_accept_rate: Some(0.5),
            verdict,
            later_harm: LaterHarm::default(),
        }
    }

    #[test]
    fn keep_rule_lines_show_later_cited_landings_for_planned_and_unplanned_jobs_beside_the_share() {
        use athena_core::status::LaterHarmGroup;
        let mut rule = keep_rule(28, 51, KeepVerdict::Keep);
        rule.later_harm = LaterHarm {
            planned: LaterHarmGroup {
                jobs: 51,
                landed: 28,
                share: Some(28.0 / 51.0),
                cited: 3,
                failures: 2,
                reverts: 0,
                fixes: 1,
            },
            unplanned: LaterHarmGroup::default(),
        };
        let text = keep_rule_lines(&rule);
        assert!(
            text.contains("landings later cited as the cause of a failure, revert or fix job:"),
            "{text}"
        );
        assert!(
            text.contains(
                "planned   3 of 28 landings cited (2 failure · 0 revert · 1 fix) · 28 of 51 jobs landed (55%)"
            ),
            "{text}"
        );
        assert!(
            text.contains(
                "unplanned 0 of 0 landings cited (0 failure · 0 revert · 0 fix) · 0 of 0 jobs landed (n/a)"
            ),
            "{text}"
        );
    }

    #[test]
    fn keep_rule_lines_show_the_measures_and_a_breach_line_only_when_breached() {
        let text = keep_rule_lines(&keep_rule(28, 51, KeepVerdict::Breach));
        for want in [
            "28 of 51 planned jobs landed (55%)",
            "baseline 64%",
            "cost per landed job $3.57",
            "first-round accept rate 50% (20 of 40)",
        ] {
            assert!(text.contains(want), "{want}: {text}");
        }
        let breach = text.lines().find(|l| l.contains("BREACH")).expect(&text);
        assert!(breach.contains("28 of 51") && breach.contains("55%") && breach.contains("64%"));

        let off = keep_rule_lines(&keep_rule(60, 100, KeepVerdict::TurnOff));
        assert!(
            off.contains("BREACH: the keep rule says turn planning off"),
            "{off}"
        );

        let kept = keep_rule_lines(&keep_rule(40, 51, KeepVerdict::Keep));
        assert!(
            kept.contains("40 of 51 planned jobs landed (78%)"),
            "{kept}"
        );
        assert!(!kept.contains("BREACH"), "{kept}");

        let mut none = keep_rule(0, 5, KeepVerdict::Breach);
        none.cost_per_landed_usd = None;
        none.first_round_accept_rate = None;
        let none = keep_rule_lines(&none);
        assert!(
            none.contains("cost per landed job n/a · first-round accept rate n/a"),
            "{none}"
        );
    }

    #[test]
    fn status_lines_show_a_non_normal_priority_and_the_plan_item_always_shows_one() {
        use athena_core::plan::Priority;
        let line = |priority| athena_core::status::JobLine {
            id: "j".into(),
            state: "open".into(),
            title: "t".into(),
            class: None,
            cost_usd: 0.0,
            plan: Some("plan accepted r1".into()),
            priority,
            hold: None,
        };
        assert_eq!(
            plan_suffix(&line(Priority::High)),
            " · priority high · plan accepted r1"
        );
        assert_eq!(plan_suffix(&line(Priority::Normal)), " · plan accepted r1");
    }

    #[test]
    fn status_renders_code_health_as_an_aligned_table_with_limits_and_suite_time() {
        use athena_core::code_health::{Breach, CodeHealth, FileHealth};
        let file = |path: &str, lines: u32, weekly: u32, over: bool| FileHealth {
            path: path.into(),
            lines,
            longest_fn: 40,
            longest_fn_name: "f".into(),
            weekly_landings: weekly,
            citations: 3,
            limit_lines: 2000,
            limit_fn: 200,
            breaches: if over {
                vec![Breach::FileLength { lines }]
            } else {
                vec![]
            },
        };
        let health = CodeHealth {
            files: vec![
                file("crates/athena-core/src/run.rs", 2412, 6, true),
                file("crates/athena-core/src/pr.rs", 310, 0, false),
            ],
            suite_seconds: Some(612.5),
        };
        let text = code_health_section(&health);
        assert!(text.contains("2000 production lines per file"), "{text}");
        assert!(text.contains("200 lines per function"), "{text}");
        assert!(
            text.contains("1500 lines with 5 or more landings"),
            "{text}"
        );
        assert!(text.contains("test suite run time: 612.5s"), "{text}");
        let header = text.lines().find(|l| l.contains("FILE")).unwrap();
        for name in [
            "LINES",
            "LONGEST FN",
            "LANDINGS 7D",
            "CITED",
            "LIMITS",
            "OVER",
        ] {
            assert!(header.contains(name), "{header}");
        }
        let over = text.lines().find(|l| l.contains("run.rs")).unwrap();
        let under = text.lines().find(|l| l.contains("pr.rs")).unwrap();
        let col = |line: &str, needle: &str| line.find(needle).unwrap();
        assert_eq!(col(header, "LINES"), col(over, "2412"));
        assert_eq!(col(header, "LINES"), col(under, "310"));
        assert_eq!(col(header, "LIMITS"), col(over, "2000 / 200"));
        assert_eq!(col(header, "OVER"), col(over, "yes: file length"));
        assert_eq!(col(header, "OVER"), col(under, "no"));
        assert!(text.contains("1 file(s) over a limit"), "{text}");
    }

    #[test]
    fn status_lists_flagged_and_paused_lines() {
        use athena_core::scope_guard::{LineMark, MarkNumbers, SignalFact};
        let mark = |level: &str, line: &str, signal: &str| LineMark {
            level: level.into(),
            line: line.into(),
            job: line.into(),
            at: "2026-10-26T00:00:00Z".into(),
            signals: vec![SignalFact {
                signal: signal.into(),
                level: level.into(),
                detail: "two rejections".into(),
            }],
            numbers: MarkNumbers {
                attempts: 2,
                rejections: 2,
                spend_usd: 0.5,
                forecast_p90_usd: 0.0,
                agent_minutes: 3,
                matched_criterion: None,
            },
            reason: "r".into(),
            lift: "narrow the request".into(),
        };
        assert_eq!(lines_section(&[]), "", "no marks, no section");
        let text = lines_section(&[mark("pause", "line-p", "T4"), mark("flag", "line-f", "T1")]);
        let paused = text.lines().find(|l| l.contains("line-p")).unwrap();
        assert!(paused.contains("paused") && paused.contains("T4"), "{text}");
        let flagged = text.lines().find(|l| l.contains("line-f")).unwrap();
        assert!(
            flagged.contains("flagged") && flagged.contains("T1"),
            "{text}"
        );
        assert!(text.contains("to lift: narrow the request"), "{text}");
    }

    #[test]
    fn a_red_report_names_the_test_and_says_the_failed_job_was_re_run_once() {
        let text = land_pr(&report(Checks::Red, 1, Some("gh::tests::x")));
        assert!(text.contains("test gh::tests::x failed"), "{text}");
        assert!(text.contains("re-run once"), "{text}");
        assert!(text.contains("nothing was merged"), "{text}");
    }

    #[test]
    fn a_red_report_names_the_test_even_when_nothing_was_re_run() {
        let text = land_pr(&report(Checks::Red, 0, Some("gh::tests::x")));
        assert!(text.contains("test gh::tests::x failed"), "{text}");
        assert!(!text.contains("re-run"), "{text}");
    }

    #[test]
    fn a_red_report_with_no_name_says_so() {
        let text = land_pr(&report(Checks::Red, 1, None));
        assert!(text.contains("could not be named"), "{text}");
    }

    #[test]
    fn a_green_report_after_a_rerun_says_so() {
        let text = land_pr(&report(Checks::Green, 1, None));
        assert!(text.contains("re-run once and then passed"), "{text}");
        assert!(!land_pr(&report(Checks::Green, 0, None)).contains("re-run"));
    }

    #[test]
    fn the_json_report_carries_the_rerun_and_the_failing_test() {
        let json = serde_json::to_value(report(Checks::Red, 1, Some("gh::tests::x"))).unwrap();
        assert_eq!(json["reruns"], 1);
        assert_eq!(json["failing_test"], "gh::tests::x");
    }

    #[test]
    fn durations_read_like_people_write_them() {
        assert_eq!(duration(42.0), "42s");
        assert_eq!(duration(240.0), "4m");
        assert_eq!(duration(365.0), "6m05s");
        assert_eq!(duration(3900.0), "1h05m");
    }
}
