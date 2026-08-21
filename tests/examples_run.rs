//! The examples are built by `cargo test` and never run by it, so they are run here.
//!
//! An example that compiles and then panics, or prints the wrong thing, is an example that
//! lies to whoever copies it. Each one is run in the feature selection it is written for,
//! and its output checked against what it claims.

use std::process::Command;

/// Runs one example and returns what it printed, failing with what cargo said if it did
/// not exit zero.
fn run_example(name: &str, features: &[&str]) -> String {
    let mut command = Command::new(env!("CARGO"));
    command
        .args(["run", "-q", "--example", name])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        // A target directory of its own, so the outer run's build lock is not held against
        // this one.
        .env("CARGO_TARGET_DIR", concat!(env!("CARGO_MANIFEST_DIR"), "/target/examples"));
    if !features.is_empty() {
        command.args(["--no-default-features", "--features", &features.join(",")]);
    }

    let output = command
        .output()
        .unwrap_or_else(|e| panic!("could not run example {name}: {e}"));

    assert!(
        output.status.success(),
        "example {name} exited {}\n--- stderr\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr),
    );

    String::from_utf8(output.stdout).expect("example printed something that is not utf-8")
}

#[test]
fn compare_backends_shows_every_backend_it_was_given() {
    let out = run_example("compare_backends", &["use_regex", "use_fancy_regex"]);

    // The example exists to show what the backends do differently, so all three have to
    // appear or it is showing nothing.
    for backend in ["charwalk:", "regex:", "fancy_regex:"] {
        assert!(out.contains(backend), "no {backend} row in:\n{out}");
    }

    // And they have to have been asked something. A run over an empty case list prints
    // three headings and no answers, which passes the check above.
    assert!(out.contains("CamelCase"), "no cases were run in:\n{out}");
    assert!(
        out.contains(r#"["camel", "case"]"#),
        "the backends produced no segmentation in:\n{out}",
    );
}

#[test]
fn the_lending_example_segments_and_refuses() {
    let out = run_example("lending", &["no_alloc"]);

    // The segmentation, from the stack arrays.
    assert!(
        out.contains("[some] [http] [request] [id]"),
        "no segmentation in:\n{out}"
    );
    assert!(
        out.contains("[parse] [xml] [document]"),
        "no segmentation in:\n{out}"
    );

    // The refusal, on each lend in turn, carrying both numbers rather than only failing.
    assert!(
        out.contains("refused: wanted at least 3, had 2"),
        "no bounds refusal in:\n{out}"
    );
    assert!(
        out.contains("refused: wanted at least 5 bytes, had 4"),
        "no text refusal in:\n{out}",
    );

    // And the final sigma, which is the one place the lending lowercasing had to reproduce
    // a rule rather than inherit it.
    assert!(
        out.contains("[οδος] [στο]"),
        "the sigma rule did not fire in:\n{out}"
    );
}
