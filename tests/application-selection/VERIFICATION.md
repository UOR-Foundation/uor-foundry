# Selection component verification

Conditional source evidence, 28 September 2026; not installed SDK, authenticated
application or deployment acceptance. Public dispatch and consumer locks are unchanged.

- Nine modules, 179 kernel-checked declarations; source SHA-256
  `f42889eb84fa97718148222e6ca52ba53317f8f446ca0a1070ee4f803c69451e`.
- Attestation: `19ef2a83edd24535054fd844890152b1d838532238b482b0d8c14ebecb55475a`.
- IR: `4db97ff2ae4e3d29adc0ed1ff11e42b02693f4bb604a9a55ec682fc74ebcf4a2`.
- All 13,000 independent exact-byte vectors pass twice in native `std`,
  `no_std` and import-free Wasm. Peak Wasm memory is 4,521,984 bytes, below
  the unchanged 64 MiB limit. Maximal valid frames and every truncation are included.
- All 13 fresh, kernel-valid source mutants fail their exact native and Wasm
  expectations. Compiler failures, traps and historical run unions do not count.
- Four registered construction checks pass; normal model write/readback retains
  all 23 IDs. Registration alone is not the complete generated behavioral owner.

Evidence is retained in `foundry-selection-owner` at
`/tmp/foundry-selection-runtime-4` and `/tmp/foundry-selection-mutants-3`.
Their result SHA-256 values are respectively
`0b90dbc5c19ae24a6c51370f858aa3e8ecd5b0210de81bd20d6ffa8f2ce225aa` and
`02d3f833c0f66aa75cc21ab077d0cb558969d156d2c049e17749125488e31000`.
Source, transitive fixture, generated package, executable and tool hashes are
checked before/after execution. The compiler
`a25445458084f6ca564af287591e5997f20f4f3db9a0c79668c304b38351e72c`
contains reviewed unmerged corrections; it does not replace the selected SDK.
The earlier noncanonical mutant-fixture failures and interrupted incomplete
fixture-capture run remain separate diagnostics, not accepted negatives.

The complete unchanged `just vv` exits 2 during locked fetch with `PP1101`.
Installed generated-owner integration, independently admitted lookup/permission,
enrollment, authenticated session, durable journal/currentness and public
dispatch/replay remain required. Selecting data confers no authority or persistence.
