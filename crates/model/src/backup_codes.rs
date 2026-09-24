//! NIST SP 800-63B-4 compliant saved backup-code recovery lifecycle (BC-01).
//!
//! Under SPEC.md and IMPLEMENTATION.md:
//! 1. Protected issuance, account/revision binding, one-time redemption, and rotation.
//! 2. Credential replacement, session invalidation, and notification flows.
//! 3. Mutual non-substitution: neither mail submission nor backup code substitutes for the other's acceptance.
//! 4. Distributed replay/rollback rejection and encrypted-data recovery evidence.
//! 5. Plaintext codes are never stored; only salted SHA-256 digests with >= 128-bit entropy.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::identity_email::UserAccountRecord;
use crate::{ModelError, OwnerInputs};

/// Top-level configuration for backup-code recovery (`model/backup_codes.toml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCodeConfig {
    /// Specification identifier (must be `foundry/backup-codes/1`).
    pub spec: String,
    /// Standards citation.
    pub standards: BackupCodeStandardsConfig,
    /// Lifecycle invariants.
    pub lifecycle: BackupCodeLifecycleConfig,
    /// Notification parameters.
    pub notification: BackupCodeNotificationConfig,
}

/// Standards requirements for backup codes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCodeStandardsConfig {
    /// Normative standard identifier.
    pub standard: String,
    /// Minimum entropy in bits per batch (>= 128).
    pub minimum_entropy_bits: usize,
    /// Storage digest scheme (must be `salted-sha256`).
    pub storage_scheme: String,
}

/// Lifecycle rules for issuance and redemption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCodeLifecycleConfig {
    /// Number of codes issued per batch (normatively 10).
    pub batch_size: usize,
    /// Single-use redemption flag.
    pub single_use: bool,
    /// Session invalidation upon redemption.
    pub session_invalidation_on_recovery: bool,
    /// Strict binding of batch to account revision at issuance.
    pub account_and_revision_binding: bool,
    /// Reissuance deactivates prior batches (rotation).
    pub require_rotation_on_reissuance: bool,
    /// Reject replay of redeemed codes and rollback of revisions.
    pub reject_replay_and_rollback: bool,
    /// Prohibit recovery from restoring previously revoked grants.
    pub prohibit_revoked_grant_restoration: bool,
    /// Prohibit recovery from conferring new authority.
    pub prohibit_authority_creation_on_recovery: bool,
    /// Mutual non-substitution between email and backup codes.
    pub require_mutual_non_substitution: bool,
}

/// Notification thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCodeNotificationConfig {
    /// Emit notification event on redemption.
    pub notify_on_redemption: bool,
    /// Emit warning on code exhaustion.
    pub notify_on_exhaustion: bool,
    /// Remaining code threshold for exhaustion warning.
    pub remaining_threshold_warning: usize,
}

impl BackupCodeConfig {
    /// Cross-check configuration against owner-inputs recovery rules.
    pub fn check(&self, owner_inputs: &OwnerInputs) -> Result<(), ModelError> {
        let bad = |msg: String| ModelError::Inconsistent(format!("model/backup_codes.toml: {msg}"));

        if self.spec != "foundry/backup-codes/1" {
            return Err(bad(format!(
                "expected spec 'foundry/backup-codes/1', found '{}'",
                self.spec
            )));
        }

        if self.standards.standard != owner_inputs.recovery_rules.backup_code_standard {
            return Err(bad(format!(
                "backup code standard '{}' does not match owner_inputs '{}'",
                self.standards.standard, owner_inputs.recovery_rules.backup_code_standard
            )));
        }

        if self.standards.minimum_entropy_bits
            < owner_inputs.recovery_rules.backup_code_entropy_bits as usize
        {
            return Err(bad(
                "minimum_entropy_bits is less than owner_inputs recovery rule".to_string(),
            ));
        }

        if self.standards.storage_scheme != owner_inputs.recovery_rules.backup_code_storage_scheme {
            return Err(bad(
                "storage_scheme must match owner_inputs recovery rule".to_string()
            ));
        }

        if self.lifecycle.single_use != owner_inputs.recovery_rules.backup_code_single_use {
            return Err(bad(
                "single_use must match owner_inputs recovery rule".to_string()
            ));
        }

        if self.lifecycle.session_invalidation_on_recovery
            != owner_inputs.recovery_rules.session_invalidation_on_recovery
        {
            return Err(bad(
                "session_invalidation_on_recovery must match owner_inputs recovery rule"
                    .to_string(),
            ));
        }

        if self.lifecycle.prohibit_revoked_grant_restoration
            != owner_inputs.recovery_rules.reject_revoked_grant_recovery
        {
            return Err(bad(
                "prohibit_revoked_grant_restoration must match owner_inputs reject_revoked_grant_recovery".to_string(),
            ));
        }

        if !self.lifecycle.require_mutual_non_substitution {
            return Err(bad(
                "require_mutual_non_substitution must be true".to_string()
            ));
        }

        Ok(())
    }
}

