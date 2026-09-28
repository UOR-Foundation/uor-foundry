//! SC-01 registration; the complete generated behavioral owner remains required.

#[test]
fn source_selection_registration_sc_01() {
    let result = std::process::Command::new("node")
        .arg("--test")
        .arg("tests/application-selection/source.test.mjs")
        .arg("tests/application-selection/corpus.test.mjs")
        .arg("tests/application-selection/mutants.test.mjs")
        .current_dir(repo_model::repo_root())
        .output()
        .expect("Node runs the registered source checks");
    assert!(
        result.status.success(),
        "selection registration failed:\n{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let transcript = String::from_utf8(result.stdout).expect("UTF-8 test report");
    for required in [
        "# tests 4",
        "# pass 4",
        "# fail 0",
        "# cancelled 0",
        "# skipped 0",
        "# todo 0",
    ] {
        assert!(
            transcript.lines().any(|line| line == required),
            "missing {required}"
        );
    }
}
