Feature: Saved backup-code recovery lifecycle

  @BC-01 @build
  Scenario: Enforce NIST SP 800-63B-4 backup-code issuance, revision binding, session invalidation, and replay rejection
    Given a configured backup-code recovery protocol compliant with NIST SP 800-63B-4
    When backup code batches are issued and redeemed for account recovery
    Then backup codes are stored as salted cryptographic digests with bounded entropy
    And redemption strictly enforces account revision binding and rejects revision rollback
    And redeeming a single-use backup code invalidates all preexisting active sessions
    And replaying a previously redeemed backup code is rejected
    And reissuing a batch deactivates and revokes all prior unredeemed codes
    And backup code recovery refuses to restore revoked authority grants or create unauthorized scopes
    And mutual non-substitution is enforced between email challenges and backup codes
