//! Empirical Challenger Stress Test Suite: Milestone 2 Governance, Organization, and Standards
//! Independent adversarial verification harness probing edge cases and boundaries.

use repo_conformance::fixtures::SyntheticModel as Model;
use repo_model::authority::ProposalApproval;
use repo_model::organization::{
    ActivateOrganizationRequest, CreateInvitationRequest, CreateOrganizationRequest,
    InvitationStatus, OrgAdministrator, OrganizationError, OrganizationManager,
};
use repo_model::{SignedOwnerAttestation, StandardsError};

const SECP256R1_N: &str = "ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551";

fn setup_org() -> (OrganizationManager, String) {
    let mut manager = OrganizationManager::new();
    let org_id = "uor:org:adversarial-stress".to_string();

    manager
        .create_organization(CreateOrganizationRequest {
            id: org_id.clone(),
            display_name: "Adversarial Stress Org".to_string(),
            creator_mailbox: "trinity@uor.foundation".to_string(),
            creator_user_id: "uor:user:trinity".to_string(),
            creator_public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                .to_string(),
            creator_scopes: vec![
                "organization".to_string(),
                "security".to_string(),
                "membership".to_string(),
            ],
        })
        .unwrap();

    let model = repo_model::Model::load_from_repo_root().expect("model loads");
    let rules = &model.organization_lifecycle.rules;

    manager
        .activate_organization(
            ActivateOrganizationRequest {
                organization_id: org_id.clone(),
                administrators: vec![
                    OrgAdministrator {
                        mailbox: "trinity@uor.foundation".to_string(),
                        user_id: "uor:user:trinity".to_string(),
                        public_key:
                            "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                                .to_string(),
                        status: "authenticated".to_string(),
                        scopes: vec![
                            "organization".to_string(),
                            "membership".to_string(),
                            "security".to_string(),
                        ],
                    },
                    OrgAdministrator {
                        mailbox: "morpheus@uor.foundation".to_string(),
                        user_id: "uor:user:morpheus".to_string(),
                        public_key:
                            "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989"
                                .to_string(),
                        status: "authenticated".to_string(),
                        scopes: vec![
                            "organization".to_string(),
                            "membership".to_string(),
                            "security".to_string(),
                        ],
                    },
                ],
                approving_mailboxes: vec![
                    "trinity@uor.foundation".to_string(),
                    "morpheus@uor.foundation".to_string(),
                ],
            },
            rules,
        )
        .unwrap();

    (manager, org_id)
}

// ============================================================================
// PART 1: ORGANIZATION INVITATION GUARDS & LIFECYCLE BOUNDARIES
// ============================================================================

