//! Leaf pin format — see SPEC § JSON shape / `pins`.

pub const LEAF_PIN_HEX_LEN: usize = 64;

pub fn assert_valid_leaf_pins_format(pins_sorted: &[String]) -> Result<(), String> {
    for (i, p) in pins_sorted.iter().enumerate() {
        if !is_valid_pin_hex(p) {
            let shown = if p.chars().all(|c| c.is_ascii_graphic()) && p.len() < 80 {
                format!("\"{p}\"")
            } else {
                format!("{:?}", p)
            };
            return Err(format!(
                "pin[{i}] must be {LEAF_PIN_HEX_LEN} lowercase hex chars (leaf DER SHA-256), got {shown}"
            ));
        }
    }
    Ok(())
}

pub fn leaf_pins_format_error(pins_sorted: &[String]) -> Option<String> {
    assert_valid_leaf_pins_format(pins_sorted).err()
}

fn is_valid_pin_hex(s: &str) -> bool {
    s.len() == LEAF_PIN_HEX_LEN && s.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_short() {
        let e = leaf_pins_format_error(&["aa".to_string()]).unwrap();
        assert!(e.contains("pin[0]"));
    }
}
