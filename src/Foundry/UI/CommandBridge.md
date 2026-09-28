# UI command component

`UC-01` is private source-owned command preparation, not an authenticated
application session, service execution or public dispatch protocol. AC-01 data
remain untrusted until their owning authentication/current-state boundaries
admit them. No SDK lock or public root is changed by this component.

## Closed routes

Each context contains the complete AC-01 state, a typed route, its exact screen,
View revision and draft epoch. View revision is not application revision.
Routes capture source-owned object references; the UI never asks users to type
digests. Mailbox and display-name text are captured exactly from field node 5;
there is no case folding, trimming, normalization or inferred mailbox proof.

| Screen | Route | Action | Field / SDK operation |
| --- | --- | --- | --- |
| 0 | Overview | 100 ReadCurrent | None |
| 1 | Enrollment | 101 BeginMailbox | Email; ST-13 Enroll |
| 2 | Sign in | 101 BeginMailbox | Email; ST-13 Login |
| 3 | Email recovery | 101 BeginMailbox | Email; ST-13 Recover |
| 4 | Change email | 101 BeginMailbox | Email; ST-13 ReplaceMailbox |
| 5 | Admitted saved-code request | 102 RedeemSavedCode | Protected ST-11 request reference only |
| 6 | Saved-code replacement | 103 ReplaceSavedCodes | Protected ST-11 issuance reference only |
| 7 | Create organization | 104 CreateOrganization | Organization name |
| 8 | Administration proposal | 105 ProposeAdministration | None |
| 9 | Administration approval | 106 ApproveAdministration | None |
| 10 | Create workspace | 107 CreateWorkspace | Workspace name |
| 11 | Open workspace | 108 OpenWorkspace | None |
| 12 | Workspace change | 109 ChangeWorkspace | None |
| 13 | Message | 110 SendMessage | Protected content reference, not message text |
| 14 | Session | 111 CloseSession | None |
| 15 | Account creation | 112 BeginAccountGenesis | Exact captured DK-33 genesis reference |
| 16 | Organization selection | 113 SelectOrganization | None |
| 17 | Clear selection | 114 ClearScope | None |
| 18 | Account selection | 115 SelectAccount | Independently looked-up account binding |

The route union is Fixed(AC-01 action), Mailbox(ST-13 operation, credential
reference), Organization(partition reference/revision, organization reference,
root-scope reference), or Workspace(workspace reference). Fixed rejects the
three editable action constructors; there is only one representation of each
route. Every route fixes its screen, label, action and complete field inventory.
The compact frame is main section 1, heading 2, helper 3, form 4, field 5 and
submit 6. With no field, submit is node 5. Input is required and admits
1–320 UTF-8 bytes for email, 1–256 for names.
Other actions admit no fields. DK-23 `intentFits` checks the generated frame.

Saved-code secrets use the existing SDK private `secretDispatch` path. Only an
independently admitted protected request reference can reach the ordinary route;
no code, password, deterministic secret hash or secret-derived public request ID
is accepted here. The complete private recovery flow remains required.

## Private observer wire

`commandBridgeBytes` is a bounded component-test boundary, not the application's
public dispatch or BrowserView protocol. Its canonical CBOR inputs are closed:

| Operation | Exact input | Successful value |
| --- | --- | --- |
| Prepare | `[1, 0, context, effectContext, intent]` | Pending continuation |
| Settle | `[1, 1, continuation, currentContext, completion]` | Continuation |
| Close | `[1, 2, continuation]` | Closed continuation |
| Present | `[1, 3, context]` | DK-23 Presentation |
| Label | `[1, 4, index]` | Text; index 0–27 |

Context is `[AC01State, route, screen, viewRevision, draftEpoch]`; effect context
is `[application, manifest, session, operation]`. Route tags are `[0, action]`,
`[1, mailboxOperation, credentialRef]`,
`[2, partitionRef, partitionRevision, organizationRef, rootScopeRef]` and
`[3, workspaceRef]`. Plan is `[context, DK23Intent, DK18Request]`.
Continuation is `[0, plan]`, `[1, plan, AC01Command]`, `[2, plan, DK18Failure]`,
`[3, plan]` or `[4, plan]` for Pending/Resolved/Failed/Unknown/Closed.
Imported records retain their existing SDK/AC-01 encoding.

Responses are `[1, 0, value]`, `[1, 1, error]` or `[1, 2, CborError]`.
Component error tags 0–6 are InvalidContext, InvalidIntent,
InvalidEffectContext, WrongCompletion, StaleContext, WrongPhase and InvalidDigest;
each is `[tag]`. Encoding and Application errors are `[7, CborError]` and
`[8, AC01ContextError]`. Unknown operations, arities, versions and noncanonical,
truncated, trailing or over-budget input reject rather than selecting defaults.

## Digest and continuation

The source consumes the four observations from DK-18
`openContextualStagedEffects().context()`: application, manifest and session references
(32 bytes each), and the next uint32 operation. Observation is not reservation
or authority. The source constructs an actual DK-18 `EffectRequest`, resource
`foundry/command-reference/1`, operation `DigestBytes`.

Its preimage is ASCII `foundry/command-reference/1`, NUL, then canonical CBOR
`[1, application, manifest, session, operation, completeState, screen,
viewRevision, draftEpoch, capturedAction]`. All non-secret action fields and
expected account, scope and revision bindings survive the await unchanged.
This private component's input/material/output ceiling is 16,384 bytes; it does
not alter DK-18's generic limits. No caller-supplied digest convenience path is
provided. The SDK performs SHA-256 and returns its canonical `sha256:` text.
Preparation checks required selection before application-revision exhaustion;
this preflight precedence differs from checking an already constructed AC-01 command.

The typed continuation is Pending, Resolved, Failed, Unknown or Closed.
Settlement requires Pending, exact complete request equality and unchanged
application context. A valid Digest result produces the complete AC-01 command
with the actual digest as request reference, then reruns AC-01 context checking.
Definitive SDK rejection becomes Failed; unknown remains Unknown, never retry
or rollback. Close preserves the captured plan and rejects late settlement.
Resolved/Failed/Unknown/Closed cannot settle again.

These are pure values. Cloning an old Pending value and settling each copy can
produce the same command twice; an explicit counterexample must preserve this
limitation. The actual DK-18 private host owns one-use release and effect
admission/completion, not this codec. Complete DK-26/DK-30 composition must own
the live continuation, authenticated currentness, journal and durable command
consumption. A second ad hoc product host is not introduced to hide that work.

## Required verification and integration

Generated source/kernel/native `std`/`no_std`/Wasm tests must cover every route,
exact fields and bounds, canonical encoding, context substitutions, currentness
changes, failures, unknown outcomes, closure and all terminal states. Actual
SDK-host tests must execute SHA-256, one-use release, wrong handle/session/
operation/manifest, concurrent current-state changes and the pure clone
counterexample. Source-registration tests alone cannot accept this component.

The full generated owner, secret recovery, authenticated session, service
transitions and public dispatch/replay integration remain mandatory. Views
must distinguish initial/restored state from accepted navigation: only the
latter announces/focuses its destination once. Entry emphasizes account creation
and sign-in; recovery links are contextual, not five equal global buttons.
Pending/replay views must preserve meaningful focus. These journey requirements
need generated browser evidence, not publisher CSS or a handwritten portal.
