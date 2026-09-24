//! Conformance tests for EC-01: UOR-native verified email identity continuity protocol.

use repo_model::identity_email::{ChallengePurpose, EmailContinuityManager, IdentityError};
use repo_model::Model;

fn test_setup() -> EmailContinuityManager {
    let model = Model::load_from_repo_root().expect("model must load and check cleanly");
    model.check().expect("model consistency check must pass");
    EmailContinuityManager::new(model.email_continuity.clone())
}

const VALID_NONCE_1: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const VALID_NONCE_2: &str = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";
const VALID_NONCE_3: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[test]
fn challenge_replay_is_rejected() {
    let mut manager = test_setup();
    let current_time = 1000;

    manager
        .issue_challenge(
            "chal-01",
            "trinity@uor.foundation",
            ChallengePurpose::Enrollment,
            VALID_NONCE_1,
            current_time,
        )
        .unwrap();

    // First completion succeeds
    manager
        .complete_enrollment(
            "chal-01",
            VALID_NONCE_1,
            "uor:user:trinity",
            "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be",
            &["organization".to_string(), "security".to_string()],
            current_time + 10,
        )
        .unwrap();

    // Second completion using same challenge must be rejected with ChallengeReplayDetected
    let err = manager
        .complete_enrollment(
            "chal-01",
            VALID_NONCE_1,
            "uor:user:trinity",
            "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be",
            &["organization".to_string()],
            current_time + 15,
        )
        .expect_err("replaying challenge must fail");
    assert!(matches!(err, IdentityError::ChallengeReplayDetected(_)));
}

#[test]
fn expired_challenge_is_rejected() {
    let mut manager = test_setup();
    let current_time = 1000;

    manager
        .issue_challenge(
            "chal-expired",
            "morpheus@uor.foundation",
            ChallengePurpose::Enrollment,
            VALID_NONCE_1,
            current_time,
        )
        .unwrap();

    // TTL is 900 seconds; attempt at current_time + 901
    let err = manager
        .complete_enrollment(
            "chal-expired",
            VALID_NONCE_1,
            "uor:user:morpheus",
            "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989",
            &["operations".to_string()],
            current_time + 901,
        )
        .expect_err("expired challenge must be rejected");
    assert!(matches!(err, IdentityError::ChallengeExpired { .. }));
}

#[test]
fn recovery_invalidates_prior_sessions() {
    let mut manager = test_setup();
    let current_time = 1000;

    // 1. Enroll
    manager
        .issue_challenge(
            "chal-enroll",
            "neo@uor.foundation",
            ChallengePurpose::Enrollment,
            VALID_NONCE_1,
            current_time,
        )
        .unwrap();
    manager
        .complete_enrollment(
            "chal-enroll",
            VALID_NONCE_1,
            "uor:user:neo",
            "cf122ae443d3fcad8b90fe30277d3c37e008c60d56c9658a44848bdb6f2078d4",
            &["releases".to_string()],
            current_time + 5,
        )
        .unwrap();

    // 2. Login to create 2 active sessions
    manager
        .issue_challenge(
            "chal-login-1",
            "neo@uor.foundation",
            ChallengePurpose::Login,
            VALID_NONCE_2,
            current_time + 10,
        )
        .unwrap();
    manager
        .complete_login(
            "chal-login-1",
            VALID_NONCE_2,
            "sess-device-alpha",
            "fingerprint-browser-linux",
            current_time + 15,
        )
        .unwrap();

    manager
        .issue_challenge(
            "chal-login-2",
            "neo@uor.foundation",
            ChallengePurpose::Login,
            VALID_NONCE_3,
            current_time + 20,
        )
        .unwrap();
    manager
        .complete_login(
            "chal-login-2",
            VALID_NONCE_3,
            "sess-device-beta",
            "fingerprint-mobile-arm",
            current_time + 25,
        )
        .unwrap();

    // Confirm sessions are valid
    let account = manager.accounts.get("neo@uor.foundation").unwrap();
    assert_eq!(account.sessions.len(), 2);
    assert!(account.sessions.iter().all(|s| s.valid));

    // 3. Perform recovery with new nonce
    const RECOVERY_NONCE: &str = "9999999999999999999999999999999999999999999999999999999999999999";
    manager
        .issue_challenge(
            "chal-recovery",
            "neo@uor.foundation",
            ChallengePurpose::Recovery,
            RECOVERY_NONCE,
            current_time + 50,
        )
        .unwrap();

    let recovered = manager
        .complete_recovery(
            "chal-recovery",
            RECOVERY_NONCE,
            "1111111111111111111111111111111111111111111111111111111111111111",
            &["releases".to_string()],
            current_time + 55,
        )
        .unwrap();

    // All prior sessions must be invalidated atomically
    assert_eq!(recovered.sessions.len(), 2);
    assert!(recovered.sessions.iter().all(|s| !s.valid));
    assert_eq!(
        recovered.primary_public_key,
        "1111111111111111111111111111111111111111111111111111111111111111"
    );
}

