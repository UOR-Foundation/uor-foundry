Feature: Producer Release reproducible generation, coverage, and pre-publication evidence closure

  @PR-01 @build
  Scenario: Validate reproducible artifact generation, complete service/control/dependency/assessment coverage, exact producer identity, and pre-publication evidence
    Given an immutable producer release specification for the staged-core stage
    When two independent clean artifact build runs are executed and compared
    Then artifact outputs are verified to be bit-for-bit identical with matching tree digests
    And complete coverage across all defined services, OSCAL controls, and conforming assessments is verified
    And draft previews and incomplete core capabilities are rejected from entering deployment authorization
    And only checks requiring actual live deployment are registered as outstanding
    And signed pre-publication evidence binds the exact producer identity, commit hash, and artifact digests
    And deployment authorization transitions only after complete readiness verification
    And final acceptance confirms that all outstanding deployment-dependent checks passed on the target
