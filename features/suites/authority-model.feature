Feature: Scoped multi-administrator authority model

  @AM-01 @build
  Scenario: Enforce scoped multi-administrator quorums, atomic coverage, and lockout protection
    Given a configured organization authority model requiring distinct users per scope
    When change proposals are submitted to mutate scoped authority grants
    Then authority actions require approval by at least two distinct authenticated administrators
    And single-owner execution shortcuts and bypasses are rejected
    And duplicate public key or email disguises attempting to meet quorum are rejected
    And mutations that drop distinct administrators below quorum fail atomic post-change coverage
    And concurrent proposals against a stale revision fail optimistic concurrency fencing
    And premature retirement of a founding bootstrap grant is rejected without replacement coverage
    And cross-organization authority operations are strictly prohibited
