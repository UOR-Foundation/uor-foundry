Feature: Holospaces threat model and browser-only durability and discovery boundary

  @HB-01 @build
  Scenario: Revalidate threat model assumptions against participant/faculty-only session realities and browser churn
    Given configured Holospaces boundary policy and participant session churn constraints
    When participants and faculty operate within isolated browser execution contexts
    Then native-relay daemon assumptions alongside browser tabs are strictly prohibited
    And private key possession must be cryptographically proven rather than assumed from public key addresses
    And local WebRTC witness proxies and out-of-band signaling are rejected as production discovery claims
    And service execution is prohibited when all participants are offline
    And participant replica quorum deficit triggers durability violations
