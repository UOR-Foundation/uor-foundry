//! Conformance tests for BC-01: NIST SP 800-63B-4 compliant saved backup-code recovery lifecycle.

use repo_model::backup_codes::{
    BackupCodeError, BackupCodeManager, CodeStatus, RedeemBackupCodeRequest,
};
use repo_model::identity_email::{AccountStatus, SessionRecord, UserAccountRecord};
use repo_model::Model;
use std::collections::HashSet;

fn test_setup() -> BackupCodeManager {
    let model = Model::load_from_repo_root().expect("model must load and check cleanly");
    model.check().expect("model consistency check must pass");
    BackupCodeManager::new(model.backup_codes.clone())
}

fn sample_plaintexts() -> Vec<String> {
    vec![
        "code-alpha-00000001".to_string(),
        "code-alpha-00000002".to_string(),
        "code-alpha-00000003".to_string(),
        "code-alpha-00000004".to_string(),
        "code-alpha-00000005".to_string(),
        "code-alpha-00000006".to_string(),
        "code-alpha-00000007".to_string(),
        "code-alpha-00000008".to_string(),
        "code-alpha-00000009".to_string(),
        "code-alpha-00000010".to_string(),
    ]
}

fn sample_user_account() -> UserAccountRecord {
    let mut granted = HashSet::new();
    granted.insert("organization".to_string());
    granted.insert("security".to_string());

    UserAccountRecord {
        user_id: "uor:user:trinity".to_string(),
        mailbox: "trinity@uor.foundation".to_string(),
        primary_public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
            .to_string(),
        status: AccountStatus::Active,
        granted_scopes: granted,
        revoked_scopes: HashSet::new(),
        sessions: vec![
            SessionRecord {
                session_id: "sess-01".to_string(),
                device_fingerprint: "linux-workstation".to_string(),
                created_at: 1000,
                valid: true,
            },
            SessionRecord {
                session_id: "sess-02".to_string(),
                device_fingerprint: "arm64-device".to_string(),
                created_at: 1010,
                valid: true,
            },
        ],
    }
}

#[test]
fn backup_code_replay_is_rejected() {
    let mut manager = test_setup();
    let plaintexts = sample_plaintexts();
    let mut account = sample_user_account();
    let current_time = 1000;

    manager
        .issue_batch(
            "trinity@uor.foundation",
            1,
            "batch-01",
            &plaintexts,
            current_time,
        )
        .unwrap();

    let req1 = RedeemBackupCodeRequest {
        mailbox: "trinity@uor.foundation".to_string(),
        code_plaintext: plaintexts[0].clone(),
        expected_account_revision: 1,
        new_public_key: "1111111111111111111111111111111111111111111111111111111111111111"
            .to_string(),
        requested_scopes: vec!["organization".to_string(), "security".to_string()],
    };

    // First redemption succeeds
    let report = manager
        .redeem_code(req1, &mut account, 1, current_time + 10)
        .expect("first redemption must succeed");
    assert_eq!(report.remaining_unused_codes, 9);

    // Attempting to reuse the same code must be rejected with CodeReplayDetected
    let req_replay = RedeemBackupCodeRequest {
        mailbox: "trinity@uor.foundation".to_string(),
        code_plaintext: plaintexts[0].clone(),
        expected_account_revision: 1,
        new_public_key: "2222222222222222222222222222222222222222222222222222222222222222"
            .to_string(),
        requested_scopes: vec!["organization".to_string()],
    };

    let err = manager
        .redeem_code(req_replay, &mut account, 1, current_time + 20)
        .expect_err("replaying redeemed code must fail");
    assert!(matches!(err, BackupCodeError::CodeReplayDetected(_)));
}

