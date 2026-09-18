# Implementation status

Audit: 17 September 2026 UTC, main `278ee0a29fc8d7262daf52791e24a845ac35a6f7`.
[SPEC.md](SPEC.md) remains the complete product contract. This record is not
an implementation claim or a reduced release plan.

Main is a development scaffold: capability, authority, and claim registers
are empty. [Bootstrap verification](https://github.com/UOR-Foundation/uor-foundry/actions/runs/35178652915)
passed on AMD64 and ARM64; it does not verify product behavior.
The documentation update passed full `just vv` in the locked AMD64 SDK;
`target/producer-status-full-vv.log` has SHA-256
`63f702d792c8eb6bc545811b8c5f4a647b3cc53b47846e0cb847ff910ea5db0b`.
[Draft PR #3](https://github.com/UOR-Foundation/uor-foundry/pull/3), audited at
`8bbb60b482eb0061beeaaf1533f39a9d7a836949`, contains only an unaccepted bounded
UTF-8 draft preview, without identity, persistence, publication, or networking.

## Authorized first release

Implement identity, roles, shared workspaces, persistence, and messaging as one
functional core under SPEC.md's staged acceptance contract. None is implemented
or accepted on main. Require actual independent-user interaction, permission
enforcement, persisted state, message delivery and fault/recovery evidence.
Workspace identities and roles must not imply verified Foundation authority.
The draft text preview does not meet this release scope.

The owner designates `trinity@uor.foundation` as the initial Foundation
administrator mailbox. Its owner has not been authenticated by this
implementation; no administrator key is enrolled. Implement and verify
the challenge/key binding and negative cases required by SPEC.md before
granting Foundation authority, including faculty-membership approval.
A verification mechanism and any external dependency still require approval;
no mailbox or provider credentials belong in repository files, public artifacts,
or chat. The owner additionally requires optional email login/recovery for every
user, scoped administration, policy-defined approval quorums and retained
multi-user ownership. Bootstrap retirement requires complete model coverage by
other users; every affected part must retain at least two administrators or its
stronger modeled minimum. These requirements are specified in SPEC.md but are
not implemented or verified. No permanent root account or universal quorum of
three is inferred.
The initial mailbox/faculty requirements update passed full `just vv` in the locked AMD64 SDK;
`target/initial-admin-faculty-authority-full-vv.log` has SHA-256
`b0235c523e15bf04c2da5db8dd3b4b09fd84d1cdfe72d69262df9ba3b72fdfd9`.
That verifies the scaffold, not administrator authentication or portal behavior.
The later scoped-authority requirements passed the complete locked-SDK gate
and [both native CI architectures](https://github.com/UOR-Foundation/uor-foundry/actions/runs/35375509288)
at `12f74b78d33f03c33cf5bec9cf6f352f7ef7b728`; this is also scaffold evidence.
The owner requires browser-executed, PrismPM-defined login/recovery, not a hosted
Foundation backend. No mailbox-proof authority is approved or implemented.
PrismPM's current Workspace/V1 single-owner operations do not implement the
required scoped multi-administrator policy. Its authority, protocol and recovery
boundaries need a modeled implementation, not merely additional View roles.

## Complete product requirements

| Boundary | Remaining work and acceptance |
| --- | --- |
| SDK and dependencies | Publish and verify the self-contained immutable OCI SDK on both architectures, including the complete offline dependency closure and digest-bound oracle inputs from a fresh cache. Source integration is not consumer acceptance. Public Cargo publication follows Foundry publication and verification. |
| Standards | Implement OSCAL catalogs, profile resolution, component/system records, inheritance and authenticated assessments. Bind every adopted edition to its complete applicable authoritative coverage. Existing structural control records are not OSCAL implementation. |
| Organization and sites | Model the approved Foundation, HQ and Foundry records, policies, responsibilities and physical/human obligations; validate their applicable assessments. |
| Services and Views | Implement every service and stakeholder journey in SPEC.md through Prism/LexLean and prism-stdlib, including state, permissions, effects, resource bounds and failures. No handwritten application substitute or draft-preview release. |
| Account and authority continuity | Implement verified optional email login/recovery, scoped grants, distinct-user approval quorums and atomic post-change ownership coverage. Reject concurrent lockout, replay, revoked-grant recovery and premature bootstrap retirement; a self-selected role or repeated key is not another administrator. |
| Browser object space | Implement browser Kappa storage, queries, inbound dispatch and verified blob transfer; model authenticated membership, confidentiality, conflicts, revocation, retention, replication, repair and recovery. |
| Network acceptance | Exercise independent participants under real discovery/connectivity constraints, suspension, eviction, partitions, hostile inputs and replica loss. Measure approved availability and recovery targets; local two-browser tests are not internet-scale evidence. |
| Producer release | Generate all artifacts twice reproducibly; verify complete service/control/dependency/assessment coverage. Bind exact producer identity, artifact tree and pre-publication evidence, with only the exact deployment-dependent checks outstanding. |
| Publication SDK | Complete source-free acquisition, readiness and authorization verification; integrate confined atomic artifact export, live verification and accepted-release rollback. Preserve unchanged bytes; reject partial, stale, substituted or unauthorized evidence. |
| Pages and final acceptance | foundry-web consumes the exact authorized producer release, uploads/deploys it through Actions, and verifies actual deployment identity, HTTPS target, every asset and complete live journeys/assessments. A successful upload is not final acceptance. |

Every row remains required for full Foundation acceptance. The authorized core
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

- legal entity, HQ/site locations, jurisdictions and site assessments;
- authenticated administrator keys, admission records, verification authority,
  exact scope policies/quorums and recovery rules beyond the owner-defined
  mailbox designation and access-continuity requirements;
- adopted standards/editions, normative-source rights and assessment authorities;
- business plan, operating procedures, brand assets and publication approvals;
- payment scope/counterparties and certification issuer/recognition rules;
- availability/workload/fault bounds, RPO/RTO, retention and replica obligations.

The mission statement does not supply these values. They must not be invented.

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
