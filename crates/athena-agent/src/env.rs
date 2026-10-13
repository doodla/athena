//! Agent and check environments (design §4 rule 1, §7.6, §10.4; M10, M25, M26).
//!
//! An agent's process starts from an empty environment. Athena adds only:
//!
//! - the variables `athena.toml` lists in `[agents] env_pass` (always) and `env_pass_if_set` (when set);
//! - `CLAUDE_CODE_DISABLE_AUTO_MEMORY=1`, a second layer under `--restricted` (E19);
//! - the Bash timeouts;
//! - `GIT_CEILING_DIRECTORIES` (the snapshot's parent), `GIT_CONFIG_GLOBAL=/dev/null`, and
//!   `GIT_CONFIG_NOSYSTEM=1`, so git inside the snapshot finds nothing of the user's;
//! - the nested-run marker, `ATHENA_NESTED=<job>`;
//! - `ATHENA_STATE_DIR` inside the job's own directory, so an Athena that an agent or a check runs keeps
//!   its state (kill switch, locks, scratch repositories) there and never in the user's (D56);
//! - the capability's environment, with `{job_dir}`, `{side}` and `{athena_home}` filled in. A build directory outside the
//!   job's own directory is refused, so one job's checks never run another job's build (D46).

use crate::NESTED_ENV;
use athena_data::config::Agents;
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

/// Which side of a job an environment serves; each side has its own build directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// The reader, which builds nothing.
    Read,
    /// The producer.
    Produce,
    /// The checks and the verifier.
    Verify,
}

impl Side {
    pub fn as_str(self) -> &'static str {
        match self {
            Side::Read => "read",
            Side::Produce => "produce",
            Side::Verify => "verify",
        }
    }
}

/// Variables that name build output locations; each must lie inside the job's own directory.
pub const BUILD_DIR_VARS: [&str; 2] = ["CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR"];

/// Where an Athena run inside a job keeps its state, relative to the job's directory.
pub const NESTED_STATE_DIR: &str = "athena-state";

/// What an environment is built from.
#[derive(Debug, Clone)]
pub struct EnvSpec<'a> {
    pub agents: &'a Agents,
    pub job: &'a str,
    /// The job's own state directory.
    pub job_dir: &'a Path,
    /// The snapshot the process works in.
    pub snapshot: &'a Path,
    pub side: Side,
    /// The capability's environment templates (`[capabilities.<id>.env]`); empty for the reader.
    pub capability_env: &'a BTreeMap<String, String>,
}

/// Builds the complete environment. `lookup` reads the parent's environment (injected for tests).
pub fn build(
    spec: &EnvSpec<'_>,
    lookup: impl Fn(&str) -> Option<String>,
) -> Result<BTreeMap<String, String>, String> {
    let mut env = BTreeMap::new();
    for key in &spec.agents.env_pass {
        if let Some(value) = lookup(key) {
            env.insert(key.clone(), value);
        }
    }
    for key in &spec.agents.env_pass_if_set {
        if let Some(value) = lookup(key).filter(|v| !v.is_empty()) {
            env.insert(key.clone(), value);
        }
    }
    env.insert("CLAUDE_CODE_DISABLE_AUTO_MEMORY".into(), "1".into());
    env.insert(
        "BASH_DEFAULT_TIMEOUT_MS".into(),
        spec.agents.bash_default_timeout_ms.to_string(),
    );
    env.insert(
        "BASH_MAX_TIMEOUT_MS".into(),
        spec.agents.bash_max_timeout_ms.to_string(),
    );
    let ceiling = spec.snapshot.parent().unwrap_or(spec.snapshot);
    env.insert(
        "GIT_CEILING_DIRECTORIES".into(),
        ceiling.display().to_string(),
    );
    env.insert("GIT_CONFIG_GLOBAL".into(), "/dev/null".into());
    env.insert("GIT_CONFIG_NOSYSTEM".into(), "1".into());
    env.insert(NESTED_ENV.into(), spec.job.to_string());
    env.insert(
        "ATHENA_STATE_DIR".into(),
        spec.job_dir.join(NESTED_STATE_DIR).display().to_string(),
    );
    for (key, template) in spec.capability_env {
        let value = expand(
            template,
            spec.job_dir,
            spec.side,
            &athena_data::paths::athena_home(),
        );
        env.insert(key.clone(), value);
    }
    check_build_dirs(&env, spec.job_dir)?;
    Ok(env)
}

/// Fills in `{job_dir}`, `{side}` and `{athena_home}` in one environment value.
pub fn expand(template: &str, job_dir: &Path, side: Side, athena_home: &Path) -> String {
    template
        .replace("{job_dir}", &job_dir.display().to_string())
        .replace("{side}", side.as_str())
        .replace("{athena_home}", &athena_home.display().to_string())
}

/// Refuses an environment whose build directory is not inside the job's own directory (M26).
pub fn check_build_dirs(env: &BTreeMap<String, String>, job_dir: &Path) -> Result<(), String> {
    let job_dir = normalize(job_dir);
    for key in BUILD_DIR_VARS {
        if let Some(value) = env.get(key) {
            let dir = normalize(Path::new(value));
            if !dir.is_absolute() || !dir.starts_with(&job_dir) || dir == job_dir {
                return Err(format!(
                    "{key}={value} is not inside this job's directory {}; each job builds in its own directories (D46)",
                    job_dir.display()
                ));
            }
        }
    }
    Ok(())
}

