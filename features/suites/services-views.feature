Feature: Services and Views stakeholder journeys and boundary enforcement
  
  @SV-01 @build
  Scenario: Enforce SPEC stakeholder journeys, resource bounds, failure recovery, raw boundary checks, and view projections
    Given registered services and views for all seven SPEC organizational domains
    When stakeholder journeys and lifecycle operations are executed
    Then workflows enforce execution timeouts, memory bounds, and artifact digest generation
    And AI inference output is strictly a proposal until authorized by workflow and verified
    And messaging and collaboration enforce size bounds and message delivery lifecycle states
    And governance proposals enforce two-admin minimum distinct quorums and revision fencing
    And financial payments strictly require independent settlement oracle receipts prior to settling
    And educational certifications strictly require accredited authority signatures prior to issuance
    And brand kits enforce WCAG 2.2 AA text and UI contrast ratios prior to publication
    And direct raw requests enforce permissions at the boundary independently of views
    And view projections redact private keys and credentials while preserving authorized data

  @SV-01 @build
  Scenario: Project and navigate the anonymous account entry without granting authority
    Given a canonical bounded anonymous selector with no account or organization authority
    When its generated presentation or navigation root executes
    Then each welcome, enrollment, sign-in, email-recovery, and saved-code-recovery screen has the modeled labels, landmarks, input purposes, and design tokens
    And only current-revision Ready navigation with an exact empty-field action binding advances the selector
    And navigation rejects pending, replay-required, closed, stale, malformed, oversized, and exhausted requests without effects
    And projection rejects invalid selectors while preserving valid non-Ready lifecycle states
    And unavailable enrollment, sign-in, and recovery cannot submit or report success
    And generated native and Wasm results agree with the independent complete-frame corpus
    And the imported browser oracle assesses the actual generated semantic presentation
    And anonymous display state cannot establish identity, membership, permissions, or functional-core acceptance
