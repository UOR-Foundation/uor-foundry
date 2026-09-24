Feature: Organization and sites lifecycle governance
  
  @OS-01 @build
  Scenario: Enforce site lifecycle, activation quorums, physical and accessibility assessments, and cross-organization isolation
    Given configured organization sites and facility assessment records
    When site lifecycle transitions and activation quorums are evaluated
    Then site activation strictly requires active parent organization status and distinct-user quorums
    And activation enforces conforming physical security and accessibility assessments
    And UOR Foundation HQ, First Foundry, and Citizen Gardens records satisfy identical rules without seeded privileges
    And cross-organization site isolation prevents unauthorized foreign administrative operations
    And identical site names across distinct organizations remain isolated without authority collision
