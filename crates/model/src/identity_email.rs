//! UOR-native verified email identity continuity protocol (EC-01).
//!
//! Under SPEC.md and IMPLEMENTATION.md:
//! 1. Implemented through the UOR Framework-native approach in browsers without a hosted platform or organization backend.
//! 2. Native delivery, mailbox-proof, and challenge-response lifecycle for enrollment, login, and recovery.
//! 3. Replay and rollback protection: single-use challenges, cryptographic nonces, and bounded TTL (900s).
//! 4. Key possession or UOR reference alone does not establish mailbox control; verifiable mailbox challenge proof is required.
//! 5. Recovery safety invariants:
//!    - Cannot bypass scoped approval.
//!    - Cannot restore revoked grants.
//!    - Cannot create new authority or unapproved scopes.
//!    - Pre-existing sessions are atomically invalidated upon recovery.
//! 6. Zero secret leakage: no verification secrets, plain nonces, or private credentials belong in public artifacts.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

use crate::{ModelError, OwnerInputs};

/// Top-level configuration for verified email continuity (`model/email_continuity.toml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailContinuityConfig {
    /// Specification identifier (must be `foundry/email-continuity/1`).
    pub spec: String,
    /// Protocol flags and rules.
    pub protocol: EmailProtocolConfig,
    /// Delivery configuration.
    pub delivery: EmailDeliveryConfig,
    /// Cryptographic and attempt security bounds.
    pub security_bounds: EmailSecurityBoundsConfig,
}

/// Email continuity protocol invariants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailProtocolConfig {
    /// Protocol name.
    pub name: String,
    /// Operates without proprietary authentication vendors.
    pub vendor_independent: bool,
    /// Executes native browser protocol without central application backend.
    pub browser_native: bool,
    /// Time-to-live for issued challenges in seconds (normatively 900).
    pub challenge_ttl_seconds: u64,
    /// Replay protection enabled.
    pub replay_protection: bool,
    /// Requires cryptographic proof of mailbox control.
    pub require_cryptographic_mailbox_proof: bool,
    /// Invalidate all active sessions upon recovery.
    pub session_invalidation_on_recovery: bool,
    /// Prohibit recovery from restoring previously revoked authority grants.
    pub prohibit_revoked_grant_restoration: bool,
    /// Prohibit recovery from conferring new authority or ungranted scopes.
    pub prohibit_authority_creation_on_recovery: bool,
    /// Prohibit public artifact disclosure of verification secrets.
    pub prohibit_secret_leakage_in_artifacts: bool,
}

/// Delivery mechanism parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailDeliveryConfig {
    /// Delivery method identifier.
    pub method: String,
    /// Reject unauthenticated mail submission.
    pub allow_unauthenticated_mail_submission: bool,
    /// Require TLS transport.
    pub require_tls_transport: bool,
}

/// Security bounds and attempt limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSecurityBoundsConfig {
    /// Minimum nonce entropy in bytes (>= 32).
    pub minimum_nonce_length_bytes: usize,
    /// Maximum verification attempts per challenge before revocation.
    pub max_attempts_per_challenge: u32,
    /// Lockout duration in seconds following exceeded attempts.
    pub lockout_duration_seconds: u64,
}

