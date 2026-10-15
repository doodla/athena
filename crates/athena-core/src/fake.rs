//! Test doubles for the core's seams, shared by this crate's tests and the CLI's lifecycle tests.

use crate::pin::Pinner;
use anyhow::{Result, bail};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// A pinner that touches no network: it counts its calls, records what it was asked, and writes one file
/// into the tree it "checks out". Clones share the same counters.
#[derive(Debug, Clone)]
pub struct CountingPinner {
    sha: String,
    /// Whether a ref that is itself a full commit id resolves to itself, as the real pinner does.
    echo_full_sha: bool,
    calls: Arc<Mutex<Vec<(String, String)>>>,
}

/// The file a [`CountingPinner`] writes into the pinned tree.
pub const FAKE_TREE_FILE: &str = "PINNED.txt";

impl CountingPinner {
    /// A pinner that resolves every ref to `sha`.
    pub fn new(sha: &str) -> CountingPinner {
        CountingPinner {
            sha: sha.to_string(),
            echo_full_sha: false,
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Makes a full 40-hex ref resolve to itself; any other ref still resolves to the fixed sha.
    pub fn echoing_full_shas(mut self) -> CountingPinner {
        self.echo_full_sha = true;
        self
    }

    /// How many times `pin` was called.
    pub fn count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    /// The `(url, ref)` pairs it was called with, in order.
    pub fn requests(&self) -> Vec<(String, String)> {
        self.calls.lock().unwrap().clone()
    }
}

impl Pinner for CountingPinner {
    fn pin(&self, url: &str, git_ref: &str, dest: &Path) -> Result<String> {
        self.calls
            .lock()
            .unwrap()
            .push((url.to_string(), git_ref.to_string()));
        if dest.exists() {
            bail!("pin destination {} already exists", dest.display());
        }
        std::fs::create_dir_all(dest)?;
        let sha = if self.echo_full_sha && crate::pin::is_full_sha(git_ref) {
            git_ref.to_string()
        } else {
            self.sha.clone()
        };
        std::fs::write(dest.join(FAKE_TREE_FILE), format!("pinned at {sha}\n"))?;
        Ok(sha)
    }
}
