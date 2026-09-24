Feature: Publication SDK source-free handoff, atomic export, and rollback closure

  @PS-01 @build
  Scenario: Validate source-free acquisition, readiness verification, target authorization, atomic export, and rollback semantics
    Given an accepted producer release and publication configuration targeting /foundry-web/
    When the publication SDK executes source-free acquisition without compiler or application source dependencies
    Then producer readiness and pre-publication binding digests are strictly verified
    And target deployment decisions require explicit authorization before asset export
    And confined atomic export extracts all required browser assets preserving unchanged bytes
    And byte substitution, truncation, or hash discrepancies trigger immediate rejection
    And live target deployment checks verify asset byte parity on the HTTPS origin
    And verification failure triggers automated atomic rollback to the previous accepted release

