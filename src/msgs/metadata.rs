use speedy::{Readable, Writable};

#[derive(Debug, PartialEq, Readable, Writable)]
pub struct MetadataMessage {
    disable_mempool: bool,
    private_node: bool,
}

impl MetadataMessage {
    pub fn new(disable_mempool: bool, private_node: bool) -> Self {
        Self {
            disable_mempool,
            private_node,
        }
    }
}
