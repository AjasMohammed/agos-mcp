use serde_json::Value;

/// Canonical key → accepted aliases. Pre-normalization rewrites top-level
/// alias keys to their canonical form before schema validation, so a model
/// that emits `recipient` instead of `to` still succeeds.
const ALIAS_MAP: &[(&str, &[&str])] = &[
    ("to", &["recipient", "recipients", "email", "to_addresses"]),
];

pub fn normalize_args(args: &mut Value) {
    let Some(obj) = args.as_object_mut() else {
        return;
    };
    for (canonical, aliases) in ALIAS_MAP {
        if obj.contains_key(*canonical) {
            for a in *aliases {
                if obj.remove(*a).is_some() {
                    tracing::debug!(
                        canonical = %canonical,
                        alias = %a,
                        "dropped alias key; canonical already present"
                    );
                }
            }
            continue;
        }
        for a in *aliases {
            if let Some(v) = obj.remove(*a) {
                tracing::debug!(canonical = %canonical, alias = %a, "rewrote alias to canonical");
                obj.insert((*canonical).to_string(), v);
                break;
            }
        }
    }
}

pub fn validate_against_schema(args: &Value, schema: &Value) -> Result<(), super::McpError> {
    let validator = jsonschema::JSONSchema::compile(schema)
        .map_err(|e| super::McpError::SchemaValidation(e.to_string()))?;

    if let Err(errors) = validator.validate(args) {
        let err_msgs: Vec<String> = errors.map(|e| e.to_string()).collect();
        let received: Vec<String> = args
            .as_object()
            .map(|o| o.keys().cloned().collect())
            .unwrap_or_default();
        let expected: Vec<String> = schema
            .get("properties")
            .and_then(|p| p.as_object())
            .map(|o| o.keys().cloned().collect())
            .unwrap_or_default();
        let required: Vec<String> = schema
            .get("required")
            .and_then(|r| r.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let detail = format!(
            "{}. Received keys: {:?}. Expected keys: {:?}. Required: {:?}",
            err_msgs.join(", "),
            received,
            expected,
            required
        );
        return Err(super::McpError::SchemaValidation(detail));
    }

    Ok(())
}