/// Resolves `.` and `..` lexically.
pub fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_data::Config;

    fn agents() -> Agents {
        Config::parse(include_str!("../../../athena.toml"))
            .unwrap()
            .agents
    }

    fn parent_env(key: &str) -> Option<String> {
        match key {
            "HOME" => Some("/home/someone".into()),
            "PATH" => Some("/usr/bin".into()),
            "LANG" => Some("C.UTF-8".into()),
            "SECRET_SESSION_ID" => Some("parent-session".into()),
            "CLAUDE_CODE_SESSION_ID" => Some("parent-session".into()),
            _ => None,
        }
    }

    #[test]
    fn only_allow_listed_variables_reach_agents() {
        // M10 (blocks): an environment variable not on the allow-list; (allows): allow-listed variables.
        let agents = agents();
        let capability_env = BTreeMap::from([(
            "CARGO_TARGET_DIR".to_string(),
            "{job_dir}/target/{side}".to_string(),
        )]);
        let spec = EnvSpec {
            agents: &agents,
            job: "20261010-153012-k7q2",
            job_dir: Path::new("/state/jobs/20261010-153012-k7q2"),
            snapshot: Path::new("/state/jobs/20261010-153012-k7q2/work"),
            side: Side::Produce,
            capability_env: &capability_env,
        };
        let env = build(&spec, parent_env).unwrap();
        assert_eq!(env.get("HOME").map(String::as_str), Some("/home/someone"));
        assert_eq!(env.get("LANG").map(String::as_str), Some("C.UTF-8"));
        assert!(!env.contains_key("SECRET_SESSION_ID"));
        assert!(
            !env.contains_key("CLAUDE_CODE_SESSION_ID"),
            "the parent's session never reaches an agent"
        );
        assert_eq!(env["CLAUDE_CODE_DISABLE_AUTO_MEMORY"], "1");
        assert_eq!(env["GIT_CONFIG_GLOBAL"], "/dev/null");
        assert_eq!(
            env["GIT_CEILING_DIRECTORIES"],
            "/state/jobs/20261010-153012-k7q2"
        );
        assert_eq!(
            env[NESTED_ENV], "20261010-153012-k7q2",
            "M25: the nested-run marker is set"
        );
        assert_eq!(
            env["ATHENA_STATE_DIR"], "/state/jobs/20261010-153012-k7q2/athena-state",
            "an Athena run inside the job keeps its state in the job's directory"
        );
        assert_eq!(
            env["CARGO_TARGET_DIR"],
            "/state/jobs/20261010-153012-k7q2/target/produce"
        );
    }

    #[test]
    fn athena_home_is_filled_in_beside_the_other_placeholders() {
        let out = expand(
            "{athena_home}/tools/sccache/1.0.0/bin/sccache",
            Path::new("/state/jobs/j"),
            Side::Verify,
            Path::new("/h/athena"),
        );
        assert_eq!(out, "/h/athena/tools/sccache/1.0.0/bin/sccache");
        let out = expand(
            "{job_dir}/{side}/{athena_home}",
            Path::new("/state/jobs/j"),
            Side::Produce,
            Path::new("/h/athena"),
        );
        assert_eq!(out, "/state/jobs/j/produce//h/athena");
        // Through `build`, no placeholder is left behind.
        let agents = agents();
        let capability_env =
            BTreeMap::from([("RUSTC_WRAPPER".to_string(), "{athena_home}/x".to_string())]);
        let env = build(
            &EnvSpec {
                agents: &agents,
                job: "j",
                job_dir: Path::new("/state/jobs/j"),
                snapshot: Path::new("/state/jobs/j/work"),
                side: Side::Produce,
                capability_env: &capability_env,
            },
            parent_env,
        )
        .unwrap();
        assert!(!env["RUSTC_WRAPPER"].contains('{'));
        assert!(env["RUSTC_WRAPPER"].ends_with("/x"));
    }

    #[test]
    fn each_jobs_environment_names_its_own_build_directories() {
        // M26 (allows): each job's environment names its own directories.
        let agents = agents();
        let capability_env = BTreeMap::from([(
            "CARGO_TARGET_DIR".to_string(),
            "{job_dir}/target/{side}".to_string(),
        )]);
        let a = Path::new("/state/jobs/a");
        let b = Path::new("/state/jobs/b");
        let env_for = |dir: &Path, side| {
            build(
                &EnvSpec {
                    agents: &agents,
                    job: "j",
                    job_dir: dir,
                    snapshot: &dir.join("verify"),
                    side,
                    capability_env: &capability_env,
                },
                parent_env,
            )
            .unwrap()
        };
        assert_ne!(
            env_for(a, Side::Verify)["CARGO_TARGET_DIR"],
            env_for(b, Side::Verify)["CARGO_TARGET_DIR"]
        );
        assert_ne!(
            env_for(a, Side::Produce)["CARGO_TARGET_DIR"],
            env_for(a, Side::Verify)["CARGO_TARGET_DIR"]
        );
    }

    #[test]
    fn a_build_directory_outside_the_job_fails_to_construct() {
        // M26 (blocks): a check or agent environment whose build directory is not inside its own job's
        // directory fails to construct.
        let agents = agents();
        for template in [
            "/shared/target",
            "{job_dir}/../other/target",
            "relative/target",
        ] {
            let capability_env =
                BTreeMap::from([("CARGO_TARGET_DIR".to_string(), template.to_string())]);
            let spec = EnvSpec {
                agents: &agents,
                job: "j",
                job_dir: Path::new("/state/jobs/j"),
                snapshot: Path::new("/state/jobs/j/verify"),
                side: Side::Verify,
                capability_env: &capability_env,
            };
            assert!(
                build(&spec, parent_env).is_err(),
                "{template} must be refused"
            );
        }
    }
}