impl EmailContinuityConfig {
    /// Cross-check configuration against owner-inputs recovery rules.
    pub fn check(&self, owner_inputs: &OwnerInputs) -> Result<(), ModelError> {
        let bad =
            |msg: String| ModelError::Inconsistent(format!("model/email_continuity.toml: {msg}"));

        if self.spec != "foundry/email-continuity/1" {
            return Err(bad(format!(
                "expected spec 'foundry/email-continuity/1', found '{}'",
                self.spec
            )));
        }

        if !self.protocol.vendor_independent {
            return Err(bad("protocol.vendor_independent must be true".to_string()));
        }

        if !self.protocol.browser_native {
            return Err(bad("protocol.browser_native must be true".to_string()));
        }

        if !self.protocol.require_cryptographic_mailbox_proof {
            return Err(bad(
                "protocol.require_cryptographic_mailbox_proof must be true".to_string(),
            ));
        }

        if self.protocol.challenge_ttl_seconds
            != owner_inputs.recovery_rules.email_challenge_ttl_seconds
        {
            return Err(bad(format!(
                "challenge_ttl_seconds ({}) does not match owner_inputs recovery rule ({})",
                self.protocol.challenge_ttl_seconds,
                owner_inputs.recovery_rules.email_challenge_ttl_seconds
            )));
        }

        if self.protocol.replay_protection
            != owner_inputs
                .recovery_rules
                .email_challenge_replay_protection
        {
            return Err(bad(
                "replay_protection must match owner_inputs recovery rule".to_string(),
            ));
        }

        if self.protocol.session_invalidation_on_recovery
            != owner_inputs.recovery_rules.session_invalidation_on_recovery
        {
            return Err(bad(
                "session_invalidation_on_recovery must match owner_inputs recovery rule"
                    .to_string(),
            ));
        }

        if self.protocol.prohibit_revoked_grant_restoration
            != owner_inputs.recovery_rules.reject_revoked_grant_recovery
        {
            return Err(bad(
                "prohibit_revoked_grant_restoration must match owner_inputs reject_revoked_grant_recovery".to_string(),
            ));
        }

        if self.security_bounds.minimum_nonce_length_bytes < 32 {
            return Err(bad(
                "minimum_nonce_length_bytes must be at least 32 bytes (256 bits)".to_string(),
            ));
        }

        Ok(())
    }
}

/// The intended purpose of an email challenge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChallengePurpose {
    /// Initial account creation and mailbox binding.
    Enrollment,
    /// Interactive session login.
    Login,
    /// Credential rotation and account recovery.
    Recovery,
}

/// Lifecycle status of a user account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AccountStatus {
    /// Normal active account.
    Active,
    /// Account suspended pending identity investigation.
    Suspended,
    /// Account undergoing credential recovery.
    RecoveryPending,
}

/// An ephemeral challenge issued to prove mailbox control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailChallenge {
    /// Unique challenge identifier.
    pub challenge_id: String,
    /// Destination mailbox.
    pub mailbox: String,
    /// Operational purpose.
    pub purpose: ChallengePurpose,
    /// SHA-256 digest of the secret challenge nonce.
    pub nonce_digest: String,
    /// Timestamp of issuance (seconds).
    pub issued_at_timestamp: u64,
    /// Expiration timestamp (seconds).
    pub expires_at_timestamp: u64,
    /// Whether this challenge has been consumed.
    pub consumed: bool,
    /// Failed verification attempts.
    pub failed_attempts: u32,
}

/// An active session record for an authenticated user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRecord {
    /// Session identifier.
    pub session_id: String,
    /// Client device fingerprint.
    pub device_fingerprint: String,
    /// Session creation timestamp.
    pub created_at: u64,
    /// Whether the session is currently valid.
    pub valid: bool,
}

/// An enrolled user account record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccountRecord {
    /// Canonical UOR user ID (e.g. `uor:user:trinity`).
    pub user_id: String,
    /// Bound verified email address.
    pub mailbox: String,
    /// Primary public key (hex-encoded Ed25519).
    pub primary_public_key: String,
    /// Account status.
    pub status: AccountStatus,
    /// Authorized scopes currently granted.
    pub granted_scopes: HashSet<String>,
    /// Permanently revoked scopes that recovery cannot reinstate.
    pub revoked_scopes: HashSet<String>,
    /// Active sessions.
    pub sessions: Vec<SessionRecord>,
}