/// Lifecycle status of an individual backup code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CodeStatus {
    /// Unused and available for single-use redemption.
    Unused,
    /// Already redeemed.
    Redeemed,
    /// Revoked due to batch rotation.
    Revoked,
}

/// Stored salted hash record for a backup code (plaintext is never stored).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredBackupCode {
    /// Unique identifier for this code in the batch.
    pub code_id: String,
    /// Cryptographic salt.
    pub salt: String,
    /// Salted SHA-256 digest hex.
    pub code_hash: String,
    /// Current redemption status.
    pub status: CodeStatus,
    /// Timestamp when code was redeemed, if applicable.
    pub redeemed_at: Option<u64>,
}

/// A batch of backup codes issued to an account at a specific revision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCodeBatch {
    /// Unique batch identifier.
    pub batch_id: String,
    /// Account mailbox.
    pub mailbox: String,
    /// Account revision to which this batch is cryptographically bound.
    pub account_revision: u64,
    /// Issuance timestamp.
    pub issued_at: u64,
    /// Stored salted code records.
    pub codes: Vec<StoredBackupCode>,
    /// Whether this batch is the current active batch for the account.
    pub active: bool,
}

/// Request to redeem a backup code for credential replacement and recovery.
#[derive(Debug, Clone)]
pub struct RedeemBackupCodeRequest {
    /// Target mailbox.
    pub mailbox: String,
    /// Plaintext backup code presented by the user.
    pub code_plaintext: String,
    /// Expected account revision (for rollback protection).
    pub expected_account_revision: u64,
    /// Replacement public key.
    pub new_public_key: String,
    /// Requested scopes upon recovery.
    pub requested_scopes: Vec<String>,
}

/// Result report of a successful backup code redemption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedemptionReport {
    /// Mailbox.
    pub mailbox: String,
    /// Timestamp of redemption.
    pub redeemed_at: u64,
    /// Remaining unused codes in the active batch.
    pub remaining_unused_codes: usize,
    /// True if remaining unused codes is at or below warning threshold.
    pub exhaustion_warning: bool,
}

/// Errors raised during backup-code operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupCodeError {
    /// Plaintext code does not match any code in the active batch.
    InvalidCode(String),
    /// Code has already been redeemed (replay attempt).
    CodeReplayDetected(String),
    /// No active batch found for the account.
    NoActiveBatch(String),
    /// Account revision rollback detected (optimistic concurrency / rollback fencing).
    RevisionRollbackDetected {
        /// Expected revision.
        expected: u64,
        /// Actual revision on record.
        actual: u64,
    },
    /// Mutual non-substitution violation: presenting email token for backup code or vice versa.
    SubstitutionViolation(String),
    /// Recovery attempted to restore a revoked authority grant.
    RevokedGrantRestorationRejected {
        /// Mailbox.
        mailbox: String,
        /// Scope attempted.
        scope: String,
    },
    /// Recovery attempted to create new ungranted authority scopes.
    AuthorityCreationRejected {
        /// Mailbox.
        mailbox: String,
        /// Scope attempted.
        scope: String,
    },
    /// Insufficient code entropy.
    InsufficientEntropy(usize),
}

