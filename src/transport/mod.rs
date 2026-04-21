//! Post Office Transport Layer - Phase 1
//!
//! QUIC protocol implementation for high-performance producer connectivity

pub mod quic_server;
pub mod certs;
pub mod quic_server_production;

pub use quic_server::{
    ConnectionPool, QuicServer, QuicServerConfig, QuicServerStats, load_certs, load_key,
};
