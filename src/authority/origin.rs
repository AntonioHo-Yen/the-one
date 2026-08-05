//! # Authority & Taxonomy Architecture (`src/authority/origin.rs`)
//!
//! ## 1. Core Principles
//! * **Identity vs. Role Decoupling:** Every entity fundamentally exists at `JobClass::Indigenous` (Level 0) as an un-delegated, sovereign baseline human identity.
//! * **Ephemeral Parallel Stacks:** Elevated operational roles (e.g., `SoftwareEngineering`, `Executive`, `MunicipalPolice`) are attached dynamically via `ParallelRoleStack` and `ActiveContext`.
//! * **Zero-Lockout Lifecycle:** When an elevated role expires, is revoked, or is collapsed, the active session cleanly reverts back to `JobClass::Indigenous` without altering or destroying the underlying origin identity.
//!
//! ---
//!
//! ## 2. Global Priority Taxonomy Hierarchy (`rank_level`)
//! The `JobClass` enum derives `PartialOrd` and `Ord` to allow direct comparison operators (`<`, `>`). Lower numerical rank levels represent higher structural precedence:
//!
//! | Level | Category Name | Primary Variants / Sectors |
//! | :--- | :--- | :--- |
//! | **0** | **Inherent Baseline** | `Indigenous` (Un-delegated sovereign human precedence) |
//! | **1** | **Education & Knowledge** | `EducationAndAcademia` |
//! | **2** | **Healthcare & Sanitation** | `PhysiciansAndSurgeons`, `HealthcareAndClinical`, `ScientificResearch`, `SanitationAndWasteManagement`, `EnvironmentalAndPublicWorks` |
//! | **3** | **Infrastructure Builders** | `InfrastructureOps`, `EnergyAndUtilities`, `SkilledTrades` |
//! | **4** | **Cyber, Systems & Coding** | `SecurityOfficer`, `SoftwareEngineering`, `DataAndAnalytics`, `QualityAssuranceAndTesting`, `FrontEndAndWebDev`, `UserExperienceAndDesign` |
//! | **5** | **Military & Defense** | `MilitaryActiveDuty`, `MilitaryReserveAndVeterans`, `DefenseAndIntelligence`, `SecurityAndTacticalOps` |
//! | **6** | **Governing Body & Legal** | `LegislativeAndPolicy`, `FederalAgenciesAndIntelligence`, `Executive`, `LegalAndRegulatory` |
//! | **7** | **Law Enforcement & Safety** | `MunicipalPolice`, `SheriffAndCorrections`, `StatePoliceAndTroopers`, `FirefightingAndRescue`, `AmbulanceAndEmergencyMedical` |
//! | **8** | **Operations & HR** | `OperationsManagement`, `TreasuryOps`, `HumanResourcesAndRecruitment` |
//! | **9** | **Freight & Industrial** | `CommercialFreightAndTrucking`, `RailAviationAndMaritime`, `WarehousingAndFulfillment`, `ForkliftAndHeavyEquipment`, `ManufacturingAndAssembly` |
//! | **10** | **Commerce & Services** | `SalesAndCommerce`, `RealEstateAndAssets`, `PassengerTransitAndBus`, `TaxiAndRideshare`, `GasStationAndConvenience`, `RetailAndMerchant`, `CustodialAndFacilities`, `PersonalServices` |
//! | **11** | **Media, Arts & Agriculture** | `AudioAndBroadcasting`, `FilmAndVisualArts`, `PerformingArts`, `ContentCreation`, `GamingAndEsports`, `AthleticsAndSports`, `HospitalityAndCulinary`, `AgricultureAndMaritime` |
//! | **12** | **Open Extension** | `Custom { sector, title, code }` |
//!
//! ---
//!
//! ## 3. Parked/Dormant Stack Management Pattern
//!
//! ### Storing vs. Activating Role Stacks
//! Unused role assignments can be stored as dormant data (`Vec<ParallelRoleStack>`) on disk or in local memory, serializing cleanly via Serde.
//!
//! ```rust
//! // A. Elevate session with a specific JobClass role stack
//! let dev_role = ParallelRoleStack {
//!     classification: Tier1Classification::Corporate {
//!         entity_id: "CORP-01".to_string(),
//!         title: "Systems Architect".to_string(),
//!         job_class: JobClass::SoftwareEngineering,
//!     },
//!     permissions: vec!["sys:deploy".to_string()],
//!     expires_at_epoch_secs: 1800000000,
//! };
//!
//! active_context.elevate_role(dev_role);
//!
//! // B. Return active context back to baseline (Indigenous)
//! active_context.collapse_to_baseline();
//!
//! // C. Re-attach a parked/unused stack whenever needed
//! if let Some(parked_stack) = dormant_role_inventory.pop() {
//!     active_context.elevate_role(parked_stack);
//! }
//! ```

