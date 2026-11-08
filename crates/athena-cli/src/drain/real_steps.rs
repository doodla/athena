//! The drain's steps against a real repository.

use super::*;

/// The default priority of the work a role raises (it applies when the job's own words give none): the `Priority: <level>` line in the standing instruction of
/// the role's newest recorded definition, normal when the role is not recorded or states none.
pub(super) fn role_priority(records: &Records, role: &str) -> athena_core::plan::Priority {
    records
        .roles
        .iter()
        .filter(|r| r.name == role)
        .max_by_key(|r| r.version)
        .map_or(athena_core::plan::Priority::Normal, |r| {
            athena_core::plan::Priority::from_text(&r.prompt)
        })
}

/// The steps against a real repository.
pub struct RealSteps<'a> {
    pub athena: &'a Athena,
    // `Sync` so that [`drain_concurrent`] can share the steps between workers.
    pub ui: &'a (dyn Ui + Sync),
    pub page: &'a dyn Page,
    pub lander: &'a (dyn Lander + Sync),
}

impl std::fmt::Debug for RealSteps<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RealSteps")
            .field("athena", &self.athena)
            .finish_non_exhaustive()
    }
}

impl<'a> RealSteps<'a> {
    /// The live explainer page, landed through the pull-request path.
    pub fn live(athena: &'a Athena, ui: &'a (dyn Ui + Sync)) -> RealSteps<'a> {
        RealSteps {
            athena,
            ui,
            page: &LivePage,
            lander: &PrLander,
        }
    }

    pub(super) fn records(&self) -> anyhow::Result<Records> {
        Records::load(&self.athena.git, &self.athena.head()?)
    }

    pub(super) fn job_cost(&self, job: &str) -> f64 {
        self.records()
            .ok()
            .and_then(|r| r.jobs.get(job).map(|j| j.spent_usd()))
            .unwrap_or(0.0)
    }

    /// The plan entry of `job` at `head`: the rung of the proposal that made it when there is one, and lookups
    /// of the paths and referenced jobs it names.
    pub(super) fn entry_of(&self, records: &Records, head: &str, job: &str) -> Option<PlanEntry> {
        let record = records.jobs.get(job)?;
        // The proposal this job carries out, if it was made by approving one.
        let proposal = records.work_items.iter().find(|work| {
            work_decisions::decision_of(records, work).is_some_and(|decision| {
                matches!(
                    &decision.decision,
                    athena_data::records::WorkDecisionState::Approved { job: made }
                        if made.as_str() == job
                )
            })
        });
        let hint = proposal.map(|work| work.rung);
        let job_exists = |id: &str| records.jobs.contains_key(id);
        // A lookup that fails counts the path as present, so a git error never invents an open question.
        let path_exists = |path: &str| self.athena.git.path_exists(head, path).unwrap_or(true);
        // The requirements and addenda a request can serve; no brief at the base commit means none.
        let brief = self
            .athena
            .git
            .show_file(head, "BRIEF.md")
            .ok()
            .flatten()
            .map(|b| athena_core::reading::brief_topics(&String::from_utf8_lossy(&b)))
            .unwrap_or_default();
        let mut entry = athena_core::reading::plan_entry(
            record,
            hint,
            &athena_core::reading::Lookup {
                job_exists: &job_exists,
                path_exists: &path_exists,
                brief: &brief,
            },
        );
        // A job made from a proposal ranks like any other item: normal unless a priority record, its words or
        // the role that raised it say otherwise.
        if let Some(work) = proposal {
            entry.priority = athena_core::plan::proposal_priority(
                (!record.priorities.is_empty()).then(|| record.priority()),
                entry.priority,
                role_priority(records, &work.role),
            );
        }
        Some(entry)
    }

    pub(super) fn audience_cost(&self) -> f64 {
        self.records()
            .map(|r| r.audience_runs.iter().map(|i| i.cost.best_lower()).sum())
            .unwrap_or(0.0)
    }

    pub(super) fn table(&self) -> InFlight {
        InFlight::at(self.athena.state.root())
    }

    /// A table write that fails is a warning line: the table is bookkeeping and never fails a job.
    pub(super) fn table_result(&self, result: anyhow::Result<()>) {
        if let Err(e) = result {
            self.ui.line(&format!(
                "warning: could not write the in-flight table: {e:#}"
            ));
        }
    }
}

