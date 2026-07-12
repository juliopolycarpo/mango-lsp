//! Orchestrate the configuration-backed workspace-symbols CLI operation.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::config::{self, ConfigError, ConfigLimits, MAX_QUERY_BYTES};
use crate::diagnostics::DiagnosticSummary;
use crate::lifecycle::{
    ChildCommand, DownstreamError, DownstreamLimits, DownstreamSession, NotificationMeta,
    WorkspaceSymbolParams,
};
use crate::output::{
    CleanupState, ErrorKind, EventWriter, write_error_envelope, write_success_envelope,
};
use crate::symbols::{self, SymbolError};

/// Inputs for one parsed `workspace-symbols` invocation.
#[derive(Debug, Clone)]
pub struct WorkspaceSymbolsRequest {
    pub config: PathBuf,
    pub workspace: PathBuf,
    pub query: String,
}

/// Run the full operation against stdout/stderr writers and return an exit code.
pub fn run_workspace_symbols(
    request: WorkspaceSymbolsRequest,
    limits: DownstreamLimits,
    config_limits: ConfigLimits,
    stdout: &mut dyn std::io::Write,
    stderr: &mut dyn std::io::Write,
) -> ExitCode {
    let mut events = EventWriter::new(stderr);
    if let Err(error) = events.operation_started(None) {
        return output_failure(stdout, None, &error.to_string());
    }

    match run_inner(request, limits, config_limits, stdout, &mut events) {
        Ok(code) => code,
        Err(fatal) => {
            let _ = write_error_envelope(
                stdout,
                None,
                ErrorKind::Output,
                &fatal,
                CleanupState::NotRequired,
            );
            ExitCode::from(1)
        }
    }
}

fn run_inner(
    request: WorkspaceSymbolsRequest,
    limits: DownstreamLimits,
    config_limits: ConfigLimits,
    stdout: &mut dyn std::io::Write,
    events: &mut EventWriter<&mut dyn std::io::Write>,
) -> Result<ExitCode, String> {
    if let Err(error) = config::validate_query(&request.query) {
        return emit_boundary_failure(stdout, events, None, ErrorKind::Query, &error.to_string());
    }

    let server = match config::load_server_config(&request.config, config_limits) {
        Ok(server) => server,
        Err(error) => {
            return emit_boundary_failure(
                stdout,
                events,
                None,
                ErrorKind::Configuration,
                &error.to_string(),
            );
        }
    };

    let (workspace_dir, root_uri) = match config::resolve_workspace(&request.workspace) {
        Ok(value) => value,
        Err(error) => {
            let kind = match &error {
                ConfigError::Io { .. } | ConfigError::Invalid { .. } => ErrorKind::Workspace,
                _ => ErrorKind::Workspace,
            };
            return emit_boundary_failure(
                stdout,
                events,
                Some(server.id.as_str()),
                kind,
                &error.to_string(),
            );
        }
    };

    let folder_name = workspace_folder_name(&workspace_dir);
    let command = ChildCommand {
        program: server.command.program,
        args: server.command.args,
        current_dir: Some(workspace_dir),
    };

    let session = match DownstreamSession::spawn(command, limits) {
        Ok(session) => session,
        Err(error) => {
            return emit_post_spawnish_failure(
                stdout,
                events,
                Some(server.id.as_str()),
                map_downstream_error(&error),
                &error.to_string(),
                CleanupState::NotRequired,
                None,
            );
        }
    };

    events
        .child_started(&server.id)
        .map_err(|error| error.to_string())?;

    let params = WorkspaceSymbolParams {
        process_id: std::process::id(),
        root_uri: &root_uri,
        workspace_folder_name: &folder_name,
        query: &request.query,
    };

    match session.run_workspace_symbols(params) {
        Ok(outcome) => {
            for notification in &outcome.notifications {
                emit_notification(events, &server.id, notification)?;
            }
            events
                .child_stopped(&server.id, &outcome.diagnostics)
                .map_err(|error| error.to_string())?;

            let symbols = match symbols::normalize_workspace_symbols(&outcome.symbols_raw) {
                Ok(symbols) => symbols,
                Err(error) => {
                    return emit_post_spawnish_failure(
                        stdout,
                        events,
                        Some(server.id.as_str()),
                        map_symbol_error(&error),
                        &error.to_string(),
                        CleanupState::Completed,
                        Some(&outcome.diagnostics),
                    );
                }
            };

            write_success_envelope(stdout, &server.id, &symbols)
                .map_err(|error| error.to_string())?;
            events
                .operation_succeeded(&server.id, symbols.len())
                .map_err(|error| error.to_string())?;
            Ok(ExitCode::SUCCESS)
        }
        Err(error) => {
            let (kind, cleanup) = map_downstream_error_with_cleanup(&error);
            let diagnostics = diagnostics_from_error(&error);
            if let Some(diagnostics) = diagnostics.as_ref() {
                let _ = events.child_stopped(&server.id, diagnostics);
            }
            emit_post_spawnish_failure(
                stdout,
                events,
                Some(server.id.as_str()),
                kind,
                &error.to_string(),
                cleanup,
                diagnostics.as_ref(),
            )
        }
    }
}

