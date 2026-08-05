use crate::authority::origin::{AuthorityTier, ExecutionContext, OriginKeyManager};
use serde::{Deserialize, Serialize};

/// Evaluation outcomes returned by the State Engine
#[derive(Debug, PartialEq, Eq)]
pub enum SystemAction {
    /// Overwrite existing state immediately (Tier 1 > Tier 2/3)
    Overwrite,
    /// Normal commit for non-conflicting background updates
    Commit,
    /// Discard stale or lower-tier automated updates without mutating state
    RejectStalePayload,
    /// Lock contested attribute and route to PENDING_HUMAN_REVIEW queue
    FlagConflict,
    /// Reject Tier 1 payload lacking signature proof
    RejectUnauthorizedTier1,
    /// Lock state defensively if duress is detected
    LockStateDuress,
}

/// Metadata header attached to all incoming payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataHeader {
    pub entity_id: String,
    pub attribute: String,
    pub tier: AuthorityTier,
    pub timestamp_utc: u64,
    pub signature: Option<String>,
}

/// Generic state container for a committed attribute value
#[derive(Debug, Clone)]
pub struct AttributeState<T> {
    pub value: T,
    pub meta: MetadataHeader,
}

pub struct Evaluator;

impl Evaluator {
    /// Evaluates incoming payload against committed state and returns the required SystemAction.
    /// Canonical Tier 1 signature and duress checks are delegated directly to `OriginKeyManager`.
    pub fn evaluate<T: PartialEq>(
        current: &AttributeState<T>,
        incoming_meta: &MetadataHeader,
        incoming_value: &T,
        key_manager: Option<&OriginKeyManager>,
    ) -> SystemAction {
        // Guardrail 1: Enforce stale check for automated operations
        if incoming_meta.tier == AuthorityTier::Tier3Observer
            && incoming_meta.timestamp_utc <= current.meta.timestamp_utc
        {
            return SystemAction::RejectStalePayload;
        }

        // Guardrail 2: Canonical Tier 1 Validation using OriginKeyManager
        if incoming_meta.tier == AuthorityTier::Tier1Origin {
            let sig = incoming_meta.signature.as_deref().unwrap_or("");
            
            if let Some(km) = key_manager {
                match km.validate_tier1_execution(sig, AuthorityTier::Tier1Origin) {
                    Ok(ExecutionContext::Decoy) => return SystemAction::LockStateDuress,
                    Err(_) => return SystemAction::RejectUnauthorizedTier1,
                    Ok(ExecutionContext::Real) => {}
                }
            } else if sig.trim().is_empty() {
                return SystemAction::RejectUnauthorizedTier1;
            }
        }

        // Guardrail 3: State machine transition matrix based on AuthorityTier ordering
        match (current.meta.tier, incoming_meta.tier) {
            // Tier 1 User Action ALWAYS overwrites lower tiers
            (_, AuthorityTier::Tier1Origin) => SystemAction::Overwrite,

            // Tier 2 Delegated overrides Tier 3 Observer, but cannot overwrite Tier 1 Origin
            (AuthorityTier::Tier3Observer, AuthorityTier::Tier2Delegated) => SystemAction::Overwrite,
            (AuthorityTier::Tier1Origin, AuthorityTier::Tier2Delegated) => SystemAction::RejectStalePayload,

            // Tier 3 Ingest vs Tier 3 Ingest
            (AuthorityTier::Tier3Observer, AuthorityTier::Tier3Observer) => {
                if current.value == *incoming_value {
                    SystemAction::Commit
                } else {
                    SystemAction::FlagConflict
                }
            }

            // Lower tiers (Tier 3 / Tier 2) cannot overwrite higher committed states
            (AuthorityTier::Tier1Origin | AuthorityTier::Tier2Delegated, AuthorityTier::Tier3Observer) => {
                SystemAction::RejectStalePayload
            }

            // Equal Tier 2 handling
            (AuthorityTier::Tier2Delegated, AuthorityTier::Tier2Delegated) => SystemAction::Overwrite,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_state(tier: AuthorityTier, timestamp: u64, val: &str) -> AttributeState<String> {
        AttributeState {
            value: val.to_string(),
            meta: MetadataHeader {
                entity_id: "usr_01".into(),
                attribute: "phone_number".into(),
                tier,
                timestamp_utc: timestamp,
                signature: None,
            },
        }
    }

    #[test]
    fn test_tier1_overwrites_tier3() {
        let current = mock_state(AuthorityTier::Tier3Observer, 1000, "+15550000");
        let incoming_meta = MetadataHeader {
            entity_id: "usr_01".into(),
            attribute: "phone_number".into(),
            tier: AuthorityTier::Tier1Origin,
            timestamp_utc: 1005,
            signature: Some("valid_sig".into()),
        };
        let action = Evaluator::evaluate(&current, &incoming_meta, &"+15559999".to_string(), None);
        assert_eq!(action, SystemAction::Overwrite);
    }

    #[test]
    fn test_tier1_missing_signature_rejected() {
        let current = mock_state(AuthorityTier::Tier3Observer, 1000, "+15550000");
        let incoming_meta = MetadataHeader {
            entity_id: "usr_01".into(),
            attribute: "phone_number".into(),
            tier: AuthorityTier::Tier1Origin,
            timestamp_utc: 1005,
            signature: None,
        };
        let action = Evaluator::evaluate(&current, &incoming_meta, &"+15559999".to_string(), None);
        assert_eq!(action, SystemAction::RejectUnauthorizedTier1);
    }
}