/// Errors raised during email identity continuity operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityError {
    /// Challenge not found.
    ChallengeNotFound(String),
    /// Challenge has expired.
    ChallengeExpired {
        /// Challenge ID.
        challenge_id: String,
        /// Current timestamp.
        now: u64,
        /// Expiration timestamp.
        expires_at: u64,
    },
    /// Challenge has already been consumed (replay attempt).
    ChallengeReplayDetected(String),
    /// Invalid challenge proof (nonce mismatch).
    InvalidChallengeProof(String),
    /// Maximum verification attempts exceeded.
    MaxAttemptsExceeded(String),
    /// Mailbox does not match challenge target.
    MailboxMismatch {
        /// Expected mailbox.
        expected: String,
        /// Actual mailbox.
        actual: String,
    },
    /// User account not found.
    AccountNotFound(String),
    /// Mailbox already registered during enrollment.
    AccountAlreadyExists(String),
    /// Key possession alone without mailbox proof is insufficient.
    MailboxProofMissing(String),
    /// Recovery attempted to restore a previously revoked grant.
    RevokedGrantRestorationRejected {
        /// Mailbox.
        mailbox: String,
        /// Revoked scope requested.
        scope: String,
    },
    /// Recovery attempted to create new unapproved authority scopes.
    AuthorityCreationRejected {
        /// Mailbox.
        mailbox: String,
        /// Scope attempted.
        scope: String,
    },
    /// Public key already bound to another mailbox.
    DuplicateKeyBinding(String),
    /// Inadequate nonce entropy.
    InsufficientNonceEntropy(usize),
}

impl std::fmt::Display for IdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChallengeNotFound(id) => write!(f, "challenge not found: {id}"),
            Self::ChallengeExpired {
                challenge_id,
                now,
                expires_at,
            } => {
                write!(
                    f,
                    "challenge '{challenge_id}' expired at {expires_at}, current time is {now}"
                )
            }
            Self::ChallengeReplayDetected(id) => write!(f, "challenge replay detected: {id}"),
            Self::InvalidChallengeProof(msg) => write!(f, "invalid challenge proof: {msg}"),
            Self::MaxAttemptsExceeded(id) => {
                write!(f, "maximum attempts exceeded for challenge: {id}")
            }
            Self::MailboxMismatch { expected, actual } => {
                write!(f, "mailbox mismatch: expected '{expected}', got '{actual}'")
            }
            Self::AccountNotFound(mb) => write!(f, "account not found for mailbox: {mb}"),
            Self::AccountAlreadyExists(mb) => write!(f, "account already exists for mailbox: {mb}"),
            Self::MailboxProofMissing(msg) => write!(f, "mailbox proof missing: {msg}"),
            Self::RevokedGrantRestorationRejected { mailbox, scope } => {
                write!(
                    f,
                    "recovery cannot restore revoked grant '{scope}' for '{mailbox}'"
                )
            }
            Self::AuthorityCreationRejected { mailbox, scope } => {
                write!(
                    f,
                    "recovery cannot create new authority scope '{scope}' for '{mailbox}'"
                )
            }
            Self::DuplicateKeyBinding(key) => {
                write!(f, "public key '{key}' already bound to another account")
            }
            Self::InsufficientNonceEntropy(len) => {
                write!(f, "nonce length {len} bytes is less than minimum 32 bytes")
            }
        }
    }
}

impl std::error::Error for IdentityError {}

/// Helper to compute SHA-256 digest hex string.
fn hash_nonce(nonce: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(nonce.as_bytes());
    let result = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for byte in result {
        use std::fmt::Write;
        let _ = write!(hex, "{:02x}", byte);
    }
    hex
}

/// Email continuity manager implementing native verification protocols.
#[derive(Debug, Clone)]
pub struct EmailContinuityManager {
    /// Enrolled user accounts indexed by mailbox.
    pub accounts: HashMap<String, UserAccountRecord>,
    /// Ephemeral challenges indexed by challenge ID.
    pub challenges: HashMap<String, EmailChallenge>,
    /// Set of consumed nonce digests for distributed replay protection.
    pub consumed_digests: HashSet<String>,
    /// Protocol configuration.
    pub config: EmailContinuityConfig,
}

