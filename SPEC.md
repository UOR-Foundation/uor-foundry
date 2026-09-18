# UOR Foundry product requirements

This is the required product contract, not evidence of implementation.
Application semantics must be authored in Prism/LexLean; generated outputs
and this requirements document cannot substitute for the accepted model.

## Purpose and ownership

The UOR Foundation is dedicated to the democratization of technology for
the well-being of humanity. It operates under the Citizen Gardens model
through a network of physical Foundries; the first also houses Foundation HQ.
No location, legal structure, financial policy, or additional authority is
inferred from that mission.

`uor-foundry` owns the complete Foundation and Foundry product model,
including its services, workflows, stakeholder roles, permissions, Views,
and acceptance. It uses the PrismPM SDK and prism-stdlib to generate and
verify the portal and its complete runtime closure.

`foundry-web` is the thin publication and deployment repository. It consumes
an immutable verified `uor-foundry` release through the SDK, without copying
source semantics, redefining controls, or rebuilding a separate application.
Deployment configuration cannot add or change product behavior. Generic
compilation, release transport, and validation belong in PrismPM and its
upstream packages.

The default `https://uor-foundation.github.io/foundry-web/` is allowed for
initial publication. Routing for `https://uor.foundation/foundry-web/` is not
established. `app.uor.foundation` remains a future address, not a bootstrap
dependency. All publication targets must consume the same modeled product,
not separately authored portals.
Reusable capabilities belong upstream, not in a vendored SDK or handwritten
application fallback. All implementation semantics flow through LexLean,
generated Lean, and lean4-prod, including the Holo profile and browser Views.

## Complete scope

The portal is the authoritative interaction point for faculty, participants,
and all other modeled stakeholder roles. It must provide the Foundation's
digital presence and operations, including:

- concept-to-production Prism workflows, Git, CI, hosting, and scheduling;
- AI inference, agentic execution, notebooks, and knowledge management;
- text and multimedia messaging, collaboration, and content creation;
- administration, governance, change management, and improvement;
- business planning, operating procedures, finances, and payments;
- learning, assessment, certification, and their authority records; and
- brand identity, approved descriptions, brand kits, and accessible presentation.

This list does not replace standards-derived coverage of the whole
organization. Physical-site responsibilities and human activities remain
modeled obligations with appropriate assessment evidence.

Each service must define its actors, permissions, inputs, outputs, state,
effects, resource requirements, failure/recovery behavior, and acceptance.
An interface, mock, third-party link, or generated document is not a service
implementation. AI output is a proposal until the applicable workflow
authorizes and verifies its effects. Recording a payment or assessment is
not evidence of settlement or certification by an independent authority.

## Initial functional release

The owner authorizes staged publication of a functional core containing all
five capabilities below; full Foundation implementation need not be published
at once. This is not authorization to publish a mock or draft preview.

- Identity: authenticate a workspace identity and its actions across sessions.
- Roles: enforce modeled membership, permissions, delegation and revocation.
- Shared workspaces: independent participants access and change shared state
  under those permissions, not separate local copies presented as collaboration.
- Persistence: retain and recover authorized workspace state across reloads
  and restarts; distinguish local writes from confirmed replication.
- Messaging: deliver and retain authorized messages between workspace members,
  with explicit delivery, failure and recovery states.

Workspace authentication and role assignment do not verify civil identity,
Foundation employment, faculty appointment, or organizational authority.
Such claims require separately approved Foundation records and authorization.
The release must expose this distinction and enforce it outside the View too.

The owner designates `trinity@uor.foundation` as the initial Foundation
administrator mailbox. Bootstrap must verify control of that exact mailbox
and bind the verified challenge to the enrolling administrator key and the
Foundation authority record. Entering the address, creating a workspace,
possessing a self-signed key, or discovering mail-routing records is not
mailbox verification. Public assets must contain no verification secrets.
Any external verification dependency must be disclosed and approved before
use. Expired, replayed, wrong-mailbox and substituted-key challenges must
fail without granting authority. Once verified and enrolled, this
initial administrator is authorized to approve faculty membership. Approval
must be an authenticated, recorded administrator action enforced at every
applicable authorization boundary, not a participant-selected role.

Every user must have the option of verified email login and account recovery.
The Foundry implementation of these flows must be PrismPM-defined and execute
in browsers; a hosted Foundation authentication/recovery backend is not authorized.
Recovery restores only that user's currently authorized access; it cannot
resurrect revoked grants, bypass an approval quorum, or grant new authority.
Verification binds the mailbox, account, operation, challenge, expiry and
replacement key/session. Changing a recovery address requires authenticated
authorization and proof of the new mailbox. External verification dependencies
still require disclosure and approval; no provider is selected by this contract.

