//! Imported oracle input integrity. These tests do not establish product conformity.
//!
//! Checks locked payload digests and selected fixture/schema structures. It does
//! not execute these specifications against generated Foundry behavior, establish
//! mailbox control, measure entropy, or assess rendered accessibility.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn hash_file(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    repo_model::sha256_hex(&bytes)
}

#[test]
fn imported_oracle_inputs_match_their_recorded_inventory() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root");

    let status = Command::new("node")
        .arg("scripts/verify-authoritative-oracles.mjs")
        .current_dir(root)
        .status()
        .expect("the locked SDK supplies Node");
    assert!(
        status.success(),
        "scripts/verify-authoritative-oracles.mjs failed"
    );
}

#[test]
fn nist_oscal_1_1_0_schemas_and_model_mapping_validation() {
    let root = repo_model::repo_root();
    let oscal_dir = root.join("standards/oracles/oscal-1.1.0");

    let required_schemas = [
        "oscal_catalog_schema.json",
        "oscal_profile_schema.json",
        "oscal_ssp_schema.json",
        "oscal_component_schema.json",
        "oscal_assessment-results_schema.json",
    ];

    for name in required_schemas {
        let schema_path = oscal_dir.join(name);
        assert!(
            schema_path.is_file(),
            "OSCAL schema file must exist: {}",
            schema_path.display()
        );
        let content = fs::read_to_string(&schema_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", schema_path.display()));
        assert!(content.contains("\"$schema\""));
        assert!(content.contains("\"$id\""));
        assert!(content.contains("\"definitions\""));
    }

    // Validate that model/standards.toml converts to OSCAL catalog and profile records
    let model = repo_model::Model::load_from_repo_root().expect("model loads");
    let standards = &model.standards;

    assert!(
        !standards.catalogs.is_empty(),
        "OSCAL catalogs must not be empty"
    );
    for cat in &standards.catalogs {
        assert!(
            !cat.catalog_id.is_empty(),
            "OSCAL catalog must have catalog_id"
        );
        assert!(
            !cat.controls.is_empty(),
            "OSCAL catalog must contain controls"
        );
        for ctrl in &cat.controls {
            assert!(!ctrl.id.is_empty(), "control id must not be empty");
            assert!(!ctrl.title.is_empty(), "control title must not be empty");
            assert!(
                !ctrl.statement.is_empty(),
                "control statement must not be empty"
            );
        }
    }

    let profile = standards
        .resolve_profile("PROF-FOUNDRY-PRODUCTION")
        .expect("production profile resolves");
    assert!(!profile.controls.is_empty());
}

#[test]
fn standards_lock_oracle_and_authority_integrity() {
    let root = repo_model::repo_root();
    let lock_path = root.join("standards.lock");
    assert!(lock_path.is_file(), "standards.lock must exist");

    let lock_content = fs::read_to_string(&lock_path).expect("read standards.lock");
    assert!(
        lock_content.contains("\"schema\":\"prismpm/standards-lock/1\""),
        "standards.lock must be a canonical prismpm/standards-lock/1 document"
    );

    // Verify required standards are declared in model/standards.toml
    let standards_path = root.join("model/standards.toml");
    assert!(standards_path.is_file(), "model/standards.toml must exist");
    let standards_content = fs::read_to_string(&standards_path).expect("read model/standards.toml");

    let required_standards = [
        "\"NIST-SP-800-63B\"",
        "\"W3C-WCAG-2-2\"",
        "\"ISO-27034-1-2011\"",
        "\"ISO-27034-5-2017\"",
        "\"ISO-27005-2022\"",
        "\"ISO-25010-2023\"",
    ];
    for std in required_standards {
        assert!(
            standards_content.contains(std),
            "Standard declaration {std} must be in model/standards.toml"
        );
    }

    // Verify cryptographic digest integrity of ingested test vectors
    let oracle_payload_checks: Vec<(&str, PathBuf, &str)> = vec![
        (
            "nist-800-63b-4.2.1.1",
            root.join("tests/oracles/nist_800_63b/recovery_codes_vectors.json"),
            "682af6fd23c04a9d1a15c4565111c3fcb4011b85b70b77b66098c3ca93227743",
        ),
        (
            "w3c-did-core-1.0",
            root.join("tests/oracles/w3c_did/did-v1.jsonld"),
            "4f3eae5568c9c5f036a082088f9e192019ee06faa78973c87ff91d5421b88dad",
        ),
        (
            "w3c-vc-data-model-2.0",
            root.join("tests/oracles/w3c_vc/credentials-v2.jsonld"),
            "59955ced6697d61e03f2b2556febe5308ab16842846f5b586d7f1f7adec92734",
        ),
        (
            "w3c-activitypub-2.0",
            root.join("tests/oracles/w3c_activitypub/activitystreams.jsonld"),
            "a27b78b82f4980963127140d0cb74f0e8f21c0e2b8efd0368232bf9823edff5a",
        ),
        (
            "nist-oscal-1.1.0",
            root.join("standards/oracles/oscal-1.1.0/oscal_catalog_schema.json"),
            "936c53978eb47880dfa8b471640ae09b111b38d837686022ef7db07e26fa629d",
        ),
        (
            "w3c-wcag-2.2-aa",
            root.join("tests/browser/helpers/axe-check.mjs"),
            "3fc6ff7bbdfb6c4f2d452ed9d7064cd768ce0b2810fd0debf85181cf2b5eb1b1",
        ),
    ];

    for (oracle_id, file_path, expected_hash) in oracle_payload_checks {
        let actual_hash = hash_file(&file_path);
        assert_eq!(
            actual_hash,
            expected_hash,
            "Cryptographic digest mismatch for oracle {oracle_id} at {}",
            file_path.display()
        );
    }
}
