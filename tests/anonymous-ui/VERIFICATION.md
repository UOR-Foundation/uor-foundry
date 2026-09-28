# Navigation component verification

Conditional source evidence, 28 September 2026; not application or deployment
acceptance. The consumer lock and complete `just vv` gate are unchanged.

- Exact root closure: 33 modules, 549 kernel-checked declarations.
- Kernel attestation: `06202dba983a70cee461f0f50aa5dafccce1d454fe7e0b384cb07349865d37fe`.
- IR: `90956429e07f1d773c2de7c181394290463417b0e7cfa77068a970f88e7d04e2`.
- All 9,664 independent vectors pass twice in native `std`, `no_std` and Wasm;
  maximum observed Wasm memory is 327,680 bytes.
- Six fresh source defects produce the intended wrong result in all three
  modes: off-screen recovery, missing navigation focus, unsolicited restoration
  focus, stale intent, request overflow and duplicate recovery landmark names.
  Compilation errors and traps do not count as detection.
- Ten registered construction/linkage tests pass. Normal model generation and
  readback preserve all 22 IDs and leave `CONFORMANCE.md` unchanged.

Evidence is retained in `foundry-anonymous-ui-owner`:
`/tmp/foundry-navigation-source-2`, `/tmp/foundry-navigation-runtime-3` and
`/tmp/foundry-navigation-mutants-QEkSwz`. Mutation receipt SHA-256:
`44e5e5912a892d1b364e12d9a4ad9aa4c35dd75f9b6d1c511b8675827a0fb648`.
The exact compiler `a25445458084f6ca564af287591e5997f20f4f3db9a0c79668c304b38351e72c`
includes reviewed, unmerged upstream corrections; it is not the accepted SDK.

Actual rendered checking caught duplicate form names; the model now distinguishes
account navigation and recovery. After the shared SDK system-palette correction,
Chromium, Firefox and WebKit pass all five screens at 1,280/320 CSS-pixel widths
with normal, forced-light and forced-dark palettes: 90 observations, no axe
violations or incomplete results. Keyboard navigation, transient heading focus,
restoration/idempotence and stale-outcome refusal pass. All 42 actual generated
browser calls replay twice in both native modes; corrupting an observed output
fails their exact equality checks.

Browser receipt: `3880765c3b34930738e66866c764503931fb5f0682def1b22f3e2a191113de46`.
Native replay receipt: `bc6dae9b1acbd79e5eaa9e3924d43473fdf043ea36ffcdb027b72630217675c1`,
retained at `/tmp/foundry-navigation-browser-replay-fevkoU` in the same container.
Exact input copies, browser observations and screenshots remain under ignored
`target/browser-evidence/recovery-labels/`; prior failing observations remain
separate. The unaccepted SDK correction uses style SHA-256
`e4ed19b1572cd0ab8f025ecb40834629895ff79db384acf68d1e542c2e7f72d6`.
This is component-level automated evidence, not full WCAG or user assessment.

Full `just vv` exits 2 during locked fetch with `PP1101` because the selected SDK
does not provide the required reviewed standards catalog. Identity, mailbox proof,
recovery, authenticated shared state and complete service journeys remain
unaccepted; navigation never enables or fabricates them.
