# The One

**The One** is a deterministic, authority-tiered state engine written in Rust. It acts as an immutable gatekeeper for state updates, guaranteeing that human user intent takes absolute precedence over background scripts, automated integrations, and conflicting administrative overrides.

---

## Key Features

* **Authority Hierarchy:** Enforces explicit privilege tiers across all incoming data payloads:
  * **Tier 1 (User Direct):** Absolute precedence. Overwrites lower tiers instantly.
  * **Tier 2 (Human Operator):** Administrative overrides. Overwrites automated data, but cannot supersede active Tier 1 state.
  * **Tier 3 (Automated Ingest):** Background syncs and webhooks. Subject to strict timestamp checking and conflict detection.
* **Deterministic Conflict Resolution:** Conflicting Tier 3 updates automatically flag the target attribute for operator review (`FlagConflict`) without corrupting or halting the rest of the application state.
* **Stale Payload Protection:** Automatically discards out-of-order or outdated payloads using strict UTC timestamp bounds.

---

## Core Architecture

Incoming Payload + Metadata
         │
         ▼
 ┌────────────────┐
 │  The One Engine│ ──► Evaluates Tier & UTC Timestamp
 └───────┬────────┘
         │
 ┌───────┴───────────────────────────────────────────┐
 │                                                   │
 ▼                                                   ▼
[SystemAction::Overwrite]                 [SystemAction::FlagConflict]
(Immediate State Mutation)                (Lock Field & Route to Queue)


git clone [https://github.com/YOUR_USERNAME/the-one.git](https://github.com/YOUR_USERNAME/the-one.git)
cd the-one
cargo test

use the_one::evaluator::{Evaluator, AttributeState, MetadataHeader, Tier, SystemAction};

fn main() {
    // Current state set by automated background sync
    let current_state = AttributeState {
        value: "555-0000".to_string(),
        meta: MetadataHeader {
            entity_id: "usr_101".into(),
            attribute: "phone_number".into(),
            tier: Tier::Tier3Automated,
            timestamp_utc: 1000,
            signature: None,
        },
    };

    // Incoming payload from end user form submission
    let user_update_meta = MetadataHeader {
        entity_id: "usr_101".into(),
        attribute: "phone_number".into(),
        tier: Tier::Tier1UserDirect,
        timestamp_utc: 1005,
        signature: Some("ed25519_signature_hash".into()),
    };

    // Evaluate transition
    let action = Evaluator::evaluate(&current_state, &user_update_meta, &"555-9999".to_string());

    match action {
        SystemAction::Overwrite => println!("State updated immediately."),
        SystemAction::FlagConflict => println!("Field locked for review."),
        SystemAction::RejectStalePayload => println!("Payload dropped."),
        SystemAction::Commit => println!("Standard update committed."),
    }
}