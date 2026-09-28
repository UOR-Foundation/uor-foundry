# Implementation status

Audit: 27 September 2026; source main `21bb2d2575c0f4d86ab7742a3e06453fcfa2ef16`.
[SPEC.md](SPEC.md) remains the complete product contract. Neither the functional
core nor the full platform has accepted implementation evidence. Publishing
static files does not establish that acceptance.

## Required corrections

| Boundary | Actual status and remaining acceptance |
| --- | --- |
| Executable model | `Foundry.dispatch`, `present`, and `replay` echo their input. Wire the complete generated application state, effects and Views to reachable service implementations. |
| Identity and recovery | Current records and nonce comparisons do not establish mailbox control. Implement actual browser mail interoperability, credential custody, saved-code generation, atomic recovery, revocation and distributed replay protection. |
| Organizations and authority | Implement ordinary arbitrary-organization onboarding, provisional creation, authenticated scoped approvals, distinct-user quorum, complete ownership coverage and safe handover. Rust simulations do not establish these boundaries. |
| Workspaces and messaging | Replace in-memory clone/relabel simulations with authenticated shared storage, independently delivered messages, durable recovery and real multi-user browser journeys. |
| Browser networking | Relay URLs and state flags are not authenticated connections. Implement actual Kappa/Veilid browser interoperability, discovery, replication, confidentiality, revocation and measured fault recovery using allowed public operators. |
| Services and design | Complete every SPEC service and its modeled user journeys, coherent navigation, accessible responsive interaction and failure/recovery states. Route labels and supplied accessibility booleans are not implementations. |
| Standards | Imported documents/fixtures are inputs, not product oracle execution. Complete normative inventories, implementation bindings, external validation and authenticated assessments; do not claim conformity from strings or digest prefixes. |
| SDK and release | Replace the local `src/Foundation/Browser/Application/V1/Model.lex.tex` SDK copy through a verified immutable SDK update, then remove the copy. Verify the SDK consumer boundary; generate twice in clean roots; authenticate complete readiness, exact artifacts, authorization and handoff. Literal digest/metadata simulations do not perform builds, publication or rollback. |
| Deployment | Verify exact published bytes and complete live stakeholder journeys for the same accepted release. Publisher delivery checks cannot replace producer acceptance. |

## Authority and evidence correction

The prior owner-input records were not supplied or authenticated by the owner.
The asserted legal registration, sites, administrator keys, assessment bodies,
financial policies and service-level approvals are not production authority.
Their original values are retained only in an explicitly synthetic fixture for
review; no special mailbox or seeded organization is required by Foundry.

Earlier `implemented`, `accepted`, `PRODUCER_READY`, signed-assessment and
reproducibility claims in TOML/Rust simulations are not accepted evidence.
For example, `model/standards.toml` records WCAG conformity using SHA-256 of empty
content, and producer-release records supply two identical literal digests.
These records must be replaced by actual assessment/build results, not promoted.

## Verification integrity

`AM-01` now rejects reopening terminal proposals, inconsistent approval records,
empty/duplicate approvers, absent proposers and zero quorums in the LexLean
source. Execution rechecks the distinct approval count and threshold. New typed
errors are `ProposalRejected`, `AlreadyApproved`, `InvalidApprovalState` and
`InvalidQuorum`; existing executed/duplicate/unauthorized errors remain distinct.
`node scripts/verify-authority-model.mjs` verifies the exact source in the pinned
SDK: 20 regression theorems, 37 zero-axiom declarations and four rejected source
mutations (terminal reopening, record trust, execution quorum, zero quorum).
The regressions first failed `LLV7002` against the prior behavior. This is scoped
source/kernel evidence, not generated native/Wasm or application acceptance.
Authenticated membership, current authority revisions and runtime integration
remain required. The consumer language lock is incompatible with the pinned
SDK; the isolated diagnostic does not alter or replace that lock.

`VI-01` rejects absent, stale, substituted and echo-only generated browser
artifacts. It strengthens the existing application gate; it is not a substitute
for complete product acceptance. The formerly ignored application test is
mandatory again. Imported-oracle checks explicitly report input integrity only.
Its Playwright owner still targets the obsolete draft-preview journey; it must
be replaced by complete generated Foundry journeys, not counted as product coverage.

The complete gate is expected to reject the current implementation. Every
unimplemented service, control, journey and fault case remains required. No
preview, fixture, handwritten application or narrowed gate is authorized.
