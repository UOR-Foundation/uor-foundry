# Application context verification

`AC-01` is implemented as a private model component. It is not accepted dispatch,
authentication, service execution or deployment. The registered source/fixture
tests do not substitute for the required generated SDK behavioral owner.

## Conditional evidence — 28 September 2026

- Initial source registration failed with missing `Reference.lex.tex` before
  implementation; the five source/fixture registration tests now pass.
- Exact eight-module closure: 161 declarations pass source/kernel verification,
  attestation `6127fa428a4a4d41c370a74a36712a00682219cf9741551a153c00f2a62b44c1`.
- All 6,621 independently authored vectors execute twice in native std/no_std
  and four import-free Wasm roots. Maximum observed Wasm memory is 1,769,472
  bytes under the unchanged 64 MiB diagnostic ceiling.
- Exact valid state/command/context inputs are 1,028/1,436/2,466 bytes at their
  structural maxima; 16,384 and 16,385 test malformed frame-budget boundaries.
- Nine kernel-valid source mutants fail actual native std/no_std and Wasm
  equality: account algorithm, hex conversion, workspace hierarchy, account
  namespace, workspace revision, application revision, exhausted revision,
  missing account selection and mailbox byte budget.
- Model readback, formatting, registered conformance test and diff checks pass.
  `just vv` remains RED at `prismpm fetch --locked`: `PP1101`, missing or changed
  `standards.lock`. No consumer lock or SDK selection was changed.

The selected SDK compiler also rejects the valid `prefix` binder (`LLV7002`).
The accepted-main code generator rejects this closure at its existing join
expansion ceiling. Those failures remain retained. Positive diagnostics use the
exact composition `431cea8` LexLean compiler and separately reviewed upstream
compiler corrections, including [PR 79](https://github.com/auser/lean4-prod/pull/79)
and [PR 83](https://github.com/auser/lean4-prod/pull/83), not an installed SDK.

Evidence is retained under `target/application-context-diagnostic.RmbGQlVA/`;
actual execution captures also reside in the pinned `uor-foundry-model-audit`
container at `/tmp/foundry-application-context-runtime-4` and
`/tmp/foundry-application-context-mutants-1`. Captures retain source, compiler
identities, kernel attestations, generated Lean/IR/packages, immutable execution
Wasm, independent corpus bytes and complete native/mutation reports.

## Required completion boundary

Install the verified immutable SDK closure, then run an unconditional owning
source/kernel/native std/no_std/Wasm gate for this entire corpus and all nine
source mutations. The source registration test alone cannot satisfy AC-01
behavioral acceptance. Host paths, retained compiler binaries, changed pins,
or a conditional diagnostic must never become a consumer CI fallback.

Source admission, current authenticated account/organization state, real
mailbox/credential/recovery proofs, SDK lifecycle extensions, session/journal
composition, all functional-core services and deployment verification remain
required. None is implied by a successful context comparison or codec.
