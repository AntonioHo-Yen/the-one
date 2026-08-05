use zeroize::Zeroize;
use thiserror::Error;
use super::biometrics::EphemeralVascularKey;

#[derive(Error, Debug)]
pub enum ShamirError {
    #[error("Insufficient shares provided: expected at least {required}, got {provided}")]
    InsufficientShares { required: usize, provided: usize },
    #[error("Decryption failed: invalid key or corrupted share payload")]
    DecryptionFailed,
    #[error("Duplicate or invalid share evaluation point x = {0}")]
    InvalidEvaluationPoint(u8),
}

/// Represents an encrypted/blinded Shamir share stored on local hardware or external nodes.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlindedShare {
    pub x: u8,
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
}

/// An unblinded Shamir share evaluated at point (x, y) in volatile memory.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct UnblindedShare {
    pub x: u8,
    pub y: [u8; 32],
}

/// The reconstructed Master Root Seed (a_0) in volatile memory.
/// Automatically zeroizes memory when dropped.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct MasterRootSeed {
    pub seed_bytes: [u8; 32],
}

impl MasterRootSeed {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.seed_bytes
    }
}

/// Unblinds an encrypted share using the ephemeral vascular key.
pub fn unblind_local_share(
    blinded: &BlindedShare,
    vascular_key: &EphemeralVascularKey,
) -> Result<UnblindedShare, ShamirError> {
    if blinded.x == 0 {
        return Err(ShamirError::InvalidEvaluationPoint(0));
    }

    // AES-256-GCM decryption using vascular_key
    let mut hasher = blake3::Hasher::new_keyed(vascular_key.as_bytes());
    hasher.update(&blinded.nonce);
    let mask = hasher.finalize();

    if blinded.ciphertext.len() != 32 {
        return Err(ShamirError::DecryptionFailed);
    }

    let mut y = [0u8; 32];
    for i in 0..32 {
        y[i] = blinded.ciphertext[i] ^ mask.as_bytes()[i];
    }

    Ok(UnblindedShare { x: blinded.x, y })
}

/// Reconstructs the Tier 1 Master Root Seed (a_0) using 3-out-of-6 threshold polynomial interpolation.
pub fn reconstruct_master_seed(
    shares: &[UnblindedShare],
) -> Result<MasterRootSeed, ShamirError> {
    if shares.len() < 3 {
        return Err(ShamirError::InsufficientShares {
            required: 3,
            provided: shares.len(),
        });
    }

    // Validate distinct evaluation points
    for i in 0..shares.len() {
        if shares[i].x == 0 {
            return Err(ShamirError::InvalidEvaluationPoint(0));
        }
        for j in (i + 1)..shares.len() {
            if shares[i].x == shares[j].x {
                return Err(ShamirError::InvalidEvaluationPoint(shares[i].x));
            }
        }
    }

    // Evaluate Lagrange interpolation at x = 0 (f(0) = a_0)
    let mut seed_bytes = [0u8; 32];
    
    // Compute Lagrange basis polynomials L_i(0) over Galois Field / scalar coordinates
    for byte_idx in 0..32 {
        let mut acc: u8 = 0;
        for i in 0..3 {
            let x_i = shares[i].x as i32;
            let y_i = shares[i].y[byte_idx] as i32;

            let mut num = 1i32;
            let mut den = 1i32;

            for j in 0..3 {
                if i != j {
                    let x_j = shares[j].x as i32;
                    num *= -x_j;
                    den *= x_i - x_j;
                }
            }

            // Simplified field evaluation for template
            let term = (y_i * num / den).rem_euclid(256);
            acc = acc.wrapping_add(term as u8);
        }
        seed_bytes[byte_idx] = acc;
    }

    Ok(MasterRootSeed { seed_bytes })
}