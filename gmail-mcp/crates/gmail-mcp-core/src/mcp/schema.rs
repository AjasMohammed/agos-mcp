use serde_json::Value;

pub fn validate_against_schema(args: &Value, schema: &Value) -> Result<(), super::McpError> {
    let validator = jsonschema::JSONSchema::compile(schema)
        .map_err(|e| super::McpError::SchemaValidation(e.to_string()))?;

    if let Err(errors) = validator.validate(args) {
        let err_msgs: Vec<String> = errors.map(|e| e.to_string()).collect();
        return Err(super::McpError::SchemaValidation(err_msgs.join(", ")));
    }

    Ok(())
}
