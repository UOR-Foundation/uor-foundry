# Repository infrastructure verification

This repository's `just vv` applies the universal verification policy in
`VERIFICATION.md` to its current scaffold, not the complete Foundry application.

| `just` recipe | Enforces | ID classes |
| --- | --- | --- |
| `just fmt-check` | the diff is reviewable | --- |
| `just model` | R1, R4, R5 | `CM-01` |
| `just lint` | clippy at `-D warnings` | --- |
| `just test` | the workspace suite | --- |
| `just features` | every optional feature compiles, with its tests | --- |
| `just bdd` | R3 and R2's behavioral half | `CM-02`, `CM-03` |
| `just deny` | R6 over the dependency graph | --- |
| `just template-check` | immutable SDK selection, template drift, and bootstrap least privilege | --- |

The register is empty, so its anti-vacuity check is armed rather than claiming
that features exist. It becomes an error as soon as an ID, scenario, or test is
added without the other two.

## Planted defects

| Gate | Planted defect | Result |
| --- | --- | --- |
| `check-model` | `CONFORMANCE.md` disagrees with the register | rejected |
| `audit-deferral` | a deferral marker in a crate and in the gate's own source | both rejected |
| honesty meta-gate | an ID with no test | armed by the empty register |
| `audit-bootstrap` | a floating/copied action, mutable SDK tag, changed universal file, narrowed boundary, shallow/credential-retaining checkout, copied project renderer, omitted standards lock, acceptance bypass, deploy-job signing privilege, or incomplete release lifecycle | all rejected |

`audit-deferral` reads every crate and `xtask`, including itself. Its token
construction therefore cannot exempt the very gate in which a deferral could
otherwise be hidden.

## Reviewed dependency-maintenance policy

The Dependabot configuration and native audit files are byte-identical to
[template `1bea460`](https://github.com/UOR-Foundation/template/commit/1bea460bac6ea50bae53a7eeb674589d7900e6cb).
SDK-managed Action and byte-bound bootstrap updates retain their reviewed
release flow; ordinary Actions and Cargo updates remain weekly.
Universal policy, SDK/template locks, model and complete `just vv` are unchanged.

Native AMD64 `just vv` passed in the locked SDK: 22 Rust tests, two Node tests,
formatting, template/model/inventory checks, Clippy, all-feature compilation,
BDD and dependency checks. The owning tests reject missing/wrong/broad
exclusions, disabled maintenance and duplicate keys. Pre-commit log SHA-256:
`278fe48b16614d2f027f585fbeb0460196e7dabec802304087e2e60c02070ce1`.
This is scaffold evidence, not application or production acceptance.

Action references are independently pinned. The current audit and runtime do
not compare the fetched Action tree with the SDK's `action` inventory digest.
Ownership exclusions do not close that execution-binding gap; an independent
Action commit need not equal the SDK build commit.
