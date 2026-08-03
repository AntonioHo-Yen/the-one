use serde::{Deserialize, Serialize};

/// System Authority Tiers as defined by the Charter
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Tier {
    Tier3Automated = 3,
    Tier2HumanOperator = 2,
    Tier1UserDirect = 1,
}

/// Evaluation outcomes returned by the State Engine
#[derive(Debug, PartialEq, Eq)]
pub enum SystemAction {
    /// Overwrite existing state immediately (Tier 1 > Tier 3/2 or Tier 2 > Tier 3)
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
    pub tier: Tier,
    pub timestamp_utc: u64,
    pub signature: Option<String>,
    pub is_duress: Option<bool>, // Quick flag for duress PINs
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
    pub fn evaluate<T: PartialEq>(
        current: &AttributeState<T>,
        incoming_meta: &MetadataHeader,
        incoming_value: &T,
    ) -> SystemAction {
        // Guardrail: Enforce Rule 3 - Reject stale Tier 3 payloads immediately
        if incoming_meta.tier == Tier::Tier3Automated 
            && incoming_meta.timestamp_utc <= current.meta.timestamp_utc 
        {
            return SystemAction::RejectStalePayload;
        }

        // Tier 1 Validation Guardrails
        if incoming_meta.tier == Tier::Tier1UserDirect {
            if incoming_meta.is_duress == Some(true) {
                return SystemAction::LockStateDuress;
            }
            if incoming_meta.signature.is_none() {
                return SystemAction::RejectUnauthorizedTier1;
            }
        }

        // Evaluate state machine transition matrix based on authority hierarchy
        match (current.meta.tier, incoming_meta.tier) {
            // Tier 1 User Action ALWAYS overwrites lower tiers
            (_, Tier::Tier1UserDirect) => SystemAction::Overwrite,

            // Tier 2 Operator Override overwrites Tier 3, but cannot overwrite newer Tier 1
            (Tier::Tier3Automated, Tier::Tier2HumanOperator) => SystemAction::Overwrite,
            (Tier::Tier1UserDirect, Tier::Tier2HumanOperator) => SystemAction::RejectStalePayload,

            // Tier 3 Ingest vs Tier 3 Ingest
            (Tier::Tier3Automated, Tier::Tier3Automated) => {
                if current.value == *incoming_value {
                    SystemAction::Commit
                } else {
                    SystemAction::FlagConflict
                }
            }

            // Automated Tier 3 cannot overwrite Tier 1 or Tier 2 committed states
            (Tier::Tier1UserDirect | Tier::Tier2HumanOperator, Tier::Tier3Automated) => {
                SystemAction::RejectStalePayload
            }

            // Equal Tier 2 handling
            (Tier::Tier2HumanOperator, Tier::Tier2HumanOperator) => SystemAction::Overwrite,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_state(tier: Tier, timestamp: u64, val: &str) -> AttributeState<String> {
        AttributeState {
            value: val.to_string(),
            meta: MetadataHeader {
                entity_id: "usr_01".into(),
                attribute: "phone_number".into(),
                tier,
                timestamp_utc: timestamp,
                signature: None,
                is_duress: None,
            },
        }
    }

    #[test]
    fn test_tier1_overwrites_tier3() {
        let current = mock_state(Tier::Tier3Automated, 1000, "+15550000");
        let incoming_meta = MetadataHeader {
            entity_id: "usr_01".into(),
            attribute: "phone_number".into(),
            tier: Tier::Tier1UserDirect,
            timestamp_utc: 1005,
            signature: Some("ed25519_sig".into()),
            is_duress: None,
        };

        let action = Evaluator::evaluate(&current, &incoming_meta, &"+15559999".to_string());
        assert_eq!(action, SystemAction::Overwrite);
    }

    #[test]
    fn test_tier1_missing_signature_rejected() {
        let current = mock_state(Tier::Tier3Automated, 1000, "+15550000");
        let incoming_meta = MetadataHeader {
            entity_id: "usr_01".into(),
            attribute: "phone_number".into(),
            tier: Tier::Tier1UserDirect,
            timestamp_utc: 1005,
            signature: None,
            is_duress: None,
        };

        let action = Evaluator::evaluate(&current, &incoming_meta, &"+15559999".to_string());
        assert_eq!(action, SystemAction::RejectUnauthorizedTier1);
    }

    #[test]
    fn test_stale_tier3_rejected() {
        let current = mock_state(Tier::Tier3Automated, 1000, "+15550000");
        let incoming_meta = MetadataHeader {
            entity_id: "usr_01".into(),
            attribute: "phone_number".into(),
            tier: Tier::Tier3Automated,
            timestamp_utc: 900,
            signature: None,
            is_duress: None,
        };

        let action = Evaluator::evaluate(&current, &incoming_meta, &"+15558888".to_string());
        assert_eq!(action, SystemAction::RejectStalePayload);
    }

    #[test]
    fn test_conflicting_tier3_flags_review() {
        let current = mock_state(Tier::Tier3Automated, 1000, "+15550000");
        let incoming_meta = MetadataHeader {
            entity_id: "usr_01".into(),
            attribute: "phone_number".into(),
            tier: Tier::Tier3Automated,
            timestamp_utc: 1005,
            signature: None,
            is_duress: None,
        };

        let action = Evaluator::evaluate(&current, &incoming_meta, &"+15559999".to_string());
        assert_eq!(action, SystemAction::FlagConflict);
    }
}