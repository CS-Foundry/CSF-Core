use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use rustls::RootCertStore;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{TlsAcceptor, TlsConnector};
use uuid::Uuid;

use super::certs;

/// Message types for P2P communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum P2PMessage {
    /// Handshake message to establish connection
    Handshake {
        agent_id: Uuid,
        agent_name: String,
        timestamp: DateTime<Utc>,
    },
    /// Heartbeat to maintain connection
    Heartbeat {
        agent_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    /// Share metrics with peer
    MetricsShare {
        agent_id: Uuid,
        timestamp: DateTime<Utc>,
        metrics: serde_json::Value,
    },
    /// Request metrics from peer
    MetricsRequest {
        agent_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    /// Response to a request
    Response {
        success: bool,
        message: String,
        data: Option<serde_json::Value>,
    },
}

/// mTLS P2P Connector for agent-to-agent communication
#[derive(Clone)]
pub struct P2PConnector {
    agent_id: Uuid,
    agent_name: String,
    listen_addr: SocketAddr,
    tls_acceptor: TlsAcceptor,
    tls_connector: TlsConnector,
}

impl P2PConnector {
    /// Creates a new P2P connector with mTLS configuration
    pub fn new(
        agent_id: Uuid,
        agent_name: String,
        listen_port: u16,
        cert_path: &Path,
        key_path: &Path,
        ca_cert_path: &Path,
    ) -> Result<Self> {
        // Load certificates
        let certs = certs::load_certs(cert_path).context("Failed to load agent certificate")?;
        let key = certs::load_private_key(key_path).context("Failed to load agent private key")?;
        let ca_certs = certs::load_certs(ca_cert_path).context("Failed to load CA certificate")?;

        // Build server config (for accepting connections)
        let server_config = Self::build_server_config(certs.clone(), key.clone_key(), &ca_certs)?;
        let tls_acceptor = TlsAcceptor::from(Arc::new(server_config));

        // Build client config (for initiating connections)
        let client_config = Self::build_client_config(certs, key, &ca_certs)?;
        let tls_connector = TlsConnector::from(Arc::new(client_config));

        let listen_addr = SocketAddr::from(([0, 0, 0, 0], listen_port));

        Ok(Self {
            agent_id,
            agent_name,
            listen_addr,
            tls_acceptor,
            tls_connector,
        })
    }

    /// Build server TLS config for accepting connections
    fn build_server_config(
        certs: Vec<CertificateDer<'static>>,
        key: PrivateKeyDer<'static>,
        ca_certs: &[CertificateDer<'static>],
    ) -> Result<rustls::ServerConfig> {
        // Create root cert store for client verification
        let mut root_store = RootCertStore::empty();
        for cert in ca_certs {
            root_store
                .add(cert.clone())
                .context("Failed to add CA certificate to root store")?;
        }

        // Create verifier that requires client certificates
        let client_verifier = rustls::server::WebPkiClientVerifier::builder(Arc::new(root_store))
            .build()
            .context("Failed to build client verifier")?;

        let config = rustls::ServerConfig::builder()
            .with_client_cert_verifier(client_verifier)
            .with_single_cert(certs, key)
            .context("Failed to build server config")?;

        Ok(config)
    }

    /// Build client TLS config for initiating connections
    fn build_client_config(
        certs: Vec<CertificateDer<'static>>,
        key: PrivateKeyDer<'static>,
        ca_certs: &[CertificateDer<'static>],
    ) -> Result<rustls::ClientConfig> {
        // Create root cert store for server verification
        let mut root_store = RootCertStore::empty();
        for cert in ca_certs {
            root_store
                .add(cert.clone())
                .context("Failed to add CA certificate to root store")?;
        }

        let config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(certs, key)
            .context("Failed to build client config")?;

        Ok(config)
    }

    /// Start listening for incoming P2P connections
    pub async fn start_server(&self) -> Result<()> {
        let listener = TcpListener::bind(self.listen_addr)
            .await
            .context(format!("Failed to bind to {}", self.listen_addr))?;

        tracing::info!("P2P server listening on {}", self.listen_addr);

        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    tracing::info!("Accepted connection from {}", peer_addr);
                    let acceptor = self.tls_acceptor.clone();
                    let agent_id = self.agent_id;
                    let agent_name = self.agent_name.clone();

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(
                            stream, acceptor, agent_id, agent_name, peer_addr,
                        )
                        .await
                        {
                            tracing::error!("Error handling connection from {}: {}", peer_addr, e);
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Error accepting connection: {}", e);
                }
            }
        }
    }

    /// Handle an incoming connection
    async fn handle_connection(
        stream: TcpStream,
        acceptor: TlsAcceptor,
        agent_id: Uuid,
        agent_name: String,
        peer_addr: SocketAddr,
    ) -> Result<()> {
        // Perform TLS handshake
        let mut tls_stream = match acceptor.accept(stream).await {
            Ok(s) => s,
            Err(e) => {
                anyhow::bail!("TLS handshake failed: {:?}", e);
            }
        };

        tracing::info!("TLS handshake completed with {}", peer_addr);

        // Send handshake message
        let handshake = P2PMessage::Handshake {
            agent_id,
            agent_name: agent_name.clone(),
            timestamp: Utc::now(),
        };
        Self::send_message(&mut tls_stream, &handshake).await?;

        // Receive handshake response
        let response = Self::receive_message(&mut tls_stream).await?;
        match response {
            P2PMessage::Handshake {
                agent_id: peer_id,
                agent_name: peer_name,
                ..
            } => {
                tracing::info!("Connected to peer: {} ({})", peer_name, peer_id);
            }
            _ => {
                anyhow::bail!("Expected handshake message, got: {:?}", response);
            }
        }

        // Keep connection alive and handle messages
        loop {
            match Self::receive_message(&mut tls_stream).await {
                Ok(message) => {
                    tracing::debug!("Received message: {:?}", message);

                    // Handle different message types
                    match message {
                        P2PMessage::Heartbeat {
                            agent_id: peer_id, ..
                        } => {
                            tracing::debug!("Heartbeat from {}", peer_id);

                            // Send heartbeat response
                            let response = P2PMessage::Response {
                                success: true,
                                message: "Heartbeat received".to_string(),
                                data: None,
                            };
                            Self::send_message(&mut tls_stream, &response).await?;
                        }
                        P2PMessage::MetricsRequest {
                            agent_id: peer_id, ..
                        } => {
                            tracing::debug!("Metrics request from {}", peer_id);

                            // TODO: Get current metrics and send them
                            let response = P2PMessage::Response {
                                success: true,
                                message: "Metrics data".to_string(),
                                data: Some(serde_json::json!({"status": "ok"})),
                            };
                            Self::send_message(&mut tls_stream, &response).await?;
                        }
                        P2PMessage::MetricsShare {
                            agent_id: peer_id,
                            metrics,
                            ..
                        } => {
                            tracing::info!("Received metrics from {}: {:?}", peer_id, metrics);
                        }
                        _ => {
                            tracing::warn!("Unhandled message type: {:?}", message);
                        }
                    }
                }
                Err(e) => {
                    tracing::info!("Connection closed: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Connect to a peer agent
    pub async fn connect_to_peer(
        &self,
        peer_addr: &str,
    ) -> Result<tokio_rustls::client::TlsStream<TcpStream>> {
        // Parse address
        let addr: SocketAddr = peer_addr
            .parse()
            .context(format!("Invalid peer address: {}", peer_addr))?;

        tracing::info!("Connecting to peer at {}", addr);

        // Connect to peer
        let stream = TcpStream::connect(addr)
            .await
            .context(format!("Failed to connect to {}", addr))?;

        // Extract hostname for SNI
        let hostname = addr.ip().to_string();
        let server_name = ServerName::try_from(hostname)
            .context("Invalid hostname")?
            .to_owned();

        // Perform TLS handshake
        let mut tls_stream = match self.tls_connector.connect(server_name, stream).await {
            Ok(s) => s,
            Err(e) => {
                anyhow::bail!("TLS handshake failed: {:?}", e);
            }
        };

        tracing::info!("TLS handshake completed with {}", addr);

        // Receive handshake from server
        let handshake = Self::receive_message(&mut tls_stream).await?;
        match handshake {
            P2PMessage::Handshake {
                agent_id: peer_id,
                agent_name: peer_name,
                ..
            } => {
                tracing::info!("Connected to peer: {} ({})", peer_name, peer_id);
            }
            _ => {
                anyhow::bail!("Expected handshake message");
            }
        }

        // Send our handshake
        let handshake = P2PMessage::Handshake {
            agent_id: self.agent_id,
            agent_name: self.agent_name.clone(),
            timestamp: Utc::now(),
        };
        Self::send_message(&mut tls_stream, &handshake).await?;

        Ok(tls_stream)
    }

    /// Send a message over any TLS stream
    async fn send_message<S>(stream: &mut S, message: &P2PMessage) -> Result<()>
    where
        S: AsyncReadExt + AsyncWriteExt + Unpin,
    {
        let json = serde_json::to_string(message)?;
        let len = json.len() as u32;

        // Send length prefix (4 bytes)
        stream.write_all(&len.to_be_bytes()).await?;
        // Send JSON data
        stream.write_all(json.as_bytes()).await?;
        stream.flush().await?;

        Ok(())
    }

    /// Receive a message from any TLS stream
    async fn receive_message<S>(stream: &mut S) -> Result<P2PMessage>
    where
        S: AsyncReadExt + AsyncWriteExt + Unpin,
    {
        // Read length prefix (4 bytes)
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        // Read JSON data
        let mut buffer = vec![0u8; len];
        stream.read_exact(&mut buffer).await?;

        let message: P2PMessage = serde_json::from_slice(&buffer)?;
        Ok(message)
    }

    /// Send a heartbeat to a peer
    pub async fn send_heartbeat<S>(&self, stream: &mut S) -> Result<()>
    where
        S: AsyncReadExt + AsyncWriteExt + Unpin,
    {
        let heartbeat = P2PMessage::Heartbeat {
            agent_id: self.agent_id,
            timestamp: Utc::now(),
        };

        Self::send_message(stream, &heartbeat).await?;

        // Wait for response
        let response = Self::receive_message(stream).await?;
        match response {
            P2PMessage::Response {
                success, message, ..
            } => {
                if success {
                    tracing::debug!("Heartbeat acknowledged: {}", message);
                } else {
                    anyhow::bail!("Heartbeat failed: {}", message);
                }
            }
            _ => {
                anyhow::bail!("Unexpected response to heartbeat");
            }
        }

        Ok(())
    }

    /// Request metrics from a peer
    #[allow(dead_code)]
    pub async fn request_metrics<S>(&self, stream: &mut S) -> Result<serde_json::Value>
    where
        S: AsyncReadExt + AsyncWriteExt + Unpin,
    {
        let request = P2PMessage::MetricsRequest {
            agent_id: self.agent_id,
            timestamp: Utc::now(),
        };

        Self::send_message(stream, &request).await?;

        // Wait for response
        let response = Self::receive_message(stream).await?;
        match response {
            P2PMessage::Response {
                success,
                data,
                message,
                ..
            } => {
                if success {
                    Ok(data.unwrap_or(serde_json::json!({})))
                } else {
                    anyhow::bail!("Metrics request failed: {}", message);
                }
            }
            _ => {
                anyhow::bail!("Unexpected response to metrics request");
            }
        }
    }
}
