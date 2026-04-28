//! Canonical manifest payload — must match Swift `PinManifestService` and the historical JSON semantics (compact `serde_json` object).

/// Comma-separated pins → trim, lowercase, filter empty, sort lexicographically.
pub fn normalize_pins_from_comma_separated(pins_arg: &str) -> Vec<String> {
    let mut v: Vec<String> = pins_arg
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    v.sort();
    v
}

pub fn normalize_pins_array(pins: &[String]) -> Vec<String> {
    let mut v: Vec<String> = pins
        .iter()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    v.sort();
    v
}

/// Exact JSON bytes as Node `JSON.stringify({ pins, validUntil, version })` with compact spacing.
pub fn canonical_payload_bytes(version: i64, valid_until: &str, pins_sorted: &[String]) -> String {
    let pins_json = pins_sorted
        .iter()
        .map(|p| format!("\"{p}\""))
        .collect::<Vec<_>>()
        .join(",");
    let vu = serde_json::to_string(valid_until).expect("validUntil serializes as JSON string");
    format!(r#"{{"pins":[{pins_json}],"validUntil":{vu},"version":{version}}}"#)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_matches_known_fixture() {
        let p_lo = format!("{}aa", "0".repeat(62));
        let p_hi = format!("{}aa", "1".repeat(62));
        let mut pins = vec![p_hi.clone(), p_lo.clone()];
        pins.sort();
        let s = canonical_payload_bytes(1, "2099-06-01T00:00:00Z", &pins);
        assert_eq!(
            s,
            format!(
                r#"{{"pins":["{p_lo}","{p_hi}"],"validUntil":"2099-06-01T00:00:00Z","version":1}}"#
            )
        );
    }

    #[test]
    fn normalize_comma() {
        assert_eq!(
            normalize_pins_from_comma_separated("BB22, aa11 "),
            vec!["aa11".to_string(), "bb22".to_string()]
        );
    }
}
