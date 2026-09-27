//! Mandatory generated application acceptance and verification integrity.

use std::path::Path;
use std::process::Command;

#[test]
fn draft_preview_executes_through_the_locked_sdk_fw_01() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root");
    let status = Command::new("node")
        .arg("scripts/verify-application.mjs")
        .current_dir(root)
        .status()
        .expect("the locked SDK supplies Node");
    assert!(status.success(), "the complete draft-preview oracle failed");
}

#[test]
fn producer_verification_rejects_missing_stale_and_echo_artifacts_vi_01() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root");
    let status = Command::new("node")
        .args([
            "--test",
            "scripts/verification-integrity.test.mjs",
            "scripts/preparation.test.mjs",
        ])
        .current_dir(root)
        .status()
        .expect("the locked SDK supplies Node");
    assert!(
        status.success(),
        "producer verification integrity regressions failed"
    );
}
