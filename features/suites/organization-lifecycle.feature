Feature: Organization lifecycle, activation, and scoped authority
  As an enrolled user of the platform
  I want open organization creation, provisional setup, and policy-compliant activation
  So that multi-administrator governance is enforced without preseeded authority or privileged names

  @OL-01 @build
  Scenario: Enforce open provisional creation, policy-compliant activation, and isolation
    Given open enrollment and open organization creation on the platform
    When any enrolled user creates an organization with any display name
    Then the organization is established in provisional lifecycle state with a sole founding grant
    And display names confer no platform privileges or reserved authority
    And activation requires at least two distinct authenticated administrators covering all required scopes
    And activation rejects single-owner bypass and duplicate key or mailbox identity disguises
    And retiring a founding grant requires full coverage by remaining distinct administrators
    And cross-organization operations are strictly isolated across all organizational boundaries
