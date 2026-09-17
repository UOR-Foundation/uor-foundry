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

## Complete product requirements

| Boundary | Remaining work and acceptance |
| --- | --- |
| SDK and dependencies | Publish and verify the complete production dependency closure; deliver lawful, digest-bound oracle inputs from a fresh cache; accept the corrected compiler/runtime in the immutable SDK on both architectures. Source integration is not consumer acceptance. |
| Standards | Implement OSCAL catalogs, profile resolution, component/system records, inheritance and authenticated assessments. Bind every adopted edition to its complete applicable authoritative coverage. Existing structural control records are not OSCAL implementation. |
| Organization and sites | Model the approved Foundation, HQ and Foundry records, policies, responsibilities and physical/human obligations; validate their applicable assessments. |
| Services and Views | Implement every service and stakeholder journey in SPEC.md through Prism/LexLean and prism-stdlib, including state, permissions, effects, resource bounds and failures. No handwritten application substitute or draft-preview release. |
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
- governance, authorized people/keys, admission, delegation and recovery policies;
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

Holospaces `96769f16be454ab1572fddff4613704ccfbebf5e` provides browser storage
and execution primitives. Its local WebRTC witness and public-key/address
tests do not establish production discovery, private-key possession, private
replication or approved durability. Its threat-model assumptions must be
revalidated against participant/faculty sessions only.

All 19 Hologram 0.13.1 public crate index entries remain absent. The
[last real publication](https://github.com/Hologram-Technologies/hologram/actions/runs/34018837931)
failed with HTTP 403; a successful dry run is not publication.
Upstream release approval and publishing authority remain external inputs.
The current PrismPM dependency couples its `.holo` archive/codec implementation
to `uor-hologram`; this does not require hosting the Hologram platform.
Any upstream separation must preserve the modeled format and authoritative
compatibility checks. The required Foundry browser services remain unimplemented
independently of this package-publication boundary.

GitHub reports no foundry-web deployments; the default HTTPS Pages URL returns
404. [Publisher status](https://github.com/UOR-Foundation/foundry-web/blob/main/IMPLEMENTATION.md)
owns target routing and publication evidence. The functional core is authorized
for staged publication only after its release gates pass; a preview is not.
