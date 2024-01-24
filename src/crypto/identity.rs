use super::{
    blake2b::Blake2bError,
    key::{PublicKey, SecretKey},
    pow::ProofOfWork,
};
use hex::FromHex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, io};
use thiserror::Error;

/// Error creating hash from bytes
#[derive(Debug, Error, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum FromBytesError {
    /// Invalid data size
    #[error("Invalid hash size")]
    InvalidSize,
}

#[derive(Debug, Error, PartialEq)]
pub enum PublicKeyError {
    #[error("Error constructing hash: {0}")]
    HashError(#[from] FromBytesError),
    #[error("Blake2b digest error: {0}")]
    Blake2bError(#[from] Blake2bError),
}

#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("I/O error: {reason}")]
    IoError { reason: io::Error },

    #[error("Serde error, reason: {reason}")]
    IdentitySerdeError { reason: serde_json::Error },

    #[error("Invalid field error, reason: {reason}")]
    IdentityFieldError { reason: String },
}

/// This node identity information compatible with Tezos
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct Identity {
    /// Peer_id is calculated hash of public_key [`crypto_box::PublicKey`]
    pub peer_id: String,
    /// Hex encoded public key: [`crypto_box::PublicKey`]
    pub public_key: PublicKey,
    /// Hex encoded secret key: [`crypto_box::SecretKey`]
    pub secret_key: SecretKey,
    /// Hex encoded pow: [`crypto::ProofOfWork`]
    pub proof_of_work_stamp: ProofOfWork,
}

impl Identity {
    pub fn from_json(json: &str) -> Result<Identity, IdentityError> {
        let identity: HashMap<String, Value> = serde_json::from_str(json)
            .map_err(|e| IdentityError::IdentitySerdeError { reason: e })?;

        let peer_id_str = identity
            .get("peer_id")
            .ok_or(IdentityError::IdentityFieldError {
                reason: "Missing 'peer_id'".to_string(),
            })?
            .as_str()
            .ok_or(IdentityError::IdentityFieldError {
                reason: "Missing valid 'peer_id'".to_string(),
            })?;
        let peer_id = peer_id_str.to_string();
        let public_key_str = identity
            .get("public_key")
            .ok_or(IdentityError::IdentityFieldError {
                reason: "Missing 'public_key'".to_string(),
            })?
            .as_str()
            .ok_or(IdentityError::IdentityFieldError {
                reason: "Missing valid 'public_key'".to_string(),
            })?;
        let public_key =
            PublicKey::from_hex(public_key_str).map_err(|e| IdentityError::IdentityFieldError {
                reason: format!("Missing valid 'public_key', reason: {}", e),
            })?;

        let secret_key_str = identity
            .get("secret_key")
            .ok_or(IdentityError::IdentityFieldError {
                reason: "Missing 'secret_key'".to_string(),
            })?
            .as_str()
            .ok_or(IdentityError::IdentityFieldError {
                reason: "Missing valid 'secret_key'".to_string(),
            })?;
        let secret_key =
            SecretKey::from_hex(secret_key_str).map_err(|e| IdentityError::IdentityFieldError {
                reason: format!("Missing valid 'secret_key', reason: {}", e),
            })?;

        let proof_of_work_stamp_str = identity
            .get("proof_of_work_stamp")
            .ok_or(IdentityError::IdentityFieldError {
                reason: "Missing 'proof_of_work_stamp'".to_string(),
            })?
            .as_str()
            .ok_or(IdentityError::IdentityFieldError {
                reason: "Missing valid 'proof_of_work_stamp'".to_string(),
            })?;
        let proof_of_work_stamp = ProofOfWork::from_hex(proof_of_work_stamp_str).map_err(|e| {
            IdentityError::IdentityFieldError {
                reason: format!("Missing valid 'proof_of_work_stamp', reason: {}", e),
            }
        })?;

        Ok(Identity {
            peer_id,
            public_key,
            secret_key,
            proof_of_work_stamp,
        })
    }

    pub fn from_json_file(identity_path: std::path::PathBuf) -> Result<Identity, IdentityError> {
        let json = std::fs::read_to_string(identity_path).map_err(|e| IdentityError::IoError {
            reason: io::Error::new(io::ErrorKind::Other, e),
        })?;
        Identity::from_json(&json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a sample identity for testing
    fn sample_identity() -> Identity {
        Identity {
            peer_id: "idsfYM6UbG2nhNS1dqhsJEchaDhmd9".to_string(),
            public_key: PublicKey::from_hex(
                "17f7d11892274a7230d969aa1335d25e637f43087b76d0e24a1a8b7d03168f5c",
            )
            .unwrap(),
            secret_key: SecretKey::from_hex(
                "0271fac86d020aebe6a1c9768381e7245e48e77524cca2a1652d0a621fac289f",
            )
            .unwrap(),
            proof_of_work_stamp: ProofOfWork::from_hex(
                "b6a4a80d765047918b037c85958c41096326a4b52ff0377e",
            )
            .unwrap(),
        }
    }

    #[test]
    fn test_identity_from_json() {
        let json: &str = r#"{ "peer_id": "idsfYM6UbG2nhNS1dqhsJEchaDhmd9",
  "public_key":
    "17f7d11892274a7230d969aa1335d25e637f43087b76d0e24a1a8b7d03168f5c",
  "secret_key":
    "0271fac86d020aebe6a1c9768381e7245e48e77524cca2a1652d0a621fac289f",
  "proof_of_work_stamp": "b6a4a80d765047918b037c85958c41096326a4b52ff0377e" }"#;
        let result = Identity::from_json(json);
        assert!(result.is_ok());

        let identity = result.unwrap();
        assert_eq!(identity, sample_identity());
    }
}