use std::time::{Duration, Instant};
use crate::error::ProtocolError;
use crate::crypto::biometrics::VascularHelperData;
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Default duress lockout cooling duration (48 hours) if none is specified.
pub const DEFAULT_DURESS_DURATION: Duration = Duration::from_secs(172_800);

/// Defines the clear hierarchical authority tiers in TO1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AuthorityTier {
    Tier1Origin,    // Human precedence / Immutable Origin Key
    Tier2Delegated, // Automated sub-routines / Delegated keys
    Tier3Observer,  // Read-only / Audit agents
}

/// Comprehensive Global Job Classes ordered by custom priority hierarchy.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum JobClass {
    // =========================================================================
    // LEVEL 0: INHERENT HUMAN PRECEDENCE
    // =========================================================================
    /// Baseline Sovereign Human capacity; un-delegated and un-revokable baseline.
    Indigenous = 0,

    // =========================================================================
    // LEVEL 1: EDUCATORS & ACADEMIA
    // =========================================================================
    /// Education, Instruction, Knowledge Preservation & Academic Research
    EducationAndAcademia,

    // =========================================================================
    // LEVEL 2: HEALTHCARE, CLINICAL SERVICES, SANITATION & PUBLIC WORKS
    // =========================================================================
    PhysiciansAndSurgeons,
    HealthcareAndClinical,
    ScientificResearch,
    SanitationAndWasteManagement,
    EnvironmentalAndPublicWorks,

    // =========================================================================
    // LEVEL 3: INFRASTRUCTURE BUILDERS, ENERGY & TRADES
    // =========================================================================
    InfrastructureOps,
    EnergyAndUtilities,
    SkilledTrades,

    // =========================================================================
    // LEVEL 4: CYBERSECURITY, SYSTEMS ENGINEERING & CODING
    // =========================================================================
    SecurityOfficer,
    SoftwareEngineering,
    DataAndAnalytics,
    QualityAssuranceAndTesting,
    FrontEndAndWebDev,
    UserExperienceAndDesign,

    // =========================================================================
    // LEVEL 5: MILITARY, DEFENSE & TACTICAL OPERATIONS
    // =========================================================================
    MilitaryActiveDuty,
    MilitaryReserveAndVeterans,
    DefenseAndIntelligence,
    SecurityAndTacticalOps,

    // =========================================================================
    // LEVEL 6: GOVERNING BODY, LEGISLATIVE & LEGAL
    // =========================================================================
    LegislativeAndPolicy,
    FederalAgenciesAndIntelligence,
    Executive,
    LegalAndRegulatory,

    // =========================================================================
    // LEVEL 7: LAW ENFORCEMENT & FIRST RESPONDERS
    // =========================================================================
    MunicipalPolice,
    SheriffAndCorrections,
    StatePoliceAndTroopers,
    FirefightingAndRescue,
    AmbulanceAndEmergencyMedical,

    // =========================================================================
    // LEVEL 8: OPERATIONS, MANAGEMENT, TREASURY & HR
    // =========================================================================
    OperationsManagement,
    TreasuryOps,
    HumanResourcesAndRecruitment,

    // =========================================================================
    // LEVEL 9: FREIGHT, LOGISTICS & INDUSTRIAL MANUFACTURING
    // =========================================================================
    CommercialFreightAndTrucking,
    RailAviationAndMaritime,
    WarehousingAndFulfillment,
    ForkliftAndHeavyEquipment,
    ManufacturingAndAssembly,

    // =========================================================================
    // LEVEL 10: COMMERCE, TRANSIT, SERVICES & RETAIL
    // =========================================================================
    SalesAndCommerce,
    RealEstateAndAssets,
    PassengerTransitAndBus,
    TaxiAndRideshare,
    GasStationAndConvenience,
    RetailAndMerchant,
    CustodialAndFacilities,
    PersonalServices,

    // =========================================================================
    // LEVEL 11: MEDIA, ENTERTAINMENT, CREATIVE ARTS & AGRICULTURE
    // =========================================================================
    AudioAndBroadcasting,
    FilmAndVisualArts,
    PerformingArts,
    ContentCreation,
    GamingAndEsports,
    AthleticsAndSports,
    HospitalityAndCulinary,
    AgricultureAndMaritime,

    // =========================================================================
    // LEVEL 12: OPEN TAXONOMY (Custom Extension)
    // =========================================================================
    Custom {
        sector: String,
        title: String,
        code: Option<String>,
    },
}

