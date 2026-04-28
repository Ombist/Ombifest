//! Sign / verify manifest — see [Ombifest/SPEC.md](../../../SPEC.md) and Swift `PinManifestService`.

use crate::canonical::{canonical_payload_bytes, normalize_pins_array, normalize_pins_from_comma_separated};
use crate::crypto::{load_private_key_pem, verify_ed25519};
use crate::pins_format::{assert_valid_leaf_pins_format, leaf_pins_format_error};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct ManifestObj {
    pins: Vec<Value>,
    #[serde(rename = "validUntil")]
    valid_until: String,
    version: Value,
    signature: String,
}

pub fn parse_manifest_json(json_utf8: &str) -> Result<ParsedManifest, String> {
    let obj: ManifestObj = serde_json::from_str(json_utf8).map_err(|e| e.to_string())?;
    let version = match &obj.version {
        Value::Number(n) => n
            .as_i64()
            .ok_or_else(|| "version must be an integer".to_string())?,
        _ => return Err("version must be an integer".to_string()),
    };
    let mut pins = Vec::new();
    for p in &obj.pins {
        let s = p
            .as_str()
            .ok_or_else(|| "pins must be strings".to_string())?
            .to_string();
        pins.push(s);
    }
    Ok(ParsedManifest {
        pins,
        valid_until: obj.valid_until,
        version,
        signature_hex: obj.signature.trim().to_lowercase(),
    })
}

pub struct ParsedManifest {
    pub pins: Vec<String>,
    pub valid_until: String,
    pub version: i64,
    pub signature_hex: String,
}

/// Swift: notAfter > referenceDate - 300s
pub fn is_valid_until_fresh(valid_until: &str, reference: DateTime<Utc>) -> bool {
    let Ok(not_after) = parse_valid_until_utc(valid_until) else {
        return false;
    };
    let threshold = reference - chrono::Duration::seconds(300);
    not_after > threshold
}

fn parse_valid_until_utc(s: &str) -> Result<DateTime<Utc>, ()> {
    if let Ok(dt) = s.parse::<DateTime<Utc>>() {
        return Ok(dt);
    }
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.3fZ")
        .map(|n| Utc.from_utc_datetime(&n))
        .or_else(|_| {
            NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ")
                .map(|n| Utc.from_utc_datetime(&n))
        })
        .map_err(|_| ())
}

pub fn verify_manifest(
    manifest_json: &str,
    public_key_hex64: &str,
    reference: DateTime<Utc>,
) -> Result<Vec<String>, String> {
    let parsed = parse_manifest_json(manifest_json)?;
    if !is_valid_until_fresh(&parsed.valid_until, reference) {
        return Err("validUntil expired or within 300s grace edge".to_string());
    }
    let pins_sorted = normalize_pins_array(&parsed.pins);
    if let Some(err) = leaf_pins_format_error(&pins_sorted) {
        return Err(err);
    }
    let canonical = canonical_payload_bytes(parsed.version, &parsed.valid_until, &pins_sorted);
    let sig = hex::decode(parsed.signature_hex.trim()).map_err(|e| e.to_string())?;
    if sig.len() != 64 {
        return Err("signature must be Ed25519 64 bytes (128 hex chars)".to_string());
    }
    let pub_raw = hex::decode(public_key_hex64.trim().to_lowercase()).map_err(|e| e.to_string())?;
    if pub_raw.len() != 32 {
        return Err("public key hex must decode to 32 bytes (Ed25519 raw)".to_string());
    }
    verify_ed25519(canonical.as_bytes(), &pub_raw, &sig).map_err(|_| "Ed25519 signature invalid".to_string())?;
    if pins_sorted.is_empty() {
        return Err("no pins after normalize".to_string());
    }
    Ok(pins_sorted)
}

#[derive(Serialize)]
struct SignedManifestOut {
    pins: Vec<String>,
    #[serde(rename = "validUntil")]
    valid_until: String,
    version: i64,
    signature: String,
}

pub fn sign_manifest_object(
    pins_sorted: &[String],
    valid_until: &str,
    version: i64,
    pem: &str,
) -> Result<serde_json::Value, String> {
    assert_valid_leaf_pins_format(pins_sorted)?;
    let keypair = load_private_key_pem(pem)?;
    let canonical = canonical_payload_bytes(version, valid_until, pins_sorted);
    let sig = keypair.sign(canonical.as_bytes());
    let out = SignedManifestOut {
        pins: pins_sorted.to_vec(),
        valid_until: valid_until.to_string(),
        version,
        signature: hex::encode(sig.as_ref()),
    };
    serde_json::to_value(&out).map_err(|e| e.to_string())
}

pub fn normalize_pins_from_csv(s: &str) -> Vec<String> {
    normalize_pins_from_comma_separated(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn swift_300s_window() {
        let ref_dt = "2025-06-15T12:00:00.000Z"
            .parse::<DateTime<Utc>>()
            .unwrap();
        assert!(!is_valid_until_fresh("2025-06-15T11:55:00.000Z", ref_dt));
        assert!(is_valid_until_fresh("2025-06-15T11:55:01.000Z", ref_dt));
    }

    #[test]
    fn sign_then_verify_round_trip() {
        let rng = ring::rand::SystemRandom::new();
        let (pem, pub_hex) = crate::crypto::generate_key_pair_pkcs8_pem(&rng).unwrap();
        let pin_a = format!("{}00", "a".repeat(62));
        let pin_b = format!("{}00", "b".repeat(62));
        let mut pins = vec![pin_b.clone(), pin_a.clone()];
        pins.sort();
        let val = sign_manifest_object(&pins, "2099-12-31T23:59:59Z", 2, &pem).unwrap();
        let json = serde_json::to_string(&val).unwrap();
        let ref_dt = "2020-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let out = verify_manifest(&json, &pub_hex, ref_dt);
        assert_eq!(out, vec![pin_a, pin_b]);
    }

    #[test]
    fn verify_rejects_wrong_public_key() {
        let rng = ring::rand::SystemRandom::new();
        let (pem_a, _) = crate::crypto::generate_key_pair_pkcs8_pem(&rng).unwrap();
        let (_, pub_b) = crate::crypto::generate_key_pair_pkcs8_pem(&rng).unwrap();
        let pin = format!("{}00", "a".repeat(62));
        let val = sign_manifest_object(&[pin.clone()], "2099-01-01T00:00:00Z", 1, &pem_a).unwrap();
        let json = serde_json::to_string(&val).unwrap();
        let ref_dt = "2020-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let err = verify_manifest(&json, &pub_b, ref_dt).unwrap_err();
        assert!(err.contains("signature") || err.contains("invalid"));
    }

    #[test]
    fn golden_sign_output_shape() {
        // Stable pin and times: manifest JSON must contain sorted pins and validUntil.
        let rng = ring::rand::SystemRandom::new();
        let (pem, _pub) = crate::crypto::generate_key_pair_pkcs8_pem(&rng).unwrap();
        let pin = "00000000000000000000000000000000000000000000000000000000000000aa";
        let val = sign_manifest_object(&[pin.to_string()], "2099-06-01T00:00:00Z", 1, &pem).unwrap();
        let s = serde_json::to_string_pretty(&val).unwrap();
        assert!(s.contains("\"pins\""));
        assert!(s.contains(pin));
        assert!(s.contains("2099-06-01T00:00:00Z"));
        assert!(s.contains("\"signature\""));
    }
}
