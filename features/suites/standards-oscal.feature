Feature: Standards and OSCAL governance boundary
  
  @ST-01 @build
  Scenario: Enforce OSCAL catalogs, profile resolution, inheritance, and authenticated assessments
    Given configured OSCAL catalogs for all adopted standards and the mandatory PrismPM base profile
    When the production profile is resolved and system implementation records are evaluated
    Then the mandatory PrismPM base profile is preserved without weakening or unauthorized exclusion
    And every control in the resolved profile is satisfied by verified local components or explicit inheritance
    And inheritance records define provider scope, exact revisions, evidence digests, and consumer responsibilities
    And authenticated assessments are verified against authorized assessment bodies and cryptographic keys
    And all assessed controls confirm a conforming verdict with valid evidence findings
    And cross-organization assessment isolation is enforced across organizational boundaries