impl Default for JobClass {
    fn default() -> Self {
        JobClass::Indigenous
    }
}

impl JobClass {
    /// Returns the exact priority rank level (0 = Indigenous, 1 = Educator, 2 = Healthcare & Sanitation, etc.)
    pub fn rank_level(&self) -> u8 {
        match self {
            JobClass::Indigenous => 0,
            JobClass::EducationAndAcademia => 1,

            JobClass::PhysiciansAndSurgeons
            | JobClass::HealthcareAndClinical
            | JobClass::ScientificResearch
            | JobClass::SanitationAndWasteManagement
            | JobClass::EnvironmentalAndPublicWorks => 2,

            JobClass::InfrastructureOps
            | JobClass::EnergyAndUtilities
            | JobClass::SkilledTrades => 3,

            JobClass::SecurityOfficer
            | JobClass::SoftwareEngineering
            | JobClass::DataAndAnalytics
            | JobClass::QualityAssuranceAndTesting
            | JobClass::FrontEndAndWebDev
            | JobClass::UserExperienceAndDesign => 4,

            JobClass::MilitaryActiveDuty
            | JobClass::MilitaryReserveAndVeterans
            | JobClass::DefenseAndIntelligence
            | JobClass::SecurityAndTacticalOps => 5,

            JobClass::LegislativeAndPolicy
            | JobClass::FederalAgenciesAndIntelligence
            | JobClass::Executive
            | JobClass::LegalAndRegulatory => 6,

            JobClass::MunicipalPolice
            | JobClass::SheriffAndCorrections
            | JobClass::StatePoliceAndTroopers
            | JobClass::FirefightingAndRescue
            | JobClass::AmbulanceAndEmergencyMedical => 7,

            JobClass::OperationsManagement
            | JobClass::TreasuryOps
            | JobClass::HumanResourcesAndRecruitment => 8,

            JobClass::CommercialFreightAndTrucking
            | JobClass::RailAviationAndMaritime
            | JobClass::WarehousingAndFulfillment
            | JobClass::ForkliftAndHeavyEquipment
            | JobClass::ManufacturingAndAssembly => 9,

            JobClass::SalesAndCommerce
            | JobClass::RealEstateAndAssets
            | JobClass::PassengerTransitAndBus
            | JobClass::TaxiAndRideshare
            | JobClass::GasStationAndConvenience
            | JobClass::RetailAndMerchant
            | JobClass::CustodialAndFacilities
            | JobClass::PersonalServices => 10,

            JobClass::AudioAndBroadcasting
            | JobClass::FilmAndVisualArts
            | JobClass::PerformingArts
            | JobClass::ContentCreation
            | JobClass::GamingAndEsports
            | JobClass::AthleticsAndSports
            | JobClass::HospitalityAndCulinary
            | JobClass::AgricultureAndMaritime => 11,

            JobClass::Custom { .. } => 12,
        }
    }
}