#[test]
fn test_challenger_org_invitation_rejection_suspended_and_retired() {
    let (mut manager, org_id) = setup_org();

    // 1. Create invitation while active
    let invite_id = manager
        .create_invitation(CreateInvitationRequest {
            id: "uor:invite:stress-01".to_string(),
            organization_id: org_id.clone(),
            target_mailbox: "candidate1@uor.foundation".to_string(),
            invited_scopes: vec!["operations".to_string()],
            inviter_mailbox: "trinity@uor.foundation".to_string(),
            inviter_user_id: "uor:user:trinity".to_string(),
            created_at: 1718000000,
            expires_at: 1718100000,
            revision_binding: 1,
        })
        .unwrap()
        .id
        .clone();

    // 2. Suspend organization
    manager
        .suspend_organization(&org_id, "trinity@uor.foundation")
        .unwrap();

    // 3. Reject acceptance while suspended
    let err_suspended = manager
        .accept_invitation(
            &invite_id,
            "candidate1@uor.foundation",
            "uor:user:cand1",
            "3344556677889900112233445566778899001122334455667788990011223344",
            1718050000,
        )
        .expect_err("acceptance while suspended must be rejected");
    assert!(matches!(
        err_suspended,
        OrganizationError::InvalidStateTransition { .. }
    ));

    // 4. Reject creating new invitations while suspended
    let err_create_suspended = manager
        .create_invitation(CreateInvitationRequest {
            id: "uor:invite:stress-02".to_string(),
            organization_id: org_id.clone(),
            target_mailbox: "candidate2@uor.foundation".to_string(),
            invited_scopes: vec!["operations".to_string()],
            inviter_mailbox: "trinity@uor.foundation".to_string(),
            inviter_user_id: "uor:user:trinity".to_string(),
            created_at: 1718050001,
            expires_at: 1718150001,
            revision_binding: 1,
        })
        .expect_err("creating invitation while suspended must be rejected");
    assert!(matches!(
        err_create_suspended,
        OrganizationError::InvalidStateTransition { .. }
    ));

    // 5. Reactivate org with quorum
    manager
        .reactivate_organization(
            &org_id,
            &[
                ProposalApproval::new(
                    "trinity@uor.foundation",
                    "uor:user:trinity",
                    "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be",
                    1718060000,
                ),
                ProposalApproval::new(
                    "morpheus@uor.foundation",
                    "uor:user:morpheus",
                    "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989",
                    1718060001,
                ),
            ],
        )
        .unwrap();

    // 6. Now that org is reactivated, the pending invitation CAN be accepted cleanly
    manager
        .accept_invitation(
            &invite_id,
            "candidate1@uor.foundation",
            "uor:user:cand1",
            "3344556677889900112233445566778899001122334455667788990011223344",
            1718060100,
        )
        .expect("acceptance must succeed after reactivation");

    // 7. Issue an invitation to candidate3, then retire org
    let invite3_id = manager
        .create_invitation(CreateInvitationRequest {
            id: "uor:invite:stress-03".to_string(),
            organization_id: org_id.clone(),
            target_mailbox: "candidate3@uor.foundation".to_string(),
            invited_scopes: vec!["operations".to_string()],
            inviter_mailbox: "trinity@uor.foundation".to_string(),
            inviter_user_id: "uor:user:trinity".to_string(),
            created_at: 1718060200,
            expires_at: 1718160200,
            revision_binding: 1,
        })
        .unwrap()
        .id
        .clone();

    manager
        .retire_organization(
            &org_id,
            &[
                ProposalApproval::new(
                    "trinity@uor.foundation",
                    "uor:user:trinity",
                    "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be",
                    1718070000,
                ),
                ProposalApproval::new(
                    "morpheus@uor.foundation",
                    "uor:user:morpheus",
                    "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989",
                    1718070001,
                ),
            ],
            false,
            0,
        )
        .unwrap();

    // 8. Reject acceptance while retired
    let err_retired = manager
        .accept_invitation(
            &invite3_id,
            "candidate3@uor.foundation",
            "uor:user:cand3",
            "4455667788990011223344556677889900112233445566778899001122334455",
            1718080000,
        )
        .expect_err("acceptance while retired must be rejected");
    assert!(matches!(
        err_retired,
        OrganizationError::InvalidStateTransition { .. }
    ));

    // 9. Reject creating invitations while retired
    let err_create_retired = manager
        .create_invitation(CreateInvitationRequest {
            id: "uor:invite:stress-04".to_string(),
            organization_id: org_id.clone(),
            target_mailbox: "candidate4@uor.foundation".to_string(),
            invited_scopes: vec!["operations".to_string()],
            inviter_mailbox: "trinity@uor.foundation".to_string(),
            inviter_user_id: "uor:user:trinity".to_string(),
            created_at: 1718080001,
            expires_at: 1718180001,
            revision_binding: 1,
        })
        .expect_err("creating invitation while retired must be rejected");
    assert!(matches!(
        err_create_retired,
        OrganizationError::InvalidStateTransition { .. }
    ));
}

