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
approach in browsers, without a hosted platform or organization backend.
The native delivery, mailbox-proof and recovery protocol is not implemented;
this is implementation work, not a requirement to choose an authentication
vendor. Bind its account/credential/recovery records, effects, trust boundaries
and complete acceptance evidence. Recovery cannot bypass scoped approval,
restore revoked grants or create authority. External dependencies still need
disclosure and approval; no verification secrets belong in public artifacts.
Key possession or a UOR reference alone does not establish mailbox control.
Existing mail infrastructure may supply authenticated submission/mailbox access,
without owning Foundry accounts or recovery decisions. Saved backup codes are
also required: protected issuance, account/revision binding, one-time redemption,
rotation, credential replacement, session invalidation and notification. Neither
mail submission nor a backup code substitutes for the other's acceptance;
distributed replay/rollback rejection and encrypted-data recovery need their
own evidence. These capabilities remain unimplemented on main.
PrismPM's current Workspace/V1 single-owner operations do not implement the
required scoped multi-administrator policy. Its authority, protocol and recovery
boundaries need a modeled implementation, not merely additional View roles.

The generic-organization contract update passed full `just vv` in the locked
AMD64 SDK; log: `target/generic-organization-contract-full-vv.log`. This is
scaffold verification, not acceptance of organization creation or isolation.

## Complete product requirements

| Boundary | Remaining work and acceptance |
| --- | --- |
| SDK and dependencies | Publish and verify the self-contained immutable OCI SDK on both architectures, including the complete offline dependency closure and digest-bound oracle inputs from a fresh cache. Source integration is not consumer acceptance. Public Cargo publication follows Foundry publication and verification. |
| Standards | Implement OSCAL catalogs, profile resolution, component/system records, inheritance and authenticated assessments. Bind every adopted edition to its complete applicable authoritative coverage. Existing structural control records are not OSCAL implementation. |
| Organization lifecycle and sites | Model normal creation, provisional setup, activation, isolated records and site lifecycles. UOR's Foundation, HQ, Foundry and Citizen Gardens records use these same workflows; validate authorized policies and applicable physical/human assessments without seeded privileges. |
| Services and Views | Implement every service and stakeholder journey in SPEC.md through Prism/LexLean and prism-stdlib, including state, permissions, effects, resource bounds and failures. No handwritten application substitute or draft-preview release. |
| Account and authority continuity | Implement UOR-native verified email enrollment/login/recovery through PrismPM, scoped grants, distinct-user approval quorums and atomic post-change ownership coverage. Reject concurrent lockout, replay, revoked-grant recovery and premature bootstrap retirement; a self-selected role or repeated key is not another administrator. |
| Browser object space | Implement browser Kappa storage, queries, inbound dispatch and verified blob transfer; model authenticated membership, confidentiality, conflicts, revocation, retention, replication, repair and recovery. |
| Network acceptance | Exercise independent participants under real discovery/connectivity constraints, suspension, eviction, partitions, hostile inputs and replica loss. Measure approved availability and recovery targets; local two-browser tests are not internet-scale evidence. |
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

No approved records for the following were found in the audited main or draft:

- organization identity, legal entity, HQ/site locations, jurisdictions and assessments;
- authenticated administrator keys, membership admission records,
  activation policies, exact scope quorums and recovery rules beyond
  the required isolation and access-continuity constraints;
- adopted standards/editions, normative-source rights and assessment authorities;
- business plan, operating procedures, brand assets and publication approvals;
- payment scope/counterparties and certification issuer/recognition rules;
- availability/workload/fault bounds, RPO/RTO, retention and replica obligations.

These are per-organization inputs, except for explicitly modeled platform policy.
UOR's mission does not supply them; they must not be invented or preinstalled.
Missing organization records block dependent claims and operations, not unrelated
generic workflows or publication of an otherwise accepted empty platform.
Approval of registrants or organization names is not an acceptance input.

## External boundaries

Kappa `2af86560a177fc9651b6c0e92e7974140ed77dd5` is a native Rust service;
its Veilid startup discards the inbound receiver, and reconciliation copies
tags/digests without referenced blob bytes. It is not an accepted browser
service. The locked Veilid browser path requires reachable transport peers;
Pages distribution alone does not provide them.

The owner authorizes disclosed public Veilid bootstrap/relay peers for bootstrap.
Their modeled bindings, transport integration, trust/failure tests and actual
network acceptance remain required; no such dependency is configured yet.
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
