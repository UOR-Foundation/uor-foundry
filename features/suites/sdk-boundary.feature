Feature: Immutable OCI SDK and offline dependency boundary

  @SB-01 @build
  Scenario: Validate multi-architecture immutable SDK index, offline dependency closure, digest-bound oracles, and consumer acceptance separation
    Given configured multi-architecture OCI SDK index for linux/amd64 and linux/arm64
    When the build and verification environments execute from a fresh cache
    Then each architecture manifest and inventory digest matches its pinned cryptographic record
    And complete offline dependency closure is verified against Cargo.lock without wildcard or git substitutions
    And external authoritative oracles are cryptographically bound by digest across architectures
    And source integration alone is strictly prohibited from passing as consumer acceptance
