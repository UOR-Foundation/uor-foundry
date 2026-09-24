Feature: Real participant network acceptance under adverse conditions
  
  @NA-01 @build
  Scenario: Validate real participant operation under adverse network conditions, partitions, suspensions, evictions, replica loss, hostile inputs, and availability targets
    Given configured independent browser participants and bootstrap routes under /foundry-web/
    When network operations and adverse simulations are executed across the mesh
    Then bootstrap routes authenticate disclosed Veilid relays and verify HTTPS-origin paths
    And network partitions reject state mutations in minority partitions and reconcile cleanly upon healing
    And browser tab suspensions are detected via heartbeats and safely resumed without corruption
    And unresponsive peers are evicted from active mesh routing tables after timeout
    And replica node failures trigger automatic re-replication to maintain quorum
    And hostile malformed frames or oversized payloads are immediately rejected and quarantined
    And observed availability and recovery measurements satisfy approved service-level targets
