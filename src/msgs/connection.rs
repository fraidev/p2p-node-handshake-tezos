use speedy::{Readable, Writable};

pub const CRYPTO_KEY_LENGTH: usize = 32;
pub const POW_LENGTH: usize = 24;
pub const NONCE_LENGTH: usize = 24;
pub const CHAIN_NAME_LENGTH: usize = 128;

#[derive(Debug, PartialEq, Readable, Writable)]
pub struct ConnectionMessage {
    pub port: u16,
    #[speedy(length  = CRYPTO_KEY_LENGTH)]
    pub public_key: Vec<u8>,
    #[speedy(length  = POW_LENGTH)]
    pub proof_of_work_stamp: Vec<u8>,
    #[speedy(length  = NONCE_LENGTH)]
    pub message_nonce: Vec<u8>,
    pub version_length: u16,
    pub version: NetworkVersion,
}

impl speedy::Context for ConnectionMessage {
    type Error = speedy::Error;
    fn endianness(&self) -> speedy::Endianness {
        speedy::Endianness::BigEndian
    }
}

impl ConnectionMessage {
    pub fn new(
        port: u16,
        public_key: Vec<u8>,
        proof_of_work_stamp: Vec<u8>,
        message_nonce: Vec<u8>,
        version: NetworkVersion,
    ) -> Self {
        Self {
            port,
            public_key,
            proof_of_work_stamp,
            message_nonce,
            version_length: 0,
            version,
        }
    }
}

#[derive(Debug, PartialEq, Readable, Writable)]
pub struct NetworkVersion {
    pub chain_name_length: u16,
    #[speedy(length = chain_name_length)]
    pub chain_name: String,
    pub distributed_db_version: u16,
    pub p2p_version: u16,
}

impl speedy::Context for NetworkVersion {
    type Error = speedy::Error;
    fn endianness(&self) -> speedy::Endianness {
        speedy::Endianness::BigEndian
    }
}

impl NetworkVersion {
    pub fn new(chain_name: String, distributed_db_version: u16, p2p_version: u16) -> Self {
        Self {
            chain_name_length: chain_name.as_bytes().len() as u16,
            chain_name,
            distributed_db_version,
            p2p_version,
        }
    }
}