#[test]
fn revision_rollback_is_rejected() {
    let mut manager = test_setup();
    let plaintexts = sample_plaintexts();
    let mut account = sample_user_account();
    let current_time = 1000;

    manager
        .issue_batch(
            "trinity@uor.foundation",
            5, // issued at revision 5
            "batch-rev-5",
            &plaintexts,
            current_time,
        )
        .unwrap();

    // Attacker or stale client attempts redemption presenting stale revision 4
    let req = RedeemBackupCodeRequest {
        mailbox: "trinity@uor.foundation".to_string(),
        code_plaintext: plaintexts[0].clone(),
        expected_account_revision: 4, // stale / rollback
        new_public_key: "3333333333333333333333333333333333333333333333333333333333333333"
            .to_string(),
        requested_scopes: vec!["organization".to_string()],
    };

    let err = manager
        .redeem_code(req, &mut account, 5, current_time + 10)
        .expect_err("revision rollback must be rejected");

    match err {
        BackupCodeError::RevisionRollbackDetected { expected, actual } => {
            assert_eq!(expected, 5);
            assert_eq!(actual, 4);
        }
        other => panic!("expected RevisionRollbackDetected, got {other:?}"),
    }
}

#[test]
fn batch_rotation_deactivates_prior_codes() {
    let mut manager = test_setup();
    let plaintexts1 = sample_plaintexts();
    let mut plaintexts2 = sample_plaintexts();
    for p in &mut plaintexts2 {
        *p = format!("{p}-rotated");
    }
    let mut account = sample_user_account();
    let current_time = 1000;

    manager
        .issue_batch(
            "trinity@uor.foundation",
            1,
            "batch-alpha",
            &plaintexts1,
            current_time,
        )
        .unwrap();

    // Reissuance creates batch-beta and deactivates batch-alpha
    manager
        .issue_batch(
            "trinity@uor.foundation",
            1,
            "batch-beta",
            &plaintexts2,
            current_time + 10,
        )
        .unwrap();

    let user_batches = manager.batches.get("trinity@uor.foundation").unwrap();
    assert_eq!(user_batches.len(), 2);
    assert!(!user_batches[0].active);
    assert!(user_batches[1].active);
    assert!(user_batches[0]
        .codes
        .iter()
        .all(|c| c.status == CodeStatus::Revoked));

    // Attempting to redeem a code from the revoked batch-alpha must fail
    let req = RedeemBackupCodeRequest {
        mailbox: "trinity@uor.foundation".to_string(),
        code_plaintext: plaintexts1[0].clone(),
        expected_account_revision: 1,
        new_public_key: "4444444444444444444444444444444444444444444444444444444444444444"
            .to_string(),
        requested_scopes: vec!["organization".to_string()],
    };

    let err = manager
        .redeem_code(req, &mut account, 1, current_time + 20)
        .expect_err("redeeming code from revoked batch must fail");
    assert!(matches!(err, BackupCodeError::InvalidCode(_)));
}

#[test]
fn mutual_non_substitution_is_enforced() {
    let manager = test_setup();

    // Backup code presented at email challenge endpoint
    let err1 = manager
        .verify_mutual_non_substitution("email_challenge_endpoint", "backup_code")
        .expect_err("backup code cannot substitute for email challenge");
    assert!(matches!(err1, BackupCodeError::SubstitutionViolation(_)));

    // Email challenge token presented at backup code endpoint
    let err2 = manager
        .verify_mutual_non_substitution("backup_code_recovery_endpoint", "email_challenge_token")
        .expect_err("email token cannot substitute for backup code");
    assert!(matches!(err2, BackupCodeError::SubstitutionViolation(_)));
}

#[test]
fn insufficient_entropy_is_rejected() {
    let mut manager = test_setup();
    let mut short_plaintexts = sample_plaintexts();
    short_plaintexts[0] = "short".to_string(); // only 5 characters, lacks 128-bit entropy

    let err = manager
        .issue_batch(
            "neo@uor.foundation",
            1,
            "batch-short",
            &short_plaintexts,
            1000,
        )
        .expect_err("codes lacking 128 bits of entropy must be rejected");
    assert!(matches!(err, BackupCodeError::InsufficientEntropy(_)));
}

