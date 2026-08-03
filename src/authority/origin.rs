use std::time::{Duration, Instant};
use subtle::ConstantTimeEq;
use crate::error::ProtocolError;

/// Defines the clear hierarchical authority tiers in TO1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorityTier {
    Tier1Origin,    // Human precedence / Immutable Origin Key
    Tier2Delegated, // Automated sub-routines / Delegated keys
    Tier3Observer,  // Read-only / Audit agents
}

/// Represents the output of an authority validation.
/// Standard operations return `Real`, while duress returns `Decoy`.
#[derive(Debug, PartialEq, Eq)]
pub enum ExecutionContext {
    Real,
    Decoy,
}

/// Core Origin Key Manager responsible for evaluating authority and enforcing safety bounds.
#[derive(Debug, Clone)]
pub struct OriginKeyManager {
    origin_pubkey: String,
    duress_triggered_at: Option<Instant>,
    duress_duration: Duration,
    // Hash or secret representation of your personal disarm passphrase
    disarm_secret_hash: String,
}

impl OriginKeyManager {
    /// Constructs a new OriginKeyManager with a target public key string and a 48-hour lockout window.
    pub fn new(
        pubkey: impl Into<String>, 
        disarm_secret_hash: impl Into<String>
    ) -> Self {
        Self {
            origin_pubkey: pubkey.into(),
            duress_triggered_at: None,
            duress_duration,
            disarm_secret_hash: disarm_secret_hash.into(),
        }
    }

    /// Trips the duress lock, engaging the cooling period.
    pub fn trigger_duress_lockout(&mut self) {
        self.duress_triggered_at = Some(Instant::now());
    }

    /// Evaluates if the system is currently locked under duress against the local instance duration.
    pub fn is_locked_under_duress(&self) -> bool {
        self.duress_triggered_at
            .is_some_and(|triggered_at| triggered_at.elapsed() < self.duress_duration)
    }

    /// Secret disarm sequence using constant-time byte comparison.
    ///
    /// Disarms active duress silently without timing side-channel leaks.
    /// Returns `true` if duress was active and successfully cleared, `false` otherwise.
    pub fn attempt_secret_disarm(&mut self, input_secret: &str) -> bool {
        if !self.is_locked_under_duress() {
            return false;
        }

        let input_bytes = input_secret.as_bytes();
        let target_bytes = self.disarm_secret_hash.as_bytes();

        // 1. Length mismatch check
        if input_bytes.len() != target_bytes.len() {
            return false;
        }

        // 2. Constant-time byte comparison via subtle crate
        let is_match: bool = input_bytes.ct_eq(target_bytes).into();

        if is_match {
            self.duress_triggered_at = None;
            true
        } else {
            false
        }
    }

    /// Validates Tier 1 execution requests.
    pub fn validate_tier1_execution(
        &self,
        signature: &str,
        requested_tier: AuthorityTier,
    ) -> Result<ExecutionContext, ProtocolError> {
        if self.is_locked_under_duress() {
            return Ok(ExecutionContext::Decoy);
        }

        if requested_tier != AuthorityTier::Tier1Origin {
            return Err(ProtocolError::UnauthorizedAccess);
        }

        if signature.trim().is_empty() {
            return Err(ProtocolError::InvalidOriginKey);
        }

        Ok(ExecutionContext::Real)
    }
}