impl EmailContinuityManager {
    /// Create a new EmailContinuityManager.
    pub fn new(config: EmailContinuityConfig) -> Self {
        Self {
            accounts: HashMap::new(),
            challenges: HashMap::new(),
            consumed_digests: HashSet::new(),
            config,
        }
    }

    /// Issue a new verification challenge with cryptographic nonce.
    pub fn issue_challenge(
        &mut self,
        challenge_id: &str,
        mailbox: &str,
        purpose: ChallengePurpose,
        nonce: &str,
        current_time: u64,
    ) -> Result<&EmailChallenge, IdentityError> {
        if nonce.len() < self.config.security_bounds.minimum_nonce_length_bytes {
            return Err(IdentityError::InsufficientNonceEntropy(nonce.len()));
        }

        let nonce_digest = hash_nonce(nonce);
        if self.consumed_digests.contains(&nonce_digest) {
            return Err(IdentityError::ChallengeReplayDetected(
                "nonce digest already consumed in prior challenge".to_string(),
            ));
        }

        let expires_at_timestamp = current_time + self.config.protocol.challenge_ttl_seconds;

        let challenge = EmailChallenge {
            challenge_id: challenge_id.to_string(),
            mailbox: mailbox.to_string(),
            purpose,
            nonce_digest,
            issued_at_timestamp: current_time,
            expires_at_timestamp,
            consumed: false,
            failed_attempts: 0,
        };

        self.challenges.insert(challenge_id.to_string(), challenge);
        Ok(self.challenges.get(challenge_id).unwrap())
    }

    /// Internal validation of challenge proof, TTL, and replay protection.
    fn verify_and_consume_challenge(
        &mut self,
        challenge_id: &str,
        nonce: &str,
        expected_purpose: ChallengePurpose,
        current_time: u64,
    ) -> Result<String, IdentityError> {
        let challenge = self
            .challenges
            .get_mut(challenge_id)
            .ok_or_else(|| IdentityError::ChallengeNotFound(challenge_id.to_string()))?;

        if challenge.consumed {
            return Err(IdentityError::ChallengeReplayDetected(
                challenge_id.to_string(),
            ));
        }

        if challenge.failed_attempts >= self.config.security_bounds.max_attempts_per_challenge {
            return Err(IdentityError::MaxAttemptsExceeded(challenge_id.to_string()));
        }

        if current_time > challenge.expires_at_timestamp {
            return Err(IdentityError::ChallengeExpired {
                challenge_id: challenge_id.to_string(),
                now: current_time,
                expires_at: challenge.expires_at_timestamp,
            });
        }

        if challenge.purpose != expected_purpose {
            return Err(IdentityError::InvalidChallengeProof(format!(
                "challenge purpose mismatch: expected {:?}, found {:?}",
                expected_purpose, challenge.purpose
            )));
        }

        let computed_digest = hash_nonce(nonce);
        if computed_digest != challenge.nonce_digest {
            challenge.failed_attempts += 1;
            return Err(IdentityError::InvalidChallengeProof(
                "nonce does not match issued challenge".to_string(),
            ));
        }

        // Atomically consume
        challenge.consumed = true;
        self.consumed_digests.insert(computed_digest);
        Ok(challenge.mailbox.clone())
    }

