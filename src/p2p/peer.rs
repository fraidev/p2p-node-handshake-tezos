use crate::{
    crypto::{
        blake2b::Blake2bError,
        identity::Identity,
        key::{CryptoError, CryptoKey, PublicKey},
        nonce::Nonce,
        peer_crypto::PeerCrypto,
    },
    msgs::{
        self,
        ack::AckStatus,
        connection::{ConnectionMessage, NetworkVersion},
        metadata::MetadataMessage,
    },
};
use speedy::{Endianness, Error, Readable, Writable};
use std::{fmt::Debug, sync::Arc};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

pub struct Peer {
    socket: std::net::SocketAddr,
    state: PeerState,
    stream: Arc<Mutex<TcpStream>>,
    identity: Identity,
    peer_crypto: Option<PeerCrypto>,
    chain_name: String,
}

#[derive(Debug, Error)]
pub enum PeerError {
    #[error("I/O error: {0}")]
    Io(std::io::Error),
    #[error("Connection failed")]
    ConnectionFailed,
    #[error("Ack failed")]
    AckFailed,
    #[error("Speedy failed: {0}")]
    SpeedyFailed(Error),
    #[error("Blake2b error: {0}")]
    BuildPeerCryptoFailed(Blake2bError),
    #[error("Peer crypto not initialized")]
    PeerCryptoNotInitialized,
    #[error("Crypto failed: {0}")]
    CryptoFailed(CryptoError),
}

enum PeerState {
    Disconnected,
    Connecting,
    Connected,
}

const CONTENT_LENGTH_FIELD_BYTES: usize = 2;

impl Peer {
    pub async fn connect(
        socket: std::net::SocketAddr,
        identity: Identity,
        chain_name: String,
    ) -> Result<Self, PeerError> {
        let addr = format!("{}:{}", socket.ip(), socket.port());
        let stream_raw = TcpStream::connect(addr).await.map_err(PeerError::Io)?;
        let stream = Arc::new(Mutex::new(stream_raw));

        Ok(Peer {
            socket,
            stream,
            state: PeerState::Connecting,
            identity,
            peer_crypto: None,
            chain_name,
        })
    }

    pub fn peer_crypto_mut(&mut self) -> &mut Option<PeerCrypto> {
        &mut self.peer_crypto
    }

    pub async fn desconnect(&mut self) -> Result<(), PeerError> {
        let mut stream = self.stream.lock().await;
        stream.flush().await.map_err(PeerError::Io)?;
        stream.shutdown().await.map_err(PeerError::Io)?;
        self.state = PeerState::Disconnected;
        Ok(())
    }

    pub async fn handshake(&mut self) -> Result<(), PeerError> {
        let connection_msg = ConnectionMessage::new(
            self.socket.port(),
            self.identity.public_key.as_ref().as_ref().to_vec(),
            self.identity.proof_of_work_stamp.as_ref().to_vec(),
            Nonce::random().get_bytes().to_vec(),
            NetworkVersion::new(self.chain_name.clone(), 2, 1),
        );

        let sent = connection_msg
            .write_to_vec_with_ctx(Endianness::BigEndian)
            .map_err(|e| PeerError::SpeedyFailed(e))?;

        // As we're a outcoming connection, we send the connection message first
        self.send_msg(sent.to_vec(), false).await?;
        println!("Sent connection message: {:?}", connection_msg);

        // Receive the connection message
        let recv = self.recv_msg(false).await?;
        let cm_msg = msgs::connection::ConnectionMessage::read_from_buffer_with_ctx(
            Endianness::BigEndian,
            &recv,
            // binary_chunk.content(),
        )
        .map_err(|e| PeerError::SpeedyFailed(e))?;
        println!("Received connection message: {:?}", cm_msg);

        // Encryption everything after this point
        let pk = PublicKey::from_bytes(&cm_msg.public_key).map_err(PeerError::CryptoFailed)?;
        *self.peer_crypto_mut() = Some(
            PeerCrypto::build(
                &self.identity.secret_key,
                &pk,
                msg_bytes_to_raw(&sent),
                msg_bytes_to_raw(&recv),
                false,
            )
            .map_err(|e| PeerError::BuildPeerCryptoFailed(e))?,
        );

        // Send metadata
        let meta_msg = MetadataMessage::new(false, false);
        let meta_msg_vec = meta_msg
            .write_to_vec()
            .map_err(|e| PeerError::SpeedyFailed(e))?;
        self.send_msg(meta_msg_vec, true).await?;
        println!("Sent metadata message: {:?}", meta_msg);

        // Receive metadata
        let meta_msg_recv = self.recv_msg(true).await?;
        MetadataMessage::read_from_buffer(&meta_msg_recv)
            .expect("Failed to parse metadata message");
        println!("Received metadata message: {:?}", meta_msg);

        // Send ack
        let ack_msg = AckStatus::Ack;
        self.send_msg(
            ack_msg.write_to_vec().map_err(PeerError::SpeedyFailed)?,
            true,
        )
        .await?;
        println!("Sent acknowledgement message: {:?}", ack_msg);

        // Receive ack
        let ack_msg_recv = self.recv_msg(true).await?;
        let ack_msg =
            AckStatus::read_from_buffer(&ack_msg_recv).expect("Failed to parse ack message");
        println!("Received acknowledgement message: {:?}", ack_msg);
        if ack_msg != AckStatus::Ack {
            return Err(PeerError::AckFailed);
        }

        self.state = PeerState::Connected;
        Ok(())
    }

    pub async fn send_msg(&mut self, bytes: Vec<u8>, encryption: bool) -> Result<(), PeerError> {
        let mut stream = self.stream.lock().await;
        let data = if encryption {
            let peer_crypt_mutable = self.peer_crypto.as_mut();
            match peer_crypt_mutable {
                Some(pc) => pc.encrypt(&bytes).map_err(PeerError::CryptoFailed)?,
                None => return Err(PeerError::PeerCryptoNotInitialized),
            }
        } else {
            bytes
        };

        let raw = msg_bytes_to_raw(&data);
        println!("Sending message length: {:?}", raw.len());
        stream.write_all(&raw).await.map_err(PeerError::Io)?;
        Ok(())
    }

    pub async fn recv_msg(&mut self, encryption: bool) -> Result<Vec<u8>, PeerError> {
        let mut buffer_len = [0u8; CONTENT_LENGTH_FIELD_BYTES];
        let mut stream = self.stream.lock().await;
        stream
            .read_exact(&mut buffer_len)
            .await
            .map_err(PeerError::Io)?;

        let mlen = u16::from_be_bytes(buffer_len);
        let mut buffer = vec![0u8; mlen as usize];
        stream
            .read_exact(&mut buffer)
            .await
            .map_err(PeerError::Io)?;
        if encryption && !buffer.is_empty() {
            let peer_crypt_mutable = self.peer_crypto.as_mut();
            return match peer_crypt_mutable {
                Some(pc) => Ok(pc.decrypt(&buffer).map_err(PeerError::CryptoFailed)?),
                None => return Err(PeerError::PeerCryptoNotInitialized),
            };
        }
        Ok(buffer)
    }
}

fn msg_bytes_to_raw(content: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(CONTENT_LENGTH_FIELD_BYTES + content.len());
    bytes.extend_from_slice(&(content.len() as u16).to_be_bytes());
    bytes.extend(content);
    bytes.clone()
}
