Feature: Authorized first-release functional core: identity, roles, shared workspaces, persistence, and messaging

  @FC-01 @build
  Scenario: Validate end-to-end multi-user identity, roles, shared workspaces, persistence recovery, messaging, and normal UOR creation
    Given an empty Foundry environment without seeded privileged accounts or organizations
    When independent users enroll and create organizations including UOR Foundation through ordinary workflows
    Then workspace identities authenticate and enforce modeled roles and permission bounds
    And shared workspace state is accessible and mutable by authorized participants
    And state mutations are persisted and successfully recovered across simulated restarts
    And authorized messages are delivered between workspace members with explicit delivery states
    And transient transmission failures recover through automatic retransmission