#[test]
fn backup_code_lifecycle_enforces_nist_standards_and_safety_invariants_bc_01() {
    let mut manager = test_setup();
    let plaintexts = sample_plaintexts();
    let mut account = sample_user_account();
    let current_time = 1000;

    // Trinity has 2 active sessions
    assert_eq!(account.sessions.len(), 2);
    assert!(account.sessions.iter().all(|s| s.valid));

    // Trinity revokes "security" scope
    account.granted_scopes.remove("security");
    account.revoked_scopes.insert("security".to_string());

    // Issue batch
    let batch = manager
        .issue_batch(
            "trinity@uor.foundation",
            1,
            "batch-nist-01",
            &plaintexts,
            current_time,
        )
        .expect("batch issuance must succeed");
    assert_eq!(batch.codes.len(), 10);
    assert!(batch.codes.iter().all(|c| c.status == CodeStatus::Unused));

    // Attempting recovery that tries to restore revoked "security" scope fails
    let bad_req = RedeemBackupCodeRequest {
        mailbox: "trinity@uor.foundation".to_string(),
        code_plaintext: plaintexts[0].clone(),
        expected_account_revision: 1,
        new_public_key: "5555555555555555555555555555555555555555555555555555555555555555"
            .to_string(),
        requested_scopes: vec!["organization".to_string(), "security".to_string()],
    };

    let err = manager
        .redeem_code(bad_req, &mut account, 1, current_time + 10)
        .expect_err("recovery must not restore revoked scope");
    assert!(matches!(
        err,
        BackupCodeError::RevokedGrantRestorationRejected { .. }
    ));

    // Proper recovery without revoked scope succeeds and invalidates all prior sessions
    let good_req = RedeemBackupCodeRequest {
        mailbox: "trinity@uor.foundation".to_string(),
        code_plaintext: plaintexts[0].clone(),
        expected_account_revision: 1,
        new_public_key: "5555555555555555555555555555555555555555555555555555555555555555"
            .to_string(),
        requested_scopes: vec!["organization".to_string()],
    };

    let report = manager
        .redeem_code(good_req, &mut account, 1, current_time + 15)
        .expect("redemption must succeed");

    assert_eq!(report.remaining_unused_codes, 9);
    assert!(!report.exhaustion_warning);
    assert_eq!(
        account.primary_public_key,
        "5555555555555555555555555555555555555555555555555555555555555555"
    );
    // All sessions invalidated atomically
    assert!(account.sessions.iter().all(|s| !s.valid));
}

fn json_extract_str<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let pattern = format!("\"{key}\":");
    let pos = json.find(&pattern)?;
    let rest = json[pos + pattern.len()..].trim_start();
    if rest.starts_with('"') {
        let after_quote = &rest[1..];
        let end = after_quote.find('"')?;
        Some(&after_quote[..end])
    } else {
        None
    }
}

fn json_extract_u64(json: &str, key: &str) -> Option<u64> {
    let pattern = format!("\"{key}\":");
    let pos = json.find(&pattern)?;
    let rest = json[pos + pattern.len()..].trim_start();
    let end = rest.find(|c: char| c == ',' || c == '}' || c == ']' || c.is_whitespace())?;
    rest[..end].parse().ok()
}

#[derive(Debug)]
struct RecoveryVectorCode {
    code_plaintext: String,
    salt: String,
    storage_digest: String,
}