#[test]
fn recovery_refuses_to_restore_revoked_grants() {
    let mut manager = test_setup();
    let current_time = 1000;

    manager
        .issue_challenge(
            "chal-trinity",
            "trinity@uor.foundation",
            ChallengePurpose::Enrollment,
            VALID_NONCE_1,
            current_time,
        )
        .unwrap();
    manager
        .complete_enrollment(
            "chal-trinity",
            VALID_NONCE_1,
            "uor:user:trinity",
            "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be",
            &["organization".to_string(), "security".to_string()],
            current_time + 5,
        )
        .unwrap();

    // Explicitly revoke "security" scope
    manager
        .revoke_scope("trinity@uor.foundation", "security")
        .unwrap();

    // Now issue recovery challenge
    manager
        .issue_challenge(
            "chal-trinity-rec",
            "trinity@uor.foundation",
            ChallengePurpose::Recovery,
            VALID_NONCE_2,
            current_time + 50,
        )
        .unwrap();

    // Attempting to include "security" scope in recovery request must fail
    let err = manager
        .complete_recovery(
            "chal-trinity-rec",
            VALID_NONCE_2,
            "2222222222222222222222222222222222222222222222222222222222222222",
            &["organization".to_string(), "security".to_string()],
            current_time + 55,
        )
        .expect_err("recovery must not restore revoked scope");

    assert!(matches!(
        err,
        IdentityError::RevokedGrantRestorationRejected { ref scope, .. } if scope == "security"
    ));
}

#[test]
fn recovery_refuses_to_create_unauthorized_authority() {
    let mut manager = test_setup();
    let current_time = 1000;

    manager
        .issue_challenge(
            "chal-morpheus",
            "morpheus@uor.foundation",
            ChallengePurpose::Enrollment,
            VALID_NONCE_1,
            current_time,
        )
        .unwrap();
    manager
        .complete_enrollment(
            "chal-morpheus",
            VALID_NONCE_1,
            "uor:user:morpheus",
            "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989",
            &["operations".to_string()],
            current_time + 5,
        )
        .unwrap();

    manager
        .issue_challenge(
            "chal-morpheus-rec",
            "morpheus@uor.foundation",
            ChallengePurpose::Recovery,
            VALID_NONCE_2,
            current_time + 50,
        )
        .unwrap();

    // Attempt to self-elevate to "certification" scope upon recovery
    let err = manager
        .complete_recovery(
            "chal-morpheus-rec",
            VALID_NONCE_2,
            "3333333333333333333333333333333333333333333333333333333333333333",
            &["operations".to_string(), "certification".to_string()],
            current_time + 55,
        )
        .expect_err("recovery must not create ungranted authority scopes");

    assert!(matches!(
        err,
        IdentityError::AuthorityCreationRejected { ref scope, .. } if scope == "certification"
    ));
}

#[test]
fn email_continuity_protocol_enforces_lifecycle_and_safety_invariants_ec_01() {
    let mut manager = test_setup();
    let current_time = 1000;

    // 1. Enrollment with verified mailbox challenge proof
    manager
        .issue_challenge(
            "chal-ec-01-enroll",
            "auditor@uor.foundation",
            ChallengePurpose::Enrollment,
            VALID_NONCE_1,
            current_time,
        )
        .unwrap();

    let account = manager
        .complete_enrollment(
            "chal-ec-01-enroll",
            VALID_NONCE_1,
            "uor:user:auditor",
            "4444444444444444444444444444444444444444444444444444444444444444",
            &["certification".to_string()],
            current_time + 10,
        )
        .expect("enrollment must succeed with valid challenge response");
    assert_eq!(account.mailbox, "auditor@uor.foundation");
    assert!(account.granted_scopes.contains("certification"));

    // 2. Interactive session login
    manager
        .issue_challenge(
            "chal-ec-01-login",
            "auditor@uor.foundation",
            ChallengePurpose::Login,
            VALID_NONCE_2,
            current_time + 20,
        )
        .unwrap();

    let session = manager
        .complete_login(
            "chal-ec-01-login",
            VALID_NONCE_2,
            "sess-auditor-1",
            "workstation-audit-01",
            current_time + 25,
        )
        .expect("login must succeed with valid challenge response");
    assert!(session.valid);

    // 3. Recovery replaces credentials and invalidates session
    manager
        .issue_challenge(
            "chal-ec-01-rec",
            "auditor@uor.foundation",
            ChallengePurpose::Recovery,
            VALID_NONCE_3,
            current_time + 40,
        )
        .unwrap();

    let recovered = manager
        .complete_recovery(
            "chal-ec-01-rec",
            VALID_NONCE_3,
            "5555555555555555555555555555555555555555555555555555555555555555",
            &["certification".to_string()],
            current_time + 45,
        )
        .expect("recovery must succeed with valid challenge response");

    assert_eq!(
        recovered.primary_public_key,
        "5555555555555555555555555555555555555555555555555555555555555555"
    );
    assert!(!recovered.sessions[0].valid);
}
