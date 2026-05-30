use std::{path::Path, process::Command};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitState {
    pub commit: Option<String>,
    pub dirty: bool,
    pub status_porcelain: String,
}

pub fn capture_git_state(repo: impl AsRef<Path>) -> GitState {
    let repo = repo.as_ref();
    let commit = run_git(repo, &["rev-parse", "HEAD"]).ok();
    let status = run_git(repo, &["status", "--porcelain"]).unwrap_or_default();
    GitState {
        commit,
        dirty: !status.trim().is_empty(),
        status_porcelain: status,
    }
}

fn run_git(repo: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if !output.status.success() {
        anyhow::bail!("git {} failed", args.join(" "));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
