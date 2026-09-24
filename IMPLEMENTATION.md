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
| SDK and dependencies | Publish and verify the self-contained immutable OCI SDK on both architectures, including the complete offline dependency closure and digest-bound oracle inputs from a fresh cache. Source integration is not consumer acceptance. Public Cargo publication follows Foundry publication and verification. |
| Standards | Implemented and accepted under ST-01: OSCAL catalogs, profile resolution, component/system records, explicit inheritance and authenticated assessment coverage across all adopted standards without weakening base profile. |
| Organization lifecycle and sites | Implemented and accepted under OL-01 and OS-01: normal creation, provisional setup, multi-admin activation quorums, physical/accessibility site assessments, and cross-organization isolation across UOR Foundation, HQ, First Foundry, and Citizen Gardens without seeded privileges. |
| Services and Views | Implemented and accepted under SV-01: complete SPEC-defined stakeholder journeys (workflows, AI inference, messaging/collaboration, admin/governance, business/finance, learning/certification, brand/presentation), state machines, permissions, effects, resource bounds, failure/recovery, and independent boundary enforcement without handwritten or draft-preview substitutes. |
| Account and authority continuity | Implemented and accepted under AM-01, EC-01, and BC-01: scoped multi-administrator policy, distinct-user approval quorums, verified email enrollment/login/recovery through PrismPM, NIST SP 800-63B-4 backup codes, single-use redemption, atomic session invalidation, and prevention of concurrent lockout, replay, or revoked-grant recovery. |
| Browser object space | Implemented and accepted under BO-01: decentralized Kappa storage, queries, inbound dispatch, verified blob transfer, authenticated membership, confidentiality, conflicts, revocation, retention, replication, repair, and offline recovery without server-hosted substitutes. |
| Network acceptance | Implemented and accepted under NA-01: exercised independent browser participants under real discovery/connectivity constraints, suspension, eviction, partitions, hostile inputs, replica loss, authenticated bootstrap routing, and measured availability targets without server-hosted proxies. |
| Producer release | Generate all artifacts twice reproducibly; verify complete service/control/dependency/assessment coverage. Bind exact producer identity, artifact tree and pre-publication evidence, with only the exact deployment-dependent checks outstanding. |
| Publication SDK | Complete source-free acquisition, readiness and authorization verification; integrate confined atomic artifact export, live verification and accepted-release rollback. Preserve unchanged bytes; reject partial, stale, substituted or unauthorized evidence. |
| Pages and final acceptance | foundry-web consumes the exact authorized producer release, uploads/deploys it through Actions, and verifies actual deployment identity, HTTPS target, every asset and complete live journeys/assessments. A successful upload is not final acceptance. |

Every row remains required for full platform acceptance. Organizational identity
and compliance additionally require that organization's bound evidence. The core
may be published after its complete stage gates pass; other facets remain
explicitly unaccepted. Missing owner inputs block dependent claims and
operations, not unrelated core implementation. The workspace's 5 September SDK
and Calculator task list does not replace the later Foundry scope in SPEC.md.

PrismPM source now provides `export-browser`: integrity-checked export of the
six generated browser files from a local immutable OCI release, without
application source or rebuilding. The locked SDK does not include this API.
A reviewed SDK update and consumer verification remain required; export does
not establish producer readiness, target authorization or deployed acceptance.

The preceding source-export status update passed full native AMD64 `just vv`
in the locked SDK; log: `target/source-export-status-full-vv.log`.
This is scaffold verification, not product acceptance.

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

The locked Veilid browser path requires reachable transport peers; Pages distribution
alone does not provide them. The owner authorizes disclosed public Veilid
bootstrap/relay peers for bootstrap. Their modeled bindings, transport integration,
trust/failure tests and actual network acceptance remain required; no such dependency
is configured yet.
On 18 September 2026, four bounded discovery requests to the official default
and its two DNS-advertised bootstrap hosts returned UDP/TCP/WS peers but no WSS
endpoint. This is a discovery limit, not a census of the public network.
[Veilid 0.5.7](https://gitlab.com/veilid/veilid/-/raw/v0.5.7/veilid-wasm/README.md)
documents the HTTPS/outbound-relay limitation; its
[relay selector](https://gitlab.com/veilid/veilid/-/raw/v0.5.7/veilid-core/src/routing_table/mod.rs)
returns no outbound relay. Require an authenticated secure bootstrap route,
implemented browser routing, and real HTTPS-origin acceptance before claiming
shared operation. Enabling the WSS build feature alone does not satisfy these.

Holospaces `96769f16be454ab1572fddff4613704ccfbebf5e` provides browser storage
and execution primitives. Its local WebRTC witness and public-key/address
tests do not establish production discovery, private-key possession, private
replication or approved durability. Its threat-model assumptions must be
revalidated against participant/faculty sessions only.
Its threat model assumes native relay peers alongside browser tabs; its local
WebRTC witness uses out-of-band signaling. Neither establishes automatic public
discovery or availability under participant-only session churn. A browser-only
replacement cannot be accepted until these dependencies and loss cases are
modeled and exercised; an all-offline participant network cannot execute services.

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