Administration is delegated over explicit parts of any modeled system, including
the Foundation and each Foundry. No permanent all-powerful user is required.
The authoritative policy defines scope, inheritance, administrator membership,
minimum retained administrators and the distinct eligible approvers required
for each grant or policy change. Quorums may require multiple administrators;
the example of three administrators is not a universal threshold.
Administrator count alone does not establish quorum availability: a two-of-two
policy still depends on both users. Approval, succession and recovery acceptance
must exercise unavailability without an implicit root or quorum bypass.

Disabling/removing an account, revoking/demoting a grant or changing scope,
inheritance or policy must leave every affected part with at least two active
authorized users and satisfy any stronger modeled minimum and approval quorum.
Aliases, additional keys and duplicate approvals do not count as extra users.
The initial sole administrator is an explicit bootstrap condition, not accepted
redundancy. `trinity@uor.foundation` may be disabled or removed only after other
users collectively cover the entire Foundry model under these rules. Retirement
is not contingent on any one successor receiving access to every part.

Evaluate approval and post-change coverage atomically against the current model
and authority revision, including concurrent changes and newly added parts.
Reject stale, replayed, out-of-scope or insufficient approvals and uncovered
parts; email recovery and direct protocol requests obey the same policy.
Acceptance must exercise partial and complete bootstrap handover, scoped
multi-party grants, duplicate approvers, concurrent removals, revoked-account
recovery, lost access and inherited-scope changes without an implicit root bypass.

The model and release evidence identify this exact stage and its complete
runtime, control, dependency and artifact closure. All five capabilities need
real multi-user journeys, persistence/recovery and adversarial acceptance.
The mandatory base profile and every applicable adopted control remain binding.
No broader compliance, certification, internet-grade availability or independent
browser-network claim follows from core acceptance.

Every other facet in Complete scope remains required and explicitly unaccepted,
not deleted, satisfied, or declared inapplicable. Release notes and Views must
distinguish accepted core behavior from these unimplemented obligations.

## Controls and standards

All facets are governed through OSCAL catalogs, resolved profiles, component
and system implementation records, and assessment evidence. Every applicable
control must have a verified local implementation, verified inheritance, or
both. Inheritance identifies provider scope, exact subjects and revisions,
evidence, validity conditions, and consumer responsibilities.

The mandatory PrismPM base profile cannot be weakened by an overlay or an
unsupported inapplicability claim. Missing implementations, planned work,
unassessed claims, and remediation records do not satisfy mandatory controls.

Every adopted standard requires a complete normative-requirement inventory,
lawfully acquired pinned sources, formal bindings, implementation mappings,
and the complete applicable authoritative oracle/assessment coverage.
Structural validation, test-corpus agreement, formal proof, human assessment,
and observed operation remain distinct. Missing coverage blocks acceptance;
it must not be hidden by narrowing the claimed standard.

The model includes human-centred design, complete accessible user journeys,
brand rules, business planning, and operating procedures. Framework and
design-system bindings must preserve the adopted web standards. Brand assets,
descriptions, forecasts, and procedures are versioned and authorized; their
publication and actual outcomes are separate evidence.

## Browser network and bootstrap

Deployed service execution and peer replication use faculty and participant
browser sessions only. Dedicated Foundation nodes and hidden service backends
are not authorized. Development and bootstrap release builds use repository
devcontainers and locked SDK CI; this does not authorize server-side substitutes
for portal functions or establish browser-resident build/CI acceptance.
Kappa provides the modeled decentralized object space, queries, references,
and service artifacts; a browser runtime executes the artifacts.

GitHub Pages initially distributes the generated portal through `foundry-web`
and bootstraps the browser network. The default Pages URL is an allowed initial
target; `https://uor.foundation/foundry-web/` awaits confirmed routing. Assets,
links, and browser acceptance must work under `/foundry-web/`, without a
repository custom domain or an `app.uor.foundation` redirect. GitHub remains
a modeled publication target after the network becomes independent.
Bootstrap hosting does not authorize undisclosed external inference,
identity, signing, agent, storage, or application-service dependencies.

The owner authorizes disclosed public Veilid bootstrap and relay peers during
bootstrap. Model their exact transport, identities, trust boundaries, effects,
failure behavior and acceptance evidence before use. This does not authorize
dedicated Foundation nodes, hosted application services, undisclosed peers,
or a claim that the browser network is independent of those peers.

The migration lifecycle has three distinct states:

1. **Bootstrap:** the accepted Pages release distributes the exact application
   and dependency closure. External dependencies are inventoried, not concealed.
2. **Candidate:** browser services run and gather complete migration evidence;
   required external hosting is not removed or claimed to be unnecessary.
