use std::time::{Duration, Instant};
use crate::error::ProtocolError;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use zeroize::Zeroize;

/// Default duress lockout cooling duration (48 hours) if none is specified.
pub const DEFAULT_DURESS_DURATION: Duration = Duration::from_secs(172_800);

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

/// -------------------------------------------------------------------
/// 1. GENESIS BOOTSTRAPPER (One-time Genesis setup)
/// -------------------------------------------------------------------
/// Handles identity genesis, derives initial Tier 1 keys, registers
/// non-sensitive biometric helper data, and zeroizes the genesis seed.
pub struct OriginBootstrapper {
    pub genesis_id: String,
    pub helper_data: Option<VascularHelperData>,
}

impl OriginBootstrapper {
    pub fn initialize(genesis_id: impl Into<String>, helper_data: VascularHelperData) -> Self {
        Self {
            genesis_id: genesis_id.into(),
            helper_data: Some(helper_data),
        }
    }
    /// Generates the operational Tier 1 Key Manager and zeroizes volatile genesis state.
    pub fn bootstrap_tier1(
        &mut self,
        pubkey: String,
        disarm_hash: String,
        duress_duration: Duration,
    ) -> OriginKeyManager {
        // Create the operational Tier 1 key manager
        let manager = OriginKeyManager::new(pubkey, disarm_hash, duress_duration);

        // Discard/Zeroize one-time bootstrap memory
        self.genesis_id.zeroize();

        manager
    }
}

/// -------------------------------------------------------------------
/// 2. TIER 1 OPERATIONAL KEY MANAGER (Daily Runtime)
/// -------------------------------------------------------------------
/// Evaluates authority signatures, enforces duress lockout, and handles disarm verification.
#[derive(Debug, Clone)]
pub struct OriginKeyManager {
    pub origin_pubkey: String,
    pub duress_triggered_at: Option<Instant>,
    pub duress_duration: Duration,
    pub disarm_secret_hash: String, // PHC-formatted Argon2id hash string
}

/// Helper function to verify a passphrase against an Argon2id PHC hash string.
#[derive(Debug, Clone)]
pub fn verify_disarm_passphrase(passphrase: &str, stored_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(stored_hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
    .verify_password(passphrase.as_bytes(), &parsed_hash)
    .is_ok()
}

impl OriginKeyManager {
    /// Constructs a new OriginKeyManager with a explicit custom `duress_duration`.
    /// Useful for unit testing, staging environments, and custom enterprise/corporate policies.
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
    
    /// Constructs an OriginKeyManager using the standard 48-hour default duress duration.
    pub fn with_default_duration(
        pubkey: impl Into<String>,
        disarm_secret_hash: impl Into<String>,
    ) -> Self {
        Self::new(pubkey, disarm_secret_hash, DEFAULT_DURESS_DURATION)
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

    pub fn attempt_secret_disarm(&mut self, input_secret: &str) -> bool {
        if !self.is_locked_under_duress() {
            return false;
        }

        if verify_disarm_passphrase(input_secret, &self.disarm_secret_hash) {
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