# The One (TO1) Protocol & State Engine

**The One** is a deterministic, origin-authoritative state engine and zero-trust egress gate written in Rust. It acts as an immutable gatekeeper for state updates and network transmissions, guaranteeing that intent originating from The One takes absolute precedence over background scripts, automated integrations, and conflicting administrative overrides. It eliminates redundant data entry, prevents silent data overwrites, and enforces self-sovereign cryptographic control at the system boundary.

*"Created out of pure developer laziness—because I got tired of re-typing my info into endless random web forms."*
---

## Key Features

* **Authority Hierarchy (AuthorityTier):** Enforces explicit privilege tiers across all incoming data payloads:

  * **Origin Anchor:** mmutable cryptographic root establishing identity genesis, key ownership, and deterministic recovery (anchored via non-custodial seed phrases, Shamir thresholds, or fuzzy biometrics).

  * **Tier 1 (Tier1Origin):** Absolute precedence representing direct, un-delegated intent from The One. Overwrites lower tiers instantly.

  * **Tier 2 (Tier2Delegated):** Delegated operator roles and parallel role stacks(`ParallelRoleStack`), Administrative overrides. Overwrites automated data, but cannot supersede active Tier 1 state.

  * **Tier 3 (Tier3Observer):** Automated background syncs, webhooks, and telemetry ingestion. Subject to strict timestamp checking and conflict detection.

* **Zero-Trust Egress & Monero Monetization:** Intercepts outbound socket traffic at the network edge. Unauthenticated queries are held behind Monero-settled 402 invoices and local Tier 1 dual-factor release

* **Deterministic Conflict Resolution:** Conflicting Tier 3 updates automatically flag the target attribute for operator review (`SystemAction::FlagConflict`) without corrupting or halting application state.

* **Stale Payload Protection:** Automatically discards out-of-order or outdated payloads using strict UTC timestamp bounds (`SystemAction::RejectStalePayload`).

* **Duress Isolation Context:** Triggering a configured Duress PIN or share switches system execution directly into decoy mode (ExecutionContext::Decoy) and locks state defensively (`SystemAction::LockStateDuress`).

---

## Core Architecture

```text
Incoming Payload (Claiming Tier 1)
         │
         ▼
 ┌────────────────────────────────────────────────────────┐
 │ Liveness Proof (Vascular/Micro-sample + Argon2id PIN)  │
 └───────────────────────┬────────────────────────────────┘
                         │
                         ├─► Missing Liveness Proof? ──────► YES ──► [RejectUnauthorizedTier1]
                         │
                         ├─► Duress PIN Detected? ─────────► YES ──► [LockStateDuress -> ExecutionContext::Decoy]
                         │
                         ▼ (Successfully Unlocks)
 ┌────────────────────────────────────────────────────────┐
 │                  Origin Root Key                       │
 └───────────────────────┬────────────────────────────────┘
                         │
                         ▼ (Generates Valid Signature)
 ┌────────────────────────────────────────────────────────┐
 │           Tier 1 Direct Precedence                     │
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
```


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
use the_one::authority::Tier1KeyManager;
use the_one::state::evaluator::{AttributeState, Evaluator, MetadataHeader, SystemAction};
use the_one::authority::AuthorityTier;

fn main() {
    // Current state set by automated background sync (Tier 3)
    let current_state = AttributeState {
        value: "555-0000".to_string(),
        meta: MetadataHeader {
            entity_id: "entity_101".into(),
            attribute: "phone_number".into(),
            tier: AuthorityTier::Tier3Observer,
            timestamp_utc: 1000,
            signature: None,
            is_duress: None,
        },
    };

    // Incoming payload authenticated via Tier 1 Key Manager
    let user_update_meta = MetadataHeader {
        entity_id: "entity_101".into(),
        attribute: "phone_number".into(),
        tier: AuthorityTier::Tier1Origin,
        timestamp_utc: 1005,
        signature: Some("ed25519_tier1_signature".into()),
        is_duress: Some(false),
    };

    // Evaluate state transition matrix
    let action = Evaluator::evaluate(
        &current_state,
        &user_update_meta,
        &"555-9999".to_string(),
    );

    match action {
        SystemAction::Overwrite => println!("Tier 1 Origin state committed immediately."),
        SystemAction::RejectUnauthorizedTier1 => println!("Rejected: Missing Tier 1 signature."),
        SystemAction::LockStateDuress => println!("Duress detected: State locked and ExecutionContext::Decoy initialized."),
        SystemAction::FlagConflict => println!("Field flagged for operator review."),
        SystemAction::RejectStalePayload => println!("Stale payload dropped."),
        SystemAction::Commit => println!("Standard update committed."),
    }
}
```