3. **Independent:** an authorized transition accepts measured browser-network
   readiness, complete control coverage, recovery, and all service acceptance.

Migration must demonstrate fresh-client acquisition, returning-client offline
launch, discovery, networking, identity, compiler/oracle execution, publication,
and recovery. DNS, TLS, signaling, relays, trust bootstrap, and counterparties
must be identified and tested; they cannot disappear from the dependency model.
Publication to GitHub, Cargo registries, or another external target is an
explicit authorized network effect, not an offline completion claim.

Migration is not automatic after a timer, download count, or passing schema
check. It needs exact-release evidence and an authorized decision. Rollback
and evidence invalidation are explicit; no silent fallback changes the model.

## Authority, resilience, and availability

Authority comes from accepted models, authenticated records, and authorized
decisions, not a browser, registry hash, or generated response alone.
Object, query, peer-protocol, and effect boundaries independently enforce
permissions; a caller cannot bypass them by avoiding a View. Views expose only
authorized content, metadata, queries, edges, and actions. Negative tests must
exercise unauthorized raw requests as well as interface interactions.
Publicly delivered assets contain no private credentials or organizational data.

The data model distinguishes local work, replicated durable acceptance, and
globally confirmed effects. It defines consistency, conflict handling,
revocation, authorized deletion, retention, replica independence, repair,
key recovery, atomic updates, and restoration from independent copies.
Content addressing detects identity mismatch; it does not recreate lost bytes.

Internet-grade availability requires approved service-level targets, workload
and fault bounds, measurements, and recovery objectives. Missing peers,
suspended browsers, storage denial, deleted copies, partitions, and exhausted
resources must be exercised. Peer outages cannot be silently removed from
measurements. No claim permits execution without an available executor or
recovery after every recoverable copy is destroyed.

Targets and operating policies not supplied by the owner remain explicit
acceptance inputs, not invented defaults. Their absence blocks the claims and
operations that depend on them, not unrelated functional-core implementation.

## Release acceptance

Acceptance is scoped to the explicitly authorized release stage. Full
Foundation acceptance still requires the complete product scope; core
acceptance does not imply it. No capability within the core may be omitted.

The complete locked Prism model for that stage must generate every declared
artifact through the accepted SDK, including the `.holo` runtime closure and
browser projection.
No independent handwritten UI, service, compiler, or policy implementation may
substitute for the model. Bootstrap trust is explicit and non-circular.

Acceptance requires complete applicable control/oracle coverage, human assessment
where required, all stage service journeys, negative/mutation tests, two clean
reproducible builds, actual browser execution, recovery and fault tests, and
verification of deployed bytes at the approved deployment address. All evidence binds
exact inputs, subjects, tools, policies, and results. Empty registers cannot
establish this.

Release evidence has three explicit states:

1. **Producer-ready:** the complete declared stage and dependency closure pass
   every pre-publication gate, including all applicable controls, assessments,
   reproducible builds, service journeys, and browser/fault/recovery tests.
   Only checks that require the actual target deployment remain outstanding;
   their exact required set is recorded. This is not final product acceptance.
2. **Deployment-authorized:** an authorized decision binds that immutable
   producer-ready release to its target. The publisher verifies both the
   complete readiness evidence and authorization before deploying unchanged
   bytes. An incomplete core capability or draft-preview release cannot enter
   this state. Authorization names the stage; it cannot imply full Foundation
   acceptance.
3. **Accepted:** post-deployment identity, bytes, live journeys, operational
   measurements, and every remaining applicable control/assessment pass for
   that exact release. Failed or missing checks prevent final acceptance;
   they do not become waivers or evidence for a different release.

This ordering permits the first deployment without claiming live evidence
before deployment or bypassing a gate. Changes invalidate affected evidence
and repeat the required checks. A deployable candidate is not a completed or
production-accepted Foundation portal.

The producer release binds its stage, source revision, model digest, complete
stage service and dependency closure, controls, assessments, browser artifacts,
and reproducible-build evidence. `foundry-web` independently verifies that
binding and the authorized target before publication. Its delivery checks
cannot replace producer acceptance. A signed SDK, a draft preview, or a valid
artifact digest does not establish the core or complete product.

Publication promotes unchanged artifacts authorized for that release state;
the publisher must reject missing, substituted, stale, stage-incomplete, or
state-inappropriate evidence. Target
changes requiring different generated assets return to `uor-foundry` for
modeling and acceptance. Live verification checks the actual deployment
identity, approved URL, artifact bytes, and complete stage stakeholder journeys.

Development and verification use repository devcontainers; consumer acceptance
uses the immutable SDK lock. Preserve template policy, keep build/cache output
untracked, and make atomic Conventional Commits. Push reviewed increments to
`main` when permitted; use upstream PRs where required and inspect their CI.
