# Implementation status

The required scope is [SPEC.md](SPEC.md). No application capability is
accepted. The template scaffold is not a running Foundry service.

| Required work | Owner | Status |
| --- | --- | --- |
| Accepted public compiler/runtime dependency closure and Prism SDK | Upstream repositories | Blocked: package gate cannot resolve `uor-hologram`; no accepted SDK lock |
| SDK/template locks, devcontainer, and full acceptance CI | template, uor-foundry, foundry-web | Blocked on accepted SDK |
| Complete OSCAL bindings, profile resolution, inheritance, assessments, and oracles | PrismPM, prism-stdlib | Required |
| Complete browser-resident SDK, compilers, oracles, and provider-independent signing | PrismPM and compiler/runtime repositories | Required |
| Foundation, Foundry, stakeholder, authority, and organizational models | uor-foundry | Required |
| Approved standards editions, policies, rights, and assessment inputs | Foundation/model owners | Required; mission alone does not supply these |
| Human-centred design, accessible authoring and user journeys, brands, and presentation | prism-stdlib, uor-foundry | Required |
| Browser Kappa services, authorization, durable events, queries, and provenance | kappa-registry, PrismPM | Required |
| Faculty/participant browser networking, discovery, peer replication, recovery, and measured availability | Kappa/runtime models, uor-foundry | Required |
| Exact-release Pages bootstrap and authorized independence/rollback lifecycle | PrismPM, foundry-web | Required |
| Git, CI, hosting, scheduling, Prism pipeline, and publication adapters | PrismPM, uor-foundry | Required |
| AI inference, agent harnesses, notebooks, and knowledge management | PrismPM, uor-foundry | Required |
| Text/multimedia messaging, collaboration, and content creation | PrismPM, uor-foundry | Required |
| Administration, governance, change management, and improvement | uor-foundry | Required |
| Business plan, operating procedures, finances, and payments | uor-foundry | Required |
| Learning, assessment, certification, and authority lifecycle | uor-foundry | Required |
| Complete oracle coverage, mutation evidence, browser journeys, fault/recovery and live deployment verification | All implementing repositories | Required |

Each implementation increment must add its modeled capability, behavioral
scenario, failing test, complete implementation, and verification evidence.
Only then may its conformance register claim the evidenced capability.
No row is a deferral, exclusion, or reduced definition of done.

## Draft-preview increment

`src/Foundry.lex.tex` defines bounded UTF-8 draft preview through the SDK's
text-application profile. It has no persistence, publication, identity, or
network service and does not satisfy the complete portal contract.

`FW-01` requires ten native/Holo/Core-Wasm vectors, five Chromium journeys,
two clean-root artifact reproductions, and the complete inherited gate.
Portable View acceptance also requires browser transport through the actual
Hologram session; the SDK's recording surface does not establish that path.
Actual application execution and rejection of a planted behavioral defect are
not yet evidenced.
Seven verification-helper tests pass in the immutable local SDK. Removing the
storage symlink guard from an isolated copy makes its test fail. Removing the
portable-evidence comparison likewise fails with `Missing expected exception`
for a substituted browser profile; restored helpers pass all seven tests.
Manifest/model/acceptance hashes, the actual successful
oracle process, and every portable browser case are now required. These are
component checks, not application acceptance.
The model and 19 supporting source/configuration/test files were relocated
from `foundry-web` without byte changes. Their capability register and complete
application gate moved with them. No application implementation remains in the
publisher. This relocation does not establish application acceptance.

Both repositories now have generated development-candidate bindings. The
publisher's binding remains in its draft PR #3; neither repository has a
production-accepted SDK. Independent signature/provenance checks and actual
never-started AMD64/ARM64 image captures agree for candidate index
`sha256:60226bc791d4c0e5613402a6be7e63f4963d3faf7f327befcf56fc0e41d0ce21`,
from PrismPM `d0174e1d64339f73091fe4c59d5d6bf532a37d1f` and template policy
`0f1245367d317d439e6752be277047eef76a9e90`. This SDK predates the required
portable-browser oracle; it cannot satisfy `FW-01` acceptance.

The reviewed `0f124536` template update preserves model and language-lock
bytes and separates private Buildx state from read-only credentials. Normal
template/SDK lock checks, native bootstrap checks, 12 xtask and two Node tests,
formatting, and Clippy passed in the exact native SDK. Log SHA-256:
`b2dffcef8d86cb92b457f6d4c284c018f126b3b3289c76270f6b23e362aee081`.
These component checks do not satisfy the application or full repository gate.

### Development SDK model/build evidence

