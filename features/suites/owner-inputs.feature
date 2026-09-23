Feature: Owner-controlled acceptance inputs

  @OI-01 @build
  Scenario: Validate approved owner acceptance records across all required scopes
    Given approved owner acceptance records for the platform organization
    When the model cross-checks identity, sites, admin keys, quorums, and recovery rules
    Then organization legal entity and site assessment records are bound to cryptographic digests
    And authenticated administrator keys cover all affected scopes with required quorums
    And adopted standards and assessment authorities match lawful acquisition records
    And business, operational, and publication approvals satisfy governance policy
    And availability, workload, fault bounds, RPO, RTO, and replica obligations are enforced
    And incomplete, unapproved, or single-owner bypass configurations are rejected

