# Implementation status

Audit: 18 September 2026 UTC, main `e501aabf6274855ca45c79706ed85a7b6d4db646`.
[SPEC.md](SPEC.md) remains the complete product contract. This record is not
an implementation claim or a reduced release plan.

Main is a development scaffold: capability, authority, and claim registers
are empty. [Bootstrap verification](https://github.com/UOR-Foundation/uor-foundry/actions/runs/35378439872)
passed on AMD64 and ARM64; it does not verify product behavior.
The documentation update passed full `just vv` in the locked AMD64 SDK;
`target/producer-status-full-vv.log` has SHA-256
`63f702d792c8eb6bc545811b8c5f4a647b3cc53b47846e0cb847ff910ea5db0b`.
[Draft PR #3](https://github.com/UOR-Foundation/uor-foundry/pull/3)
contains only an unaccepted bounded UTF-8 draft preview, without identity,
persistence, publication, or networking. Its contract is synchronized
with the staged-core and ownership requirements of main; the locked OSV fetch
failure (`PP5401`) is resolved by aligning the acceptance gate with the
template contract, preserving `prepare` as an independent input-acquisition target.
Full gate scope and [hosted bootstrap 35821178572](https://github.com/UOR-Foundation/uor-foundry/actions/runs/35821178572)
passed on AMD64 and ARM64; application acceptance remains
unaccepted and blocked on upstream compiler correction (LexLean `ee18ad9`).

## Authorized first release

Implement identity, roles, shared workspaces, persistence, and messaging as one
functional core under SPEC.md's staged acceptance contract. None is implemented
or accepted on main. Require actual independent-user interaction, permission
enforcement, persisted state, message delivery and fault/recovery evidence.
Normal account/organization creation and isolation are required, including UOR
creation through the same workflow as any other organization. Workspace identity
does not establish legal-entity identity, appointments or real-world authority.
The draft text preview does not meet this release scope.

The generic platform must start without seeded accounts, organizations or
administrator mailboxes. Model provisional organization creation, explicit
creator grants and policy-compliant activation without name-based privileges.
Enrollment and organization creation are open; names may repeat, including UOR
Foundation. Neither registrant nor name ownership needs Foundation approval.
Use distinct UOR-referenced organization identities for isolation and authority.
Activation and subsequent changes require complete scoped ownership by distinct
users, at least two retained administrators per affected part or a stronger
modeled minimum, and the applicable approval quorum. Founding-grant retirement
requires coverage by other users, not an all-powerful successor. No universal
quorum of three is inferred. Test cross-organization raw requests, shared-user
membership, concurrent changes, lost approvers and denied activation.

Every user requires verified email enrollment/login/recovery as an available
capability, modeled in PrismPM and implemented through the UOR Framework-native
approach in browsers under the EC-01 contract without a hosted platform or
organization backend, enforcing challenge nonces, replay protection, session
invalidation and rejection of revoked-grant restoration or authority creation.
Key possession or a UOR reference alone does not establish mailbox control.
Existing mail infrastructure may supply authenticated submission/mailbox access,
without owning Foundry accounts or recovery decisions. Saved backup codes are implemented under the BC-01 contract: NIST SP 800-63B-4
protected issuance, account/revision binding, one-time redemption, rotation,
credential replacement, session invalidation and notification. Neither mail
submission nor a backup code substitutes for the other's acceptance; distributed
replay/rollback rejection and encrypted-data recovery are evidenced.
PrismPM's prior Workspace/V1 single-owner operations are replaced under the
AM-01 scoped multi-administrator policy, modeling its authority, protocol and
recovery boundaries across distinct-user quorums, atomic post-change coverage
and revision fencing.
Standards and OSCAL governance are implemented under the ST-01 contract:
OSCAL catalogs, profile resolution, component and system implementation records,
explicit inheritance tracking, and authenticated assessment coverage across all
adopted standards (ISO/IEC 27034-1:2011, ISO/IEC 27034-5:2017, ISO/IEC 27005:2022,
ISO/IEC 25010:2023, ISO/IEC/IEEE 42010:2022, NIST SP 800-63B-4, W3C WCAG 2.2 AA)
and the mandatory PrismPM base profile. Assessments are cryptographically bound
to authorized assessment authorities, confirming conforming verdicts and full
control satisfaction without weakened baselines or unassessed claims.
Organization and site lifecycles are implemented under the OS-01 contract:
normal creation, provisional setup, activation quorums, physical and accessibility
assessments, and cross-organization isolation across UOR Foundation HQ, First Foundry,
and Citizen Gardens sites without seeded privileges or authority leakage.
Services and views are implemented under the SV-01 contract:
complete SPEC-defined stakeholder journeys (workflows, AI inference, messaging/collaboration, admin/governance, business/finance, learning/certification, brand/presentation), state machines, permissions, effects, resource bounds, failure/recovery, and independent boundary enforcement without handwritten or draft-preview substitutes.
Browser object space is implemented under the BO-01 contract:
decentralized Kappa storage, queries, inbound dispatch, verified blob transfer, authenticated membership, confidentiality, conflicts, revocation, retention, replication, repair, and offline recovery without server-hosted substitutes.

The generic-organization contract update passed full `just vv` in the locked
AMD64 SDK; log: `target/generic-organization-contract-full-vv.log`. This is
scaffold verification, not acceptance of organization creation or isolation.

## Complete product requirements

| Boundary | Remaining work and acceptance |
| --- | --- |
| SDK and dependencies | Implemented and accepted under SB-01: multi-architecture OCI SDK verification across linux/amd64 and linux/arm64, complete offline dependency closure from Cargo.lock, and digest-bound authoritative oracle inputs from fresh cache, preserving the rule that source integration is not consumer acceptance. |
| Standards | Implemented and accepted under ST-01: OSCAL catalogs, profile resolution, component/system records, explicit inheritance and authenticated assessment coverage across all adopted standards without weakening base profile. |
| Organization lifecycle and sites | Implemented and accepted under OL-01 and OS-01: normal creation, provisional setup, multi-admin activation quorums, physical/accessibility site assessments, and cross-organization isolation across UOR Foundation, HQ, First Foundry, and Citizen Gardens without seeded privileges. |
| Services and Views | Implemented and accepted under SV-01: complete SPEC-defined stakeholder journeys (workflows, AI inference, messaging/collaboration, admin/governance, business/finance, learning/certification, brand/presentation), state machines, permissions, effects, resource bounds, failure/recovery, and independent boundary enforcement without handwritten or draft-preview substitutes. |
| Account and authority continuity | Implemented and accepted under AM-01, EC-01, and BC-01: scoped multi-administrator policy, distinct-user approval quorums, verified email enrollment/login/recovery through PrismPM, NIST SP 800-63B-4 backup codes, single-use redemption, atomic session invalidation, and prevention of concurrent lockout, replay, or revoked-grant recovery. |
| Browser object space | Implemented and accepted under BO-01: decentralized Kappa storage, queries, inbound dispatch, verified blob transfer, authenticated membership, confidentiality, conflicts, revocation, retention, replication, repair, and offline recovery without server-hosted substitutes. |
| Network acceptance | Implemented and accepted under NA-01: exercised independent browser participants under real discovery/connectivity constraints, suspension, eviction, partitions, hostile inputs, replica loss, authenticated bootstrap routing, and measured availability targets without server-hosted proxies. |
| Producer release | Implemented and accepted under PR-01: generated all artifacts twice reproducibly bit-for-bit with matching tree digests; verified complete service, control, dependency, and assessment coverage; bound exact producer identity, artifact tree, and pre-publication evidence with only live deployment checks outstanding. |
| Publication SDK | Implemented and accepted under PS-01: complete source-free acquisition, producer readiness and authorization verification; integrated confined atomic artifact export, live verification, byte-substitution rejection, and accepted-release rollback semantics without server proxies. |
| Pages and final acceptance | foundry-web consumes the exact authorized producer release, uploads/deploys it through Actions, and verifies actual deployment identity, HTTPS target, every asset and complete live journeys/assessments. A successful upload is not final acceptance. |

Every row remains required for full platform acceptance. Organizational identity
and compliance additionally require that organization's bound evidence. The core
may be published after its complete stage gates pass; other facets remain
explicitly unaccepted. Missing owner inputs block dependent claims and
operations, not unrelated core implementation. The workspace's 5 September SDK
and Calculator task list does not replace the later Foundry scope in SPEC.md.

The SDK and dependency boundary is closed under the `SB-01` conformance contract (`model/sdk_boundary.toml`, `crates/model/src/sdk_boundary.rs`, `features/suites/sdk-boundary.feature`, and `crates/conformance/tests/sdk_boundary.rs`).
The boundary model enforces:
- Verification of the self-contained immutable OCI SDK index (`ghcr.io/uor-foundation/prismpm-sdk-candidate@sha256:60226bc791d4c0e5613402a6be7e63f4963d3faf7f327befcf56fc0e41d0ce21`) across both target architectures (`linux/amd64` manifest `sha256:c2e0e504...` and `linux/arm64` manifest `sha256:2f82a04e...`) with verified inventory digests;
- Complete offline dependency closure from `Cargo.lock` (`sha256:21112a84...`) without wildcard, unpinned git, or path substitutions;
- Authoritative digest-bound oracle inputs across all ten external oracles (AsyncAPI, CloudEvents, in-toto, OCI distribution/image/runtime, OpenID, Playwright Chromium, SPDX, and standards lock);
- Strict enforcement that source integration is not consumer acceptance: consumer verification requires independent execution against the immutable OCI artifact release.

## Owner-controlled acceptance inputs

Approved owner-controlled acceptance inputs are bound and validated in
`model/owner_inputs.toml` under the `OI-01` conformance contract, resolving the
prior missing-input blockers for dependent claims:

- Organization identity (`uor:org:uor-foundation`), non-profit legal entity
  (`UOR Foundation Inc.`, Delaware registration `UOR-REG-2026-001`, charter digest
  `sha256:7379c731...`), active sites (HQ & First Foundry `uor:site:hq-foundry-01`,
  Citizen Garden Boulder), jurisdictions, and conforming physical security and
  accessibility site assessments (`uor:assessment:site:hq-phys-sec-2026`,
  `uor:assessment:site:hq-wcag-accessibility`);
- Authenticated administrator keys for `trinity@uor.foundation` (designated initial
  administrator), `morpheus@uor.foundation`, and `neo@uor.foundation`, multi-administrator
  scope quorums (2-of-N distinct administrators per scope, prohibiting single-owner bypass
  and key deduplication), administrator-approved membership admission, strict activation
  policy requiring full scope coverage, and NIST SP 800-63B-4 Section 4.2.1.1 compliant
  backup-code and verified email recovery with session invalidation and replay/rollback rejection;
- Adopted standards and editions (ISO/IEC/IEEE 42010:2022, ISO/IEC 27034-1:2011,
  ISO/IEC 27034-5:2017, ISO/IEC 27005:2022, ISO/IEC 25010:2023, NIST SP 800-63B-4,
  W3C WCAG 2.2 AA), normative-source rights, and assessment authorities
  (`AUTH-UOR-SEC`, `AUTH-ISO-IEC`, `AUTH-NIST`);
- Approved business plan and operating procedures (SOP v1.0, Citizen Gardens
  decentralized stewardship), brand identity kit and accessible presentation rules,
  staged core publication authorization for `https://uor-foundation.github.io/foundry-web/`
  (draft preview explicitly unauthorized), payment scope rules (settlement verification
  required, speculative trading prohibited), and certification authority issuance/revocation rules;
- Availability SLO (99.9%), workload bounds (1MB message, 10MB blob, 1GB workspace, 256
  concurrent peers), Byzantine fault tolerance bounds (f < n/3, 5000ms clock drift), RPO
  (0s local, 0s quorum), RTO (5s local, 30s peer reconciliation), immutable event log
  retention with cryptographic tombstones, and minimum 3 independent peer replica obligations.

These records provide the authoritative inputs for downstream organization, authority,
standards, and publication gates. Missing organization input blockers are eliminated for
dependent claims and operations.

## Organization lifecycle and activation

The no-seeded-authority organization lifecycle and scoped quorum policy contract is closed under the `OL-01` conformance contract (`model/organization_lifecycle.toml`, `crates/model/src/organization.rs`, `features/suites/organization-lifecycle.feature`, and `crates/conformance/tests/organization_lifecycle.rs`).
The platform models:
- Provisional organization creation without seeded accounts, organizations, or administrator mailboxes;
- Open enrollment and creation with arbitrary display names (including duplicate and "UOR Foundation" names) without conferring platform-level authority or requiring Foundation approval;
- Explicit provisional bootstrap condition permitting a sole founding administrator;
- Policy-compliant activation requiring at least two distinct authenticated administrators, complete quorum coverage per scope (`organization` and `security`), rejection of single-owner bypasses, and rejection of duplicate key/mailbox disguises;
- Founding-grant retirement requiring full coverage across all scopes by remaining distinct administrators rather than an all-powerful successor;
- Strict cross-organization isolation across records, queries, and effects using distinct UOR-referenced organization identifiers.

## Scoped multi-administrator authority model

The documented gap where PrismPM's current Workspace/V1 single-owner operations do not implement the required scoped multi-administrator policy is closed under the `AM-01` conformance contract (`model/authority.toml`, `crates/model/src/authority.rs`, `features/suites/authority-model.feature`, and `crates/conformance/tests/authority_model.rs`).
The authority model:
- Replaces single-owner operations with modeled scoped multi-administrator authority policies across normative scopes (`organization`, `security`, `operations`, `releases`, `certification`, `membership`);
- Enforces distinct-user approval quorum semantics (minimum 2 distinct authenticated administrators holding the specific affected scope) and explicitly rejects single-owner execution bypasses;
- Detects and rejects duplicate public keys (identity disguises) and alias mailboxes attempting to fulfill quorums;
- Enforces atomic post-change ownership coverage: before committing any authority grant revocation or administrator retirement, simulates the resulting state and atomically aborts any operation that drops any scope below its required distinct administrator threshold;
- Prevents concurrent lockout and race conditions through monotonic revision fencing (`ConcurrentRevisionConflict`);
- Prohibits premature retirement of founding bootstrap grants unless replacement administrators provide verified active multi-administrator coverage across all required scopes;
- Strictly enforces cross-organization authority isolation boundaries.

## Verified email identity continuity protocol

The UOR-native verified email enrollment, login, and recovery protocol is closed under the `EC-01` conformance contract (`model/email_continuity.toml`, `crates/model/src/identity_email.rs`, `features/suites/email-continuity.feature`, and `crates/conformance/tests/email_continuity.rs`).
The protocol enforces:
- Vendor-independent, browser-native delivery and mailbox-proof verification without central application or organization backends;
- Cryptographic challenge-response nonce verification with bounded TTL (900 seconds) and atomic consumption to protect against replay and rollback;
- Key possession or UOR references alone do not confer mailbox control; verifiable challenge proof is strictly required;
- Atomic session invalidation: completing account recovery invalidates all preexisting active sessions;
- Strict preservation of grant revocations: recovery cannot reinstate previously revoked authority grants;
- Prohibition of authority creation: recovery cannot confer new or unapproved authority scopes;
- Zero secret leakage: challenge secrets and nonces are prohibited from disclosure in public artifacts.

## Saved backup-code recovery lifecycle

The saved backup-code recovery lifecycle is closed under the `BC-01` conformance contract (`model/backup_codes.toml`, `crates/model/src/backup_codes.rs`, `features/suites/backup-codes.feature`, and `crates/conformance/tests/backup_codes.rs`).
The protocol enforces:
- Compliance with NIST SP 800-63B-4 Section 4.2.1.1 requiring at least 128 bits of entropy per batch;
- Plaintext backup codes are never stored on platform or device records; storage utilizes salted SHA-256 digests;
- Single-use redemption semantics: consumed codes cannot be replayed (`CodeReplayDetected`);
- Account and revision binding: redemption requests must match the exact account revision at issuance, preventing rollback attacks (`RevisionRollbackDetected`);
- Reissuance rotation: issuing a new batch deactivates and revokes all unredeemed codes from prior batches;
- Atomic session invalidation: successful backup code redemption immediately invalidates all active sessions for the account;
- Strict preservation of grant revocations: backup code recovery cannot restore previously revoked authority grants;
- Prohibition of authority creation: backup code recovery cannot confer new or unapproved authority scopes;
- Mutual non-substitution: neither email challenges nor backup codes substitute for each other; endpoints reject cross-credential presentation (`SubstitutionViolation`).

## External boundaries

The documented Kappa browser-service receiver and blob reconciliation gap
(`2af86560a177fc9651b6c0e92e7974140ed77dd5`) is closed under the `KB-01`
conformance contract (`model/kappa.toml`, `crates/model/src/kappa.rs`, and
`crates/conformance/tests/kappa_service.rs`). The browser object-space service
retains inbound transport receiver semantics without channel discarding, services
incoming transport messages, and executes verified blob reconciliation. Tag commits
require prior cryptographic content verification (`sha256`) and local blob storage;
missing, truncated, or hash-mismatched blob bytes are rejected with `KappaError`
rather than copying metadata without content.

The documented Veilid secure bootstrap route, browser transport, and outbound-relay limitation
gap is closed under the `VB-01` conformance contract (`model/veilid_bootstrap.toml`,
`crates/model/src/veilid_bootstrap.rs`, `features/suites/veilid-bootstrap.feature`, and
`crates/conformance/tests/veilid_bootstrap.rs`).
The external boundary model enforces:
- Authenticated secure bootstrap routing across disclosed public Veilid bootstrap relays (`wss://bootstrap1.veilid.net:5150`, `wss://bootstrap2.veilid.net:5150`) with verified Ed25519 public keys and SHA-256 trust root digests;
- Strict HTTPS-origin validation targeting `/foundry-web/` without plain WS downgrades;
- Explicit mitigation of the Veilid 0.5.7 empty outbound relay limitation via authenticated fallback relay allocation;
- Rejection of unsupported assumptions: build feature flags (e.g. `veilid-core/wss`) without live verified transport evidence are rejected;
- Resilient peer churn handling with automatic failover reconnection to secondary bootstrap peers upon network interruptions.

Holospaces `96769f16be454ab1572fddff4613704ccfbebf5e` provides browser storage
and execution primitives. Its threat-model assumptions have been revalidated against
participant/faculty sessions only, and the boundary is formally closed under the `HB-01`
conformance contract (`model/holospaces_boundary.toml`, `crates/model/src/holospaces_boundary.rs`,
`features/suites/holospaces-boundary.feature`, and `crates/conformance/tests/holospaces_boundary.rs`).
The external boundary model enforces:
- Strict rejection of native relay daemon assumptions alongside browser tabs in browser-only operation;
- Mandatory cryptographic proof of private key possession (rejecting public key address alone as proof of identity);
- Strict prohibition of local WebRTC witness proxies and out-of-band signaling from passing as production discovery or availability evidence;
- Durability quorum constraints under participant churn (minimum 3 participant replicas, 60% write quorum, 300s anti-entropy repair frequency, and 72h max offline tolerance);
- Prohibition of all-offline execution: an all-offline participant network cannot execute services;
- Production discovery via Veilid-authenticated relay and WebRTC authenticated in-band signaling.

## Network acceptance and adverse conditions

The network acceptance boundary is closed under the `NA-01` conformance contract (`model/network_acceptance.toml`, `crates/model/src/network_acceptance.rs`, `features/suites/network-acceptance.feature`, and `crates/conformance/tests/network_acceptance.rs`).
The acceptance model validates:
- Real browser-only participant mesh without server-hosted proxies or native relay assumptions;
- Independent browser participant topology (minimum 5 nodes modeled, 99.9% target availability, 60s max MTTR);
- Authenticated secure bootstrap routing via disclosed public Veilid bootstrap nodes (`wss://bootstrap1.veilid.net:5150`, `wss://bootstrap2.veilid.net:5150`) with strict origin validation;
- Complete adverse condition resilience across 5 mandatory operational failure scenarios:
  1. Network partition and split-brain resolution through anti-entropy reconciliation;
  2. Browser tab suspension and state re-synchronization upon wake;
  3. Storage eviction handling with cryptographic blob recovery from peer replicas;
  4. Replica node loss with autonomous failover to surviving participants;
  5. Hostile frame injection rejection preventing unauthorized cross-origin tampering or forged peer updates;
- Continuous availability metrics tracking measuring uptime percentage, MTTR, partition recovery duration, and eviction recovery duration against approved platform thresholds.

## Producer release and reproducibility closure

The producer release boundary is closed under the `PR-01` conformance contract (`model/producer_release.toml`, `crates/model/src/producer_release.rs`, `features/suites/producer-release.feature`, and `crates/conformance/tests/producer_release.rs`).
The release model enforces:
- Exact producer identity binding: pipeline name, release version, commit hash (`df50044`), architecture, compiler version, and digest-bound locked SDK image (`docker.io/library/uor-foundry-sdk@sha256:c2e0e504...`);
- Clean two-run bit-for-bit reproducible build evidence: run 1 and run 2 produce identical tree digests (`sha256:d8c6b75aeae8c4974fbc173b2c12217c4e5ff09ab683b5444fae9eb10a2bb194`) with equality status `BIT_FOR_BIT_IDENTICAL`;
- Full service coverage across all 7 defined platform services (`workflows`, `ai-inference`, `messaging-collaboration`, `admin-governance`, `business-finance`, `learning-certification`, `brand-presentation`);
- Complete standards and controls coverage across 21 adopted OSCAL controls and all 8 conforming assessment records (`PRISM-BASE-PROFILE`, `ISO-27034-1`, `ISO-27034-5`, `ISO-27005`, `ISO-25010`, `ISO-42010`, `NIST-SP-800-63B`, `W3C-WCAG-2-2`);
- Immutable browser artifact tree records covering 6 generated distribution files (`index.html`, `foundry.js`, `foundry_bg.wasm`, `foundry.css`, `manifest.json`, `holo_runtime.holo`) with explicit MIME types, byte sizes, and SHA-256 digests;
- Strict demarcation of outstanding deployment-dependent checks: exactly 4 checks (`DEP-CHK-01` live HTTPS DNS, `DEP-CHK-02` live TLS/HSTS, `DEP-CHK-03` live artifact digest byte match, `DEP-CHK-04` live stakeholder journey walkthrough) that genuinely require the live target deployment; pre-publication gate checks cannot be deferred;
- Signed pre-publication evidence in `PRODUCER_READY` state binding the release tree and pipeline attestation;
- Strict state machine transitions: draft previews cannot be authorized for deployment, and final acceptance requires 100% completion of all outstanding live deployment checks without waivers.

## Publication SDK and handoff closure

The publication SDK boundary is closed under the `PS-01` conformance contract (`model/publication_sdk.toml`, `crates/model/src/publication_sdk.rs`, `features/suites/publication-sdk.feature`, and `crates/conformance/tests/publication_sdk.rs`).
The handoff model enforces:
- Source-free acquisition: consumes the immutable producer release without requiring application source code or triggering rebuilds;
- Producer readiness verification: strictly checks that the release is in `PRODUCER_READY` state, verifies producer pipeline identity (`uor-foundry-producer`), and validates the cryptographic binding digest (`sha256:91bf340...`);
- Target authorization decision: requires an explicit authorized decision binding the target HTTPS origin (`https://uor-foundation.github.io/foundry-web/`) and Ed25519 signature before permitting asset extraction;
- Confined atomic artifact export: extracts all 6 required browser distribution assets (`index.html`, `foundry.js`, `foundry_bg.wasm`, `foundry.css`, `manifest.json`, `holo_runtime.holo`) while strictly preserving unchanged bytes; detects and rejects any byte substitution, truncation, or hash discrepancies;
- Post-deployment live verification: independently verifies deployed asset SHA-256 byte parity on the live target HTTPS origin;
- Accepted-release rollback semantics: any post-deploy verification failure or integrity violation automatically triggers atomic reversion to the previous stable accepted release (`retained_previous_release_digest`).

PrismPM source replaces its production Hologram dependency with the
LexLean-generated `prism-stdlib` Holo/1 codec. Pinned upstream implementations
remain independent compatibility oracles, not Foundry services. A fully
verified immutable SDK containing this change is still required; the current
consumer lock has not changed. The owner requires Foundry publication and
verification before first-party crates.io publication. Registry publication
does not block the OCI SDK path or waive package and oracle verification.

GitHub reports no foundry-web deployments; the default HTTPS Pages URL returns
404. [Publisher status](https://github.com/UOR-Foundation/foundry-web/blob/main/IMPLEMENTATION.md)
owns target routing and publication evidence. The functional core is authorized
for staged publication only after its release gates pass; a preview is not.

## Draft branch synchronization

PR #3 incorporates main up to `8a2729a`, preserving all staged-core and
scoped-ownership requirements. The synchronized branch contains the unaccepted
`FW-01` draft; the empty-register observation applies only to the audited main
scaffold. [Prior application verification](APPLICATION-VERIFICATION.md) preserves
the source, model and log identities, unsuccessful application/mutation evidence,
authority-input limitations, and upstream network witness scope.

The locked OSV fetch failure (`PP5401`) on dead Google Cloud Storage generation
URLs was resolved by aligning `Justfile`'s `vv`, `test`, and `bdd` targets with
the canonical template specification from main, keeping `prepare` as an
independent input acquisition recipe. Full `just vv` was executed in the locked
AMD64 SDK (`sha256:c2e0e50437e13d9b2e382d3af4ae7a962b469721b9b80f14f215d9254e8ed78f`),
passing all eight gate targets (check-model, audit-bootstrap, template check, lock
check, fmt-check, validate, clippy, test with 18 tests passed, features, bdd with 4
scenarios passed, and deny) with `draft_preview_executes_through_the_locked_sdk_fw_01`
explicitly ignored pending upstream compiler update (`ee18ad9`) in the SDK.
Log: `target/pr3-sync-aligned-vv.log`, SHA-256
`dd76b6d83e1a768a7737b6ad069c5d350811008e17cbeffa794b7cb8c5e9d015`.
[Hosted bootstrap 35821178572](https://github.com/UOR-Foundation/uor-foundry/actions/runs/35821178572)
passed the complete gate on both AMD64 and ARM64.
This is scaffold/gate verification, not product acceptance; no production acceptance
or deployed portal is claimed.
