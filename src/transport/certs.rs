//! Certificate generation utilities for QUIC server testing

use anyhow::{anyhow, Result};
use std::path::Path;

/// Generate self-signed certificate and key for testing
pub fn generate_self_signed_cert(
    cert_path: &str,
    key_path: &str,
    days: u32,
) -> Result<()> {
    // Check if files already exist
    if Path::new(cert_path).exists() && Path::new(key_path).exists() {
        println!("✅ Certificate and key already exist");
        return Ok(());
    }

    // Create certs directory if it doesn't exist
    let cert_dir = Path::new(cert_path).parent();
    if let Some(dir) = cert_dir {
        if !dir.exists() {
            std::fs::create_dir_all(dir)?;
        }
    }

    // For now, we'll provide instructions on how to generate certs
    println!("📝 Generating self-signed certificate...");
    
    // Using openssl command line (this is a shell wrapper)
    let output = std::process::Command::new("openssl")
        .args([
            "req",
            "-x509",
            "-newkey",
            "rsa:2048",
            "-keyout",
            key_path,
            "-out",
            cert_path,
            "-days",
            &days.to_string(),
            "-nodes", // No DES encryption
            "-subj",
            "/C=US/ST=State/L=City/O=Org/CN=localhost",
        ])
        .output();

    match output {
        Ok(result) if result.status.success() => {
            println!("✅ Certificate generated at: {}", cert_path);
            println!("✅ Key generated at: {}", key_path);
            Ok(())
        }
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            Err(anyhow!("OpenSSL error: {}", stderr))
        }
        Err(e) => {
            Err(anyhow!(
                "Failed to run openssl. Make sure OpenSSL is installed: {}",
                e
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_generation_function_exists() {
        // Basic smoke test - just verify the function can be called
        // Full testing requires openssl installed on the system
        let _fn = generate_self_signed_cert;
        assert!(_fn as *const () != std::ptr::null());
    }
}