fn emit_notification(
    events: &mut EventWriter<&mut dyn std::io::Write>,
    server: &str,
    notification: &NotificationMeta,
) -> Result<(), String> {
    events
        .downstream_notification(server, &notification.method, notification.severity)
        .map_err(|error| error.to_string())
}

fn emit_boundary_failure(
    stdout: &mut dyn std::io::Write,
    events: &mut EventWriter<&mut dyn std::io::Write>,
    server: Option<&str>,
    kind: ErrorKind,
    message: &str,
) -> Result<ExitCode, String> {
    write_error_envelope(stdout, server, kind, message, CleanupState::NotRequired)
        .map_err(|error| error.to_string())?;
    events
        .operation_failed(server, kind, CleanupState::NotRequired)
        .map_err(|error| error.to_string())?;
    Ok(ExitCode::from(kind.exit_status() as u8))
}

fn emit_post_spawnish_failure(
    stdout: &mut dyn std::io::Write,
    events: &mut EventWriter<&mut dyn std::io::Write>,
    server: Option<&str>,
    kind: ErrorKind,
    message: &str,
    cleanup: CleanupState,
    _diagnostics: Option<&DiagnosticSummary>,
) -> Result<ExitCode, String> {
    write_error_envelope(stdout, server, kind, message, cleanup)
        .map_err(|error| error.to_string())?;
    events
        .operation_failed(server, kind, cleanup)
        .map_err(|error| error.to_string())?;
    Ok(ExitCode::from(kind.exit_status() as u8))
}

fn output_failure(
    stdout: &mut dyn std::io::Write,
    server: Option<&str>,
    message: &str,
) -> ExitCode {
    let _ = write_error_envelope(
        stdout,
        server,
        ErrorKind::Output,
        message,
        CleanupState::NotRequired,
    );
    ExitCode::from(1)
}

fn workspace_folder_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "workspace".to_owned())
}

fn map_downstream_error(error: &DownstreamError) -> ErrorKind {
    match error {
        DownstreamError::Spawn(_) => ErrorKind::Spawn,
        DownstreamError::UnsupportedCapability { .. } => ErrorKind::UnsupportedCapability,
        DownstreamError::Timeout { .. } => ErrorKind::Timeout,
        DownstreamError::Cleanup { .. } => ErrorKind::Cleanup,
        DownstreamError::Protocol { source, .. } => match source {
            crate::protocol::LspError::ResponseError(_) => ErrorKind::Downstream,
            _ => ErrorKind::Protocol,
        },
        DownstreamError::Frame { .. }
        | DownstreamError::Io { .. }
        | DownstreamError::ChildExited { .. } => ErrorKind::Protocol,
    }
}

fn map_downstream_error_with_cleanup(error: &DownstreamError) -> (ErrorKind, CleanupState) {
    let kind = map_downstream_error(error);
    let cleanup = match error {
        DownstreamError::Cleanup { .. } => CleanupState::Failed,
        DownstreamError::Spawn(_) => CleanupState::NotRequired,
        _ => CleanupState::Completed,
    };
    (kind, cleanup)
}

fn map_symbol_error(error: &SymbolError) -> ErrorKind {
    match error {
        SymbolError::Invalid(_) | SymbolError::Oversized { .. } => ErrorKind::Protocol,
    }
}

fn diagnostics_from_error(error: &DownstreamError) -> Option<DiagnosticSummary> {
    match error {
        DownstreamError::ChildExited { diagnostics, .. }
        | DownstreamError::Cleanup { diagnostics, .. }
        | DownstreamError::Timeout { diagnostics, .. }
        | DownstreamError::UnsupportedCapability { diagnostics, .. } => Some(diagnostics.clone()),
        _ => None,
    }
}

/// Documented query byte limit for help text and tests.
#[must_use]
pub fn query_byte_limit() -> usize {
    MAX_QUERY_BYTES
}
