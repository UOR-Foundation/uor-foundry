//! Conformance tests for Services and Views stakeholder journeys, state machines,
//! resource bounds, failure recovery, raw boundary checks, and view projections (SV-01).

use std::collections::HashMap;

use repo_model::{
    AiInferenceProposal, AiProposalState, BrandKit, BrandKitState, CertificationState,
    CreateAiProposalRequest, FinancialInvoice, FinancialInvoiceState, GovernanceProposal,
    GovernanceProposalState, LearnerAssessment, MessageDeliveryState, MessageRecord, Model,
    RawBoundaryExecutor, ServiceError, ViewProjector, WorkflowRun, WorkflowRunState,
};

/// SV-01: Services and Views enforce SPEC-defined stakeholder journeys, explicit state machines,
/// permissions, effects, resource bounds, failure recovery, and independent raw-request boundary
/// verification without mock or draft-preview substitutes.
#[test]
fn services_and_views_enforce_spec_journeys_bounds_and_boundaries_sv_01() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    model
        .check()
        .expect("model checks and validates services and views");

    let cfg = &model.services;
    assert_eq!(cfg.services.len(), 7);
    assert_eq!(cfg.views.len(), 7);

    // -------------------------------------------------------------------------
    // 1. Workflows Journey & Resource Bounds
    // -------------------------------------------------------------------------
    let mut run = WorkflowRun::new(
        "uor:run:wf-001",
        "uor:org:uor-foundation",
        "ci-normative-acceptance",
        "9f8a3c4b5e2d1a0f8b7c6d5e4f3a2b1c0d9e8f7a",
    );
    assert_eq!(run.state, WorkflowRunState::Queued);

    // Advance execution within bounds (120s, 2048MB mem, 4096MB disk)
    run.advance(120, 2048, 4096, 3600, 16384, 51200)
        .expect("workflow advance within bounds");
    assert_eq!(run.state, WorkflowRunState::Running);
    assert_eq!(run.elapsed_seconds, 120);

    // Complete successfully with artifact digest
    run.succeed("sha256:d8a95f9c5d2b7e1f4a3c2b1a0e9f8d7c6b5a4e3d2c1b0a9f8e7d6c5b4a3e2d1c")
        .expect("workflow succeeds");
    assert_eq!(run.state, WorkflowRunState::Succeeded);
    assert!(run.artifact_digest.is_some());

    // Adversarial: execution exceeding maximum timeout (3600s) must fail
    let mut failing_run = WorkflowRun::new(
        "uor:run:wf-timeout",
        "uor:org:uor-foundation",
        "long-running-ci",
        "1111222233334444555566667777888899990000",
    );
    let err = failing_run
        .advance(3601, 1024, 1024, 3600, 16384, 51200)
        .expect_err("timeout exceeds bound");
    assert_eq!(failing_run.state, WorkflowRunState::Failed);
    match err {
        ServiceError::ResourceBoundsExceeded { metric, .. } => {
            assert_eq!(metric, "workflow execution seconds");
        }
        other => panic!("expected ResourceBoundsExceeded, got {other:?}"),
    }

    // -------------------------------------------------------------------------
    // 2. AI Inference Journey & Strict Proposal Gating Invariant
    // -------------------------------------------------------------------------
    let mut proposal = AiInferenceProposal::new(CreateAiProposalRequest {
        proposal_id: "uor:proposal:ai-01",
        organization_id: "uor:org:uor-foundation",
        model_uri: "uor:model:reasoning-v2",
        prompt: "Optimize governance quorum evaluation",
        proposed_content: "Proposed refactoring of quorum evaluation logic",
        context_tokens: 4096,
        generation_tokens: 512,
        max_context_tokens: 131072,
        max_generation_tokens: 4096,
    })
    .expect("ai proposal created within token bounds");
    assert_eq!(proposal.state, AiProposalState::Proposed);

    // Invariant: AI output is strictly a proposal until authorized by workflow.
    // Attempting to apply effects while in Proposed state MUST be rejected.
    let err = proposal
        .apply_effect()
        .expect_err("cannot apply effect without workflow authorization");
    assert_eq!(
        err,
        ServiceError::AiProposalNotWorkflowAuthorized("uor:proposal:ai-01".to_string())
    );

    // Authorize proposal through workflow
    proposal
        .authorize_by_workflow("uor:run:wf-001")
        .expect("workflow authorizes proposal");
    assert_eq!(proposal.state, AiProposalState::WorkflowAuthorized);

    // Now verified and applied
    proposal
        .apply_effect()
        .expect("apply authorized proposal effect");
    assert_eq!(proposal.state, AiProposalState::VerifiedApplied);

    // Adversarial: context token bound violation
    let err = AiInferenceProposal::new(CreateAiProposalRequest {
        proposal_id: "uor:proposal:ai-overflow",
        organization_id: "uor:org:uor-foundation",
        model_uri: "uor:model:reasoning-v2",
        prompt: "Prompt with excessive tokens",
        proposed_content: "Output",
        context_tokens: 131073, // exceeds 131072 max
        generation_tokens: 512,
        max_context_tokens: 131072,
        max_generation_tokens: 4096,
    })
    .expect_err("context token bound exceeded");
    match err {
        ServiceError::ResourceBoundsExceeded { metric, .. } => {
            assert_eq!(metric, "AI context tokens");
        }
        other => panic!("expected ResourceBoundsExceeded, got {other:?}"),
    }

    // -------------------------------------------------------------------------
    // 3. Messaging & Collaboration Journey & Size Bounds
    // -------------------------------------------------------------------------
    let mut msg = MessageRecord::send(
        "uor:msg:collab-01",
        "uor:channel:core-dev",
        "trinity@uor.foundation",
        "Release candidate for milestone accepted.",
        1024,
        65536,
        26214400,
    )
    .expect("send message within bounds");
    assert_eq!(msg.state, MessageDeliveryState::Sent);

    msg.mark_delivered();
    assert_eq!(msg.state, MessageDeliveryState::Delivered);

    msg.acknowledge();
    assert_eq!(msg.state, MessageDeliveryState::Acknowledged);

    // Adversarial: payload exceeding 64KB (65536 bytes) must be rejected
    let huge_content = "X".repeat(65537);
    let err = MessageRecord::send(
        "uor:msg:collab-huge",
        "uor:channel:core-dev",
        "trinity@uor.foundation",
        &huge_content,
        0,
        65536,
        26214400,
    )
    .expect_err("payload exceeds bound");
    match err {
        ServiceError::ResourceBoundsExceeded { metric, .. } => {
            assert_eq!(metric, "message payload bytes");
        }
        other => panic!("expected ResourceBoundsExceeded, got {other:?}"),
    }

    // -------------------------------------------------------------------------
    // 4. Administration & Governance Journey & 2-Admin Minimum Quorum
    // -------------------------------------------------------------------------
    let mut gov = GovernanceProposal::new(
        "uor:gov:prop-01",
        "uor:org:uor-foundation",
        "Upgrade site accessibility requirements",
        "morpheus@uor.foundation",
        2, // min 2 distinct approvers
        42,
    );
    assert_eq!(gov.state, GovernanceProposalState::Voting);

    // First administrator votes
    assert!(gov.approve("trinity@uor.foundation"));
    // Duplicate vote by same administrator does not add distinct approvers
    assert!(!gov.approve("trinity@uor.foundation"));

    // Attempting to enact with only 1 approver fails quorum verification
    let err = gov
        .enact(42)
        .expect_err("enact with 1 of 2 approvers must fail");
    assert_eq!(
        err,
        ServiceError::GovernanceQuorumDeficit {
            actual_approvals: 1,
            required_approvals: 2,
        }
    );

    // Second distinct administrator votes
    assert!(gov.approve("neo@uor.foundation"));

    // Adversarial: stale revision fencing check
    let err = gov
        .enact(41)
        .expect_err("enact with stale revision must fail");
    assert_eq!(
        err,
        ServiceError::StaleRevision {
            target_revision: 42,
            current_revision: 41,
        }
    );

    // Valid enactment with matching revision and 2 distinct approvers
    gov.enact(42).expect("governance proposal enacted");
    assert_eq!(gov.state, GovernanceProposalState::Enacted);

    // -------------------------------------------------------------------------
    // 5. Business & Finance Journey & Independent Settlement Oracle
    // -------------------------------------------------------------------------
    let mut invoice = FinancialInvoice::new(
        "uor:invoice:fin-01",
        "uor:org:uor-foundation",
        250000, // $2,500.00
        "USD",
    );
    assert_eq!(invoice.state, FinancialInvoiceState::Authorized);

    // Invariant: Recording a payment is NOT evidence of settlement.
    // Recording payment moves invoice to PendingSettlement, NEVER directly to Settled.
    invoice.record_payment().expect("record payment");
    assert_eq!(invoice.state, FinancialInvoiceState::PendingSettlement);

    // Settle with independent authoritative oracle receipt
    invoice
        .settle_with_oracle(
            "uor:oracle:settlement-authority-01",
            "sha256:4b9a1e8c7d6f5a3b2c1e0f9d8c7b6a5e4d3c2b1a0f9e8d7c6b5a4e3d2c1b0a9f",
        )
        .expect("settlement with oracle receipt");
    assert_eq!(invoice.state, FinancialInvoiceState::Settled);

    // Adversarial: attempt to settle with invalid non-sha256 digest
    let mut fake_invoice = FinancialInvoice::new(
        "uor:invoice:fin-fake",
        "uor:org:uor-foundation",
        1000,
        "USD",
    );
    fake_invoice.record_payment().expect("record payment");
    let err = fake_invoice
        .settle_with_oracle("uor:oracle:settlement-authority-01", "not-a-hash")
        .expect_err("invalid receipt digest");
    match err {
        ServiceError::PaymentNotSettledByOracle(msg) => {
            assert!(msg.contains("sha256:"));
        }
        other => panic!("expected PaymentNotSettledByOracle, got {other:?}"),
    }

    // -------------------------------------------------------------------------
    // 6. Learning & Certification Journey & Accredited Authority Verification
    // -------------------------------------------------------------------------
    let mut cert = LearnerAssessment::enroll(
        "uor:assess:learn-01",
        "uor:org:uor-foundation",
        "learner@uor.foundation",
        "uor:course:formal-verification-101",
    );
    assert_eq!(cert.state, CertificationState::Enrolled);

    // Record assessment score
    cert.record_assessment(95).expect("record score");
    assert_eq!(cert.state, CertificationState::Assessed);

    // Issue certificate with accredited authority signature
    cert.issue_certificate(
        "uor:authority:accredited-cert-board-01",
        "sha256:1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b",
        80, // passing score is 80
    )
    .expect("issue certificate");
    assert_eq!(cert.state, CertificationState::Certified);

    // Adversarial: failing score (< 80) cannot be certified
    let mut failing_cert = LearnerAssessment::enroll(
        "uor:assess:learn-fail",
        "uor:org:uor-foundation",
        "learner@uor.foundation",
        "uor:course:formal-verification-101",
    );
    failing_cert.record_assessment(65).expect("record score");
    let err = failing_cert
        .issue_certificate(
            "uor:authority:accredited-cert-board-01",
            "sha256:abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            80,
        )
        .expect_err("score below passing");
    assert_eq!(failing_cert.state, CertificationState::Uncertified);
    match err {
        ServiceError::CertificationNotAccredited(msg) => {
            assert!(msg.contains("below passing score"));
        }
        other => panic!("expected CertificationNotAccredited, got {other:?}"),
    }

    // -------------------------------------------------------------------------
    // 7. Brand Identity & WCAG 2.2 AA Contrast Enforcement
    // -------------------------------------------------------------------------
    let mut brand = BrandKit::new(
        "uor:brand:kit-01",
        "uor:org:uor-foundation",
        4.8, // normal text contrast ratio >= 4.5
        3.2, // UI contrast ratio >= 3.0
    );
    assert_eq!(brand.state, BrandKitState::Review);

    // Validate WCAG 2.2 AA contrast ratios
    brand
        .validate_wcag_contrast(4.5, 3.0)
        .expect("contrast meets WCAG 2.2 AA");
    assert_eq!(brand.state, BrandKitState::AccessibilityValidated);

    // Publish brand kit asset
    brand
        .publish("https://assets.uor.foundation/brand/logo.svg")
        .expect("publish asset");
    assert_eq!(brand.state, BrandKitState::Published);

    // Adversarial: brand kit failing normal text contrast (< 4.5) must be rejected
    let mut bad_brand = BrandKit::new(
        "uor:brand:kit-bad-contrast",
        "uor:org:uor-foundation",
        3.8, // fails 4.5
        3.5,
    );
    let err = bad_brand
        .validate_wcag_contrast(4.5, 3.0)
        .expect_err("contrast deficit");
    assert_eq!(bad_brand.state, BrandKitState::Rejected);
    match err {
        ServiceError::WcagContrastDeficit { contrast_type, .. } => {
            assert_eq!(contrast_type, "normal text");
        }
        other => panic!("expected WcagContrastDeficit, got {other:?}"),
    }

    // -------------------------------------------------------------------------
    // 8. Direct Raw Request Boundary Verification (Cannot bypass View)
    // -------------------------------------------------------------------------
    let workflow_svc = cfg
        .services
        .iter()
        .find(|s| s.id == "uor:service:workflows")
        .expect("find workflow svc");

    // Authorized raw request
    RawBoundaryExecutor::execute_raw_request(
        workflow_svc,
        "dev@uor.foundation",
        &["workflow:create", "workflow:execute"],
        "workflow:execute",
        "/api/raw/workflows/execute",
    )
    .expect("authorized raw request");

    // Adversarial: caller attempting direct raw request without required permission
    let err = RawBoundaryExecutor::execute_raw_request(
        workflow_svc,
        "unauthorized@external.com",
        &["workflow:read"], // lacks workflow:execute
        "workflow:execute",
        "/api/raw/workflows/execute",
    )
    .expect_err("unauthorized raw request must fail at the boundary");
    assert_eq!(
        err,
        ServiceError::UnauthorizedRawRequest {
            caller: "unauthorized@external.com".to_string(),
            required_permission: "workflow:execute".to_string(),
            endpoint: "/api/raw/workflows/execute".to_string(),
        }
    );

    // -------------------------------------------------------------------------
    // 9. View Projections & Credential Isolation
    // -------------------------------------------------------------------------
    let wf_view = cfg
        .views
        .iter()
        .find(|v| v.id == "uor:view:workflows")
        .expect("find workflows view");

    let mut raw_record = HashMap::new();
    raw_record.insert("run_id".to_string(), "uor:run:wf-001".to_string());
    raw_record.insert(
        "workflow_name".to_string(),
        "ci-normative-acceptance".to_string(),
    );
    raw_record.insert("status".to_string(), "succeeded".to_string());
    raw_record.insert("commit_sha".to_string(), "9f8a3c...".to_string());
    raw_record.insert("elapsed_seconds".to_string(), "120".to_string());
    raw_record.insert("artifact_digest".to_string(), "sha256:d8a...".to_string());
    // Sensitive fields in backend store that must NEVER leak to View
    raw_record.insert(
        "signing_private_key".to_string(),
        "SECRET_KEY_MATERIAL".to_string(),
    );
    raw_record.insert("build_secret".to_string(), "SECRET_TOKEN_XYZ".to_string());

    let projected = ViewProjector::project(wf_view, &raw_record).expect("project view fields");
    assert!(projected.contains_key("run_id"));
    assert!(projected.contains_key("status"));
    assert!(projected.contains_key("artifact_digest"));
    // Assert sensitive fields are completely redacted from projected view
    assert!(!projected.contains_key("signing_private_key"));
    assert!(!projected.contains_key("build_secret"));
    assert!(!projected.contains_key("ci_token"));

    // Adversarial: view projection detecting accidental leak in exposed field
    let mut leaking_record = HashMap::new();
    leaking_record.insert(
        "run_id".to_string(),
        "BEGIN PRIVATE KEY leaking...".to_string(),
    );
    let err = ViewProjector::project(wf_view, &leaking_record)
        .expect_err("projection must reject private key leak in exposed fields");
    assert_eq!(
        err,
        ServiceError::ViewExposedCredential("run_id".to_string())
    );
}

