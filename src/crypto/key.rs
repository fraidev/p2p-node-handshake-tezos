use super::{
    blake2b::{self, Blake2bError},
    nonce::Nonce,
};
use hex::{FromHex, FromHexError};
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::box_;
use std::fmt::Debug;
use thiserror::Error;

pub const BOX_ZERO_BYTES: usize = 32;
pub const CRYPTO_KEY_SIZE: usize = 32;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Error)]
pub enum CryptoError {
    #[error("Invalid key size, expected: {expected}, actual: {actual}")]
    InvalidKeySize { expected: usize, actual: usize },
    #[error("Invalid key, reason: {reason}")]
    InvalidKey { reason: String },
    #[error("Failed to encrypt")]
    FailedToDecrypt,
}

pub trait CryptoKey: Sized {
    fn from_bytes<B: AsRef<[u8]>>(buf: B) -> Result<Self, CryptoError>;
}

fn ensure_crypto_key_bytes<B: AsRef<[u8]>>(buf: B) -> Result<[u8; CRYPTO_KEY_SIZE], CryptoError> {
    let buf = buf.as_ref();

    // check size
    if buf.len() != CRYPTO_KEY_SIZE {
        return Err(CryptoError::InvalidKeySize {
            expected: CRYPTO_KEY_SIZE,
            actual: buf.len(),
        });
    };

    // convert to correct key size
    let mut arr = [0u8; CRYPTO_KEY_SIZE];
    arr.copy_from_slice(buf);
    Ok(arr)
}

/// Convenience wrapper around [`sodiumoxide::crypto::box_::PublicKey`]
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct PublicKey(box_::PublicKey);

impl CryptoKey for PublicKey {
    fn from_bytes<B: AsRef<[u8]>>(buf: B) -> Result<Self, CryptoError> {
        ensure_crypto_key_bytes(buf).map(|key_bytes| PublicKey(box_::PublicKey(key_bytes)))
    }
}

impl AsRef<box_::PublicKey> for PublicKey {
    fn as_ref(&self) -> &box_::PublicKey {
        &self.0
    }
}

impl FromHex for PublicKey {
    type Error = CryptoError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        Self::from_bytes(hex::decode(hex)?)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
/// Convenience wrapper around [`sodiumoxide::crypto::box_::SecretKey`]
pub struct SecretKey(box_::SecretKey);

impl CryptoKey for SecretKey {
    fn from_bytes<B: AsRef<[u8]>>(buf: B) -> Result<Self, CryptoError> {
        ensure_crypto_key_bytes(buf).map(|key_bytes| SecretKey(box_::SecretKey(key_bytes)))
    }
}

impl AsRef<box_::SecretKey> for SecretKey {
    fn as_ref(&self) -> &box_::SecretKey {
        &self.0
    }
}

impl FromHex for SecretKey {
    type Error = CryptoError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        Self::from_bytes(hex::decode(hex)?)
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
/// Convenience wrapper around [`sodiumoxide::crypto::box_::PrecomputedKey`]
pub struct PrecomputedKey(box_::PrecomputedKey);

impl PrecomputedKey {
    pub fn precompute(pk: &PublicKey, sk: &SecretKey) -> Self {
        Self(box_::precompute(pk.as_ref(), sk.as_ref()))
    }

    pub fn from_bytes(bytes: [u8; box_::PRECOMPUTEDKEYBYTES]) -> Self {
        Self(box_::PrecomputedKey(bytes))
    }

    pub fn encrypt(&self, msg: &[u8], nonce: &Nonce) -> Result<Vec<u8>, CryptoError> {
        let box_nonce = box_::Nonce(nonce.get_bytes());
        Ok(box_::seal_precomputed(msg, &box_nonce, &self.0))
    }

    pub fn decrypt(&self, enc: &[u8], nonce: &Nonce) -> Result<Vec<u8>, CryptoError> {
        let box_nonce = box_::Nonce(nonce.get_bytes());
        match box_::open_precomputed(enc, &box_nonce, &self.0) {
            Ok(msg) => Ok(msg),
            Err(()) => Err(CryptoError::FailedToDecrypt),
        }
    }
}

impl From<FromHexError> for CryptoError {
    fn from(e: FromHexError) -> Self {
        CryptoError::InvalidKey {
            reason: format!("{}", e),
        }
    }
}

const INIT_TO_RESP_SEED: &[u8] = b"Init -> Resp";
const RESP_TO_INIT_SEED: &[u8] = b"Resp -> Init";
pub const NONCE_SIZE: usize = 24;

macro_rules! merge_slices {
    ( $($x:expr),* ) => {{
        let mut res = vec![];
        $(
            res.extend_from_slice($x);
        )*
        res
    }}
}

/// Pair of local/remote nonces
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct NoncePair {
    pub local: Nonce,
    pub remote: Nonce,
}

pub fn generate_nonces(
    sent_msg: &[u8],
    recv_msg: &[u8],
    incoming: bool,
) -> Result<NoncePair, Blake2bError> {
    let (init_msg, resp_msg) = if incoming {
        (recv_msg, sent_msg)
    } else {
        (sent_msg, recv_msg)
    };

    let nonce_init_to_resp: [u8; NONCE_SIZE] =
        match blake2b::digest_256(&merge_slices!(init_msg, resp_msg, INIT_TO_RESP_SEED))?
            [0..NONCE_SIZE]
            .try_into()
        {
            Ok(value) => value,
            Err(_) => return Err(Blake2bError::InvalidLenght),
        };

    let nonce_resp_to_init: [u8; NONCE_SIZE] =
        match blake2b::digest_256(&merge_slices!(init_msg, resp_msg, RESP_TO_INIT_SEED))?
            [0..NONCE_SIZE]
            .try_into()
        {
            Ok(value) => value,
            Err(_) => return Err(Blake2bError::InvalidLenght),
        };

    Ok(if incoming {
        NoncePair {
            local: Nonce::new(&nonce_init_to_resp),
            remote: Nonce::new(&nonce_resp_to_init),
        }
    } else {
        NoncePair {
            local: Nonce::new(&nonce_resp_to_init),
            remote: Nonce::new(&nonce_init_to_resp),
        }
    })
}
