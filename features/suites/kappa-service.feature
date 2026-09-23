Feature: Kappa browser-service receiver and verified blob reconciliation

  @KB-01 @build
  Scenario: Retain inbound receiver semantics and reconcile verified blob bytes
    Given a configured Kappa browser object-space service with inbound receiver and verified reconciliation
    When an inbound peer sends transport messages across the serviced receiver channel
    Then inbound messages are received and processed without dropped receiver channels
    And reconciling a peer preserves referenced blob bytes with cryptographic content verification
    And tags are only committed after their referenced blobs are verified and persisted
    And reconciliation rejects missing blobs, content hash mismatches, and truncated payloads
