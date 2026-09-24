Feature: Implementation closure: complete uor-foundry producer model and acceptance evidence

  @IC-01 @build
  Scenario: Validate complete product requirements closure, zero deferred scope, and release handoff readiness
    Given an authoritative product requirements contract defined in SPEC.md and IMPLEMENTATION.md
    When all fifteen model boundaries are independently evaluated and verified accepted
    Then every remaining-work table row is confirmed implemented and accepted with evidence
    And no deferred or narrowed scope exists across any functional or security domain
    And exact producer release identity, target URL, and artifact tree digest are verified bound
    And the handoff package is validated ready for consumption by foundry-web