#[test]
fn ai_proposal_gating_strictly_enforces_workflow_authorization() {
    let mut proposal = AiInferenceProposal::new(CreateAiProposalRequest {
        proposal_id: "uor:proposal:ai-unauthorized",
        organization_id: "uor:org:uor-foundation",
        model_uri: "uor:model:reasoning-v2",
        prompt: "Execute unverified code mutation",
        proposed_content: "rm -rf /",
        context_tokens: 100,
        generation_tokens: 10,
        max_context_tokens: 131072,
        max_generation_tokens: 4096,
    })
    .expect("create proposal");

    assert_eq!(proposal.state, AiProposalState::Proposed);
    let err = proposal
        .apply_effect()
        .expect_err("unauthorized proposal must not apply");
    assert_eq!(
        err,
        ServiceError::AiProposalNotWorkflowAuthorized("uor:proposal:ai-unauthorized".to_string())
    );
}

#[test]
fn payment_settlement_requires_independent_oracle() {
    let mut invoice = FinancialInvoice::new(
        "uor:invoice:unsettled",
        "uor:org:uor-foundation",
        50000,
        "USD",
    );
    assert_eq!(invoice.state, FinancialInvoiceState::Authorized);
    invoice.record_payment().expect("record payment");
    assert_eq!(invoice.state, FinancialInvoiceState::PendingSettlement);

    let err = invoice
        .settle_with_oracle("uor:oracle:unknown", "corrupted-hash")
        .expect_err("invalid oracle receipt format");
    match err {
        ServiceError::PaymentNotSettledByOracle(msg) => assert!(msg.contains("sha256:")),
        other => panic!("expected PaymentNotSettledByOracle, got {other:?}"),
    }
}

