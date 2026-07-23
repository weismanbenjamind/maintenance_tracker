//! # Integration tests
//!
//! Tests public API

use std::path::Path;

use assert_cmd::Command;
use indoc::indoc;
use predicates;
use tempfile::{NamedTempFile, tempdir};

fn build_log() -> NamedTempFile {
    let tmpfile = NamedTempFile::with_suffix(".toml").unwrap();
    let contents = indoc! {"
        [services.oil_change]
        name = \"Oil Change\"
        notes = [
        \"Doing 3000 miles instead of 5000 mile intervals\",
        \"Still doing 6 months\",
        ]

        [services.oil_change.service_interval]
        miles = 3000
        months = 5

        [services.oil_change.next_service]
        miles = 100000
        date = \"2027-01-01\"

        [[services.oil_change.previous_services]]
        miles = 72000
        date = \"2026-06-19\"

        [[services.oil_change.previous_services]]
        miles = 100000
        date = \"2026-06-11\"
    "};
    std::fs::write(&tmpfile, contents).unwrap();
    tmpfile
}

#[test]
fn test_run_no_log() {
    let path = Path::new("some/path/that/does/not/exist");
    assert!(!path.exists());

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(path)
        .arg("list")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Failed to resolve path to config",
        ));
}

/*
 * THIS TEST WILL ATTEMPT TO READ THE ENVIRONMENT
 * ENVIRONMENT VARIABLES ARE PROCESS WIDE ACROSS THREADS
 * IF TWO THREADS ATTEMPT TO SET/UNSET maintenance_log_env
 * THEY WILL INTERFERE WITH EACH OTHER
 * RUN cargo test -- --test-threads=1 TO AVOID RACE CONDITIONS
 * LIKE THE ONE JUST DESCRIBED
*/
#[test]
fn test_run_use_env() {
    let tempfile = build_log();
    let bad_path = Path::new("some/path/that/does/not/exist");
    assert!(!bad_path.exists());

    let maintenance_log_env = "MAINTENANCE_LOG_ENV";
    assert!(std::env::var(maintenance_log_env).is_err());

    temp_env::with_var(maintenance_log_env, Some(tempfile.path()), || {
        Command::cargo_bin("maintenance_tracker")
            .unwrap()
            .arg("-m")
            .arg(bad_path)
            .arg("-e")
            .arg(maintenance_log_env)
            .arg("list")
            .assert()
            .success();
    })
}

