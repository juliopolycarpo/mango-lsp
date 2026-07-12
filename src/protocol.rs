//! Minimal JSON-RPC / LSP message types for the S002 lifecycle subset.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 version token required by the lifecycle subset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JsonRpcVersion {
    #[serde(rename = "2.0")]
    V2,
}

/// JSON-RPC request/response identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcId {
    Number(i64),
    String(String),
}

impl JsonRpcId {
    #[must_use]
    pub fn number(value: i64) -> Self {
        Self::Number(value)
    }
}

/// A JSON-RPC request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestMessage {
    pub jsonrpc: JsonRpcVersion,
    pub id: JsonRpcId,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// A JSON-RPC notification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub jsonrpc: JsonRpcVersion,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// A JSON-RPC response error object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseError {
    pub code: i64,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// A JSON-RPC response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub jsonrpc: JsonRpcVersion,
    pub id: Option<JsonRpcId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

/// Any JSON-RPC message the lifecycle subset may observe.
///
/// Variant order matters for `untagged` deserialization: requests carry both
/// `id` and `method`, notifications carry `method` without `id`, and responses
/// carry `id` with `result`/`error` and no `method`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(RequestMessage),
    Notification(NotificationMessage),
    Response(ResponseMessage),
}

/// Protocol-level failures distinct from transport errors.
#[derive(Debug)]
pub enum LspError {
    InvalidJson(serde_json::Error),
    InvalidMessage(String),
    UnexpectedMessage(String),
    Correlation {
        expected: JsonRpcId,
        actual: Option<JsonRpcId>,
    },
    InvalidJsonRpcVersion,
    ResponseError(ResponseError),
}

impl std::fmt::Display for LspError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJson(error) => write!(f, "invalid JSON-RPC payload: {error}"),
            Self::InvalidMessage(message) => write!(f, "invalid JSON-RPC message: {message}"),
            Self::UnexpectedMessage(message) => write!(f, "unexpected JSON-RPC message: {message}"),
            Self::Correlation { expected, actual } => {
                write!(
                    f,
                    "response id mismatch: expected {expected:?}, got {actual:?}"
                )
            }
            Self::InvalidJsonRpcVersion => write!(f, "response missing or invalid jsonrpc version"),
            Self::ResponseError(error) => {
                write!(f, "JSON-RPC error {}: {}", error.code, error.message)
            }
        }
    }
}

impl std::error::Error for LspError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidJson(error) => Some(error),
            _ => None,
        }
    }
}

impl RequestMessage {
    #[must_use]
    pub fn new(id: JsonRpcId, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JsonRpcVersion::V2,
            id,
            method: method.into(),
            params,
        }
    }
}

impl NotificationMessage {
    #[must_use]
    pub fn new(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JsonRpcVersion::V2,
            method: method.into(),
            params,
        }
    }
}

/// Parse a framed body into a JSON-RPC message.
pub fn parse_message(body: &[u8]) -> Result<JsonRpcMessage, LspError> {
    serde_json::from_slice(body).map_err(LspError::InvalidJson)
}

/// Serialize a JSON-RPC message to UTF-8 JSON bytes.
pub fn encode_message(message: &JsonRpcMessage) -> Result<Vec<u8>, LspError> {
    serde_json::to_vec(message).map_err(LspError::InvalidJson)
}

/// Require a successful response that matches `expected_id`.
pub fn expect_result(message: JsonRpcMessage, expected_id: &JsonRpcId) -> Result<Value, LspError> {
    let JsonRpcMessage::Response(response) = message else {
        return Err(LspError::UnexpectedMessage(
            "expected a JSON-RPC response".to_owned(),
        ));
    };

    // serde rejects unknown jsonrpc enum values, so reaching a ResponseMessage
    // already proves jsonrpc == "2.0". Missing version fails deserialize.
    if response.id.as_ref() != Some(expected_id) {
        return Err(LspError::Correlation {
            expected: expected_id.clone(),
            actual: response.id,
        });
    }

    if let Some(error) = response.error {
        return Err(LspError::ResponseError(error));
    }

    // serde maps JSON `null` onto `Option::None`; treat that as a null result.
    Ok(response.result.unwrap_or(Value::Null))
}

