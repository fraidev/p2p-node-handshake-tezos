use byteorder::{BigEndian, ByteOrder};
use rand::Rng;
use serde::{Deserialize, Serialize};

pub const NONCE_SIZE: usize = 24;
const NONCE_WORDS: usize = 12;

/// Arbitrary number that can be used once in communication.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Nonce {
    value: [u16; NONCE_WORDS],
}

impl Nonce {
    /// Create new nonce from raw bytes
    pub fn new(bytes: &[u8; NONCE_SIZE]) -> Self {
        let mut buf: [u8; NONCE_SIZE] = [0; NONCE_SIZE];
        buf.copy_from_slice(bytes);
        let mut value: [u16; NONCE_WORDS] = [0; NONCE_WORDS];
        BigEndian::read_u16_into(&buf, &mut value);
        Nonce { value }
    }

    /// Generate new random nonce
    pub fn random() -> Self {
        let mut value: [u16; NONCE_WORDS] = [0; NONCE_WORDS];
        rand::thread_rng().fill(&mut value);
        Nonce { value }
    }

    /// Increment this nonce by one
    pub fn increment(&self) -> Self {
        let mut value: [u16; NONCE_WORDS] = [0; NONCE_WORDS];
        value.copy_from_slice(&self.value);

        let mut pos = NONCE_WORDS - 1;
        loop {
            let result: u32 = value[pos] as u32 + 1u32;
            value[pos] = (result & 0xffffu32) as u16;

            if result < 0x10000u32 || pos == 0 {
                break;
            }

            pos -= 1;
        }

        Nonce { value }
    }

    /// Create bytes representation equal to this nonce with correct nonce size, else return error
    pub fn get_bytes(&self) -> [u8; NONCE_SIZE] {
        let mut result: [u8; NONCE_SIZE] = [0; NONCE_SIZE];
        BigEndian::write_u16_into(&self.value, &mut result);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_nonce() {
        let raw_bytes: [u8; NONCE_SIZE] = [1; NONCE_SIZE];
        let nonce = Nonce::new(&raw_bytes);
        assert_eq!(nonce.get_bytes(), raw_bytes);
    }

    #[test]
    fn test_random_nonce() {
        let nonce1 = Nonce::random();
        let nonce2 = Nonce::random();
        assert_ne!(nonce1, nonce2);
    }

    #[test]
    fn test_increment_nonce() {
        let initial_nonce = Nonce::random();
        let incremented_nonce = initial_nonce.increment();
        assert_ne!(initial_nonce, incremented_nonce);
    }

    #[test]
    fn test_get_bytes() {
        let nonce = Nonce::random();
        let bytes = nonce.get_bytes();
        let reconstructed_nonce = Nonce::new(&bytes);
        assert_eq!(nonce, reconstructed_nonce);
    }
}
