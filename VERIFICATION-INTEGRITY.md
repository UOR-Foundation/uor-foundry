# Producer verification integrity

`VI-01` is an integrity gate, not product acceptance. Its owning Rust test invokes
the complete `scripts/verification-integrity.test.mjs` suite in the locked SDK.

Before correction, the browser check returned success for absent build output
and for echo Wasm, calling echoed route names service routing and markup strings
WCAG verification. The oracle-input script also succeeded with all application
sources hidden. These were measured with read-only container mounts.

The correction requires the SDK's current build and verification result, checks
artifact bytes and current source closure, rejects empty/echo-only vectors and
executes non-echo vectors. Generated dispatch, presentation and replay definitions
must reach their required domain modules through actual definition calls;
imports, theorem statements and caller-supplied completion flags do not count.
Static reachability is necessary but does not prove branch behavior.
It does not infer service, design or standards
completion from this smoke check. The full generated/browser application gate
is mandatory and shares its build with this integrity check, avoiding a second
production build just for the smoke check.

Regression scope: missing identities/output, stale source identity, modified or
omitted source, missing/substituted/empty artifacts, duplicate inventory entries,
path escape, symlink substitution, echo-only vectors, execution mismatch and
oracle-input scope disclosure. Synthetic negative fixtures cannot authorize a
release. Real acceptance remains blocked by the implementation gaps in
[IMPLEMENTATION.md](IMPLEMENTATION.md).

The production organization registry is empty and valid. Explicit synthetic
fixtures exercise legacy configuration validators without establishing external
facts or creating accounts. Independent recovery-policy checks still apply to
an empty platform; incomplete product closure remains unaccepted.

## Verification record

Executed in the immutable SDK image pinned by `prismpm.lock`:

- 22 Node integrity/reachability regressions passed through the registered VI-01 owner.
- Five empty-registry regressions, 35 synthetic configuration unit tests and the
  unaccepted-closure refusal passed; these are not product acceptance.
- Formatting, all-target Clippy, model regeneration, template and SDK-lock checks passed.
- Full `just vv` failed at mandatory FW-01: `PP2001` / `LLC0102`, compiler semantics
  differ from the pre-existing feature-branch `lexlean.lock`. That lock was preserved.
- Retained build `bd7a8cd5498a7cf9996ea2a23e2dac6a751434428cbdb1694f2604980a3385b6`
  was independently rejected for echo-only behavior and disconnected generated roots.

No product acceptance, release or deployment was performed.

Follow-up integrity checks reject source/artifact hardlinks and require the
standalone command to check the locked SDK before build/verification. Mock
process replies test ordering and refusal only, never SDK or product acceptance.
All 27 follow-up Node regressions passed in the same exact SDK image.

The source-origin follow-up first failed two new regressions. All 29 now pass,
including an executed removed-guard mutant. Snapshot module names and source
paths must be unique: Foundry declarations come from producer `src/`, while
standard-library declarations come from the exact locked SDK input path.
Matching source/artifact hashes cannot legitimize a consumer SDK copy.
The existing local Browser Application model is such a copy; it remains
unaccepted until a real SDK update permits its removal. This check neither
implements the missing SDK capability nor replaces SDK source authentication.
Independent review found no source-origin bypass. The full `just vv` rerun
passed model/template/lock checks, formatting, audits and all-target Clippy;
VI-01 passed all 29 cases, while mandatory FW-01 again failed with the retained
`PP2001` / `LLC0102` compiler-semantics mismatch. Full acceptance remains RED.
