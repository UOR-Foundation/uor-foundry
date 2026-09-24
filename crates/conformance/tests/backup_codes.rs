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
