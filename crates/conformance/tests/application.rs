//! Executable acceptance for the LexLean-authored draft preview.

use std::path::Path;
use std::process::Command;

#[test]
#[ignore = "blocked on upstream compiler update (LexLean ee18ad9) in locked SDK"]
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
fn pure_model_browser_wasm_executes_acceptance_vectors_and_routing() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root");
    let status = Command::new("node")
        .arg("scripts/verify-browser-wasm.mjs")
        .current_dir(root)
        .status()
        .expect("the locked SDK supplies Node");
    assert!(status.success(), "browser Wasm verification failed");
}
