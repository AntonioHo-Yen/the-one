# `The-One(TO1)` Architecture Specification: Evaluator & Egress Control Engine

**Version:** 1.0.0  
**Status:** Architecture Draft / Active Spec  
**Target Platform:** Linux Workstation (`cobraLair` / Arch Linux Environment)  
**Core Language:** Rust (`edition = "2021"`)  

---

## 1. Executive Summary & Core Mission

`the-one` is an autonomous, origin-authoritative system built on the principle of **absolute local data sovereignty**. Rather than relying on third-party policies, legal requests, or remote database scraping to protect data, `the-one` establishes a **Zero-Trust Egress Gate** at the local system boundary.

The core objective of the **Evaluator Engine** (`state::evaluator::Evaluator`) is to intercept, deserialize, and inspect all outbound data transmissions in real time. If an outgoing payload contains sensitive data—such as Personally Identifiable Information (PII), device fingerprints, or state keys—without explicit, cryptographically signed Tier 1 authorization from `The One`, the Evaluator blocks the network payload instantly before a single byte leaves the machine.

---

## 2. Structural Authority Hierarchy

System operations and payload state transitions are governed strictly by a three-tiered authority model anchored to an Origin root:

```
┌───────────────────────────────────────────────────────────┐
│              ORIGIN KEY (Genesis Anchor)                  │
│  - One-time bootstrapping secret                          │
│  - Generates & signs operational Tier1Origin Key          │
│  - Discarded / Zeroized permanently after initialization  │
└─────────────────────────────┬─────────────────────────────┘
                              │ (One-Time Execution)
                              ▼
┌───────────────────────────────────────────────────────────┐
│          TIER 1: AuthorityTier::Tier1Origin               │
│  - Absolute operational authority belonging to The One    │
│  - Executes high-privilege operations & selective grants  │
│  - Integrated directly with DURESS FLAGS & Decoy Contexts │
└─────────────────────────────┬─────────────────────────────┘
                              │
                              ▼
┌───────────────────────────────────────────────────────────┐
│        TIER 2 & 3: Delegated & Observer Tiers             │
│  - Tier 2 (Tier2Delegated): Sub-routines & Parallel Stacks│
│  - Tier 3 (Tier3Observer): Read-only ingest & telemetry   │
└───────────────────────────────────────────────────────────┘
```

### 2.1 Origin Key Genesis (`OriginBootstrapper`)
* **Role:** One-time root identity used strictly during initial system setup to anchor identity genesis.
* **Lifecycle:** Issues the operational Tier 1 Key (`AuthorityTier::Tier1Origin`), signs the dual-attestation genesis commitment, and is immediately consumed/zeroized in memory. It never touches network sockets or daily runtime processes.

### 2.2 Tier 1 Key (`Tier1KeyManager`)
* **Role:** The active, operational root key used for daily system operations, rule signing, and delegating sub-routines via `ParallelRoleStack`.
* **Duress Integration:** Because Tier 1 is exposed to daily operations, it holds the primary **Duress Override Flag**. If triggered, all high-privilege executions and key releases are rejected immediately.

---

## 3. Serialization, Deserialization, & Network Boundary

Network interfaces and transport protocols operate strictly on flat streams of raw bytes (`0s` and `1s`). For the Evaluator to inspect network traffic effectively, it relies on strict **Serialization and Deserialization** boundaries.

```
[ Active Memory Structs ] ──► ( Serialize ) ──► [ Flat Bytes / JSON / Proto ]
                                                       │
                                                       ▼
                                            ┌─────────────────────┐
                                            │  EVALUATOR ENGINE   │
                                            │ (Egress Inspection) │
                                            └──────────┬──────────┘
                                                       │
                                 ┌─────────────────────┴─────────────────────┐
                                 ▼                                           ▼
                     [ Tier 1 Consent Valid ]                   [ Unauthorized / Duress ]
                                 │                                           │
                                 ▼                                           ▼
                        ( Transmit to Wire )                      ( SystemAction::RejectPayload )
```

1. **Serialization (Outbound Conversion):** Programs convert live RAM data structures into flat byte formats (e.g., JSON, Protocol Buffers, BSON) for transmission over network sockets.