#[test]
fn nist_800_63b_authoritative_vectors_validation() {
    use std::fs;

    let root = repo_model::repo_root();

    // 1. Verify NIST CAVP SHA-256 test vectors
    let cavp_path = root.join("tests/oracles/nist_800_63b/cavp_sha256_vectors.json");
    let cavp_content = fs::read_to_string(&cavp_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", cavp_path.display()));

    let mut cavp_checked = 0;
    for chunk in cavp_content.split("\"description\":").skip(1) {
        let msg = json_extract_str(chunk, "msg").expect("msg present");
        let expected_md = json_extract_str(chunk, "md").expect("md present");
        let actual_md = repo_model::sha256_hex(msg.as_bytes());
        assert_eq!(
            actual_md, expected_md,
            "NIST CAVP SHA-256 test vector failed for msg: '{msg}'"
        );
        cavp_checked += 1;
    }
    assert_eq!(cavp_checked, 3, "expected 3 NIST CAVP vectors");

    // 2. Verify NIST SP 800-63B-4 §4.2.1.1 Recovery Codes test vectors
    let recovery_path = root.join("tests/oracles/nist_800_63b/recovery_codes_vectors.json");
    let recovery_content = fs::read_to_string(&recovery_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", recovery_path.display()));

    let spec = json_extract_str(&recovery_content, "spec").unwrap();
    assert!(
        spec == "NIST-SP-800-63B-4 §4.2.1.1" || spec == "NIST-SP-800-63B-4 \\u00a74.2.1.1",
        "spec mismatch: {spec}"
    );
    assert_eq!(json_extract_u64(&recovery_content, "batch_size").unwrap(), 10);
    assert!(json_extract_u64(&recovery_content, "total_entropy_bits").unwrap() >= 128);
    assert_eq!(
        json_extract_str(&recovery_content, "storage_scheme").unwrap(),
        "salted-sha256"
    );

    let mut codes = Vec::new();
    for chunk in recovery_content.split("\"code_index\":").skip(1) {
        let code_plaintext = json_extract_str(chunk, "code_plaintext")
            .expect("code_plaintext")
            .to_string();
        let salt = json_extract_str(chunk, "salt").expect("salt").to_string();
        let storage_digest = json_extract_str(chunk, "storage_digest")
            .expect("storage_digest")
            .to_string();
        codes.push(RecoveryVectorCode {
            code_plaintext,
            salt,
            storage_digest,
        });
    }

    assert_eq!(codes.len(), 10, "expected 10 authoritative NIST recovery codes");

    let plaintexts: Vec<String> = codes.iter().map(|c| c.code_plaintext.clone()).collect();

    let mut manager = test_setup();
    let mut account = sample_user_account();
    let current_time = 2000;

    // Issue batch using authoritative NIST recovery codes
    let batch = manager
        .issue_batch(
            &account.mailbox,
            1,
            "batch-nist-authoritative",
            &plaintexts,
            current_time,
        )
        .expect("batch issuance with authoritative NIST codes must succeed");

    assert_eq!(batch.codes.len(), 10);
    assert!(batch.codes.iter().all(|c| c.status == CodeStatus::Unused));

    // Verify salted SHA-256 hashing for each code
    for (i, code_meta) in codes.iter().enumerate() {
        let expected_digest = &code_meta.storage_digest;
        let salt = &code_meta.salt;
        let plaintext = &code_meta.code_plaintext;
        let computed_digest = format!(
            "sha256:{}",
            repo_model::sha256_hex(format!("{salt}:{plaintext}").as_bytes())
        );
        assert_eq!(
            computed_digest, *expected_digest,
            "authoritative test vector digest must match salted SHA-256 specification"
        );
        assert_eq!(batch.codes[i].code_hash.len(), 64);
    }

    // Ensure sessions exist and are active prior to redemption
    assert!(!account.sessions.is_empty());
    assert!(account.sessions.iter().all(|s| s.valid));

    // Redeem first code
    let redeem_req = RedeemBackupCodeRequest {
        mailbox: account.mailbox.clone(),
        code_plaintext: plaintexts[0].clone(),
        expected_account_revision: 1,
        new_public_key: "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            .to_string(),
        requested_scopes: vec!["organization".to_string()],
    };

    let report = manager
        .redeem_code(redeem_req, &mut account, 1, current_time + 10)
        .expect("redemption with valid NIST backup code must succeed");

    assert_eq!(report.remaining_unused_codes, 9);
    // Atomic session invalidation: all prior sessions must be terminated
    assert!(
        account.sessions.iter().all(|s| !s.valid),
        "NIST SP 800-63B requires atomic invalidation of all prior sessions upon redemption"
    );

    // Replay rejection: redeeming the same code again must be rejected
    let replay_req = RedeemBackupCodeRequest {
        mailbox: account.mailbox.clone(),
        code_plaintext: plaintexts[0].clone(),
        expected_account_revision: 1,
        new_public_key: "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
            .to_string(),
        requested_scopes: vec!["organization".to_string()],
    };

    let replay_err = manager
        .redeem_code(replay_req, &mut account, 1, current_time + 20)
        .expect_err("replayed backup code must be rejected");

    assert!(matches!(
        replay_err,
        BackupCodeError::CodeReplayDetected { .. }
    ));
}
