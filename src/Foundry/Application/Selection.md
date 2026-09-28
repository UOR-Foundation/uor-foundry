# Selection transition

`SC-01` is a private source-owned selection reducer. It reuses AC-01 state,
commands and references; it neither authenticates lookup observations nor
creates accounts, organizations, grants, sessions or effects. The integrating
service must independently admit current lookup data and permission before
calling it. No public dispatch or UI action is enabled by this component.
The reducer is pure: cloned old state, command and observation can produce the
same result again. It does not provide one-use consumption or durable currentness.

## Closed operation

`reduceApplicationSelection(current, command, observation)` first runs the
complete AC-01 command/context check. Observation retains the exact captured
command, including request reference, instance, all account/scope fields and
expected revision. Its canonical command bytes must equal the checked command;
matching object IDs alone cannot reuse an observation from another request.

| Command | Required observation | State change |
| --- | --- | --- |
| ReadCurrent | NoLookup | ReadOnly: exact state, no revision increment. |
| SelectAccount | Account | Exact namespace/account ID; replace account and clear organization/workspace. |
| SelectOrganization | Organization | Enrolled selected account; exact organization reference; replace organization and clear workspace. |
| OpenWorkspace | Workspace | Enrolled account, Provisional/Active organization; exact workspace and parent organization references. |
| ClearScope | NoLookup | Retain account; clear organization/workspace. |

Every Changed result advances only the application revision by one; domain
record revisions, epochs, references, labels and phases remain byte-exact.
Selecting the already-selected account or organization still clears subordinate
scopes and advances the application revision. ClearScope also advances it when
the scope is already empty; these explicit commands are not ReadCurrent aliases.
Selecting an account permits every account phase, including states needing
enrollment/recovery. Selecting an organization preserves every organization
phase for its appropriate read-only/status view. Suspended/Retired organizations
cannot open a workspace through this transition. These checks are not proof
of current state or permission, and cannot replace the owning service policy.

Lookup commands also accept Cancelled, producing Cancelled with the exact prior
state before phase eligibility checks; cancellation grants no access, even for
a suspended account or retired organization. NotFound, Unavailable and Denied are distinct typed failures with no
replacement state. None permits stale-data fallback. NoLookup, successful
snapshot types and live failure/cancellation outcomes are not interchangeable.
All other AC-01 actions reject as UnsupportedAction; their service transitions
remain required, not simulated here.

## Canonical wire

The observation is `[1, capturedCommand, outcome]`. Outcome tags are NoLookup
`[0]`, Account `[1, snapshot]`, Organization `[2, snapshot]`, Workspace
`[3, snapshot]`, NotFound `[4]`, Unavailable `[5]`, Denied `[6]`, Cancelled `[7]`.
Snapshots and commands retain their exact AC-01 encodings.

`selectionObservationBytes` round-trips the observation.
`applicationSelectionBytes` consumes `[1, currentState, command, observation]`.
Replies are `[1,0,value]`, `[1,1,error]` or `[1,2,CborError]`. Transition values
are ReadOnly `[0,state]`, Changed `[1,state]` or Cancelled `[2,state]`.
Errors are Context `[0,AC01Error]`, WrongObservation `[1]`, UnsupportedAction
`[2]`, WrongOutcome `[3]`, WrongNamespace `[4]`, WrongAccount `[5]`,
WrongOrganization `[6]`, WrongWorkspace `[7]`, IneligibleAccount `[8]`,
IneligibleOrganization `[9]`, NotFound `[10]`, Unavailable `[11]`, Denied `[12]`.

The existing 16,384-byte frame ceiling, shortest uint32 encoding, UTF-8/digest
bounds and exact end-of-input rules apply. Structural errors precede reduction;
then current-command validation, complete observation binding, supported action,
lookup cancellation, eligible phase, outcome kind and returned identity/parent
checks apply in order. A rejected result leaves the complete prior state intact;
it never replaces selected data with an empty or fabricated snapshot.
AC-01 revision exhaustion precedes lookup cancellation; failure never changes
state. ReadCurrent remains valid at the maximum revision.

## Acceptance boundary

The required complete generated owner covers canonical round trips, every
state/action/outcome combination, complete context substitutions, account
namespace and parent chains, subordinate clearing, all phases, maximum
revisions, cancellation and every truncation/trailing/one-over input. Genuine
kernel-valid source mutants must fail exact native std/no_std/Wasm cases.
Source registration alone is not behavioral or application acceptance.
Enrollment/provider proofs, authorized lookup, organization lifecycle, durable
currentness, DK-26/DK-30 ownership and public dispatch/replay remain required.
