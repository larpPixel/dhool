use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;

fn sample_config() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../configs/inferno.production.yaml")
}

#[test]
fn validate_accepts_sample_config() {
    let mut command = Command::cargo_bin("dhool").expect("dhool binary");
    command
        .arg("validate")
        .arg(sample_config())
        .assert()
        .success()
        .stdout(predicate::str::contains("valid config"));
}

#[test]
fn device_info_reports_cuda() {
    let mut command = Command::cargo_bin("dhool").expect("dhool binary");
    command
        .arg("device-info")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"backend\": \"cuda\""))
        .stdout(predicate::str::contains("\"available\": true"));
}

#[test]
fn still_rejects_cpu_reference_backend() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mut command = Command::cargo_bin("dhool").expect("dhool binary");
    command
        .arg("still")
        .arg(sample_config())
        .arg("--backend")
        .arg("cpu-reference")
        .arg("--output-dir")
        .arg(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "cpu-reference is only for setup validation tests",
        ));
}