#[test]
fn test_challenger_org_invitation_duplicate_pending_mailbox_guards() {
    let (mut manager, org_id) = setup_org();

    // 1. Create initial invitation for candidate@uor.foundation
    manager
        .create_invitation(CreateInvitationRequest {
            id: "uor:invite:dup-01".to_string(),
            organization_id: org_id.clone(),
            target_mailbox: "candidate@uor.foundation".to_string(),
            invited_scopes: vec!["operations".to_string()],
            inviter_mailbox: "trinity@uor.foundation".to_string(),
            inviter_user_id: "uor:user:trinity".to_string(),
            created_at: 1718000000,
            expires_at: 1718100000,
            revision_binding: 1,
        })
        .unwrap();

    // 2. Reject duplicate invitation with same mailbox, different ID and different scopes
    let err_dup1 = manager
        .create_invitation(CreateInvitationRequest {
            id: "uor:invite:dup-02".to_string(),
            organization_id: org_id.clone(),
            target_mailbox: "candidate@uor.foundation".to_string(),
            invited_scopes: vec!["security".to_string()],
            inviter_mailbox: "trinity@uor.foundation".to_string(),
            inviter_user_id: "uor:user:trinity".to_string(),
            created_at: 1718000010,
            expires_at: 1718100010,
            revision_binding: 1,
        })
        .expect_err("duplicate pending invitation to candidate@uor.foundation must fail");
    assert!(matches!(
        err_dup1,
        OrganizationError::InvalidInvitationState(_)
    ));

    // 3. Reject duplicate invitation from a different inviter (morpheus)
    let err_dup2 = manager
        .create_invitation(CreateInvitationRequest {
            id: "uor:invite:dup-03".to_string(),
            organization_id: org_id.clone(),
            target_mailbox: "candidate@uor.foundation".to_string(),
            invited_scopes: vec!["operations".to_string()],
            inviter_mailbox: "morpheus@uor.foundation".to_string(),
            inviter_user_id: "uor:user:morpheus".to_string(),
            created_at: 1718000020,
            expires_at: 1718100020,
            revision_binding: 1,
        })
        .expect_err("duplicate pending invitation from different inviter must fail");
    assert!(matches!(
        err_dup2,
        OrganizationError::InvalidInvitationState(_)
    ));

    // 4. Decline the first invitation
    manager
        .decline_invitation("uor:invite:dup-01", "candidate@uor.foundation")
        .unwrap();

    // 5. Once declined, creating a new invitation for candidate@uor.foundation MUST SUCCEED
    let fresh_invite = manager
        .create_invitation(CreateInvitationRequest {
            id: "uor:invite:dup-04".to_string(),
            organization_id: org_id.clone(),
            target_mailbox: "candidate@uor.foundation".to_string(),
            invited_scopes: vec!["operations".to_string()],
            inviter_mailbox: "trinity@uor.foundation".to_string(),
            inviter_user_id: "uor:user:trinity".to_string(),
            created_at: 1718000030,
            expires_at: 1718100030,
            revision_binding: 1,
        })
        .expect("invitation creation after decline must succeed");
    assert_eq!(fresh_invite.status, InvitationStatus::Pending);

    // 6. Revoke the fresh invitation
    manager
        .revoke_invitation("uor:invite:dup-04", "trinity@uor.foundation")
        .unwrap();

    // 7. Once revoked, creating another invitation for candidate@uor.foundation MUST SUCCEED
    let fresh_invite2 = manager
        .create_invitation(CreateInvitationRequest {
            id: "uor:invite:dup-05".to_string(),
            organization_id: org_id.clone(),
            target_mailbox: "candidate@uor.foundation".to_string(),
            invited_scopes: vec!["operations".to_string()],
            inviter_mailbox: "trinity@uor.foundation".to_string(),
            inviter_user_id: "uor:user:trinity".to_string(),
            created_at: 1718000040,
            expires_at: 1718100040,
            revision_binding: 1,
        })
        .expect("invitation creation after revocation must succeed");
    assert_eq!(fresh_invite2.status, InvitationStatus::Pending);
}

// ============================================================================
// PART 2: OWNER INPUTS & SECP256R1 SCALAR BOUNDS (0 < r, s < n)
// ============================================================================

fn base_charter_attestation() -> (Model, SignedOwnerAttestation) {
    let root = repo_model::repo_root();
    let syn_model = Model::load(&root.join("model")).expect("model loads");
    let att = SignedOwnerAttestation {
        attestation_id: "att-charter-01".to_string(),
        organization_id: "uor:org:uor-foundation".to_string(),
        attestation_type: "charter".to_string(),
        document_digest: "sha256:7379c7314f6e635202e221f787737531eefe1ac28cbbf034d04e490e769e6d41".to_string(),
        signer_mailbox: "trinity@uor.foundation".to_string(),
        signer_public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be".to_string(),
        signature_hex: "304402202b8d00938f45a6c4ef76921319c5932560ef7b8ec5d1b7d5ca4c1e4c7304f29102200259b64ea32313d3957eb64ea32313d3957eb64ea32313d3957eb64ea32313d3".to_string(),
        timestamp: 1718000000,
        valid_until: "2027-09-01".to_string(),
    };
    (syn_model, att)
}

