//! AC-01 source registration; generated behavioral execution remains mandatory.

#[test]
fn source_context_and_oracle_registration_ac_01() {
    let result = std::process::Command::new("node")
        .arg("--test")
        .arg("tests/application-context/source.test.mjs")
        .arg("tests/application-context/corpus.test.mjs")
        .arg("tests/application-context/mutants.test.mjs")
        .current_dir(repo_model::repo_root())
        .output()
        .expect("Node runs the registered source check");
    assert!(
        result.status.success(),
        "source context registration failed:\n{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let transcript = String::from_utf8(result.stdout).expect("UTF-8 test report");
    for required in [
        "# tests 5",
        "# pass 5",
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
