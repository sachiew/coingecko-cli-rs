//! Integration tests — deterministic CLI parsing checks (no network).

use std::process::Command;

fn cg() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cg"))
}

#[test]
fn bare_cg_exits_zero() {
    let output = cg().output().expect("failed to run");
    assert!(output.status.success());
}

#[test]
fn help_flag_exits_zero() {
    let output = cg().arg("--help").output().expect("failed to run");
    assert!(output.status.success());
}

#[test]
fn help_contains_about() {
    let output = cg().arg("--help").output().expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("CoinGecko CLI"));
}

#[test]
fn version_flag_exits_zero() {
    let output = cg().arg("--version").output().expect("failed to run");
    assert!(output.status.success());
}

#[test]
fn version_contains_semver() {
    let output = cg().arg("--version").output().expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn price_help_shows_options() {
    let output = cg()
        .args(["price", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--ids"));
    assert!(stdout.contains("--vs"));
}

#[test]
fn markets_help_shows_options() {
    let output = cg()
        .args(["markets", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--total"));
    assert!(stdout.contains("--order"));
    assert!(stdout.contains("--category"));
}

#[test]
fn history_help_shows_options() {
    let output = cg()
        .args(["history", "--help"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--date"));
    assert!(stdout.contains("--days"));
}

#[test]
fn unknown_subcommand_fails() {
    let output = cg().arg("nonexistent").output().expect("failed to run");
    assert!(!output.status.success());
}