#[test]
fn test_challenger_ecdsa_secp256r1_scalar_bounds_raw() {
    let (model, base_att) = base_charter_attestation();
    let inputs = &model.owner_inputs;

    // Baseline valid scalar components for reference:
    let valid_r = "2b8d00938f45a6c4ef76921319c5932560ef7b8ec5d1b7d5ca4c1e4c7304f291";
    let valid_s = "0259b64ea32313d3957eb64ea32313d3957eb64ea32313d3957eb64ea32313d3";

    // 1. Zero scalar r rejected
    let mut att = base_att.clone();
    att.signature_hex = format!("{}{}", "0".repeat(64), valid_s);
    let err = inputs
        .verify_attestation(&att)
        .expect_err("r == 0 must be rejected");
    assert!(err.to_string().contains("scalar 'r' is zero"));

    // 2. Zero scalar s rejected
    let mut att = base_att.clone();
    att.signature_hex = format!("{}{}", valid_r, "0".repeat(64));
    let err = inputs
        .verify_attestation(&att)
        .expect_err("s == 0 must be rejected");
    assert!(err.to_string().contains("scalar 's' is zero"));

    // 3. Exact curve order n for r: r == n rejected!
    let mut att = base_att.clone();
    att.signature_hex = format!("{}{}", SECP256R1_N, valid_s);
    let err = inputs
        .verify_attestation(&att)
        .expect_err("r == n must be rejected");
    assert!(err.to_string().contains("exceeds secp256r1 curve order"));

    // 4. Exact curve order n for s: s == n rejected!
    let mut att = base_att.clone();
    att.signature_hex = format!("{}{}", valid_r, SECP256R1_N);
    let err = inputs
        .verify_attestation(&att)
        .expect_err("s == n must be rejected");
    assert!(err.to_string().contains("exceeds secp256r1 curve order"));

    // 5. r == n + 1 rejected!
    let n_plus_1 = "ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632552";
    let mut att = base_att.clone();
    att.signature_hex = format!("{}{}", n_plus_1, valid_s);
    let err = inputs
        .verify_attestation(&att)
        .expect_err("r == n + 1 must be rejected");
    assert!(err.to_string().contains("exceeds secp256r1 curve order"));

    // 6. s == n + 1 rejected!
    let mut att = base_att.clone();
    att.signature_hex = format!("{}{}", valid_r, n_plus_1);
    let err = inputs
        .verify_attestation(&att)
        .expect_err("s == n + 1 must be rejected");
    assert!(err.to_string().contains("exceeds secp256r1 curve order"));

    // 7. Maximum 256-bit integer (all Fs) for r rejected!
    let all_fs = "f".repeat(64);
    let mut att = base_att.clone();
    att.signature_hex = format!("{}{}", all_fs, valid_s);
    let err = inputs
        .verify_attestation(&att)
        .expect_err("r == all Fs must be rejected");
    assert!(err.to_string().contains("exceeds secp256r1 curve order"));

    // 8. Maximum 256-bit integer (all Fs) for s rejected!
    let mut att = base_att.clone();
    att.signature_hex = format!("{}{}", valid_r, all_fs);
    let err = inputs
        .verify_attestation(&att)
        .expect_err("s == all Fs must be rejected");
    assert!(err.to_string().contains("exceeds secp256r1 curve order"));

    // 9. Boundary: r == n - 1 (strictly less than n)
    // n - 1 = ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632550
    let n_minus_1 = "ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632550";
    let mut att = base_att.clone();
    att.signature_hex = format!("{}{}", n_minus_1, valid_s);
    // n_minus_1 is strictly < n, so it MUST NOT error on "exceeds secp256r1 curve order"
    let res = inputs.verify_attestation(&att);
    if let Err(e) = res {
        assert!(
            !e.to_string().contains("exceeds secp256r1 curve order"),
            "n - 1 must not exceed curve order: {e}"
        );
    }

    // 10. Identical r and s scalars rejected
    let mut att = base_att.clone();
    att.signature_hex = format!("{}{}", valid_r, valid_r);
    let err = inputs
        .verify_attestation(&att)
        .expect_err("r == s must be rejected");
    assert!(err.to_string().contains("identical r and s"));

    // 11. Dummy repetitive pattern rejected (length 2 cycle)
    let dummy_pat_2 = "12".repeat(32);
    let mut att = base_att.clone();
    att.signature_hex = format!("{}{}", dummy_pat_2, valid_s);
    let err = inputs
        .verify_attestation(&att)
        .expect_err("cycle 2 pattern must be rejected");
    assert!(err.to_string().contains("dummy repetitive pattern"));

    // 12. Dummy repetitive pattern rejected (length 8 cycle)
    let dummy_pat_8 = "12345678".repeat(8);
    let mut att = base_att.clone();
    att.signature_hex = format!("{}{}", dummy_pat_8, valid_s);
    let err = inputs
        .verify_attestation(&att)
        .expect_err("cycle 8 pattern must be rejected");
    assert!(err.to_string().contains("dummy repetitive pattern"));
}

