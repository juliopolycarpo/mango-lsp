//! Version 1 result envelope and redacted JSON Lines event writers.

use serde::Serialize;
use serde_json::Value;

use crate::diagnostics::DiagnosticSummary;

/// Public operation name embedded in envelopes and events.
pub const OPERATION_WORKSPACE_SYMBOLS: &str = "workspace_symbols";

/// Bounded public error kinds for parsed `workspace-symbols` invocations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    Configuration,
    Workspace,
    Query,
    Spawn,
    UnsupportedCapability,
    Protocol,
    Downstream,
    Timeout,
    Cleanup,
    Output,
}

impl ErrorKind {
    /// Exit status for a parsed operation that failed with this kind.
    #[must_use]
    pub fn exit_status(self) -> i32 {
        match self {
            Self::Configuration | Self::Workspace | Self::Query => 2,
            _ => 1,
        }
    }
}

/// Cleanup outcome reported in failure envelopes and terminal events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupState {
    NotRequired,
    Completed,
    Failed,
}

/// Normalized symbol emitted in a successful result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedSymbol {
    pub name: String,
    pub kind: String,
    pub container_name: Option<String>,
    pub location: NormalizedLocation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedLocation {
    pub uri: String,
    pub range: NormalizedRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedRange {
    pub start: NormalizedPosition,
    pub end: NormalizedPosition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedPosition {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct ResultEnvelope<'a> {
    schema_version: u32,
    operation: &'static str,
    status: &'static str,
    server: Option<&'a str>,
    result: Option<SymbolsResult<'a>>,
    error: Option<ErrorObject<'a>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct SymbolsResult<'a> {
    symbols: &'a [NormalizedSymbol],
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct ErrorObject<'a> {
    kind: ErrorKind,
    message: &'a str,
    cleanup: CleanupState,
}

/// Write the success envelope as one compact JSON object plus newline.
pub fn write_success_envelope(
    out: &mut dyn std::io::Write,
    server: &str,
    symbols: &[NormalizedSymbol],
) -> Result<(), std::io::Error> {
    let envelope = ResultEnvelope {
        schema_version: 1,
        operation: OPERATION_WORKSPACE_SYMBOLS,
        status: "ok",
        server: Some(server),
        result: Some(SymbolsResult { symbols }),
        error: None,
    };
    write_compact_json_line(out, &envelope)
}

/// Write the failure envelope as one compact JSON object plus newline.
pub fn write_error_envelope(
    out: &mut dyn std::io::Write,
    server: Option<&str>,
    kind: ErrorKind,
    message: &str,
    cleanup: CleanupState,
) -> Result<(), std::io::Error> {
    let envelope = ResultEnvelope {
        schema_version: 1,
        operation: OPERATION_WORKSPACE_SYMBOLS,
        status: "error",
        server,
        result: None,
        error: Some(ErrorObject {
            kind,
            message,
            cleanup,
        }),
    };
    write_compact_json_line(out, &envelope)
}

fn write_compact_json_line<T: Serialize>(
    out: &mut dyn std::io::Write,
    value: &T,
) -> Result<(), std::io::Error> {
    serde_json::to_writer(&mut *out, value).map_err(json_io_error)?;
    out.write_all(b"\n")?;
    Ok(())
}

fn json_io_error(error: serde_json::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, error)
}

/// Event severity for version 1 JSON Lines records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventLevel {
    Info,
    Warn,
    Error,
}

/// Authoritative redacted event writer for stderr.
pub struct EventWriter<W> {
    out: W,
}

impl<W: std::io::Write> EventWriter<W> {
    #[must_use]
    pub fn new(out: W) -> Self {
        Self { out }
    }

    pub fn operation_started(&mut self, server: Option<&str>) -> Result<(), std::io::Error> {
        self.emit(
            EventLevel::Info,
            "operation_started",
            server,
            serde_json::Map::new(),
        )
    }

    pub fn child_started(&mut self, server: &str) -> Result<(), std::io::Error> {
        self.emit(
            EventLevel::Info,
            "child_started",
            Some(server),
            serde_json::Map::new(),
        )
    }

    pub fn downstream_notification(
        &mut self,
        server: &str,
        source: &str,
        severity: Option<i64>,
    ) -> Result<(), std::io::Error> {
        let mut extra = serde_json::Map::new();
        extra.insert("source".to_owned(), Value::String(source.to_owned()));
        if let Some(severity) = severity {
            extra.insert("severity".to_owned(), Value::Number(severity.into()));
        }
        // Never include notification text — only redacted metadata.
        self.emit(
            EventLevel::Info,
            "downstream_notification",
            Some(server),
            extra,
        )
    }

    pub fn child_stopped(
        &mut self,
        server: &str,
        diagnostics: &DiagnosticSummary,
    ) -> Result<(), std::io::Error> {
        let mut extra = serde_json::Map::new();
        extra.insert(
            "observed_bytes".to_owned(),
            Value::Number(diagnostics.total_observed.into()),
        );
        extra.insert(
            "retained_bytes".to_owned(),
            Value::Number(diagnostics.bytes.len().into()),
        );
        extra.insert("truncated".to_owned(), Value::Bool(diagnostics.truncated));
        self.emit(EventLevel::Info, "child_stopped", Some(server), extra)
    }

    pub fn operation_succeeded(
        &mut self,
        server: &str,
        symbol_count: usize,
    ) -> Result<(), std::io::Error> {
        let mut extra = serde_json::Map::new();
        extra.insert(
            "symbol_count".to_owned(),
            Value::Number(symbol_count.into()),
        );
        self.emit(EventLevel::Info, "operation_succeeded", Some(server), extra)
    }

    pub fn operation_failed(
        &mut self,
        server: Option<&str>,
        kind: ErrorKind,
        cleanup: CleanupState,
    ) -> Result<(), std::io::Error> {
        let mut extra = serde_json::Map::new();
        extra.insert(
            "kind".to_owned(),
            serde_json::to_value(kind).map_err(json_io_error)?,
        );
        extra.insert(
            "cleanup".to_owned(),
            serde_json::to_value(cleanup).map_err(json_io_error)?,
        );
        self.emit(EventLevel::Error, "operation_failed", server, extra)
    }

    fn emit(
        &mut self,
        level: EventLevel,
        event: &str,
        server: Option<&str>,
        extra: serde_json::Map<String, Value>,
    ) -> Result<(), std::io::Error> {
        let mut object = serde_json::Map::new();
        object.insert("schema_version".to_owned(), Value::Number(1.into()));
        object.insert(
            "level".to_owned(),
            serde_json::to_value(level).map_err(json_io_error)?,
        );
        object.insert("event".to_owned(), Value::String(event.to_owned()));
        object.insert(
            "operation".to_owned(),
            Value::String(OPERATION_WORKSPACE_SYMBOLS.to_owned()),
        );
        if let Some(server) = server {
            object.insert("server".to_owned(), Value::String(server.to_owned()));
        }
        for (key, value) in extra {
            object.insert(key, value);
        }
        write_compact_json_line(&mut self.out, &Value::Object(object))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_envelope_is_one_compact_line() {
        let mut buf = Vec::new();
        write_success_envelope(
            &mut buf,
            "fixture",
            &[NormalizedSymbol {
                name: "Widget".to_owned(),
                kind: "class".to_owned(),
                container_name: None,
                location: NormalizedLocation {
                    uri: "file:///workspace/src/widget.rs".to_owned(),
                    range: NormalizedRange {
                        start: NormalizedPosition {
                            line: 0,
                            character: 0,
                        },
                        end: NormalizedPosition {
                            line: 0,
                            character: 6,
                        },
                    },
                },
            }],
        )
        .unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.ends_with('\n'));
        assert_eq!(text.lines().count(), 1);
        let value: Value = serde_json::from_str(text.trim_end()).unwrap();
        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["status"], "ok");
        assert_eq!(value["result"]["symbols"][0]["kind"], "class");
        assert!(value["error"].is_null());
    }
}
