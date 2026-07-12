//! Typed validation and normalization of workspace symbol responses.

use serde_json::Value;

use crate::output::{NormalizedLocation, NormalizedPosition, NormalizedRange, NormalizedSymbol};

/// Maximum symbols retained from one workspace/symbol response.
pub const MAX_SYMBOLS: usize = 10_000;

/// Failures while validating a workspace/symbol result payload.
#[derive(Debug)]
pub enum SymbolError {
    Invalid(String),
    Oversized { count: usize, limit: usize },
}

impl std::fmt::Display for SymbolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) => write!(f, "{message}"),
            Self::Oversized { count, limit } => {
                write!(
                    f,
                    "workspace/symbol returned {count} symbols, exceeding limit of {limit}"
                )
            }
        }
    }
}

impl std::error::Error for SymbolError {}

/// Normalize a JSON-RPC `workspace/symbol` result into the public representation.
pub fn normalize_workspace_symbols(result: &Value) -> Result<Vec<NormalizedSymbol>, SymbolError> {
    if result.is_null() {
        return Ok(Vec::new());
    }
    let Some(items) = result.as_array() else {
        return Err(SymbolError::Invalid(
            "workspace/symbol result must be an array or null".to_owned(),
        ));
    };
    if items.len() > MAX_SYMBOLS {
        return Err(SymbolError::Oversized {
            count: items.len(),
            limit: MAX_SYMBOLS,
        });
    }

    let mut symbols = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        symbols.push(normalize_one(item, index)?);
    }
    Ok(symbols)
}

fn normalize_one(item: &Value, index: usize) -> Result<NormalizedSymbol, SymbolError> {
    let Some(object) = item.as_object() else {
        return Err(SymbolError::Invalid(format!(
            "symbol[{index}] must be an object"
        )));
    };

    let name = required_string(object, "name", index)?;
    let kind = map_symbol_kind(required_u64(object, "kind", index)?, index)?;
    let container_name = optional_string(object, "containerName");

    let location = object
        .get("location")
        .ok_or_else(|| SymbolError::Invalid(format!("symbol[{index}] is missing location")))?;
    let location = normalize_location(location, index)?;

    Ok(NormalizedSymbol {
        name,
        kind,
        container_name,
        location,
    })
}

fn normalize_location(value: &Value, index: usize) -> Result<NormalizedLocation, SymbolError> {
    let Some(object) = value.as_object() else {
        return Err(SymbolError::Invalid(format!(
            "symbol[{index}].location must be an object"
        )));
    };
    let uri = required_string(object, "uri", index)?;
    let range = object.get("range").ok_or_else(|| {
        SymbolError::Invalid(format!(
            "symbol[{index}].location is missing range (unresolved locations are unsupported)"
        ))
    })?;
    let range = normalize_range(range, index)?;
    Ok(NormalizedLocation { uri, range })
}

fn normalize_range(value: &Value, index: usize) -> Result<NormalizedRange, SymbolError> {
    let Some(object) = value.as_object() else {
        return Err(SymbolError::Invalid(format!(
            "symbol[{index}].location.range must be an object"
        )));
    };
    let start = normalize_position(
        object.get("start").ok_or_else(|| {
            SymbolError::Invalid(format!("symbol[{index}].location.range missing start"))
        })?,
        index,
        "start",
    )?;
    let end = normalize_position(
        object.get("end").ok_or_else(|| {
            SymbolError::Invalid(format!("symbol[{index}].location.range missing end"))
        })?,
        index,
        "end",
    )?;
    Ok(NormalizedRange { start, end })
}

fn normalize_position(
    value: &Value,
    index: usize,
    field: &str,
) -> Result<NormalizedPosition, SymbolError> {
    let Some(object) = value.as_object() else {
        return Err(SymbolError::Invalid(format!(
            "symbol[{index}].location.range.{field} must be an object"
        )));
    };
    Ok(NormalizedPosition {
        line: required_u32(object, "line", index, field)?,
        character: required_u32(object, "character", index, field)?,
    })
}

fn required_string(
    object: &serde_json::Map<String, Value>,
    key: &str,
    index: usize,
) -> Result<String, SymbolError> {
    match object.get(key) {
        Some(Value::String(value)) => Ok(value.clone()),
        Some(_) => Err(SymbolError::Invalid(format!(
            "symbol[{index}].{key} must be a string"
        ))),
        None => Err(SymbolError::Invalid(format!(
            "symbol[{index}] is missing {key}"
        ))),
    }
}

fn optional_string(object: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    object.get(key).and_then(Value::as_str).map(str::to_owned)
}

fn required_u64(
    object: &serde_json::Map<String, Value>,
    key: &str,
    index: usize,
) -> Result<u64, SymbolError> {
    match object.get(key).and_then(Value::as_u64) {
        Some(value) => Ok(value),
        None => Err(SymbolError::Invalid(format!(
            "symbol[{index}] is missing a numeric {key}"
        ))),
    }
}

fn required_u32(
    object: &serde_json::Map<String, Value>,
    key: &str,
    index: usize,
    field: &str,
) -> Result<u32, SymbolError> {
    match object.get(key).and_then(Value::as_u64) {
        Some(value) if value <= u64::from(u32::MAX) => Ok(value as u32),
        Some(_) => Err(SymbolError::Invalid(format!(
            "symbol[{index}].location.range.{field}.{key} is out of range"
        ))),
        None => Err(SymbolError::Invalid(format!(
            "symbol[{index}].location.range.{field} is missing numeric {key}"
        ))),
    }
}

fn map_symbol_kind(kind: u64, index: usize) -> Result<String, SymbolError> {
    let name = match kind {
        1 => "file",
        2 => "module",
        3 => "namespace",
        4 => "package",
        5 => "class",
        6 => "method",
        7 => "property",
        8 => "field",
        9 => "constructor",
        10 => "enum",
        11 => "interface",
        12 => "function",
        13 => "variable",
        14 => "constant",
        15 => "string",
        16 => "number",
        17 => "boolean",
        18 => "array",
        19 => "object",
        20 => "key",
        21 => "null",
        22 => "enum_member",
        23 => "struct",
        24 => "event",
        25 => "operator",
        26 => "type_parameter",
        _ => {
            return Err(SymbolError::Invalid(format!(
                "symbol[{index}] has unsupported SymbolKind"
            )));
        }
    };
    Ok(name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn null_becomes_empty_list() {
        assert!(
            normalize_workspace_symbols(&Value::Null)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn normalizes_symbol_information() {
        let result = json!([{
            "name": "Widget",
            "kind": 5,
            "location": {
                "uri": "file:///workspace/src/widget.rs",
                "range": {
                    "start": {"line": 0, "character": 0},
                    "end": {"line": 0, "character": 6}
                }
            }
        }]);
        let symbols = normalize_workspace_symbols(&result).unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].kind, "class");
        assert_eq!(symbols[0].container_name, None);
    }

    #[test]
    fn rejects_unresolved_workspace_symbol() {
        let result = json!([{
            "name": "Widget",
            "kind": 5,
            "location": { "uri": "file:///workspace/src/widget.rs" }
        }]);
        let error = normalize_workspace_symbols(&result).unwrap_err();
        assert!(error.to_string().contains("range"));
    }
}
