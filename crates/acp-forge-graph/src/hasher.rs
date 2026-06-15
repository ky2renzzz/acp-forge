//! Content-addressable hashing for event integrity.

use sha2::{Digest, Sha256};

/// Compute a SHA-256 hex digest of the given bytes.
#[must_use]
pub fn sha256_hex(data: &[u8]) -> String {
    let hash = Sha256::digest(data);
    hex::encode(hash)
}

/// Hash a serializable value via its canonical JSON representation.
///
/// Canonical JSON: keys sorted, no trailing whitespace, UTF-8.
pub fn hash_json<T: serde::Serialize>(value: &T) -> crate::Result<String> {
    let json = canonical_json(value)?;
    Ok(sha256_hex(json.as_bytes()))
}

/// Produce canonical JSON (deterministic key ordering).
pub fn canonical_json<T: serde::Serialize>(value: &T) -> crate::Result<String> {
    // serde_json with sorted keys via Value round-trip.
    let v = serde_json::to_value(value)?;
    Ok(canonical_value(&v))
}

fn canonical_value(v: &serde_json::Value) -> String {
    use serde_json::Value;
    match v {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let entries: Vec<String> = keys
                .iter()
                .map(|k| format!("{}:{}", serde_json::to_string(k).unwrap(), canonical_value(&map[*k])))
                .collect();
            format!("{{{}}}", entries.join(","))
        }
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(canonical_value).collect();
            format!("[{}]", items.join(","))
        }
        _ => serde_json::to_string(v).unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn canonical_ordering() {
        let v = json!({"b": 1, "a": 2});
        let c = canonical_value(&v);
        assert_eq!(c, r#"{"a":2,"b":1}"#);
    }

    #[test]
    fn hash_deterministic() {
        let v = json!({"z": [1, 2], "a": "hello"});
        let h1 = hash_json(&v).unwrap();
        let h2 = hash_json(&v).unwrap();
        assert_eq!(h1, h2);
    }
}