#[test]
fn learning_certification_requires_accredited_authority_and_passing_score() {
    let mut assessment = LearnerAssessment::enroll(
        "uor:assess:cert-fail",
        "uor:org:uor-foundation",
        "learner@uor.foundation",
        "uor:course:security",
    );
    assessment
        .record_assessment(74)
        .expect("record failing score");
    let err = assessment
        .issue_certificate(
            "uor:authority:cert-board",
            "sha256:0000111122223333444455556666777788889999000011112222333344445555",
            75,
        )
        .expect_err("score 74 < passing score 75");
    assert_eq!(assessment.state, CertificationState::Uncertified);
    match err {
        ServiceError::CertificationNotAccredited(msg) => {
            assert!(msg.contains("below passing score"))
        }
        other => panic!("expected CertificationNotAccredited, got {other:?}"),
    }
}

#[test]
fn brand_kit_requires_wcag_aa_contrast() {
    let mut kit = BrandKit::new("uor:brand:bad-ui", "uor:org:uor-foundation", 5.0, 2.5); // UI contrast 2.5 < 3.0
    let err = kit
        .validate_wcag_contrast(4.5, 3.0)
        .expect_err("ui contrast failure");
    assert_eq!(kit.state, BrandKitState::Rejected);
    match err {
        ServiceError::WcagContrastDeficit { contrast_type, .. } => {
            assert_eq!(contrast_type, "ui component")
        }
        other => panic!("expected WcagContrastDeficit, got {other:?}"),
    }
}

#[test]
fn direct_raw_requests_enforce_permissions_at_boundary() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let gov_svc = model
        .services
        .services
        .iter()
        .find(|s| s.id == "uor:service:admin-governance")
        .expect("find governance svc");

    let err = RawBoundaryExecutor::execute_raw_request(
        gov_svc,
        "intruder@external.com",
        &[],
        "policy:enact",
        "/api/raw/governance/enact",
    )
    .expect_err("raw boundary request without permission fails");
    assert_eq!(
        err,
        ServiceError::UnauthorizedRawRequest {
            caller: "intruder@external.com".to_string(),
            required_permission: "policy:enact".to_string(),
            endpoint: "/api/raw/governance/enact".to_string(),
        }
    );
}