/// Build the minimal `initialize` request used by the S002 lifecycle proof.
#[must_use]
pub fn initialize_request(id: JsonRpcId) -> RequestMessage {
    RequestMessage::new(
        id,
        "initialize",
        Some(serde_json::json!({
            "processId": null,
            "capabilities": {},
            "clientInfo": {
                "name": "mango-lsp",
                "version": env!("CARGO_PKG_VERSION")
            },
            "rootUri": null
        })),
    )
}

/// Parameters for the configuration-backed workspace initialize request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceInitializeParams {
    pub process_id: u32,
    pub root_uri: String,
    pub workspace_folder_name: String,
}

/// Build the S003 initialize request with workspace folders and static caps.
#[must_use]
pub fn workspace_initialize_request(
    id: JsonRpcId,
    params: &WorkspaceInitializeParams,
) -> RequestMessage {
    RequestMessage::new(
        id,
        "initialize",
        Some(serde_json::json!({
            "processId": params.process_id,
            "rootUri": params.root_uri,
            "capabilities": {
                "workspace": {
                    "workspaceFolders": true,
                    "symbol": {
                        "dynamicRegistration": false
                    }
                }
            },
            "clientInfo": {
                "name": "mango-lsp",
                "version": env!("CARGO_PKG_VERSION")
            },
            "workspaceFolders": [{
                "uri": params.root_uri,
                "name": params.workspace_folder_name
            }]
        })),
    )
}

/// Build a `workspace/symbol` request with the validated query text.
#[must_use]
pub fn workspace_symbol_request(id: JsonRpcId, query: &str) -> RequestMessage {
    RequestMessage::new(
        id,
        "workspace/symbol",
        Some(serde_json::json!({ "query": query })),
    )
}

/// Return whether initialize capabilities advertise static workspace symbols.
#[must_use]
pub fn supports_workspace_symbol(initialize_result: &Value) -> bool {
    match initialize_result.pointer("/capabilities/workspaceSymbolProvider") {
        Some(Value::Bool(value)) => *value,
        Some(Value::Object(_)) => true,
        _ => false,
    }
}

/// Build a response to `workspace/workspaceFolders`.
#[must_use]
pub fn workspace_folders_response(
    id: JsonRpcId,
    root_uri: &str,
    folder_name: &str,
) -> ResponseMessage {
    ResponseMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some(id),
        result: Some(serde_json::json!([{
            "uri": root_uri,
            "name": folder_name
        }])),
        error: None,
    }
}

#[must_use]
pub fn initialized_notification() -> NotificationMessage {
    NotificationMessage::new("initialized", Some(serde_json::json!({})))
}

#[must_use]
pub fn shutdown_request(id: JsonRpcId) -> RequestMessage {
    RequestMessage::new(id, "shutdown", None)
}

#[must_use]
pub fn exit_notification() -> NotificationMessage {
    NotificationMessage::new("exit", None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expect_result_rejects_mismatched_id() {
        let message = JsonRpcMessage::Response(ResponseMessage {
            jsonrpc: JsonRpcVersion::V2,
            id: Some(JsonRpcId::number(99)),
            result: Some(Value::Null),
            error: None,
        });
        let error = expect_result(message, &JsonRpcId::number(1)).unwrap_err();
        assert!(matches!(error, LspError::Correlation { .. }));
    }

    #[test]
    fn parse_message_distinguishes_notification_from_empty_response() {
        let body = br#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#;
        let message = parse_message(body).unwrap();
        assert!(matches!(
            message,
            JsonRpcMessage::Notification(NotificationMessage { method, .. }) if method == "initialized"
        ));
    }

    #[test]
    fn parse_message_rejects_missing_jsonrpc_version() {
        let body = br#"{"id":1,"result":null}"#;
        let error = parse_message(body).unwrap_err();
        assert!(matches!(error, LspError::InvalidJson(_)));
    }
}