    /// Complete new account enrollment upon verified mailbox challenge.
    pub fn complete_enrollment(
        &mut self,
        challenge_id: &str,
        nonce: &str,
        user_id: &str,
        public_key: &str,
        initial_scopes: &[String],
        current_time: u64,
    ) -> Result<&UserAccountRecord, IdentityError> {
        let mailbox = self.verify_and_consume_challenge(
            challenge_id,
            nonce,
            ChallengePurpose::Enrollment,
            current_time,
        )?;

        if self.accounts.contains_key(&mailbox) {
            return Err(IdentityError::AccountAlreadyExists(mailbox));
        }

        // Verify public key is not already bound to another account
        if self
            .accounts
            .values()
            .any(|a| a.primary_public_key == public_key)
        {
            return Err(IdentityError::DuplicateKeyBinding(public_key.to_string()));
        }

        let account = UserAccountRecord {
            user_id: user_id.to_string(),
            mailbox: mailbox.clone(),
            primary_public_key: public_key.to_string(),
            status: AccountStatus::Active,
            granted_scopes: initial_scopes.iter().cloned().collect(),
            revoked_scopes: HashSet::new(),
            sessions: Vec::new(),
        };

        self.accounts.insert(mailbox.clone(), account);
        Ok(self.accounts.get(&mailbox).unwrap())
    }

    /// Complete login and issue an active session record upon verified challenge.
    pub fn complete_login(
        &mut self,
        challenge_id: &str,
        nonce: &str,
        session_id: &str,
        device_fingerprint: &str,
        current_time: u64,
    ) -> Result<&SessionRecord, IdentityError> {
        let mailbox = self.verify_and_consume_challenge(
            challenge_id,
            nonce,
            ChallengePurpose::Login,
            current_time,
        )?;

        let account = self
            .accounts
            .get_mut(&mailbox)
            .ok_or_else(|| IdentityError::AccountNotFound(mailbox.clone()))?;

        let session = SessionRecord {
            session_id: session_id.to_string(),
            device_fingerprint: device_fingerprint.to_string(),
            created_at: current_time,
            valid: true,
        };

        account.sessions.push(session);
        Ok(account.sessions.last().unwrap())
    }

    /// Complete account recovery: replaces public key, invalidates all sessions,
    /// verifies that revoked grants are NOT restored, and prohibits authority creation.
    pub fn complete_recovery(
        &mut self,
        challenge_id: &str,
        nonce: &str,
        new_public_key: &str,
        requested_scopes: &[String],
        current_time: u64,
    ) -> Result<&UserAccountRecord, IdentityError> {
        let mailbox = self.verify_and_consume_challenge(
            challenge_id,
            nonce,
            ChallengePurpose::Recovery,
            current_time,
        )?;

        let account = self
            .accounts
            .get_mut(&mailbox)
            .ok_or_else(|| IdentityError::AccountNotFound(mailbox.clone()))?;

        // 1. Prohibit restoring revoked grants
        if self.config.protocol.prohibit_revoked_grant_restoration {
            for scope in requested_scopes {
                if account.revoked_scopes.contains(scope) {
                    return Err(IdentityError::RevokedGrantRestorationRejected {
                        mailbox: mailbox.clone(),
                        scope: scope.clone(),
                    });
                }
            }
        }

        // 2. Prohibit creating new unapproved authority
        if self.config.protocol.prohibit_authority_creation_on_recovery {
            for scope in requested_scopes {
                if !account.granted_scopes.contains(scope) {
                    return Err(IdentityError::AuthorityCreationRejected {
                        mailbox: mailbox.clone(),
                        scope: scope.clone(),
                    });
                }
            }
        }

        // 3. Atomically invalidate all prior sessions
        if self.config.protocol.session_invalidation_on_recovery {
            for session in &mut account.sessions {
                session.valid = false;
            }
        }

        // 4. Update credentials
        account.primary_public_key = new_public_key.to_string();
        account.status = AccountStatus::Active;

        Ok(account)
    }

    /// Explicitly revoke an authority scope from an account.
    pub fn revoke_scope(&mut self, mailbox: &str, scope: &str) -> Result<(), IdentityError> {
        let account = self
            .accounts
            .get_mut(mailbox)
            .ok_or_else(|| IdentityError::AccountNotFound(mailbox.to_string()))?;

        account.granted_scopes.remove(scope);
        account.revoked_scopes.insert(scope.to_string());
        Ok(())
    }
}
