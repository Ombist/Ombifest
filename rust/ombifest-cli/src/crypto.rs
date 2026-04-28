//! Ed25519 signing and verification (ring).

use ring::rand::SecureRandom;
use ring::signature::Ed25519KeyPair;
use ring::signature::{UnparsedPublicKey, ED25519};

pub fn load_private_key_pem(pem_utf8: &str) -> Result<Ed25519KeyPair, String> {
    let pem = pem::parse(pem_utf8).map_err(|e| e.to_string())?;
    if pem.tag() != "PRIVATE KEY" {
        return Err("expected PKCS#8 PEM tag PRIVATE KEY".to_string());
    }
    let der = pem.contents();
    Ed25519KeyPair::from_pkcs8(der)
        .or_else(|_| Ed25519KeyPair::from_pkcs8_maybe_unchecked(der))
        .map_err(|_| "invalid Ed25519 PKCS#8 private key".to_string())
}

/// Raw 32-byte public key hex from keypair (matches Node `rawPublicKeyHexFromKeyPair`).
pub fn raw_public_key_hex_from_keypair(keypair: &Ed25519KeyPair) -> String {
    hex::encode(keypair.public_key().as_ref())
}

pub fn generate_key_pair_pkcs8_pem(rng: &dyn SecureRandom) -> Result<(String, String), String> {
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(rng).map_err(|_| "keygen failed".to_string())?;
    let keypair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref())
        .or_else(|_| Ed25519KeyPair::from_pkcs8_maybe_unchecked(pkcs8.as_ref()))
        .map_err(|_| "internal key parse".to_string())?;
    let pub_hex = raw_public_key_hex_from_keypair(&keypair);
    let pem = pem::encode(&pem::Pem::new(
        "PRIVATE KEY",
        pkcs8.as_ref().to_vec(),
    ));
    Ok((pem, pub_hex))
}

pub fn verify_ed25519(canonical: &[u8], pub_raw_32: &[u8], signature: &[u8]) -> Result<(), ring::error::Unspecified> {
    let pk = UnparsedPublicKey::new(&ED25519, pub_raw_32);
    pk.verify(canonical, signature)
}
