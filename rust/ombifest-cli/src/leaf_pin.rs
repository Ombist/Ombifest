//! Leaf certificate DER SHA-256 hex — matches Ombifest leaf-pin (openssl x509 -outform DER).

use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::Command;

pub fn leaf_pin_from_cert_pem(cert_path: &Path) -> Result<String, String> {
    let out = Command::new("openssl")
        .args(["x509", "-in"])
        .arg(cert_path)
        .args(["-outform", "DER"])
        .output()
        .map_err(|e| format!("openssl x509 failed: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!("openssl x509 failed: {err}"));
    }
    let digest = Sha256::digest(&out.stdout);
    Ok(hex::encode(digest))
}
