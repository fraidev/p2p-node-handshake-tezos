pub mod cli;
pub mod constants;
pub mod crypto;
pub mod msgs;
pub mod p2p;

use clap::Parser;
use std::{net::SocketAddr, str::FromStr};

use crate::{
    cli::Cli,
    constants::{BOOTSTRAP_DEFAULT_PORT, BOOTSTRAP_PEERS, DEFAUL_IDENTITY_JSON},
    crypto::identity::Identity,
    p2p::{dns, peer::Peer},
};

#[tokio::main]
async fn main() {
    println!("Starting... 🚀");
    let args = Cli::parse();

    println!("Resolving peer address... 🧭");
    let peer_addr = if let Some(peer) = args.peer {
        SocketAddr::from_str(&peer).expect("Failed to parse peer address")
    } else {
        println!("Looking for active nodes... 🔎");
        let boostrap_peers = dns::lookup_active_nodes(BOOTSTRAP_PEERS, BOOTSTRAP_DEFAULT_PORT);
        let rand = rand::random::<usize>() % boostrap_peers.len();
        boostrap_peers[rand]
    };

    println!("Getting identity... 🪪");
    let identity = if let Some(identity_path) = args.identity_path {
        Identity::from_json_file(identity_path).expect("Failed to get identity")
    } else {
        Identity::from_json(DEFAUL_IDENTITY_JSON).expect("Failed to get identity")
    };

    println!("Connecting to peer {}... 🛜", peer_addr);
    let chain_name = args
        .chain_name
        .unwrap_or("TEZOS_MAINNET".to_string())
        .to_uppercase();
    let mut peer = Peer::connect(peer_addr, identity, chain_name)
        .await
        .unwrap_or_else(|e| panic!("Failed to connect to peer, Error: {}", e));

    println!("Handshaking with peer {}... 🤝", peer_addr);
    peer.handshake()
        .await
        .unwrap_or_else(|e| panic!("Failed to handshake with peer, Error: {}", e));

    println!("Done, Handshake completed! 🎉");

    peer.desconnect()
        .await
        .unwrap_or_else(|e| panic!("Failed to disconnect from peer, Error: {}", e));
    println!("Disconnected from peer {}... 👋", peer_addr);
}
