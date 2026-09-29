Feature: Source-owned application data and command boundary

  @AC-01 @build
  Scenario: Decode exact bounded context without granting authority
    Given the modeled state starts with no selected account or organization
    When canonical state, command and reference records are decoded and re-encoded
    Then native and Wasm execution preserve every admitted value and byte
    And wrong tags, widths, UTF-8, arities, truncation and trailing data are rejected
    And canonical account text preserves the exact DK-33 digest and namespace
    And substituted account, organization, workspace, epoch or revision bindings are rejected
    And exhausted revisions cannot admit a mutating command
    And all organization lifecycle states remain distinct without authorizing transitions
    And decoded records never create authenticated authority, effects or seeded privileges
    And planted source defects fail the generated behavioral corpus
