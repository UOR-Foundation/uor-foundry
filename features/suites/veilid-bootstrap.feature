Feature: Veilid secure bootstrap route and browser transport boundary

  @VB-01 @build
  Scenario: Validate authenticated Veilid bootstrap routes, browser transport integration, relay verification, outbound-relay limitation mitigations, and peer churn resilience
    Given configured public Veilid bootstrap peers and HTTPS-origin routing targeting /foundry-web/
    When the browser initializes transport connections to disclosed bootstrap relays
    Then bootstrap routes authenticate relay public keys and verify cryptographic trust root digests
    And insecure non-WSS endpoints and path mismatches are immediately rejected
    And Veilid 0.5.7 outbound relay limitations are mitigated via authenticated relay fallback
    And build feature flags without verified live transport evidence are rejected as unsupported assumptions
    And peer churn and network interruptions trigger seamless reconnection to alternative bootstrap relays
