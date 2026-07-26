// JSONC parsing and recursive config-value merging, split out of
// `config.rs` to stay under the 300-line file limit.

/// Recursively merges `b` into `a`.
/// For the "extensions" key, it performs key-based merging using the "name" field.
pub(crate) fn merge_json_value(a: &mut serde_json::Value, b: serde_json::Value) {
    match (a, b) {
        (serde_json::Value::Object(a_map), serde_json::Value::Object(b_map)) => {
            for (k, v) in b_map {
                if k == "extensions" {
                    if let Some(serde_json::Value::Array(a_exts)) = a_map.get_mut("extensions") {
                        if let serde_json::Value::Array(b_exts) = v {
                            merge_extensions_array(a_exts, b_exts);
                        }
                    } else {
                        a_map.insert(k, v);
                    }
                } else {
                    let entry = a_map.entry(k).or_insert(serde_json::Value::Null);
                    merge_json_value(entry, v);
                }
            }
        }
        (a, b) => {
            *a = b;
        }
    }
}

fn merge_extensions_array(a_exts: &mut Vec<serde_json::Value>, b_exts: Vec<serde_json::Value>) {
    for b_ext in b_exts {
        if let Some(b_name) = b_ext.get("name").and_then(serde_json::Value::as_str) {
            let mut found = false;
            for a_ext in a_exts.iter_mut() {
                if a_ext.get("name").and_then(serde_json::Value::as_str) == Some(b_name) {
                    merge_json_value(a_ext, b_ext.clone());
                    found = true;
                    break;
                }
            }
            if !found {
                a_exts.push(b_ext);
            }
        } else {
            a_exts.push(b_ext);
        }
    }
}

/// Parses a JSONC string into `serde_json::Value`.
pub(crate) fn parse_jsonc(content: &str) -> Result<serde_json::Value, crate::error::UnifiedError> {
    jsonc_parser::parse_to_serde_value(content, &jsonc_parser::ParseOptions::default())
        .map_err(|e| crate::error::UnifiedError::l1(format!("JSONC parse error: {e:?}"), "Config"))?
        .ok_or_else(|| crate::error::UnifiedError::l1("JSONC parsed to empty value", "Config"))
}
