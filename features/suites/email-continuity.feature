Feature: UOR-native verified email identity continuity protocol

  @EC-01 @build
  Scenario: Enforce native browser-based email enrollment, login, and recovery with safety invariants
    Given a configured UOR-native verified email continuity protocol
    When users enroll, log in, or recover credentials using ephemeral cryptographic challenges
    Then enrollment and login require verified proof of mailbox control
    And replaying or reusing consumed challenge nonces is rejected
    And expired challenges exceeding bounded time-to-live are rejected
    And account recovery invalidates all prior active sessions
    And recovery strictly refuses to restore previously revoked authority grants
    And recovery cannot create new unauthorized authority scopes
