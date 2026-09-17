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

Repository bootstrap only, from [UOR template](https://github.com/UOR-Foundation/template/tree/e0e11ecb1b38e202116d9806887363848629d439).
The authorized first functional release comprises identity, roles, shared
workspaces, persistence, and messaging. Workspace identities and roles do not
assert verified Foundation appointments or organizational authority.
This core is not implemented or accepted yet; a draft preview does not satisfy it.
The complete organizational model, standards bindings, services, and portal
remain required and explicitly unaccepted. Core acceptance will not establish
complete Foundation implementation or standards compliance.
The audited remaining work and owner inputs are in [IMPLEMENTATION.md](IMPLEMENTATION.md).

PrismPM is consumed as an SDK, following
[Calculator](https://github.com/UOR-Foundation/calculator-example).

The SDK/template binding selects an authenticated development candidate.
Reviewed development infrastructure belongs on `main` after the complete
repository gate passes. Main-branch integration does not qualify the SDK or
portal for production release; publication retains its separate acceptance
requirements. Source checkouts and host tools are not substitutes for the
locked SDK.

Application work and its additional acceptance gates remain in
[PR #3](https://github.com/UOR-Foundation/uor-foundry/pull/3), without narrowing
the product requirements in [SPEC.md](SPEC.md). Passing this empty scaffold's
gate is not application or production acceptance.

Repository policy is in [AGENTS.md](AGENTS.md) and
[TEMPLATE-CONTRACT.md](TEMPLATE-CONTRACT.md).
