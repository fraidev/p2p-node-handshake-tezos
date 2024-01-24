use std::net::{SocketAddr, ToSocketAddrs};

pub fn lookup_active_nodes(dns: &[&str], port: u16) -> Vec<SocketAddr> {
    dns.iter()
        .flat_map(|d| {
            let t = (*d, port);
            ToSocketAddrs::to_socket_addrs(&t).unwrap_or_default()
        })
        .collect::<Vec<_>>()
}
