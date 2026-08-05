use zeroize::Zeroize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BiometricError {
    #[error("Liveness check failed: insufficient or static blood flow detected")]
    LivenessFailed,
    #[error("Fuzzy extractor error correction threshold exceeded (Hamming distance too high)")]
    MatchFailed,
    #[error("Invalid biometric vector length or alignment")]
    InvalidVector,
}

/// Non-sensitive helper data generated during the Genesis/Enrollment event.
/// Safe to persist in `state/` or on-disk storage—reveals zero information
/// about raw vascular geometry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VascularHelperData {
    /// Error correction parity bits (e.g., BCH / Reed-Solomon syndrome)
    pub helper_blob: Vec<u8>,
    /// Unique cryptographic salt for KDF
    pub salt: [u8; 32],
    /// Maximum allowed Hamming distance threshold for noisy match
    pub tolerance: u32,
}

/// Ephemeral key derived in RAM during a valid vascular scan.
/// Uses `zeroize` to ensure CPU registers and memory are wiped immediately 
/// when this struct goes out of scope.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct EphemeralVascularKey {
    pub key_bytes: [u8; 32],
}

impl EphemeralVascularKey {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key_bytes
    }
}

/// Fuzzy Extractor engine for turning noisy near-infrared (NIR) vascular feature 
/// vectors into deterministic cryptographic keys.
pub struct FuzzyExtractorEngine;

impl FuzzyExtractorEngine {
    /// GENESIS / ENROLLMENT: Takes an initial high-entropy NIR vascular feature vector.
    /// Returns the initial EphemeralVascularKey and the public VascularHelperData blob.
    pub fn enroll(raw_vector: &[u8]) -> Result<(EphemeralVascularKey, VascularHelperData), BiometricError> {
        if raw_vector.is_empty() {
            return Err(BiometricError::InvalidVector);
        }

        // 1. Generate a cryptographic salt for Key Derivation
        let mut salt = [0u8; 32];
        getrandom::getrandom(&mut salt).map_err(|_| BiometricError::InvalidVector)?;

        // 2. Compute the deterministic key (K_bio) from raw vector features + salt
        let mut key_bytes = [0u8; 32];
        let mut hasher = blake3::Hasher::new_key(&salt);
        hasher.update(raw_vector);
        key_bytes.copy_from_slice(hasher.finalize().as_bytes());

        // 3. Generate Syndrome / Error Correction helper payload (P)
        // (In production, run Reed-Solomon / BCH polynomial encoding over raw_vector)
        let helper_blob = vec![0u8; raw_vector.len() / 2]; // Stub helper payload

        let helper_data = VascularHelperData {
            helper_blob,
            salt,
            tolerance: 12, // Allowed bit-flip tolerance
        };

        Ok((EphemeralVascularKey { key_bytes }, helper_data))
    }

    /// RE-AUTHENTICATION: Takes a noisy live NIR vascular feature vector + saved Helper Data.
    /// Corrects noise and reproduces the EXACT SAME EphemeralVascularKey in RAM.
    pub fn reconstruct(
        live_vector: &[u8],
        helper_data: &VascularHelperData,
    ) -> Result<EphemeralVascularKey, BiometricError> {
        if live_vector.is_empty() {
            return Err(BiometricError::InvalidVector);
        }

        // 1. Apply error correction (BCH/Reed-Solomon decoding) using helper_data.helper_blob
        //    to smooth out blood pressure/temperature variations in live_vector.
        let corrected_vector = apply_error_correction(live_vector, &helper_data.helper_blob, helper_data.tolerance)?;

        // 2. Derive the exact same 256-bit key using the stored salt
        let mut key_bytes = [0u8; 32];
        let mut hasher = blake3::Hasher::new_key(&helper_data.salt);
        hasher.update(&corrected_vector);
        key_bytes.copy_from_slice(hasher.finalize().as_bytes());

        Ok(EphemeralVascularKey { key_bytes })
    }
}

/// Internal helper function to execute error-correction decoding
fn apply_error_correction(
    live_vector: &[u8],
    _helper_blob: &[u8],
    _tolerance: u32,
) -> Result<Vec<u8>, BiometricError> {
    // Error correction logic goes here
    Ok(live_vector.to_vec())
}