On 16 September 2026, the exact candidate above, initially bound to template
`a21a5426c290aeac92df1c0b1c63d9701420ae68`, ran as UID 1000 with the project
mounted at its real host path. Normal `just model-write`,
`lake update`, and `lexlean --project lexlean.toml lock` generated conformance
and language locks. The relocated model's package repository was explicitly
corrected to `https://github.com/UOR-Foundation/uor-foundry`; its crate identity
remains `prism-foundry-web`. Its semantic JSON object keys were canonicalized
after the genuine SDK reported `LLT4001`; declaration order, all ten vectors,
and their bytes were preserved. The producer gate checks the repository URL.

`prismpm check`, template/SDK lock checks, a fresh-target native
`xtask check-model`, formatter checking, and all seven evidence-helper tests
passed. The checked source SHA-256 is
`dc631ca5a0dd8f85705d01d612fcdfea332e6493cb1fcab402f326f194867666`.
The model ID is
`7379c7314f6e635202e221f787737531eefe1ac28cbbf034d04e490e769e6d41`.

`prismpm --json fetch --locked` completed: 37 authority inputs reused,
Cargo and npm fetches successful, and both actual indexed image subjects
scanned offline. AMD64/ARM64 package counts were 3,552/3,547, with 629 retained
findings on each and zero rejected by the locked policy; this is not a claim
of zero vulnerabilities. The pinned crates.io and Ubuntu OSV snapshot URLs
returned HTTP 404; all five locked OSV snapshots were recovered as exact
SHA-256-verified bytes from the preserved local PrismPM content-addressed
cache without changing a pin or skipping an oracle.
Fresh-client retrieval of those locked inputs still requires an upstream
distribution correction; preserved local cache bytes do not prove that path.
[Hosted bootstrap 35107899416](https://github.com/UOR-Foundation/uor-foundry/actions/runs/35107899416)
confirmed this at `75cd64f3d208d9eaec1db0e71918e505af238caf`: both AMD64 and
ARM64 passed Buildx, native bootstrap, and template/SDK lock checks, then
failed `PP5401` fetching the pinned crates.io OSV archive after four HTTP 404
responses. No source or application acceptance gate was reached.
Independent follow-up found all five locked OSV generation URLs unavailable.
Their retained bytes match the lock, but PrismPM records them as
`citation-only`. Durable SDK delivery requires an evidence-backed rights and
attribution review, digest-bound distribution of the complete acquired-input
closure, and fresh-cache acquisition tests on both architectures. Replacing
the archived facts with current downloads is a different authority update.

Both the unchanged application and an isolated, normally locked mutation
changing only the dispatch bound from 4,096 to 4,095 failed before native
execution: `PP5001`, generated `PrismFoundry/Foundry.lean:11:21`, deterministic
`transform` timeout at Lean's default `maxHeartbeats` of 200,000. That line
contains the complete generated application declaration and its unchanged
vectors. This is a baseline compiler-resource failure, not evidence that the
intended behavioral defect was caught. No generated Lean was edited, no
vector was removed, and no runtime/browser result was accepted. Upstream
correction and a newly bound SDK must precede those remaining checks.

Raw diagnostics and results are retained under ignored `target/` as
`sdk-canonical-model-check.log`, `sdk-final-helper-checks.log`,
`sdk-preserved-osv-cache.log`, `sdk-fetch-and-build.log`,
`sdk-unmodified-model-build.log`, and `sdk-mutated-model-build-verify.log`.

## Kappa/Veilid baseline

Source inspection, not execution or accepted conformance:

- On 16 September 2026, remote `uor-foundry` remained
  [`c226fdee`](https://github.com/UOR-Foundation/uor-foundry/tree/c226fdee098cb82841d7562f2cb7f44be4420869):
  then bootstrap-only, without an organizational model or SDK lock. Kappa's remote
  head still matched the revision below; it has no browser/Prism service package.
- Kappa [2af86560](https://github.com/UOR-Foundation/kappa-registry/tree/2af86560a177fc9651b6c0e92e7974140ed77dd5)
  locks Rekindle `3fb5b80f2d5d3d5b5a59dded1a56b477cb9f23ca` and Veilid 0.5.7.
  Kappa owns object identity, storage, references, queries, and authorization;
  Rekindle adapts Veilid routing, DHT records, and application messages.
- The optional [Kappa startup](https://github.com/UOR-Foundation/kappa-registry/blob/2af86560a177fc9651b6c0e92e7974140ed77dd5/crates/kappa-server/src/main.rs#L748)
  discards the inbound channel. Its [reconciliation loop](https://github.com/UOR-Foundation/kappa-registry/blob/2af86560a177fc9651b6c0e92e7974140ed77dd5/crates/kappa-transport-veilid/src/reconcile.rs#L146)
  copies tags, not referenced blob bytes. These paths do not establish
  authenticated end-to-end replication, conflict handling, or recovery.
- Veilid 0.5.7 has a [browser transport](https://gitlab.com/veilid/veilid/-/blob/f5cdcca38cecf4845eb9bd5e21ddca382a357a75/veilid-core/src/network_manager/network/wasm/protocol/mod.rs)
  using outbound WS/WSS, without inbound listeners or implemented WebRTC.
  Reachable transport peers are therefore a dependency of that path, distinct
  from application hosting. GitHub Pages bootstrap does not supply them.

Required modeled work includes authenticated peer/content discovery, inbound
dispatch, verified blob transfer, durable replica receipts, conflict/revocation
rules, repair, and browser-native storage/execution. Test the complete path
across networks and peer failures. Inventory relay/bootstrap dependencies;
faculty/participant browser-only service execution and peer replication remain
required. Hologram's WebRTC path is not evidence about Veilid's capabilities.

## Holospaces threat-model inputs

Reuse the [threat model](https://github.com/Hologram-Technologies/hologram/blob/96769f16be454ab1572fddff4613704ccfbebf5e/specs/holospaces/src/arc42/adoc/13_product_security.adoc)
and its witnesses alongside the [network design](https://github.com/Hologram-Technologies/hologram/blob/96769f16be454ab1572fddff4613704ccfbebf5e/specs/refactor/04-networks.md).
The latter distinguishes restricted access from private encrypted content and
identifies open public-network abuse economics and durability/replication policy.
Operator-owned storage and NIC-capable egress peers in the former are not
automatically valid assumptions for Foundry's browser-session deployment.

At that revision, the Hologram devcontainer passed:

- `bash vv/suites/cc40-product-security.sh`: eight tests, none ignored.
- `bash vv/suites/cc38-content-net.sh`: four in-process exchange/tampering
  tests, none ignored; bare-metal and browser-target compatibility builds,
  not cross-network browser deployment tests.

These witness the tested properties, not large-scale availability, hostile
storage confidentiality, or resistance to a compromised initial verifier.
CC-40's public-key identity and roster checks do not demonstrate private-key
possession, authenticated sessions, or Foundation role admission.
Foundry must bind inherited claims to their exact evidence and validate
changed assumptions explicitly.

Additional inspected witnesses, not rerun here, include real loopback
[TCP/DHT exchanges](https://github.com/Hologram-Technologies/hologram/blob/96769f16be454ab1572fddff4613704ccfbebf5e/crates/hologram-net/tests/uor_native_dht.rs)
and [CC-49 browser WebRTC](https://github.com/Hologram-Technologies/hologram/blob/96769f16be454ab1572fddff4613704ccfbebf5e/spaces/holospaces-browser/web/webrtc-content-net-test.mjs).
CC-49 uses two Chromium contexts, host ICE, and harness-carried signaling;
it does not establish cross-NAT or geographically distributed operation.
The [DHT implementation](https://github.com/Hologram-Technologies/hologram/blob/96769f16be454ab1572fddff4613704ccfbebf5e/crates/hologram-net/src/tcp/dht.rs#L82-L104)
retains incumbent entries without liveness probes. Neither these tests nor
cache-on-fetch establish churn/eclipsing resistance or durable replica policy.
Foundry acceptance still requires adversarial churn/partition, browser eviction,
and last-replica-loss evidence against its modeled recovery obligations.

## Verification of the bootstrap

- On 16 September 2026, the owner requested
  `https://uor.foundation/foundry-web/`. The Pages API assigns `uor.foundation`
  to the `website` project, while `foundry-web` has Actions deployment, no
  repository custom domain, and the default `uor-foundation.github.io` address.
  Both Foundry URLs return HTTP 404. Confirm the target; do not implicitly
  migrate the existing website. Generated assets and live verification must
  support `/foundry-web/`; `app.uor.foundation` DNS is not a prerequisite.
- Whitespace checks passed; inherited agent policy, bootstrap workflow, and
  generated conformance document remain byte-identical to the template.
- [CI at a006253](https://github.com/UOR-Foundation/foundry-web/actions/runs/34928559731)
  failed on both architectures because `prismpm.lock` is absent. No gate was
  disabled and no SDK or application acceptance was claimed.
- The PrismPM source devcontainer's `cargo xtask package-api` fails to resolve
  `uor-hologram`. The last real upstream
  [publication attempt](https://github.com/Hologram-Technologies/hologram/actions/runs/34018837931)
  failed with a publishing-permission error; later successful dry runs do not
  demonstrate publication. The upstream owner must establish authorized
  publication before the SDK dependency closure can be accepted.
- Full `just vv` at PrismPM
  [`d0174e1`](https://github.com/UOR-Foundation/PrismPM/commit/d0174e1d64339f73091fe4c59d5d6bf532a37d1f)
  passed gates 1–14, including 319 workspace tests and two-root reproduction,
  then failed gate 15 on that missing public package. No passing release receipt
  was produced; portable-View browser execution remains additional required work.
