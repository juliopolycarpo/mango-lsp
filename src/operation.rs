//! Orchestrate the configuration-backed workspace-symbols CLI operation.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::config::{self, ConfigLimits};
use crate::diagnostics::DiagnosticSummary;
use crate::lifecycle::{
    ChildCommand, DownstreamError, DownstreamLimits, DownstreamSession, WorkspaceSymbolParams,
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

/// Failure raised before any result envelope was committed to stdout.
struct Fatal {
    message: String,
    cleanup: CleanupState,
}

fn fatal(cleanup: CleanupState) -> impl Fn(std::io::Error) -> Fatal {
    move |error| Fatal {
        message: error.to_string(),
        cleanup,
    }
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
        let _ = write_error_envelope(
            stdout,
            None,
            ErrorKind::Output,
            &error.to_string(),
            CleanupState::NotRequired,
        );
        return ExitCode::from(1);
    }

    match run_inner(request, limits, config_limits, stdout, &mut events) {
        Ok(code) => code,
        Err(error) => {
            let _ = write_error_envelope(
                stdout,
                None,
                ErrorKind::Output,
                &error.message,
                error.cleanup,
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
) -> Result<ExitCode, Fatal> {
    if let Err(error) = config::validate_query(&request.query) {
        return emit_failure(
            stdout,
            events,
            None,
            ErrorKind::Query,
            &error.to_string(),
            CleanupState::NotRequired,
        );
    }

    let server = match config::load_server_config(&request.config, config_limits) {
        Ok(server) => server,
        Err(error) => {
            return emit_failure(
                stdout,
                events,
                None,
                ErrorKind::Configuration,
                &error.to_string(),
                CleanupState::NotRequired,
            );
        }
    };

    let (workspace_dir, root_uri) = match config::resolve_workspace(&request.workspace) {
        Ok(value) => value,
        Err(error) => {
            return emit_failure(
                stdout,
                events,
                Some(server.id.as_str()),
                ErrorKind::Workspace,
                &error.to_string(),
                CleanupState::NotRequired,
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
            return emit_failure(
                stdout,
                events,
                Some(server.id.as_str()),
                map_downstream_error(&error),
                &error.to_string(),
                CleanupState::NotRequired,
            );
        }
    };

    // From here on the session drop kills and reaps the child on early
    // returns, so fatal failures report cleanup as completed.
    events
        .child_started(&server.id)
        .map_err(fatal(CleanupState::Completed))?;

    let params = WorkspaceSymbolParams {
        process_id: std::process::id(),
        root_uri: &root_uri,
        workspace_folder_name: &folder_name,
        query: &request.query,
    };

    match session.run_workspace_symbols(params) {
        Ok(outcome) => {
            for notification in &outcome.notifications {
                events
                    .downstream_notification(
                        &server.id,
                        &notification.method,
                        notification.severity,
                    )
                    .map_err(fatal(CleanupState::Completed))?;
            }
            events
                .child_stopped(&server.id, &outcome.diagnostics)
                .map_err(fatal(CleanupState::Completed))?;

            let symbols = match symbols::normalize_workspace_symbols(&outcome.symbols_raw) {
                Ok(symbols) => symbols,
                Err(error) => {
                    return emit_failure(
                        stdout,
                        events,
                        Some(server.id.as_str()),
                        map_symbol_error(&error),
                        &error.to_string(),
                        CleanupState::Completed,
                    );
                }
            };

            write_success_envelope(stdout, &server.id, &symbols)
                .map_err(fatal(CleanupState::Completed))?;
            // The success envelope is committed; a failed event write must
            // not fail the invocation or add a second envelope.
            let _ = events.operation_succeeded(&server.id, symbols.len());
            Ok(ExitCode::SUCCESS)
        }
        Err(error) => {
            let (kind, cleanup) = map_downstream_error_with_cleanup(&error);
            if let Some(diagnostics) = diagnostics_from_error(&error) {
                let _ = events.child_stopped(&server.id, diagnostics);
            }
            emit_failure(
                stdout,
                events,
                Some(server.id.as_str()),
                kind,
                &error.to_string(),
                cleanup,
            )
        }
    }
}

fn emit_failure(
    stdout: &mut dyn std::io::Write,
    events: &mut EventWriter<&mut dyn std::io::Write>,
    server: Option<&str>,
    kind: ErrorKind,
    message: &str,
    cleanup: CleanupState,
) -> Result<ExitCode, Fatal> {
    write_error_envelope(stdout, server, kind, message, cleanup).map_err(fatal(cleanup))?;
    // The error envelope is committed; a failed event write must not add a
    // second envelope or change the documented exit status.
    let _ = events.operation_failed(server, kind, cleanup);
    Ok(ExitCode::from(kind.exit_status() as u8))
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

fn diagnostics_from_error(error: &DownstreamError) -> Option<&DiagnosticSummary> {
    match error {
        DownstreamError::ChildExited { diagnostics, .. }
        | DownstreamError::Cleanup { diagnostics, .. }
        | DownstreamError::Timeout { diagnostics, .. }
        | DownstreamError::UnsupportedCapability { diagnostics, .. } => Some(diagnostics),
        _ => None,
    }
}
