//! Every backend and performance-flag combination builds.
//!
//! This crate's optional dependencies are gated on the performance features, so naming one
//! outside those gates breaks a configuration nobody builds by default. That is what
//! happened: the shared compiled pattern reached for `once_cell`, which is optional and
//! gated on `optimize_for_cpu` / `optimize_for_memory`, so every build with a regex backend
//! and neither flag failed to resolve it while the default build was fine.
//!
//! Running the combinations by hand found it once. This is that check, kept.

use std::process::Command;

/// Runs `cargo check` for one feature selection.
fn check(features: &str) -> (bool, String) {
    let mut command = Command::new(env!("CARGO"));
    command
        .args(["check", "--quiet"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env(
            "CARGO_TARGET_DIR",
            concat!(env!("CARGO_MANIFEST_DIR"), "/target/feature-matrix"),
        );
    if features.is_empty() {
        // The default selection.
    } else {
        command.args(["--no-default-features", "--features", features]);
    }
    let output = command.output().expect("cargo runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

#[test]
fn the_default_selection_builds() {
    let (ok, err) = check("");
    assert!(ok, "the default features build:\n{err}");
}

#[test]
fn each_backend_builds_without_a_performance_flag() {
    // The configuration the `once_cell` reference broke, and the reason it went unnoticed:
    // `optimize_for_cpu` is on by default, so the default build never reached it.
    for backend in ["use_regex", "use_fancy_regex"] {
        let (ok, err) = check(backend);
        assert!(ok, "{backend} alone builds:\n{err}");
    }
}

#[test]
fn each_backend_builds_with_each_performance_flag() {
    for backend in ["use_regex", "use_fancy_regex"] {
        for flag in ["optimize_for_cpu", "optimize_for_memory"] {
            let features = format!("{backend},{flag}");
            let (ok, err) = check(&features);
            assert!(ok, "{features} builds:\n{err}");
        }
    }
}

#[test]
fn the_enhanced_accuracy_selection_builds() {
    let (ok, err) = check("use_fancy_regex,enhanced_accuracy");
    assert!(ok, "enhanced_accuracy builds:\n{err}");
}

#[test]
fn the_parity_cases_actually_run_when_both_backends_are_present() {
    // `tests/backend_parity.rs` is gated on a regex backend, so under the default selection
    // it reports `running 0 tests`, which is what a suite that stopped compiling reports
    // too. This runs it in the configuration it exists for and insists it executed
    // something, rather than merely that it built.
    let output = Command::new(env!("CARGO"))
        .args([
            "test",
            "--test",
            "backend_parity",
            "--features",
            "use_regex,use_fancy_regex",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env(
            "CARGO_TARGET_DIR",
            concat!(env!("CARGO_MANIFEST_DIR"), "/target/feature-matrix"),
        )
        .output()
        .expect("cargo runs");

    let report = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "the parity suite passes:\n{report}");

    let ran: usize = report
        .lines()
        .find_map(|line| line.strip_prefix("test result: ok. "))
        .and_then(|rest| rest.split(' ').next())
        .and_then(|count| count.parse().ok())
        .expect("the suite reported a result line");

    assert!(
        ran >= 8,
        "the parity suite ran {ran} cases, where it has at least 8. A suite that compiles \
         and executes nothing reports success just as loudly."
    );
}