#[test]
fn test_challenger_ecdsa_secp256r1_scalar_bounds_der() {
    let (model, base_att) = base_charter_attestation();
    let inputs = &model.owner_inputs;

    // Baseline DER signature in base_att:
    // 30 44
    //   02 20 2b8d00938f45a6c4ef76921319c5932560ef7b8ec5d1b7d5ca4c1e4c7304f291 (r, 32 bytes)
    //   02 20 0259b64ea32313d3957eb64ea32313d3957eb64ea32313d3957eb64ea32313d3 (s, 32 bytes)

    // 1. Construct DER with r == SECP256R1_N (with 00 prefix because high bit is set: length 33 = 0x21)
    // 30 45 02 21 00<SECP256R1_N> 02 20 <valid_s>
    let valid_s = "0259b64ea32313d3957eb64ea32313d3957eb64ea32313d3957eb64ea32313d3";
    let der_r_n = format!("3045022100{}0220{}", SECP256R1_N, valid_s);
    let mut att = base_att.clone();
    att.signature_hex = der_r_n;
    let err = inputs
        .verify_attestation(&att)
        .expect_err("DER r == n must be rejected");
    assert!(err.to_string().contains("exceeds secp256r1 curve order"));

    // 2. Construct DER with s == SECP256R1_N
    let valid_r = "2b8d00938f45a6c4ef76921319c5932560ef7b8ec5d1b7d5ca4c1e4c7304f291";
    let der_s_n = format!("30450220{}022100{}", valid_r, SECP256R1_N);
    let mut att = base_att.clone();
    att.signature_hex = der_s_n;
    let err = inputs
        .verify_attestation(&att)
        .expect_err("DER s == n must be rejected");
    assert!(err.to_string().contains("exceeds secp256r1 curve order"));

    // 3. Construct DER with r == 0
    let der_r_zero = format!("30440220{}0220{}", "00".repeat(32), valid_s);
    let mut att = base_att.clone();
    att.signature_hex = der_r_zero;
    let err = inputs
        .verify_attestation(&att)
        .expect_err("DER r == 0 must be rejected");
    assert!(err.to_string().contains("scalar 'r' is zero"));

    // 4. Construct DER with corrupted tag (04 instead of 02)
    let der_bad_tag = format!("30440420{}0220{}", valid_r, valid_s);
    let mut att = base_att.clone();
    att.signature_hex = der_bad_tag;
    let err = inputs
        .verify_attestation(&att)
        .expect_err("DER bad tag must be rejected");
    assert!(err.to_string().contains("malformed ASN.1 DER"));
}

// ============================================================================
// PART 3: STANDARDS COMPLIANCE & ROOT KEY ENFORCEMENT
// ============================================================================

#[test]
fn test_challenger_standards_legacy_bypass_removal_enforced() {
    let root = repo_model::repo_root();
    let mut model = Model::load(&root.join("model")).expect("model loads");

    let profile = model
        .standards
        .resolve_profile("PROF-FOUNDRY-PRODUCTION")
        .expect("profile resolves");

    // 1. Genuine assessment with authentic key passes
    let res = model.standards.verify_assessments(
        &profile,
        &model.owner_inputs.assessment_authorities,
        &model.owner_inputs.standards,
        &model.owner_inputs.organization.id,
    );
    assert!(res.is_ok(), "genuine assessments must verify");

    // 2. Plant defect: replace enrolled authority key with legacy hardcoded key
    // SHA256:ISO-IEC-TRUSTED-ROOT-2026 was the legacy bypassed string
    model.standards.assessments[1].assessor_signature_key =
        "SHA256:ISO-IEC-TRUSTED-ROOT-2026".to_string();

    let err = model
        .standards
        .verify_assessments(
            &profile,
            &model.owner_inputs.assessment_authorities,
            &model.owner_inputs.standards,
            &model.owner_inputs.organization.id,
        )
        .expect_err("legacy root key must NOT be bypassed and must trigger AssessorKeyMismatch");

    assert!(
        matches!(err, StandardsError::AssessorKeyMismatch { .. }),
        "expected AssessorKeyMismatch, got: {err}"
    );
}