/// Specialized Parallel Role Classifications for Tier 1 Contexts.
/// Allows a baseline human identity to step into elevated institutional roles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier1Classification {
    Sovereign {
        jurisdiction_id: String,
        job_class: JobClass,
    },
    Corporate {
        entity_id: String,
        title: String,
        job_class: JobClass,
    },
    GoverningBody {
        agency_id: String,
        office_title: String,
        job_class: JobClass,
    },
}

/// Ephemeral Parallel Permission Stack.
/// Attached dynamically to an identity during an active role assignment
/// and automatically detached upon termination or expiration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelRoleStack {
    pub classification: Tier1Classification,
    pub permissions: Vec<String>,
    pub expires_at_epoch_secs: u64,
}

impl ParallelRoleStack {
    pub fn is_expired(&self, current_epoch_secs: u64) -> bool {
        current_epoch_secs >= self.expires_at_epoch_secs
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }

    pub fn job_class(&self) -> &JobClass {
        match &self.classification {
            Tier1Classification::Sovereign { job_class, .. } => job_class,
            Tier1Classification::Corporate { job_class, .. } => job_class,
            Tier1Classification::GoverningBody { job_class, .. } => job_class,
        }
    }
}

/// Represents the active execution context combining baseline human identity
/// with an optional elevated parallel role stack.
#[derive(Debug, Clone)]
pub struct ActiveContext {
    pub key_manager: OriginKeyManager,
    pub parallel_stack: Option<ParallelRoleStack>,
}

impl ActiveContext {
    pub fn new(key_manager: OriginKeyManager) -> Self {
        Self {
            key_manager,
            parallel_stack: None,
        }
    }

    pub fn elevate_role(&mut self, stack: ParallelRoleStack) {
        self.parallel_stack = Some(stack);
    }

    pub fn collapse_to_baseline(&mut self) {
        self.parallel_stack = None;
    }

    pub fn active_job_class(&self) -> JobClass {
        self.parallel_stack
            .as_ref()
            .map(|stack| stack.job_class().clone())
            .unwrap_or_default()
    }
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

    pub fn bootstrap_tier1(
        &mut self,
        pubkey: String,
        disarm_hash: String,
        duress_duration: Duration,
    ) -> OriginKeyManager {
        let manager = OriginKeyManager::new(pubkey, disarm_hash, duress_duration);
        self.genesis_id.zeroize();
        manager
    }
}

/// -------------------------------------------------------------------
/// 2. TIER 1 OPERATIONAL KEY MANAGER (Daily Runtime)
/// -------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct OriginKeyManager {
    pub origin_pubkey: String,
    pub duress_triggered_at: Option<Instant>,
    pub duress_duration: Duration,
    pub disarm_secret_hash: String, // PHC-formatted Argon2id hash string
}

/// Helper function to verify a passphrase against an Argon2id PHC hash string.
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
    pub fn new(
        pubkey: impl Into<String>, 
        disarm_secret_hash: impl Into<String>,
        duress_duration: Duration,
    ) -> Self {
        Self {
            origin_pubkey: pubkey.into(),
            duress_triggered_at: None,
            duress_duration,
            disarm_secret_hash: disarm_secret_hash.into(),
        }
    }
    
    pub fn with_default_duration(
        pubkey: impl Into<String>,
        disarm_secret_hash: impl Into<String>,
    ) -> Self {
        Self::new(pubkey, disarm_secret_hash, DEFAULT_DURESS_DURATION)
    }

    pub fn trigger_duress_lockout(&mut self) {
        self.duress_triggered_at = Some(Instant::now());
    }

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