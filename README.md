# The One (TO1) Protocol & State Engine

**The One** is a deterministic, authority-tiered state engine written in Rust. It acts as an immutable gatekeeper for state updates, guaranteeing that human user intent takes absolute precedence over background scripts, automated integrations, and conflicting administrative overrides. Eliminating redundant PII forms, preventing silent data overwrites, and giving individuals true digital state sovereignty.

*"Created out of pure developer laziness—because I got tired of re-typing my info into endless random web forms."*
---

## Key Features

* **Authority Hierarchy:** Enforces explicit privilege tiers across all incoming data payloads:
    **Origin Anchor:** Immutable cryptographic root establishing identity genesis, key ownership, and deterministic recovery (anchored via device hardware, passkeys, or fuzzy biometrics).
  * **Tier 1 (User Direct):** Absolute precedence. Overwrites lower tiers instantly.
  * **Tier 2 (Human Operator):** Administrative overrides. Overwrites automated data, but cannot supersede active Tier 1 state.
  * **Tier 3 (Automated Ingest):** Background syncs and webhooks. Subject to strict timestamp checking and conflict detection.
* **Deterministic Conflict Resolution:** Conflicting Tier 3 updates automatically flag the target attribute for operator review (`FlagConflict`) without corrupting or halting the rest of the application state.
* **Stale Payload Protection:** Automatically discards out-of-order or outdated payloads using strict UTC timestamp bounds.

---

## Core Architecture

Incoming Payload (Claiming Tier 1)
         │
         ▼
 ┌────────────────────────────────────────────────────────┐
 │ Liveness Proof (Option 1/2 + Argon2id PIN)             │
 └───────────────────────┬────────────────────────────────┘
                         │
                         ├─► Missing Liveness Proof? ──────► YES ──► [RejectUnauthorizedTier1]
                         │
                         ├─► Duress PIN Detected? ─────────► YES ──► [LockStateDuress]
                         │
                         ▼ (Successfully Unlocks)
 ┌────────────────────────────────────────────────────────┐
 │                  Origin Root Key                       │
 └───────────────────────┬────────────────────────────────┘
                         │
                         ▼ (Generates Valid Signature)
 ┌────────────────────────────────────────────────────────┐
 │           Tier 1 User Direct Precedence                │
 └───────────────────────┬────────────────────────────────┘
                         │
                         ▼
 ┌────────────────────────────────────────────────────────┐
 │ Evaluator::evaluate()                                  │
 └───────────────────────┬────────────────────────────────┘
                         │
                         ├─► Valid Signature + Tier 1? ───► YES ──► [SystemAction::Overwrite]
                         │
                         └─► Stale Timestamp? ────────────► YES ──► [RejectStalePayload]

## Quickstart

### Prerequisites
* Rust 1.80+ / Cargo

### Running Tests

Clone the repository and run the unit test suite:

```bash
git clone [https://github.com/AntonioHo-Yen/the-one.git](https://github.com/AntonioHo-Yen/the-one.git)
cd the-one
cargo test
```

---

## Usage Example

```rust
use the_one::evaluator::{AttributeState, Evaluator, MetadataHeader, SystemAction, Tier};

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
            is_duress: None,
        },
    };

    // Incoming payload authenticated via Origin Key
    let user_update_meta = MetadataHeader {
        entity_id: "usr_101".into(),
        attribute: "phone_number".into(),
        tier: Tier::Tier1UserDirect,
        timestamp_utc: 1005,
        signature: Some("ed25519_origin_signature".into()),
        is_duress: Some(false),
    };

    // Evaluate transition
    let action = Evaluator::evaluate(
        &current_state,
        &user_update_meta,
        &"555-9999".to_string(),
    );

    match action {
        SystemAction::Overwrite => println!("Tier 1 Origin State committed immediately."),
        SystemAction::RejectUnauthorizedTier1 => println!("Rejected: Missing Tier 1 Origin signature."),
        SystemAction::LockStateDuress => println!("Duress detected: Local state locked defensively."),
        SystemAction::FlagConflict => println!("Field locked for review."),
        SystemAction::RejectStalePayload => println!("Payload dropped."),
        SystemAction::Commit => println!("Standard update committed."),
    }
}
```