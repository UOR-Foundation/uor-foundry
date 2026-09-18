# Foundry platform requirements

This is the required product contract, not evidence of implementation.
Application semantics must be authored in Prism/LexLean; generated outputs
and this requirements document cannot substitute for the accepted model.

## Purpose and ownership

Foundry is a generic organization-management platform defined with PrismPM.
It starts without seeded accounts, organizations, administrator mailboxes or
memberships. Organizations are modeled instances created through ordinary
Foundry workflows; physical sites are parts of those organizations, not
separate platform implementations.

`uor-foundry` owns the complete platform model: services, organization lifecycle,
workflows, stakeholder roles, permissions, Views and acceptance. It uses the
PrismPM SDK and prism-stdlib to generate and verify the portal and its complete
runtime closure. Each organization owns its authorized records and policies.

The UOR Foundation is one organization created through those same workflows.
Its mission is the democratization of technology for the well-being of humanity;
it operates under the Citizen Gardens model through physical Foundries, the
first also housing Foundation HQ. These are organization-specific requirements,
not seeded data or special privileges. No location, legal structure, financial
policy or additional authority is inferred from the mission.

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

The portal is the authoritative interaction point for each organization's
modeled stakeholders, including UOR faculty and participants. It must support
the complete organizational digital presence and operations, including:

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
five capabilities below; full platform implementation need not be published
at once. This is not authorization to publish a mock or draft preview.

- Identity: authenticate a workspace identity and its actions across sessions.
- Roles: enforce modeled membership, permissions, delegation and revocation.
- Shared workspaces: independent participants access and change shared state
  under those permissions, not separate local copies presented as collaboration.
- Persistence: retain and recover authorized workspace state across reloads
  and restarts; distinguish local writes from confirmed replication.
- Messaging: deliver and retain authorized messages between workspace members,
  with explicit delivery, failure and recovery states.

Normal account and organization creation are required core journeys, including
creating UOR through the same workflow as any other organization. Creation is
an authenticated operation under a modeled creation policy. It establishes only
the new organization's provisional scope and explicit creator grants, never
authority over another organization or the platform. Names, domains, mailbox
strings, repository ownership and self-selected roles confer no privileges.

Provisional setup is distinct from policy-compliant activation. It may collect
authorized records and enroll administrators, but cannot claim accepted
ownership redundancy or perform operations whose approval/coverage conditions
are unmet. Activation requires the modeled policy, distinct-user approvals and
complete administrative coverage of that organization's parts. Every transition
must preserve isolation of data, queries, keys, grants and effects across
organizations, including when one user belongs to several organizations.

Workspace authentication and organization creation do not verify civil identity,
legal-entity identity, employment, faculty appointment or real-world authority.
These claims require separately authenticated organizational records and
assessments. Role admission, including UOR faculty membership, requires recorded
approval by the organization's currently authorized administrators, enforced at
every applicable boundary rather than only in the View.

Every user must have the option of verified email login and account recovery.
The Foundry implementation of these flows must be PrismPM-defined and execute
in browsers; a hosted platform or organization authentication/recovery backend
is not authorized.
Recovery restores only that user's currently authorized access; it cannot
resurrect revoked grants, bypass an approval quorum, or grant new authority.
Verification binds the mailbox, account, operation, challenge, expiry and
replacement key/session. Changing a recovery address requires authenticated
authorization and proof of the new mailbox. External verification dependencies
still require disclosure and approval; no provider is selected by this contract.
Entering an address, a self-signed key or mail-routing records is not mailbox
proof. Expired, replayed, wrong-mailbox and substituted-key challenges must fail
without granting authority; public assets contain no verification secrets.

Administration is delegated over explicit parts of any modeled system, including
each organization and its sites. No permanent all-powerful user is required.
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
A sole founding administrator is an explicit provisional bootstrap condition,
not accepted redundancy. Activation must satisfy the same retained-ownership
requirements; it cannot carry this exception into active operation. A founding
grant may be retired only after other users collectively cover every affected
part under these rules. Retirement does not require one successor to control
every part, and gives no authority over another organization's bootstrap.

Evaluate approval and post-change coverage atomically against the current model
and authority revision, including concurrent changes and newly added parts.
Reject stale, replayed, out-of-scope or insufficient approvals and uncovered
parts; email recovery and direct protocol requests obey the same policy.
Acceptance must exercise partial and complete bootstrap handover, scoped
multi-party grants, duplicate approvers, concurrent removals, revoked-account
recovery, lost access and inherited-scope changes without an implicit root bypass.
It must also exercise empty-platform onboarding, normal UOR creation, denied
activation, organization-name impersonation and cross-organization raw requests.

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

Platform and organization facets are governed through OSCAL catalogs, resolved
profiles, component/system implementation records and assessment evidence. Every applicable
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
Platform acceptance does not establish any organization's legal identity or
compliance. Organization-specific standards, policies and assessments bind that
organization and revision; names or membership cannot transfer those claims.

The model includes human-centred design, complete accessible user journeys,
brand rules, business planning, and operating procedures. Framework and
design-system bindings must preserve the adopted web standards. Brand assets,
descriptions, forecasts, and procedures are versioned and authorized; their
publication and actual outcomes are separate evidence.

## Browser network and bootstrap

Deployed service execution and peer replication use participating user browser
sessions only, including UOR faculty and participants. Dedicated platform or
organization nodes and hidden service backends are not authorized. Development
and bootstrap release builds use repository
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
dedicated platform or organization nodes, hosted application services,
undisclosed peers, or a claim of browser-network independence from those peers.

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
platform acceptance still requires the complete product scope; core
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
   this state. Authorization names the stage; it cannot imply full platform
   acceptance.
3. **Accepted:** post-deployment identity, bytes, live journeys, operational
   measurements, and every remaining applicable control/assessment pass for
   that exact release. Failed or missing checks prevent final acceptance;
   they do not become waivers or evidence for a different release.

This ordering permits the first deployment without claiming live evidence
before deployment or bypassing a gate. Changes invalidate affected evidence
and repeat the required checks. A deployable candidate is not a completed or
production-accepted organization-management platform.

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
