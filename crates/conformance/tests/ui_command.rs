//! UC-01 source registration; the generated behavioral and SDK-host owner is required.

#[test]
fn source_bridge_registration_uc_01() {
    let result = std::process::Command::new("node")
        .arg("--test")
        .arg("tests/ui-command/source.test.mjs")
        .arg("tests/ui-command/corpus.test.mjs")
        .arg("tests/ui-command/mutants.test.mjs")
        .current_dir(repo_model::repo_root())
        .output()
        .expect("Node runs the registered source check");
    assert!(
        result.status.success(),
        "UI command registration failed:\n{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let transcript = String::from_utf8(result.stdout).expect("UTF-8 test report");
    for required in [
        "# tests 3",
        "# pass 3",
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
