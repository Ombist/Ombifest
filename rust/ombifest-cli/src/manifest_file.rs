//! Limited manifest read — bounded file size per SPEC.

use std::fs;
use std::path::Path;

pub const DEFAULT_MAX_MANIFEST_BYTES: u64 = 1024 * 1024;

pub fn read_manifest_utf8_limited(path: &Path, max_bytes: u64) -> Result<String, String> {
    let meta = fs::metadata(path).map_err(|e| e.to_string())?;
    if !meta.is_file() {
        return Err("manifest path must be a regular file".to_string());
    }
    let size = meta.len();
    if size > max_bytes {
        return Err(format!(
            "manifest file exceeds {max_bytes} bytes (size={size})"
        ));
    }
    fs::read_to_string(path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn rejects_too_large() {
        let dir = std::env::temp_dir();
        let p = dir.join("ombifest-oversize-test.json");
        let mut f = fs::File::create(&p).unwrap();
        let body = vec![b' '; (DEFAULT_MAX_MANIFEST_BYTES as usize) + 1];
        f.write_all(&body).unwrap();
        drop(f);
        let r = read_manifest_utf8_limited(&p, DEFAULT_MAX_MANIFEST_BYTES);
        let _ = fs::remove_file(&p);
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("exceeds"));
    }
}
