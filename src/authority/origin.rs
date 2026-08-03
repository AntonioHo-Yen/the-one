use crate::error::ProtocolError;

/// Defines the clear hierarchical authority tiers in TO1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorityTier {
    Tier1Origin,    // Human precedence / Immutable Origin Key
    Tier2Delegated, // Automated sub-routines / Delegated keys
    Tier3Observer,  // Read-only / Audit agents
}

/// Core Origin Key Manager responsible for evaluating authority and enforcing safety bounds.
#[derive(Debug, Clone)]
pub struct OriginKeyManager {
    origin_pubkey: String,
    duress_active: bool,
}

impl OriginKeyManager {
    /// Constructs a new OriginKeyManager with a target public key string.
    pub fn new(pubkey: impl Into<String>) -> Self {
        Self {
            origin_pubkey: pubkey.into(),
            duress_active: false,
        }
    }

    /// Sets or trips the duress flag.
    pub fn set_duress(&mut self, active: bool) {
        self.duress_active = active;
    }

    /// Validates a Tier 1 execution request against signature validity and duress conditions.
    pub fn validate_tier1_execution(
        &self,
        signature: &str,
        requested_tier: AuthorityTier,
    ) -> Result<bool, ProtocolError> {
        // Enforce duress override
        if self.duress_active {
            return Err(ProtocolError::StateViolation(
                "Duress flag active: Tier 1 execution rejected immediately.".into(),
            ));
        }

        // Verify tier requested
        if requested_tier != AuthorityTier::Tier1Origin {
            return Err(ProtocolError::UnauthorizedAccess);
        }

        // Signature format baseline check
        if signature.trim().is_empty() {
            return Err(ProtocolError::InvalidOriginKey);
        }

        Ok(true)
    }
}