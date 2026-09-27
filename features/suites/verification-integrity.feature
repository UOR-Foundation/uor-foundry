Feature: Producer verification integrity

  @VI-01 @build
  Scenario: Reject missing, stale and echo-only artifacts without accepting fixture ingestion as product behavior
    Given the exact source revision and SDK-selected build and verification results
    When browser artifacts are missing, substituted, stale or echo-only
    Then the owning producer gate fails rather than skipping its checks
    And oracle input integrity alone cannot establish product or accessibility acceptance
