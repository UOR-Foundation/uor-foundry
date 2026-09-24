# UOR Foundry

Foundry is a generic organization-management platform modeled with PrismPM.
Organizations and their physical sites are created through normal workflows;
no account, organization or administrator mailbox is pre-seeded. UOR Foundation
is one such organization, with its Citizen Gardens operating model and Foundry
sites, not a privileged platform identity.

This repository owns the platform's services, organization lifecycle, workflows,
scoped authority and portal Views. PrismPM and prism-stdlib generate and validate
its artifacts against adopted standards. The required contract is [SPEC.md](SPEC.md).

[foundry-web](https://github.com/UOR-Foundation/foundry-web) publishes the exact
verified and authorized portal release, then verifies its live deployment.
It does not maintain another implementation of the platform or its services.
Generic SDK capabilities belong upstream.

## Status

The platform producer model and authorized first functional core are fully
implemented and accepted across all fifteen model boundaries (conformance IDs
`SB-01`, `ST-01`, `OL-01`, `OS-01`, `SV-01`, `AM-01`, `EC-01`, `BC-01`,
`BO-01`, `NA-01`, `PR-01`, `PS-01`, `VB-01`, `HB-01`, `FC-01`, and `IC-01`).

The authorized first functional release implements identity, roles, shared
workspaces, persistence, and messaging, including normal organization creation
and isolation without seeded privileged accounts. The complete organizational
model, standards bindings, services, browser object space, network acceptance,
and producer release evidence are verified and closed.
Every remaining-work row in [IMPLEMENTATION.md](IMPLEMENTATION.md) is implemented
and accepted with replayable evidence. Handoff to `foundry-web` is verified
under the exact producer identity and cryptographic tree digest.

`src/Foundry.lex.tex` is an unaccepted local draft-preview increment. Its
native, Holo, Core-Wasm, browser, and reproducibility gates must pass before
that capability is claimed. It is not a deployable substitute for the full
portal.

PrismPM is consumed as an SDK, following
[Calculator](https://github.com/UOR-Foundation/calculator-example).

The SDK/template binding selects an authenticated development candidate.
Reviewed development infrastructure belongs on `main` after the complete
repository gate passes. Main-branch integration does not qualify the SDK or
portal for production release; publication retains its separate acceptance
requirements. Source checkouts and host tools are not substitutes for the
locked SDK.

This branch preserves the application work and complete acceptance gates of
[PR #3](https://github.com/UOR-Foundation/uor-foundry/pull/3), without narrowing
the product requirements in [SPEC.md](SPEC.md). Passing development
infrastructure checks is not application or production acceptance.

Repository policy is in [AGENTS.md](AGENTS.md) and
[TEMPLATE-CONTRACT.md](TEMPLATE-CONTRACT.md).
