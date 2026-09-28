Feature: Source-owned application selection transition

  @SC-01 @build
  Scenario: Select exact looked-up data without granting authority
    Given the application has an exact current account and scope selection
    When a checked command consumes its complete captured lookup observation
    Then account namespace and organization workspace parent bindings are preserved
    And changing account or organization clears every subordinate selection
    And read-only and cancelled results retain the exact prior state and revision
    And missing denied unavailable stale and exhausted operations cannot replace state
    And lifecycle eligibility never substitutes for authenticated current permission
    And canonical bounded codecs reject malformed truncated trailing and oversized input
    And planted source defects fail actual generated native and Wasm execution