2. **Egress Interception:** The Evaluator intercepts serialized payloads at the system egress boundary before they hit external physical network interfaces.

3. **Deserialization & Deep Inspection:** The Evaluator parses/deserializes the payload back into structured, strongly-typed fields to verify against active PII hashes, Monero cryptographic proofs (get_spend_proof, get_reserve_proof), and active Tier 1 authorization signatures.
---

## 4. Evaluator Engine Logic & Egress Rules

The Evaluator functions as a deterministic data loss prevention (DLP) engine and local privacy firewall (state::evaluator::Evaluator).

### 4.1 Scope & Action Matrix

| Category | Mechanics | System Action |
| :--- | :--- | :--- |
| **Real-Time Telemetry & Tracking** | Intercepts background analytics, tracking pixels, and unauthorized third-party telemetry calls (`AuthorityTier::Tier3Observer`). | `SystemAction::RejectStalePayload` or `RejectUnauthorized` |

| **Unauthorized Data Exfiltration** | Blocks unauthorized outbound transmissions containing recognized PII or system signatures lacking Tier 1 signatures. | `SystemAction::RejectUnauthorizedTier1` |

| **Duress Activation** | Evaluates payload during active duress signal trigger. | `SystemAction::LockStateDuress` -> `ExecutionContext::Decoy` |

| **Authorized Third-Party Queries** | Monetized remote access matching Monero subaddress settlement and dual-factor Tier 1 authorization. | `SystemAction::Overwrite` / `DeliverPayload` |

### Data Licensing Header & Monero Attestation Injection
For authorized outbound requests (e.g., voluntary third-party attribute queries), the Evaluator injects machine-readable licensing metadata headers into the request payload:

* Declares single-use processing rights cryptographically bound to strict time expiration bounds (timestamp_utc).

* Contains Monero challenge authentication nonces (sign_message).

* Explicitly forbids downstream data resale, profiling, or persistent database storage, stripping cryptographic proof of accuracy from un-monetized or un-authorized copies across the network.


---

## 5. Architectural File Hierarchy (`src/`)

```text
src/
├── lib.rs                      # Core library root, feature flags, and crate exports
├── main.rs                     # Runtime entry point & CLI daemon harness
│
├── authority/                  # Authority Tier Enforcement & Bootstrapping
│   ├── mod.rs                  # Authority module gatekeeping & tier definitions
│   └── origin.rs               # OriginBootstrapper & Tier1KeyManager implementation
│
├── crypto/                     # Cryptographic Utilities & Backup Mechanisms
│   ├── mod.rs                  # Crypto module root
│   ├── biometrics.rs           # Ephemeral sensor key derivation & fuzzy extractors
│   └── shamir.rs               # Multi-party secret sharing & seed recovery logic
│
├── Desktop/                    # Local Client Intercept & DOM Sensor Components
│   ├── mod.rs                  # Desktop module root
│   ├── injector.rs             # DOM field auto-fill & attestation payload injector
│   ├── reader.rs               # Local state/DOM observer and reader interface
│   └── sensor.rs               # Local hardware sensor listener & liveness trigger
│
├── egress/                     # Zero-Trust Network Boundary & Gateway
│   ├── mod.rs                  # Egress module root
│   ├── gate.rs                 # Outbound network interceptor & DLP firewall
│   ├── invoice.rs              # Monero 402 Payment Required & invoice generator
│   └── monero.rs               # Monero RPC, mempool scanner, & proof verification
│
└── state/                      # State Engine & Transition Evaluator
    ├── mod.rs                  # State module root & storage definitions
    └── evaluator.rs            # StateEvaluator engine & SystemAction transition matrix
```

---

## 6. Implementation Roadmap

1. **`src/authority/origin.rs`:** Finalize `OriginBootstrapper` (one-time burn) and `Tier1KeyManager` (`AuthorityTier::Tier1Origin` operational root with `ExecutionContext::Decoy` support).

2. **`src/authority/evaluator.rs`:** Build the egress payload parser and transition matrix (`SystemAction`) for PII hashes, Monero challenge nonces, and Tier 1 authorization signatures.

3. **`src/authority/mod.rs`:** Wire module gatekeeping rules to enforce compile-time boundary isolation and `JobClass::TheOne` priority enforcement.