#[test]
fn test_run_list_ok() {
    let tmpfile = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(tmpfile.path().to_str().unwrap())
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_run_vebosity() {
    let log = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(log.path())
        .arg("-vv")
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_run_help() {
    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_run_version() {
    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("--version")
        .assert()
        .success();
}

#[test]
fn test_complete_ok() {
    let tmpfile = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(tmpfile.path().to_str().unwrap())
        .arg("complete")
        .arg("oil_change")
        .arg("103000")
        .arg("-d")
        .arg("2027-01-01")
        .assert()
        .success();
}

#[test]
fn test_complete_err() {
    let tmpfile = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(tmpfile.path().to_str().unwrap())
        .arg("complete")
        .arg("-d")
        .arg("2027-01-01")
        .assert()
        .failure();
}

#[test]
fn test_delete_ok() {
    let tmpfile = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(tmpfile.path().to_str().unwrap())
        .arg("delete")
        .arg("oil_change")
        .assert()
        .success();
}

#[test]
fn test_delete_err() {
    let tmpfile = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(tmpfile.path().to_str().unwrap())
        .arg("delete")
        .assert()
        .failure();
}

#[test]
fn test_detail_ok() {
    let tmpfile = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(tmpfile.path().to_str().unwrap())
        .arg("detail")
        .arg("oil_change")
        .assert()
        .success();
}

#[test]
fn test_detail_err() {
    let tmpfile = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(tmpfile.path().to_str().unwrap())
        .arg("detail")
        .assert()
        .failure();
}

#[test]
fn test_diff_threshold_ok() {
    let tmpfile = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(tmpfile.path().to_str().unwrap())
        .arg("diff")
        .arg("oil_change")
        .arg("threshold")
        .arg("-m")
        .arg("100100")
        .arg("-c")
        .arg("97000")
        .arg("-d")
        .arg("2027-01-02")
        .assert()
        .success();
}

#[test]
fn test_diff_threshold_missing_curr_miles() {
    let tmpfile = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(tmpfile.path().to_str().unwrap())
        .arg("diff")
        .arg("oil_change")
        .arg("threshold")
        .arg("-m")
        .arg("100100")
        .arg("-d")
        .arg("2027-01-02")
        .assert()
        .failure();
}

#[test]
fn test_diff_threshold_missing_miles() {
    let tmpfile = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(tmpfile.path().to_str().unwrap())
        .arg("diff")
        .arg("oil_change")
        .arg("threshold")
        .arg("-c")
        .arg("97000")
        .arg("-d")
        .arg("2027-01-02")
        .assert()
        .failure();
}

#[test]
fn test_diff_threshold_missing_all() {
    let tmpfile = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(tmpfile.path().to_str().unwrap())
        .arg("diff")
        .arg("oil_change")
        .arg("threshold")
        .assert()
        .failure();
}

#[test]
fn test_diff_interval_ok() {
    let tmpfile = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(tmpfile.path().to_str().unwrap())
        .arg("diff")
        .arg("oil_change")
        .arg("interval")
        .arg("-m")
        .arg("5000")
        .arg("-c")
        .arg("97000")
        .arg("-n")
        .arg("2")
        .arg("-t")
        .arg("2026-12-31")
        .assert()
        .success();
}

#[test]
fn test_diff_interval_missing_curr_miles() {
    let tmpfile = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(tmpfile.path().to_str().unwrap())
        .arg("diff")
        .arg("oil_change")
        .arg("interval")
        .arg("-m")
        .arg("5000")
        .arg("-n")
        .arg("2")
        .arg("-t")
        .arg("2026-12-31")
        .assert()
        .failure();
}

#[test]
fn test_diff_interval_missing_miles() {
    let tmpfile = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(tmpfile.path().to_str().unwrap())
        .arg("diff")
        .arg("oil_change")
        .arg("interval")
        .arg("-c")
        .arg("97000")
        .arg("-n")
        .arg("2")
        .arg("-t")
        .arg("2026-12-31")
        .assert()
        .failure();
}

#[test]
fn test_diff_interval_missing_months_pass_date() {
    let tmpfile = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(tmpfile.path().to_str().unwrap())
        .arg("diff")
        .arg("oil_change")
        .arg("interval")
        .arg("-m")
        .arg("5000")
        .arg("-c")
        .arg("97000")
        .arg("-t")
        .arg("2026-12-31")
        .assert()
        .success();
}

#[test]
fn test_diff_interval_missing_all() {
    let tmpfile = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(tmpfile.path().to_str().unwrap())
        .arg("diff")
        .arg("oil_change")
        .arg("interval")
        .assert()
        .failure();
}

#[test]
fn test_init_ok() {
    let tmpfile = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(tmpfile.path().to_str().unwrap())
        .arg("init")
        .arg("Rear Diff")
        .arg("rear_diff")
        .arg("14000")
        .arg("17")
        .arg("114000")
        .arg("2028-05-01")
        .arg("-n")
        .arg("Test note")
        .arg("-p")
        .arg("100000;2027-01-01")
        .assert()
        .success();
}

#[test]
fn test_init_err() {
    let tmpfile = build_log();

    // Missing next service date - should cause the error
    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(tmpfile.path().to_str().unwrap())
        .arg("init")
        .arg("Rear Diff")
        .arg("rear_diff")
        .arg("14000")
        .arg("17")
        .arg("114000")
        .arg("-n")
        .arg("Test note")
        .arg("-p")
        .arg("100000;2027-01-01")
        .assert()
        .failure();
}

#[test]
fn test_log_ok() {
    let tmpdir = tempdir().unwrap();
    let log = tmpdir.path().join("maintenance_log.toml");
    assert!(!log.exists());

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("log")
        .arg(&log)
        .assert()
        .success();

    assert!(log.exists());
    assert!(!std::fs::read(log).unwrap().is_empty())
}

#[test]
fn test_log_err() {
    let log = NamedTempFile::with_suffix(".toml").unwrap();
    let log_path = log.path();
    assert!(log_path.exists());
    assert!(std::fs::read_to_string(log_path).unwrap().is_empty());

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("log")
        .arg(log_path)
        .assert()
        .success()
        .stdout(predicates::str::contains("Maintenance log already exists"));

    assert!(std::fs::read_to_string(log_path).unwrap().is_empty());
}

#[test]
fn test_next_ok() {
    let log = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(log.path())
        .arg("next")
        .arg("oil_change")
        .assert()
        .success();
}

#[test]
fn test_next_err() {
    let log = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(log.path())
        .arg("next")
        .assert()
        .failure();
}

#[test]
fn test_status_all() {
    let log = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(log.path())
        .arg("status")
        .arg("97000")
        .arg("-t")
        .arg("2026-12-15")
        .assert()
        .success();
}

#[test]
fn test_status_id() {
    let log = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(log.path())
        .arg("status")
        .arg("97000")
        .arg("-i")
        .arg("oil_change")
        .arg("-t")
        .arg("2026-12-15")
        .assert()
        .success();
}

#[test]
fn test_status_today_defaut() {
    let log = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(log.path())
        .arg("status")
        .arg("97000")
        .assert()
        .success();
}

#[test]
fn test_status_err() {
    let log = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(log.path())
        .arg("status")
        .assert()
        .failure();
}

#[test]
fn test_update_ok() {
    let log = build_log();
    let new_name = "new_name";
    let path = log.path();
    assert!(!std::fs::read_to_string(path).unwrap().contains(new_name));

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(path)
        .arg("update")
        .arg("oil_change")
        .arg("name")
        .arg(new_name)
        .assert()
        .success();

    assert!(std::fs::read_to_string(path).unwrap().contains(new_name));
}

#[test]
fn test_update_err() {
    let log = build_log();

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(log.path())
        .arg("update")
        .arg("name")
        .arg("lol")
        .assert()
        .failure();
}

#[test]
fn test_update_notes() {
    let log = build_log();
    let new_note = "new_note";
    let path = log.path();
    assert!(!std::fs::read_to_string(path).unwrap().contains(new_note));

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(path)
        .arg("update")
        .arg("oil_change")
        .arg("notes")
        .arg("append")
        .arg(new_note)
        .assert()
        .success();

    assert!(std::fs::read_to_string(path).unwrap().contains(new_note));
}

#[test]
fn test_update_previous_services() {
    let log = build_log();
    let new_miles = "50000";
    let new_date = "2026-04-11";

    let path = log.path();
    let path_contents = std::fs::read_to_string(path).unwrap();
    assert!(!path_contents.contains(new_miles));
    assert!(!path_contents.contains(new_date));

    Command::cargo_bin("maintenance_tracker")
        .unwrap()
        .arg("-m")
        .arg(path)
        .arg("update")
        .arg("oil_change")
        .arg("previous-services")
        .arg("append")
        .arg(new_miles)
        .arg("-d")
        .arg(new_date)
        .assert()
        .success();

    let path_contents = std::fs::read_to_string(path).unwrap();
    assert!(path_contents.contains(new_miles));
    assert!(path_contents.contains(new_date));
}
