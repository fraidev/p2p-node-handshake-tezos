use speedy::{Readable, Writable};

#[derive(Debug, PartialEq, Readable, Writable)]
#[speedy(tag_type = u8)]
pub enum AckStatus {
    #[speedy(tag = 0x00)]
    Ack,
    #[speedy(tag = 0xFF)]
    NackV1,
    #[speedy(tag = 0x01)]
    NackV2,
}
