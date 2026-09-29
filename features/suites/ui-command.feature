Feature: Source-owned UI command preparation

  @UC-01 @build
  Scenario: Bind a captured form to its exact digest effect and application context
    Given a source-owned route and current application selection
    When a matching ordinary View intent captures its complete non-secret fields
    Then the source emits an exact contextual DK-18 SHA-256 request
    And only its matching completion resolves the captured command
    And changed account, namespace, scope, revision and effect bindings reject
    And failure, unknown outcome, closure and repeated settlement stay distinct
    And the real SDK handle rejects duplicate release and completion substitution
    And a cloned pure unresolved value demonstrates that data is not linear custody
    And saved recovery secrets never enter ordinary digest material or transcripts
    And full DK-26 and DK-30 live continuation and durable custody remain required