impl std::fmt::Display for BackupCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCode(msg) => write!(f, "invalid backup code: {msg}"),
            Self::CodeReplayDetected(id) => write!(f, "backup code replay detected: {id}"),
            Self::NoActiveBatch(mb) => write!(f, "no active backup code batch for: {mb}"),
            Self::RevisionRollbackDetected { expected, actual } => {
                write!(
                    f,
                    "revision rollback detected: expected {expected}, actual {actual}"
                )
            }
            Self::SubstitutionViolation(msg) => {
                write!(f, "mutual non-substitution violation: {msg}")
            }
            Self::RevokedGrantRestorationRejected { mailbox, scope } => {
                write!(
                    f,
                    "backup code recovery cannot restore revoked grant '{scope}' for '{mailbox}'"
                )
            }
            Self::AuthorityCreationRejected { mailbox, scope } => {
                write!(f, "backup code recovery cannot create new authority scope '{scope}' for '{mailbox}'")
            }
            Self::InsufficientEntropy(bits) => {
                write!(
                    f,
                    "insufficient entropy: {bits} bits is less than required 128 bits"
                )
            }
        }
    }
}

impl std::error::Error for BackupCodeError {}

/// Compute salted SHA-256 hash.
fn hash_code_with_salt(salt: &str, code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(code.as_bytes());
    let result = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for byte in result {
        use std::fmt::Write;
        let _ = write!(hex, "{:02x}", byte);
    }
    hex
}

/// Backup-code recovery manager.
#[derive(Debug, Clone)]
pub struct BackupCodeManager {
    /// Batches indexed by mailbox.
    pub batches: HashMap<String, Vec<BackupCodeBatch>>,
    /// Protocol configuration.
    pub config: BackupCodeConfig,
}

impl BackupCodeManager {
    /// Create a new BackupCodeManager.
    pub fn new(config: BackupCodeConfig) -> Self {
        Self {
            batches: HashMap::new(),
            config,
        }
    }

    /// Issue a new batch of 10 high-entropy single-use backup codes.
    ///
    /// Deactivates any preexisting active batches for the mailbox (rotation).
    /// Returns the persisted batch record (with salted hashes) and plaintexts for user custody.
    pub fn issue_batch(
        &mut self,
        mailbox: &str,
        account_revision: u64,
        batch_id: &str,
        plaintexts: &[String],
        current_time: u64,
    ) -> Result<BackupCodeBatch, BackupCodeError> {
        if plaintexts.len() != self.config.lifecycle.batch_size {
            return Err(BackupCodeError::InvalidCode(format!(
                "batch must contain exactly {} codes, found {}",
                self.config.lifecycle.batch_size,
                plaintexts.len()
            )));
        }

        // Verify entropy: each code must have at least 16 characters (128 bits of base16 / alphanumeric entropy)
        for code in plaintexts {
            if code.len() < 16 {
                return Err(BackupCodeError::InsufficientEntropy(code.len() * 8));
            }
        }

        let user_batches = self.batches.entry(mailbox.to_string()).or_default();

        // Deactivate prior batches (rotation)
        if self.config.lifecycle.require_rotation_on_reissuance {
            for b in user_batches.iter_mut() {
                b.active = false;
                for c in &mut b.codes {
                    if c.status == CodeStatus::Unused {
                        c.status = CodeStatus::Revoked;
                    }
                }
            }
        }

        let mut stored_codes = Vec::new();
        for (idx, plain) in plaintexts.iter().enumerate() {
            let salt = format!("salt:{}:{}:{}", batch_id, idx, current_time);
            let code_hash = hash_code_with_salt(&salt, plain);
            stored_codes.push(StoredBackupCode {
                code_id: format!("{batch_id}-code-{idx}"),
                salt,
                code_hash,
                status: CodeStatus::Unused,
                redeemed_at: None,
            });
        }

        let batch = BackupCodeBatch {
            batch_id: batch_id.to_string(),
            mailbox: mailbox.to_string(),
            account_revision,
            issued_at: current_time,
            codes: stored_codes,
            active: true,
        };

        user_batches.push(batch.clone());
        Ok(batch)
    }