impl Steps for RealSteps<'_> {
    /// Asks [`Athena::line_pause`], which recomputes the line's level from the records, so a stored pause that
    /// no longer holds is not reported. Records that cannot be loaded report nothing paused: the read then
    /// refuses a paused line with the pause text, as it did before.
    fn paused(&self, jobs: &[String]) -> Vec<(String, String)> {
        let Ok(records) = self.records() else {
            return Vec::new();
        };
        jobs.iter()
            .filter_map(|job| {
                let mark = self.athena.line_pause(&records, job)?;
                Some((job.clone(), self.athena.pause_refusal(&mark)))
            })
            .collect()
    }

    /// Asks [`Athena::line_flag`], which recomputes the line's level from the records, so a stored flag that
    /// no longer holds is not reported. Records that cannot be loaded report nothing flagged, so the item then
    /// ends as a plain double failure (`ItemEnd::Failed`), which is passed over too.
    fn flagged(&self, jobs: &[String]) -> Vec<(String, String)> {
        let Ok(records) = self.records() else {
            return Vec::new();
        };
        jobs.iter()
            .filter_map(|job| {
                let mark = self.athena.line_flag(&records, job)?;
                Some((job.clone(), self.athena.flag_note(&mark)))
            })
            .collect()
    }

    fn review_passes(&self) -> Vec<String> {
        self.athena.review_passes(self.ui)
    }

    fn caps(&self) -> anyhow::Result<Caps> {
        let jc = athena_core::capability::JobConfig::at(&self.athena.git, &self.athena.head()?)?;
        Ok(Caps {
            read: jc.config.budget.reader_usd,
            run: jc.config.budget.job_usd,
            rate: jc.config.budget.verifier_usd,
        })
    }

    fn queue(&self) -> anyhow::Result<QueueView> {
        let records = self.records()?;
        let waiting = |items: Vec<&athena_data::records::WorkItem>| -> Vec<Waiting> {
            items
                .into_iter()
                .map(|w| Waiting {
                    id: w.id.to_string(),
                    title: w.title.clone(),
                    rung: w.rung.as_str().to_string(),
                })
                .collect()
        };
        let (within, above) = work_decisions::split_open(&records);
        let item = |w: steward::QueuedWork| Item {
            job: w.job.as_str().to_string(),
            title: w.title,
            read: w.read,
            interrupted: false,
        };
        let mut items: Vec<Item> = steward::approved_work(&records)
            .into_iter()
            .map(item)
            .collect();
        // A job a work decision approved can also sit on a drained channel: it is listed once.
        let queued: Vec<Item> = steward::queued_work(&records)
            .into_iter()
            .map(item)
            .filter(|i| !items.iter().any(|a| a.job == i.job))
            .collect();
        items.extend(queued);
        // A job a restart interrupted is worked by retrying it. The registry is cleaned first, so a job whose
        // process is gone counts as interrupted.
        self.athena.state.registry().reap();
        items.extend(
            steward::interrupted_work(&records, &|j| self.athena.running(j))
                .into_iter()
                .map(|w| Item {
                    interrupted: true,
                    ..item(w)
                }),
        );
        // Planning: every open request is fleshed out. Nothing is defaulted: a job that came from a steward
        // proposal serves the proposal's rung; any other request's rung is inferred or flagged.
        let head = self.athena.head()?;
        let plan = items
            .iter()
            .filter_map(|i| self.entry_of(&records, &head, &i.job))
            .collect();
        Ok(QueueView {
            items,
            waiting: waiting(above),
            approvable: waiting(within),
            plan,
        })
    }

    fn stop_requested(&self) -> bool {
        stop_file(self.athena.state.root()).exists()
    }

    fn plan_entry(&self, job: &str) -> Option<PlanEntry> {
        let records = self.records().ok()?;
        let head = self.athena.head().ok()?;
        self.entry_of(&records, &head, job)
    }

    fn job_history(&self) -> Vec<JobEnd> {
        self.records()
            .map(|r| athena_core::reading::job_history(&r))
            .unwrap_or_default()
    }

    fn priority_caps(&self) -> athena_core::plan::TierCaps {
        let caps = self
            .athena
            .head()
            .and_then(|head| athena_core::capability::JobConfig::at(&self.athena.git, &head))
            .map(|jc| jc.config.priority_caps())
            .unwrap_or_default();
        athena_core::plan::TierCaps {
            urgent: caps.urgent as usize,
            high: caps.high as usize,
        }
    }

    fn improvement(&self) -> athena_core::plan::Improvement {
        self.records()
            .map(|records| athena_core::hestia::improvement_of(&records))
            .unwrap_or_default()
    }

    fn person_set(&self) -> Vec<String> {
        self.records()
            .map(|records| {
                records
                    .jobs
                    .iter()
                    .filter(|(_, job)| !job.priorities.is_empty())
                    .map(|(id, _)| id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn approve(&self, proposal: &str) -> Approval {
        match self.athena.approve_in_limit(proposal, self.ui) {
            Ok(p) => Approval {
                job: Some(p.job.as_str().to_string()),
                cost: p.spent_usd,
                error: p.reader_error.clone(),
            },
            Err(error) => Approval {
                job: None,
                cost: 0.0,
                error: Some(format!("{error:#}")),
            },
        }
    }

    fn previous_rating(&self) -> Option<u8> {
        self.records()
            .ok()?
            .audience_ratings
            .last()
            .map(|r| r.rating)
    }

    fn read(&self, what: ReadOf<'_>) -> Read {
        let (result, before) = match what {
            ReadOf::Queued(job) => (
                match self.athena.go(job, AcceptedBy::Flag, self.ui) {
                    Ok(athena_core::run::GoResult::Reread(p)) => Ok(*p),
                    Ok(athena_core::run::GoResult::Finished(_)) => {
                        Err(anyhow::anyhow!("the request was already read and run"))
                    }
                    Err(error) => Err(error),
                },
                self.job_cost(job),
            ),
            ReadOf::Retry(job) => (self.athena.retry(job, self.ui).map(|(p, _)| p), 0.0),
        };
        if let ReadOf::Retry(old) = what {
            // The retry is a new job. A row waiting for it (the checks or verifier the old job reached) goes
            // to the new job, keeping its stage, until the new job's producer enters; a retry that cannot run
            // leaves no row.
            let moved = match &result {
                Ok(p)
                    if p.reader_error.is_none()
                        && p.reading.as_ref().is_some_and(|r| r.capability.is_some()) =>
                {
                    self.table()
                        .move_row(old, p.job.as_str(), self.athena.state.work_dir(&p.job))
                }
                _ => self.table().remove(old),
            };
            self.table_result(moved);
        }
        match result {
            Ok(p) => Read {
                runnable: p.reading.as_ref().is_some_and(|r| r.capability.is_some()),
                error: p.reader_error.clone(),
                cost: p.spent_usd,
                job: Some(p.job.as_str().to_string()),
                held: false,
            },
            Err(error) => Read {
                job: None,
                cost: match what {
                    ReadOf::Queued(job) => (self.job_cost(job) - before).max(0.0),
                    ReadOf::Retry(_) => 0.0,
                },
                runnable: false,
                error: Some(format!("{error:#}")),
                held: error
                    .downcast_ref::<athena_core::reading::BudgetShort>()
                    .is_some(),
            },
        }
    }

    fn reclaim_interrupted(&self) {
        self.athena.state.registry().reap();
        self.athena.reclaim_interrupted(self.ui);
    }

    fn run(&self, job: &str) -> Run {
        let before = self.job_cost(job);
        // The run reports each stage it enters; the table holds the latest.
        let sink = TableSink {
            table: self.table(),
            athena: self.athena,
            ui: self.ui,
        };
        let result = self
            .athena
            .go_with_stages(job, AcceptedBy::Flag, self.ui, &sink);
        let cost = (self.job_cost(job) - before).max(0.0);
        match result {
            Ok(athena_core::run::GoResult::Finished(f)) => Run {
                outcome: Some(f.state),
                reason: f.reason,
                cost,
                needs_accepting: false,
                held: false,
            },
            Ok(athena_core::run::GoResult::Reread(p)) => Run {
                outcome: None,
                reason: format!(
                    "the request was read again after a configuration change and the new reading's scope differs from the earlier one{}; the reading needs accepting (`athena go {job}`)",
                    p.reader_error
                        .as_ref()
                        .map_or(String::new(), |e| format!(" ({e})"))
                ),
                cost,
                needs_accepting: true,
                held: false,
            },
            Err(error) => {
                let short = error
                    .downcast_ref::<athena_core::reading::BudgetShort>()
                    .is_some();
                Run {
                    outcome: None,
                    reason: if short {
                        format!("{BUDGET_SHORT}: {error:#}")
                    } else {
                        format!("{error:#}")
                    },
                    cost,
                    needs_accepting: false,
                    held: short
                        || error.downcast_ref::<athena_core::disk::LowDisk>().is_some()
                        || error
                            .downcast_ref::<athena_core::disk::SlotWait>()
                            .is_some(),
                }
            }
        }
    }

    fn land(&self, job: &str) -> Landing {
        // `go` does not land a verified job; this is where the drain enters the landing stage.
        if let Ok(id) = JobId::parse(job) {
            TableSink {
                table: self.table(),
                athena: self.athena,
                ui: self.ui,
            }
            .enter(&id, Stage::Landing);
        }
        let landing = match self.lander.land(self.athena, job, self.ui) {
            // What the landing changed is what the job's candidate changed; a landing whose candidate cannot be
            // read is taken to have changed nothing the page is built from.
            Landing::Landed { .. } => Landing::Landed {
                changed: self
                    .records()
                    .ok()
                    .and_then(|r| {
                        r.jobs.get(job).and_then(|j| {
                            j.candidate
                                .as_ref()
                                .map(|c| c.changes.iter().map(|e| e.path.clone()).collect())
                        })
                    })
                    .unwrap_or_default(),
            },
            other => other,
        };
        // The landing is over, landed or not: a landing that failed is tried again by the drain's next
        // attempt, which sets the row again.
        self.table_result(self.table().remove(job));
        landing
    }

    fn config_after_landing(&self) -> Option<String> {
        // Any read error here is taken as "the landed config is newer than this build": the config was
        // readable before the landing (`caps` parsed it at the start), so the landing is what changed it.
        // The error text is kept so damage of another kind stays visible in the report.
        let head = match self.athena.head() {
            Ok(head) => head,
            Err(error) => return Some(format!("{error:#}")),
        };
        athena_core::capability::JobConfig::unreadable_at(&self.athena.git, &head)
    }

    fn in_flight(&self) -> Vec<String> {
        let found = self.table().read();
        let mut lines: Vec<String> = found
            .rows
            .iter()
            .map(|r| {
                format!(
                    "the in-flight table holds a row left by an earlier run: job {} was at stage {} in {}; it is not acted on",
                    r.job,
                    r.stage.as_str(),
                    r.worktree.display()
                )
            })
            .collect();
        lines.extend(found.corrupt.map(|why| {
            format!("the in-flight table was not read and the drain goes on without it: {why}")
        }));
        lines
    }

    fn item_over(&self, job: &str) {
        self.table_result(self.table().remove(job));
    }

    fn rebuild(&self) -> Result<Rebuild, String> {
        run_setup(self.athena.git.dir())
    }

    fn check_cap(&self) -> f64 {
        let studies = self
            .athena
            .head()
            .and_then(|head| study_files(&self.athena.git, &head))
            .map_or(1, |s| s.len().max(1));
        let one = self.caps().map_or(0.0, |c| c.rate);
        one * studies as f64
    }

    fn check_page(&self) -> Check {
        let studies = match self
            .athena
            .head()
            .and_then(|head| study_files(&self.athena.git, &head))
        {
            Ok(studies) => studies,
            Err(error) => return Check::fail(format!("the studies could not be read: {error:#}")),
        };
        if studies.is_empty() {
            return Check::fail(
                "no study was found under studies/, so there is no page text to check",
            );
        }
        // The floor: each model must be consistent with itself. It costs nothing, so it goes first.
        let mut problems = Vec::new();
        for (name, files) in &studies {
            if let Some((_, bytes)) = files.iter().find(|(f, _)| f == "model.json") {
                match serde_json::from_slice::<serde_json::Value>(bytes) {
                    Ok(model) => problems.extend(
                        model_contradictions(&model)
                            .into_iter()
                            .map(|p| format!("studies/{name}/model.json: {p}")),
                    ),
                    Err(error) => problems.push(format!(
                        "studies/{name}/model.json: not valid JSON: {error}"
                    )),
                }
            }
        }
        if !problems.is_empty() {
            return Check::fail(problems.join("; "));
        }
        // The source check: every page text file of every study, against that study's pinned source.
        let mut cost = 0.0;
        let mut files_checked = 0;
        for (name, files) in &studies {
            let study = match study_text(name, files) {
                Ok(study) => study,
                Err(why) => {
                    problems.push(format!("studies/{name}: {why}"));
                    continue;
                }
            };
            files_checked += study.files.len();
            let report = self.athena.check_study_text(&study);
            cost += report.cost_usd;
            problems.extend(report.problems);
        }
        if problems.is_empty() {
            Check::pass(format!(
                "{} study(ies), {files_checked} page text file(s), match the pinned source and are consistent with themselves",
                studies.len()
            ))
            .costing(cost)
        } else {
            Check::fail(problems.join("; ")).costing(cost)
        }
    }

    fn rate(&self) -> Rate {
        let before = self.audience_cost();
        let result = self.athena.audience(self.page, &Target::explainer());
        let cost = (self.audience_cost() - before).max(0.0);
        match result {
            Ok(Some(rated)) => Rate {
                rating: Some(rated.rating.rating),
                cost,
                error: None,
            },
            Ok(None) => Rate {
                rating: None,
                cost,
                error: Some("a nested run makes no rating".into()),
            },
            Err(error) => Rate {
                rating: None,
                cost,
                error: Some(format!("{error:#}")),
            },
        }
    }
}
