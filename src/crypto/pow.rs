use super::{blake2b::Blake2bError, key::CryptoError, nonce::NONCE_SIZE};
use hex::FromHex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const POW_SIZE: usize = NONCE_SIZE;

#[derive(Serialize, Deserialize, Error, Debug, Clone)]
pub enum PowError {
    #[error("Proof-of-work check failed")]
    CheckFailed,
    #[error("Proof-of-work blake2b error: {0}")]
    Blake2b(Blake2bError),
}

pub type PowResult = Result<(), PowError>;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct ProofOfWork([u8; POW_SIZE]);

impl AsRef<[u8]> for ProofOfWork {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl FromHex for ProofOfWork {
    type Error = CryptoError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        let bytes = hex::decode(hex)?;

        if bytes.len() != POW_SIZE {
            return Err(CryptoError::InvalidKeySize {
                expected: POW_SIZE,
                actual: bytes.len(),
            });
        }

        let mut arr = [0u8; POW_SIZE];
        arr.copy_from_slice(&bytes);
        Ok(ProofOfWork(arr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_of_work_from_hex() {
        let hex_string = "b6a4a80d765047918b037c85958c41096326a4b52ff0377e";
        let expected_bytes: [u8; POW_SIZE] = [
            182, 164, 168, 13, 118, 80, 71, 145, 139, 3, 124, 133, 149, 140, 65, 9, 99, 38, 164,
            181, 47, 240, 55, 126,
        ];

        let proof_of_work_result = ProofOfWork::from_hex(hex_string);
        assert!(proof_of_work_result.is_ok());

        let proof_of_work = proof_of_work_result.unwrap();
        assert_eq!(proof_of_work.as_ref(), &expected_bytes);
    }

    #[test]
    fn test_proof_of_work_from_hex_invalid_size() {
        let hex_string = "0123456789abcdef0123456789abcdef"; // Invalid size

        let proof_of_work_result = ProofOfWork::from_hex(hex_string);
        assert!(proof_of_work_result.is_err());

        match proof_of_work_result.unwrap_err() {
            CryptoError::InvalidKeySize { expected, actual } => {
                assert_eq!(expected, POW_SIZE);
                assert_eq!(actual, hex_string.len() / 2);
            }
            _ => panic!("Unexpected error type"),
        }
    }
}