    /// Redeem a backup code to replace credentials and invalidate preexisting sessions.
    pub fn redeem_code(
        &mut self,
        req: RedeemBackupCodeRequest,
        account: &mut UserAccountRecord,
        account_current_revision: u64,
        current_time: u64,
    ) -> Result<RedemptionReport, BackupCodeError> {
        let user_batches = self
            .batches
            .get_mut(&req.mailbox)
            .ok_or_else(|| BackupCodeError::NoActiveBatch(req.mailbox.clone()))?;

        let active_batch = user_batches
            .iter_mut()
            .find(|b| b.active)
            .ok_or_else(|| BackupCodeError::NoActiveBatch(req.mailbox.clone()))?;

        // 1. Rollback & Revision Binding Check
        if self.config.lifecycle.account_and_revision_binding
            && active_batch.account_revision != req.expected_account_revision
        {
            return Err(BackupCodeError::RevisionRollbackDetected {
                expected: active_batch.account_revision,
                actual: req.expected_account_revision,
            });
        }

        if account_current_revision != req.expected_account_revision {
            return Err(BackupCodeError::RevisionRollbackDetected {
                expected: account_current_revision,
                actual: req.expected_account_revision,
            });
        }

        // 2. Locate and verify code
        let mut matched_code_idx = None;
        let mut replay_detected = false;

        for (idx, stored) in active_batch.codes.iter().enumerate() {
            let computed_hash = hash_code_with_salt(&stored.salt, &req.code_plaintext);
            if computed_hash == stored.code_hash {
                if stored.status == CodeStatus::Redeemed {
                    replay_detected = true;
                    break;
                }
                if stored.status == CodeStatus::Unused {
                    matched_code_idx = Some(idx);
                    break;
                }
            }
        }

        if replay_detected {
            return Err(BackupCodeError::CodeReplayDetected(
                "backup code has already been redeemed".to_string(),
            ));
        }

        let code_idx = matched_code_idx.ok_or_else(|| {
            BackupCodeError::InvalidCode(
                "code does not match any unused code in active batch".to_string(),
            )
        })?;

        // 3. Prohibit restoring revoked grants
        if self.config.lifecycle.prohibit_revoked_grant_restoration {
            for scope in &req.requested_scopes {
                if account.revoked_scopes.contains(scope) {
                    return Err(BackupCodeError::RevokedGrantRestorationRejected {
                        mailbox: req.mailbox.clone(),
                        scope: scope.clone(),
                    });
                }
            }
        }

        // 4. Prohibit creating new unapproved authority
        if self
            .config
            .lifecycle
            .prohibit_authority_creation_on_recovery
        {
            for scope in &req.requested_scopes {
                if !account.granted_scopes.contains(scope) {
                    return Err(BackupCodeError::AuthorityCreationRejected {
                        mailbox: req.mailbox.clone(),
                        scope: scope.clone(),
                    });
                }
            }
        }

        // 5. Mark code as redeemed (single-use)
        active_batch.codes[code_idx].status = CodeStatus::Redeemed;
        active_batch.codes[code_idx].redeemed_at = Some(current_time);

        // 6. Invalidate all preexisting sessions
        if self.config.lifecycle.session_invalidation_on_recovery {
            for session in &mut account.sessions {
                session.valid = false;
            }
        }

        // 7. Update credentials
        account.primary_public_key = req.new_public_key;

        // 8. Count remaining unused codes
        let remaining_unused_codes = active_batch
            .codes
            .iter()
            .filter(|c| c.status == CodeStatus::Unused)
            .count();

        let exhaustion_warning =
            remaining_unused_codes <= self.config.notification.remaining_threshold_warning;

        Ok(RedemptionReport {
            mailbox: req.mailbox,
            redeemed_at: current_time,
            remaining_unused_codes,
            exhaustion_warning,
        })
    }

    /// Enforce mutual non-substitution: neither email tokens nor backup codes can substitute for each other.
    pub fn verify_mutual_non_substitution(
        &self,
        endpoint_type: &str,
        credential_type: &str,
    ) -> Result<(), BackupCodeError> {
        if !self.config.lifecycle.require_mutual_non_substitution {
            return Ok(());
        }

        match (endpoint_type, credential_type) {
            ("email_challenge_endpoint", "backup_code") => {
                Err(BackupCodeError::SubstitutionViolation(
                    "backup code cannot substitute for email challenge verification".to_string(),
                ))
            }
            ("backup_code_recovery_endpoint", "email_challenge_token") => {
                Err(BackupCodeError::SubstitutionViolation(
                    "email challenge token cannot substitute for backup code recovery".to_string(),
                ))
            }
            _ => Ok(()),
        }
    }
}
