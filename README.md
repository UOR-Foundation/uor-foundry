# UOR Foundry

The UOR Foundation is dedicated to the democratization of technology for the
well-being of humanity. It operates under the Citizen Gardens model through
a network of Foundries; the first Foundry also houses Foundation HQ.

This repository owns the complete PrismPM model: the organization, sites,
services, workflows, stakeholder roles, and portal Views. PrismPM and
prism-stdlib generate and validate its artifacts against adopted standards.
The required product contract is [SPEC.md](SPEC.md).

[foundry-web](https://github.com/UOR-Foundation/foundry-web) publishes the exact
verified and authorized portal release, then verifies its live deployment.
It does not maintain another implementation of the
Foundation or its services. Generic SDK capabilities belong upstream.

## Status

Created from [UOR template](https://github.com/UOR-Foundation/template/tree/e0e11ecb1b38e202116d9806887363848629d439).
The complete organizational model, standards bindings, services, and portal
are not implemented or accepted. Requirements are not implementation evidence.

`src/Foundry.lex.tex` is an unaccepted local draft-preview increment. Its
native, Holo, Core-Wasm, browser, and reproducibility gates must pass before
that capability is claimed. It is not a deployable substitute for the full
portal. Remaining work and evidence are in [IMPLEMENTATION.md](IMPLEMENTATION.md).

PrismPM is consumed as an SDK, following
[Calculator](https://github.com/UOR-Foundation/calculator-example).

The draft SDK/template binding selects an authenticated development candidate,
not a production-accepted release. Complete `just vv` and production dependency
qualification remain required before merge or publication.
Repository policy is in [AGENTS.md](AGENTS.md) and
[TEMPLATE-CONTRACT.md](TEMPLATE-CONTRACT.md).
