# Application context component

`AC-01` is a private source-owned data/codec boundary, not an authenticated
session, enrollment service, authority reducer or public dispatch protocol.
`State`, `Command` and `Reference` use existing SDK byte/UTF-8/CBOR primitives
and `Foundation.Holo.V1.Identity.Digest`; no `Foundation` namespace is defined here.

## References

Account identity is the exact DK-33 SHA-256 digest of its immutable genesis.
Its 32-byte namespace remains explicit and is checked independently. Binary
account IDs convert exactly to `sha256:` plus 64 lowercase hexadecimal digits
for the ST-12/13/15 string-reference boundary. General object references use
the SDK's BLAKE3/SHA-256 `Digest` and the corresponding 71-byte Kappa label.
Both conversions are modeled, bounded and round-trip tested; unknown prefixes,
uppercase, aliases, wrong lengths and non-ASCII digits reject.

This representation follows [UOR content addressing](https://github.com/UOR-Foundation/UOR-Framework/blob/51c01382200b0179d6640b07e9c8119364ab69a1/docs/content/concepts/content-addressing.md)
and [Kappa labels](https://github.com/UOR-Foundation/kappa-registry/blob/2af86560a177fc9651b6c0e92e7974140ed77dd5/crates/kappa-core/src/kappa/label.rs).
It does not allocate identities, compute digests, establish human uniqueness or
authenticate referenced content. Kappa's storage namespace UUID is a distinct
16-byte identifier; it is never substituted for the account namespace.

## State

The canonical array is `[1, instance, revision, account?, organization?, workspace?]`.
`instance` is the immutable 32-byte DK-26 application-instance binding, not an
account namespace. The account record independently retains its exact DK-33
namespace; neither value can replace the other.
Optional records are `[0]` or `[1, record]`, never null or a zero sentinel.

- Account: `[namespace, accountId, revision, credentialEpoch, recoveryEpoch, credentialRef,
  authorizationRef, stateRef, phase]`.
- Organization: `[organizationRef, revision, authorityRevision, policyRef,
  stateRef, workspaceIndexRef, phase, label]`.
- Workspace: `[organizationRef, workspaceRef, revision, stateRef, label]`.

Account phases are Allocated/Enrolled/Suspended/RecoveryRequired (`0..3`).
Organization phases are Provisional/Active/Suspended/Retired (`0..3`). These are
decoded data, not evidence that their transitions were authorized. Workspace
selection requires a matching selected organization. Empty selection contains
no account or organization and does not constrain the size of any registry.
Names are non-unique display data, never identifiers or reserved privileges.
State references bind complete records; these compact selections cannot replace
current authenticated account, registry, policy or administration snapshots.

## Commands

The canonical array is `[1, instance, requestRef, expectedRevision, account?,
scope, action]`. Account binding is the complete account record, including its
namespace, phase and every credential/current-state field. Scope is `[0]`,
`[1, organization]` or `[2, organization, workspace]`, using complete records
above, not caller-selected subsets of their binding fields.

| Tag | Action fields | Required selection / integration |
| --- | --- | --- |
| 0 | ReadCurrent | Any; read authorization remains required. |
| 1 | BeginMailbox, operation `0..3`, mailbox, candidateCredentialRef | Account; ST-13 Enroll/Login/Recover/ReplaceMailbox and actual proof adapters. |
| 2 | RedeemSavedCode, recoveryRequestRef | Account; complete ST-11 request, secret admission and current state. |
| 3 | ReplaceSavedCodes, issuanceRequestRef | Account; ST-11 current-credential authorization and protected issuance. |
| 4 | CreateOrganization, partitionRef, partitionRevision, organizationRef, rootScopeRef, label | Account, any current selection; ST-15 creation and exact canonical reference spelling. Existing selection grants no new-organization authority. |
| 5 | ProposeAdministration, proposalRef, planRef | Organization; complete ST-12 plan, not a caller approval flag. |
| 6 | ApproveAdministration, proposalRef | Organization; authenticated approval over the complete ST-12 request. |
| 7 | CreateWorkspace, workspaceRef, label | Organization; scoped creation, durable allocation and isolation. |
| 8 | OpenWorkspace, workspaceRef | Organization; current membership and authorized lookup. |
| 9 | ChangeWorkspace, changeRef | Workspace; complete base-bound change and durable compare/commit. |
| 10 | SendMessage, messageRef, channelRef, contentRef | Workspace; membership, delivery and retained-message protocol. |
| 11 | CloseSession | Account; DK-26 Close and journal/custody cleanup. |
| 12 | BeginAccountGenesis, namespace, genesisRef | Any selection, including empty; fetch exact DK-33 genesis, check namespace/digest, prove key possession and durably allocate only if absent. No decoded account is implicitly created. |
| 13 | SelectOrganization, organizationRef | Account, any current selection; authorized current registry lookup, never name-based ownership. |
| 14 | ClearScope | Any selection; clear organization/workspace context without deleting records or grants. |
| 15 | SelectAccount, namespace, accountId | Any selection, including a new device; independently validate current DK-33 account/genesis lookup and clear prior organization/workspace selection. Selecting data grants no account authority. |

Referenced requests/plans/content must be independently fetched, hash-checked,
decoded and admitted by their owning source boundary before execution. A digest
is not that admission. The command records no authority, completion, delivery,
durability or permission booleans. Unknown actions and extra fields reject.
Email-based account discovery and recovery remain required; SelectAccount does
not require users to remember an account ID or create another genesis to log in.

Context checking compares instance, complete account namespace/binding, selected scope
and every expected revision/reference to current supplied state. It returns
only the unchanged typed command, never `SourceAuthority`, a new state or an
effect. Mutating commands reject exhausted application revisions; ReadCurrent
does not advance a revision. Request-reference freshness and authenticated
currentness require the journal; equality alone does not prevent replay.

## Wire and integration

Canonical definite CBOR uses shortest uint32 arguments and consumes the entire
input. A digest is `[algorithm, bytes32]`, where `0` is BLAKE3 and `1` is SHA-256.
All revisions/epochs are uint32. Labels are 1–256 UTF-8 bytes; mailboxes 1–320.
State/command/context input and output ceilings are 16,384 bytes, including
wrappers; the largest admitted field remains 320 bytes. Exact valid maxima,
every truncation, one-over bounds, trailing input and substitutions are tested.
No byte limit clamps or normalizes a value.

Private observation replies are `[1,0,value]`, `[1,1,contextError]` or
`[1,2,cborError]`; errors are typed and return no replacement state.
`applicationStateBytes` and `applicationCommandBytes` consume their raw arrays;
`applicationContextBytes` consumes `[1, state, command]` and returns the exact
unchanged command only after context checking. Context errors are InvalidState
`0`, InvalidCommand `1`, WrongInstance `2`, WrongAccount `3`, WrongScope `4`,
StaleRevision `5`, CounterExhausted `6`, MissingSelection `7`, checked in that
order. Structural CBOR errors retain the SDK's existing codes. Context checks
validate both supplied typed records before comparing bindings; they do not
turn a well-formed current-state argument into an authenticated one.
The source/codec component does not update `dispatch`, `replay`, SDK locks or
any accepted boundary. DK-33 authentication, ST-11/13 recovery, ST-12/15
authority, session composition, durable currentness and all network/service
journeys remain required. Suspended/retired organization transitions require
an explicit SDK lifecycle extension; they cannot be coerced to Active or
claimed implemented because their state tags decode.
