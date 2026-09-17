Feature: Local content-draft preview

  @FW-01 @build
  Scenario: Preview modeled draft text without publishing or approving it
    Given the immutable Prism SDK and the Foundry LexLean application
    When the generated application receives valid, invalid, empty, and oversized draft input
    Then the generated native and Hologram runtimes agree with every modeled response
    And the portable View executes in Chromium through the authoritative Hologram session
    And byte-bound SDK evidence rejects recording-only and incomplete browser reports
    And the complete generated artifact tree reproduces from two clean absolute roots
    And the browser renders valid text without interpreting markup or sending network effects
    And disabled or failed scripts cannot submit draft content through native HTML forms
    And exact-file Chromium evidence rejects skipped, stale, retried, and wrong-file results
    And the output is identified as an unpublished local draft
