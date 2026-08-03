# `the-one` Architecture Specification: Evaluator & Egress Control Engine

**Version:** 1.0.0 
**Status:** Architecture Draft / Active Spec 
**Target Platform:** Linux Workstation (`cobraLair` / Arch Linux Environment) 
**Core Language:** Rust (`edition = "2021"`) 
---

## 1. Executive Summary & Core Mission

`the-one` is an autonomous, privacy-first system built on the principle of **absolute local data sovereignty**. Rather than relying on third-party policies, legal requests, or remote database scraping to protect user data, `the-one` establishes a **Zero-Trust Egress Gate** at the local system boundary.

The core objective of the **Evaluator Engine** is to intercept, deserialize, and inspect all outbound data transmissions in real time. If an outgoing payload contains sensitive data—such as Personally Identifiable Information (PII), device fingerprints, or state keys—without explicit, cryptographically signed Tier 1 authorization, the Evaluator blocks the network payload instantly before a single byte leaves the machine.

---

## 2. Structural Authority Hierarchy

The system enforces a strict two-stage cryptographic authority chain, separating one-time bootstrapping from daily operational control.

```
┌───────────────────────────────────────────────────────────┐
│                    ORIGIN KEY (Genesis)                   │
│  - One-time bootstrapping secret                          │
│  - Generates & signs operational Tier 1 Key               │
│  - Discarded / Zeroized permanently after initialization  │
└─────────────────────────────┬─────────────────────────────┘
                              │ (One-Time Execution)
                              ▼
┌───────────────────────────────────────────────────────────┐
│               TIER 1 KEY (Daily Operational)              │
│  - Active root key for high-privilege operations          │
│  - Delegates Tier 2 automated sub-routines                │
│  - Integrated directly with DURESS FLAGS                  │
└───────────────────────────────────────────────────────────┘
```

### 2.1 Origin Key (`OriginBootstrapper`)
* **Role:** One-time root identity used strictly during initial system setup.
* **Lifecycle:** Issues the operational Tier 1 Key, signs the genesis configuration, and is immediately consumed/zeroized in memory. It never touches network sockets or daily runtime processes.

### 2.2 Tier 1 Key (`Tier1KeyManager`)
* **Role:** The active, operational root key used for daily system operations, rule signing, and delegating sub-routines.
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
                        ( Transmit to Wire )                      ( BLOCK Payload & Log )
```

1. **Serialization (Outbound Conversion):** Programs convert live RAM data structures into flat byte formats (e.g., JSON, Protocol Buffers, BSON) for transmission over network sockets.
2. **Egress Interception:** The Evaluator intercepts serialized payloads at the system egress boundary before they hit external physical network interfaces.
3. **Deserialization & Deep Inspection:** The Evaluator parses/deserializes the payload back into structured, strongly-typed fields to verify against active PII hashes and Tier 1 policy rules.

---

## 4. Evaluator Engine Logic & Egress Rules

The Evaluator functions as a deterministic data loss prevention (DLP) engine and local privacy firewall.

### 4.1 Scope & Effectiveness

| Category | Mechanics | Effectiveness |
| :--- | :--- | :--- |
| **Real-Time Telemetry & Tracking** | Intercepts background analytics, tracking pixels, and unauthorized third-party telemetry calls. | **95%+ (Blocked Pre-Execution)** |
| **New Data Exfiltration** | Blocks unauthorized outbound transmissions containing recognized PII or system signatures. | **100% (Pre-Wire Drop)** |
| **Historical Data Leaks** | Data previously disclosed to external databases prior to system deployment. | **Out of Scope (Requires Local Sphere Focus)** |

### 4.2 Data Licensing Header Injection
For authorized outbound requests (e.g., voluntary e-commerce transactions), the Evaluator injects machine-readable licensing metadata headers into the request payload:
* Declares single-use processing rights.
* Explicitly forbids downstream data resale, profiling, or persistent database storage.
* Establishes cryptographic proof of non-consent for unauthorized retention.

---

## 5. Architectural File Hierarchy (`src/`)

```text
src/
├── lib.rs                   # Core module declarations & public API export
├── error.rs                 # Custom ProtocolError definitions
├── authority/
│   ├── mod.rs               # Gatekeeper module & tier exposure
│   ├── origin.rs            # OriginBootstrapper & Tier1KeyManager structs
│   └── evaluator.rs         # Egress inspection matrix & PII policy engine
└── crypto/
    ├── mod.rs               # Isolated cryptographic backend wrappers
    └── monero.rs            # C++ memory safety containment layer
```

---

## 6. Implementation Roadmap

1. **`src/authority/origin.rs`:** Finalize `OriginBootstrapper` (one-time burn) and `Tier1KeyManager` (duress-enabled operational root).
2. **`src/authority/evaluator.rs`:** Build the egress payload parser and pattern matching engine for PII hashes and authorization signatures.
3. **`src/authority/mod.rs`:** Wire module gatekeeping rules to enforce compile-time boundary isolation.
