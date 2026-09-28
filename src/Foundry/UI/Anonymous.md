# Anonymous entry contract

`SV-01` product-owned component, not an authenticated application/session state.
`Journey.lex.tex` owns selection and bounds; `Presentation.lex.tex` owns DK-23
frames, DK-29 semantics/design, labels and navigation. Both depend on existing
prism-stdlib contracts. No consumer-defined `Foundation` API is introduced.

| Private byte root | Input | Successful output |
| --- | --- | --- |
| `Journey.anonymousSelectorBytes` | Selector | Same canonical selector |
| `Presentation.anonymousPresentBytes` | Selector | DK-23 Presentation |
| `Presentation.anonymousSemanticPresentBytes` | Selector | DK-29 SemanticPresentation |
| `Presentation.anonymousNavigateBytes` | `[1, selector, intent]` | Advanced selector |
| `Presentation.anonymousNavigationSemanticBytes` | `[1, selector, intent]` | Destination semantics with transient heading focus |
| `Presentation.anonymousDesignBytes` | Empty bytes | DK-29 design catalogue |

The selector is canonical CBOR `[1, revision, phase, screen, draftEpoch]`:
revision/draftEpoch are unsigned 32-bit values; phase is Ready/Pending/
ReplayRequired/Closed (`0..3`); screen is Welcome/Enrollment/SignIn/
EmailRecovery/SavedCodeRecovery (`0..4`). Its maximum encoded size is 14 bytes.
Navigation embeds the DK-23 Intent directly, accepts at most 64 bytes and uses
a bounded SDK value budget. Successful navigation is at most 25 input bytes.
Both readers reject trailing bytes, indefinite/noncanonical encodings,
unknown versions, wrong arities and out-of-domain values without coercion.

Actions `1..5` select screens `0..4`. Entry navigation contains two relevant
destinations, never the current screen. Recovery is contextual to sign-in and
the two recovery screens; it is not a parallel primary entry choice.
Navigation reconstructs the exact frame and applies SDK `intentFits`: Ready,
matching revision, enabled action, exact empty field binding. Revision and
draft epoch each advance once; either exhausted counter rejects without wrap.
The selector carries no principal, mailbox fact, organization, grant or effect.
It must never authorize account, organization, storage or network operations.

Failures use the existing DK-23 error encoding `[1, 1, CborError]`. Wrong
version/short arity or rejected navigation is WrongType (3). Oversized input,
oversized arity, invalid selector bounds or exhausted counters is ValueLimit
(6). SDK canonicality, truncation and trailing-input errors remain distinct.
The design root rejects nonempty input with WrongType.

Frames use one source-owned sorted label catalogue, native controls, a banner,
account navigation, main landmark and skip link. Primary navigation and recovery
forms have distinct accessible names. Closed frames are empty;
Pending/ReplayRequired disable navigation. Account fields remain disabled and
have no submission action until the actual identity/recovery transitions are
connected. Every open screen says these services are unavailable; no mailbox,
account, success, recovery or authority is fabricated. The component's neutral
light/dark tokens are not an approved organizational brand kit.

The base public BrowserView protocol remains `prismpm/browser-presentation/1`.
The semantic/design roots are private; they do not invent a public host protocol.
The locked SDK currently cannot accept these dependencies. Independent
source/kernel/native/Wasm/DOM diagnostics against an exact upstream closure
are conditional evidence only, never a replacement SDK or release gate.
Full identity, roles, shared workspaces, persistence, messaging, all SPEC
services and complete rendered/journey assessment remain mandatory.

## Navigation focus

Initial/restored selector projections never request focus. The private
`anonymousNavigationSemanticBytes` root admits the complete navigation request
before projecting the destination with heading focus. Its result is a transient
navigation outcome, not persisted state or a focus flag supplied by a caller.
Pending/replay/idempotent updates must
preserve context, and closing must retain a logical surviving focus target.
Assess actual keyboard order and visible focus at narrow widths/zoom, not merely
a focus node number. These are product expectations informed by
[WCAG focus order](https://www.w3.org/WAI/WCAG22/Understanding/focus-order.html)
and [APG keyboard guidance](https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/),
not a claim that WCAG prescribes one universal focus target.

The independently reviewed SDK renderer correction retains an established live
region and updates it without stealing focus. It is not yet the locked consumer
SDK. Pending/replay/completion and exactly-once announcements remain required.
[ARIA22](https://www.w3.org/WAI/WCAG22/Techniques/aria/ARIA22) checks that the status
container exists before the message, and recommends explicit atomicity for
interoperability. DOM/axe checks cannot establish actual assistive-technology
speech; complete user-journey and screen-reader assessment remains required.
