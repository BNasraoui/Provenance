//! Canonical bytes, and the hashes taken over them.
//!
//! A graph digest and a reference id are claims about bytes somebody else
//! holds, so the bytes have to be produced the same way every time: object
//! keys sorted, no incidental whitespace, one SHA-256 over the result. Two
//! machines that agree on the graph must agree on the digit string, or the
//! pin means nothing.

use super::{incomplete, GraphReferenceError};
use serde::Serialize;
use sha2::{Digest, Sha256};

pub(super) fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, GraphReferenceError> {
    let value = serde_json::to_value(value).map_err(incomplete)?;
    let mut bytes = Vec::new();
    write_canonical_json(&value, &mut bytes)?;
    Ok(bytes)
}

fn write_canonical_json(
    value: &serde_json::Value,
    output: &mut Vec<u8>,
) -> Result<(), GraphReferenceError> {
    match value {
        serde_json::Value::Object(map) => {
            output.push(b'{');
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by_key(|(key, _)| key.as_str());
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                output.extend(serde_json::to_vec(key).map_err(incomplete)?);
                output.push(b':');
                write_canonical_json(value, output)?;
            }
            output.push(b'}');
        }
        serde_json::Value::Array(values) => {
            output.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                write_canonical_json(value, output)?;
            }
            output.push(b']');
        }
        _ => output.extend(serde_json::to_vec(value).map_err(incomplete)?),
    }
    Ok(())
}

pub(super) fn digest(bytes: &[u8]) -> String {
    format!("sha256:{}", sha256(bytes))
}

pub(super) fn sha256(bytes: &[u8]) -> String {
    use std::fmt::Write;

    let digest = Sha256::digest(bytes);
    digest.iter().fold(
        String::with_capacity(digest.len() * 2),
        |mut output, byte| {
            write!(output, "{byte:02x}").expect("writing to a String cannot fail");
            output
        },
    )
}
