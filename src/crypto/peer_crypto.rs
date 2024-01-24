use super::{
    blake2b::Blake2bError,
    key::{generate_nonces, NoncePair, PrecomputedKey, PublicKey, SecretKey, CryptoError},
    nonce::Nonce,
};

/// PeerCrypto is responsible for encrypting/decrypting messages and
/// managing nonces.
#[derive(Debug, Clone)]
pub struct PeerCrypto {
    precomputed_key: PrecomputedKey,
    nonce_pair: NoncePair,
}

impl PeerCrypto {
    #[inline]
    pub fn new(precomputed_key: PrecomputedKey, nonce_pair: NoncePair) -> Self {
        Self {
            precomputed_key,
            nonce_pair,
        }
    }

    pub fn build(
        node_secret_key: &SecretKey,
        peer_public_key: &PublicKey,
        sent_conn_msg: Vec<u8>,
        received_conn_msg: Vec<u8>,
        incoming: bool,
    ) -> Result<Self, Blake2bError> {
        let nonce_pair = generate_nonces(
            sent_conn_msg.as_slice(),
            received_conn_msg.as_slice(),
            incoming,
        )?;
        let key = PrecomputedKey::precompute(peer_public_key, node_secret_key);
        Ok(PeerCrypto::new(key, nonce_pair))
    }

    #[inline]
    fn local_nonce_fetch_increment(&mut self) -> Nonce {
        let nonce = self.nonce_pair.local.increment();
        std::mem::replace(&mut self.nonce_pair.local, nonce)
    }

    #[inline]
    fn remote_nonce_fetch_increment(&mut self) -> Nonce {
        let nonce = self.nonce_pair.remote.increment();
        std::mem::replace(&mut self.nonce_pair.remote, nonce)
    }

    /// Increments local nonce and encrypts the message.
    #[inline]
    pub fn encrypt<T: AsRef<[u8]>>(&mut self, data: &T) -> Result<Vec<u8>, CryptoError> {
        let nonce = self.local_nonce_fetch_increment();
        self.precomputed_key.encrypt(data.as_ref(), &nonce)
    }

    /// Increments remote nonce and encrypts the message.
    #[inline]
    pub fn decrypt<T: AsRef<[u8]>>(&mut self, data: &T) -> Result<Vec<u8>, CryptoError> {
        let nonce = self.remote_nonce_fetch_increment();
        self.precomputed_key.decrypt(data.as_ref(), &nonce)
    }
}

