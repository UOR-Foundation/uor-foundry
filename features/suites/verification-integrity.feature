Feature: Producer verification integrity

  @VI-01 @build
  Scenario: Reject missing, stale and echo-only artifacts without accepting fixture ingestion as product behavior
    Given the exact source revision and SDK-selected build and verification results
    And the complete gate acquires immutable SDK inputs through locked fetch before product checks
    When browser artifacts are missing, substituted, stale or echo-only
    Then the owning producer gate fails rather than skipping its checks
    And Foundry modules originate in the producer while SDK modules originate only in the locked SDK
    And oracle input integrity alone cannot establish product or accessibility acceptance
