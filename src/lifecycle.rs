//! Direct-child STDIO LSP lifecycle: spawn, correlate, shut down, reap.

use std::io::Read;
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::diagnostics::{self, DiagnosticSummary};
use crate::frame::{self, FrameError, FrameLimits, write_frame};
use crate::protocol::{
    JsonRpcId, JsonRpcMessage, LspError, encode_message, exit_notification, expect_result,
    initialize_request, initialized_notification, parse_message, shutdown_request,
};

/// Explicit child command constructed by the caller. Never interpreted by a shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChildCommand {
    pub program: std::path::PathBuf,
    pub args: Vec<String>,
    /// Optional working directory for the direct child.
    pub current_dir: Option<std::path::PathBuf>,
}

impl ChildCommand {
    #[must_use]
    pub fn new(program: impl Into<std::path::PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            current_dir: None,
        }
    }

    #[must_use]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }

    #[must_use]
    pub fn current_dir(mut self, dir: impl Into<std::path::PathBuf>) -> Self {
        self.current_dir = Some(dir.into());
        self
    }
}

/// Finite bounds for one downstream lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DownstreamLimits {
    pub frame: FrameLimits,
    /// Maximum retained stderr bytes. Truncation is reported explicitly.
    pub max_stderr_bytes: usize,
    /// Bound for waiting on a correlated response or graceful child exit.
    pub operation_timeout: Duration,
    /// Bound for forced termination and final reap after a timeout path.
    pub force_shutdown_timeout: Duration,
}

impl Default for DownstreamLimits {
    fn default() -> Self {
        Self {
            frame: FrameLimits::default(),
            max_stderr_bytes: 64 * 1024,
            operation_timeout: Duration::from_secs(5),
            force_shutdown_timeout: Duration::from_secs(2),
        }
    }
}

/// Minimal initialize result retained after a successful lifecycle.
#[derive(Debug, Clone, PartialEq)]
pub struct InitializeResult {
    pub raw: Value,
}

/// Successful lifecycle evidence after the direct child has been reaped.
#[derive(Debug, Clone, PartialEq)]
pub struct LifecycleOutcome {
    pub initialize: InitializeResult,
    pub diagnostics: DiagnosticSummary,
    pub exit_status: ExitStatus,
}

/// Inputs for one configuration-backed workspace/symbol session.
#[derive(Debug, Clone, Copy)]
pub struct WorkspaceSymbolParams<'a> {
    pub process_id: u32,
    pub root_uri: &'a str,
    pub workspace_folder_name: &'a str,
    pub query: &'a str,
}

/// Redacted metadata for an observed downstream notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationMeta {
    pub method: String,
    pub severity: Option<i64>,
}

/// Successful workspace/symbol evidence after the direct child has been reaped.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceSymbolOutcome {
    pub initialize: InitializeResult,
    pub symbols_raw: Value,
    pub notifications: Vec<NotificationMeta>,
    pub diagnostics: DiagnosticSummary,
    pub exit_status: ExitStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingPhase {
    Initialize,
    Operation,
}

/// Failures from the bounded downstream lifecycle.
#[derive(Debug)]
pub enum DownstreamError {
    Spawn(std::io::Error),
    Io {
        operation: &'static str,
        source: std::io::Error,
    },
    Frame {
        operation: &'static str,
        source: FrameError,
    },
    Protocol {
        operation: &'static str,
        source: LspError,
    },
    Timeout {
        operation: &'static str,
        diagnostics: DiagnosticSummary,
    },
    ChildExited {
        operation: &'static str,
        status: Option<ExitStatus>,
        diagnostics: DiagnosticSummary,
    },
    Cleanup {
        operation: &'static str,
        message: String,
        diagnostics: DiagnosticSummary,
    },
    /// Server did not advertise workspace symbol support.
    UnsupportedCapability {
        message: String,
        diagnostics: DiagnosticSummary,
    },
}

impl std::fmt::Display for DownstreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spawn(error) => write!(f, "failed to spawn downstream child: {error}"),
            Self::Io { operation, source } => {
                write!(f, "{operation} failed with I/O error: {source}")
            }
            Self::Frame { operation, source } => {
                write!(f, "{operation} failed while framing: {source}")
            }
            Self::Protocol { operation, source } => {
                write!(f, "{operation} failed with protocol error: {source}")
            }
            Self::Timeout { operation, .. } => {
                write!(f, "{operation} exceeded its configured bound")
            }
            Self::ChildExited {
                operation,
                status,
                diagnostics,
            } => write!(
                f,
                "{operation} failed because the child exited early ({status:?}); diagnostics truncated={}, observed={}",
                diagnostics.truncated, diagnostics.total_observed
            ),
            Self::Cleanup {
                operation,
                message,
                diagnostics,
            } => write!(
                f,
                "{operation} cleanup failed: {message}; diagnostics truncated={}, observed={}",
                diagnostics.truncated, diagnostics.total_observed
            ),
            Self::UnsupportedCapability { message, .. } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for DownstreamError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Spawn(error) | Self::Io { source: error, .. } => Some(error),
            Self::Frame { source, .. } => Some(source),
            Self::Protocol { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Owns one direct child and runs the initialize → exit lifecycle.
pub struct DownstreamSession {
    child: Child,
    stdin: Option<ChildStdin>,
    reader: Option<JoinHandle<()>>,
    reader_rx: mpsc::Receiver<ReaderEvent>,
    stderr: Option<JoinHandle<Result<DiagnosticSummary, std::io::Error>>>,
    limits: DownstreamLimits,
    next_id: i64,
    reaped: bool,
}

enum ReaderEvent {
    Message(Vec<u8>),
    Failed(FrameError),
    Eof,
}

impl DownstreamSession {
    /// Spawn `command` with piped STDIO and start concurrent stderr draining.
    ///
    /// The child inherits the parent environment. When `current_dir` is set it
    /// becomes the child's working directory.
    pub fn spawn(command: ChildCommand, limits: DownstreamLimits) -> Result<Self, DownstreamError> {
        let mut builder = Command::new(&command.program);
        builder
            .args(&command.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(dir) = &command.current_dir {
            builder.current_dir(dir);
        }
        let mut child = builder.spawn().map_err(DownstreamError::Spawn)?;

        let stdin = child.stdin.take().ok_or_else(|| DownstreamError::Cleanup {
            operation: "spawn",
            message: "child stdin missing after spawn".to_owned(),
            diagnostics: DiagnosticSummary::default(),
        })?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| DownstreamError::Cleanup {
                operation: "spawn",
                message: "child stdout missing after spawn".to_owned(),
                diagnostics: DiagnosticSummary::default(),
            })?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| DownstreamError::Cleanup {
                operation: "spawn",
                message: "child stderr missing after spawn".to_owned(),
                diagnostics: DiagnosticSummary::default(),
            })?;

        let (reader_tx, reader_rx) = mpsc::channel();
        let frame_limits = limits.frame;
        let reader = thread::Builder::new()
            .name("mango-lsp-stdout-reader".to_owned())
            .spawn(move || read_stdout(stdout, frame_limits, reader_tx))
            .expect("stdout reader thread should spawn");
        let stderr_handle = diagnostics::spawn_stderr_drainer(stderr, limits.max_stderr_bytes);

        Ok(Self {
            child,
            stdin: Some(stdin),
            reader: Some(reader),
            reader_rx,
            stderr: Some(stderr_handle),
            limits,
            next_id: 1,
            reaped: false,
        })
    }

    /// Run the specification-ordered lifecycle and reap the direct child.
    pub fn run_lifecycle(mut self) -> Result<LifecycleOutcome, DownstreamError> {
        let initialize_id = self.allocate_id();
        let shutdown_id = self.allocate_id();

        let result = (|| {
            self.write_message(
                "initialize",
                &JsonRpcMessage::Request(initialize_request(initialize_id.clone())),
            )?;
            let initialize_body =
                self.wait_for_message("initialize response", self.limits.operation_timeout)?;
            let initialize_value = expect_result(
                parse_protocol("initialize response", &initialize_body)?,
                &initialize_id,
            )
            .map_err(|source| DownstreamError::Protocol {
                operation: "initialize response",
                source,
            })?;

            self.write_message(
                "initialized",
                &JsonRpcMessage::Notification(initialized_notification()),
            )?;
            self.write_message(
                "shutdown",
                &JsonRpcMessage::Request(shutdown_request(shutdown_id.clone())),
            )?;
            let shutdown_body =
                self.wait_for_message("shutdown response", self.limits.operation_timeout)?;
            expect_result(
                parse_protocol("shutdown response", &shutdown_body)?,
                &shutdown_id,
            )
            .map_err(|source| DownstreamError::Protocol {
                operation: "shutdown response",
                source,
            })?;

            self.write_message("exit", &JsonRpcMessage::Notification(exit_notification()))?;
            // Close stdin after exit so servers that read until EOF can finish.
            self.stdin.take();
            let exit_status = self.wait_for_exit("graceful exit", self.limits.operation_timeout)?;
            if !exit_status.success() {
                return Err(DownstreamError::ChildExited {
                    operation: "graceful exit",
                    status: Some(exit_status),
                    diagnostics: DiagnosticSummary::default(),
                });
            }

            let diagnostics = self.join_workers();
            self.reaped = true;
            Ok(LifecycleOutcome {
                initialize: InitializeResult {
                    raw: initialize_value,
                },
                diagnostics,
                exit_status,
            })
        })();

        match result {
            Ok(outcome) => Ok(outcome),
            Err(error) => Err(self.fail_closed(error)),
        }
    }

    /// Run initialize → workspace/symbol → shutdown for one configured workspace.
    pub fn run_workspace_symbols(
        mut self,
        params: WorkspaceSymbolParams<'_>,
    ) -> Result<WorkspaceSymbolOutcome, DownstreamError> {
        let initialize_id = self.allocate_id();
        let symbol_id = self.allocate_id();
        let shutdown_id = self.allocate_id();
        let mut notifications = Vec::new();

        let result = (|| {
            let init_params = crate::protocol::WorkspaceInitializeParams {
                process_id: params.process_id,
                root_uri: params.root_uri.to_owned(),
                workspace_folder_name: params.workspace_folder_name.to_owned(),
            };
            self.write_message(
                "initialize",
                &JsonRpcMessage::Request(crate::protocol::workspace_initialize_request(
                    initialize_id.clone(),
                    &init_params,
                )),
            )?;

            let initialize_value = self.wait_for_correlated_result(
                "initialize response",
                &initialize_id,
                PendingPhase::Initialize,
                params.root_uri,
                params.workspace_folder_name,
                &mut notifications,
            )?;

            if !crate::protocol::supports_workspace_symbol(&initialize_value) {
                return Err(DownstreamError::UnsupportedCapability {
                    message: "server does not advertise workspaceSymbolProvider".to_owned(),
                    diagnostics: DiagnosticSummary::default(),
                });
            }

            self.write_message(
                "initialized",
                &JsonRpcMessage::Notification(initialized_notification()),
            )?;
            self.write_message(
                "workspace/symbol",
                &JsonRpcMessage::Request(crate::protocol::workspace_symbol_request(
                    symbol_id.clone(),
                    params.query,
                )),
            )?;

            let symbol_value = self.wait_for_correlated_result(
                "workspace/symbol response",
                &symbol_id,
                PendingPhase::Operation,
                params.root_uri,
                params.workspace_folder_name,
                &mut notifications,
            )?;

            self.write_message(
                "shutdown",
                &JsonRpcMessage::Request(shutdown_request(shutdown_id.clone())),
            )?;
            self.wait_for_correlated_result(
                "shutdown response",
                &shutdown_id,
                PendingPhase::Operation,
                params.root_uri,
                params.workspace_folder_name,
                &mut notifications,
            )?;

            self.write_message("exit", &JsonRpcMessage::Notification(exit_notification()))?;
            self.stdin.take();
            let exit_status = self.wait_for_exit("graceful exit", self.limits.operation_timeout)?;
            if !exit_status.success() {
                return Err(DownstreamError::ChildExited {
                    operation: "graceful exit",
                    status: Some(exit_status),
                    diagnostics: DiagnosticSummary::default(),
                });
            }

            let diagnostics = self.join_workers();
            self.reaped = true;
            Ok(WorkspaceSymbolOutcome {
                initialize: InitializeResult {
                    raw: initialize_value,
                },
                symbols_raw: symbol_value,
                notifications,
                diagnostics,
                exit_status,
            })
        })();

        match result {
            Ok(outcome) => Ok(outcome),
            Err(error) => Err(self.fail_closed(error)),
        }
    }

    fn allocate_id(&mut self) -> JsonRpcId {
        let id = JsonRpcId::number(self.next_id);
        self.next_id += 1;
        id
    }

    fn write_message(
        &mut self,
        operation: &'static str,
        message: &JsonRpcMessage,
    ) -> Result<(), DownstreamError> {
        let body = encode_message(message)
            .map_err(|source| DownstreamError::Protocol { operation, source })?;
        let stdin = self.stdin.as_mut().ok_or(DownstreamError::Io {
            operation,
            source: std::io::Error::new(std::io::ErrorKind::BrokenPipe, "child stdin closed"),
        })?;
        write_frame(stdin, &body).map_err(|source| match source {
            FrameError::Io(error) => DownstreamError::Io {
                operation,
                source: error,
            },
            other => DownstreamError::Frame {
                operation,
                source: other,
            },
        })
    }

    fn wait_for_message(
        &mut self,
        operation: &'static str,
        timeout: Duration,
    ) -> Result<Vec<u8>, DownstreamError> {
        self.wait_for_message_until(operation, Instant::now() + timeout)
    }

    fn wait_for_message_until(
        &mut self,
        operation: &'static str,
        deadline: Instant,
    ) -> Result<Vec<u8>, DownstreamError> {
        loop {
            // Error paths must not join pipe workers here: the child may still
            // be alive and holding stderr open, so joining could block past the
            // configured bound. fail_closed() terminates and reaps first, then
            // joins and enriches the error with the real diagnostic summary.
            if let Some(status) = self.try_reap()? {
                return Err(DownstreamError::ChildExited {
                    operation,
                    status: Some(status),
                    diagnostics: DiagnosticSummary::default(),
                });
            }

            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(DownstreamError::Timeout {
                    operation,
                    diagnostics: DiagnosticSummary::default(),
                });
            }

            let slice = remaining.min(Duration::from_millis(50));
            match self.reader_rx.recv_timeout(slice) {
                Ok(ReaderEvent::Message(body)) => return Ok(body),
                Ok(ReaderEvent::Failed(source)) => {
                    return Err(DownstreamError::Frame { operation, source });
                }
                Ok(ReaderEvent::Eof) => {
                    let status = self.try_reap()?;
                    return Err(DownstreamError::ChildExited {
                        operation,
                        status,
                        diagnostics: DiagnosticSummary::default(),
                    });
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    let status = self.try_reap()?;
                    return Err(DownstreamError::ChildExited {
                        operation,
                        status,
                        diagnostics: DiagnosticSummary::default(),
                    });
                }
            }
        }
    }

    fn wait_for_correlated_result(
        &mut self,
        operation: &'static str,
        expected_id: &JsonRpcId,
        phase: PendingPhase,
        root_uri: &str,
        folder_name: &str,
        notifications: &mut Vec<NotificationMeta>,
    ) -> Result<Value, DownstreamError> {
        let deadline = Instant::now() + self.limits.operation_timeout;
        loop {
            let body = self.wait_for_message_until(operation, deadline)?;
            let message = parse_protocol(operation, &body)?;
            match message {
                JsonRpcMessage::Response(_) => {
                    return expect_result(message, expected_id)
                        .map_err(|source| DownstreamError::Protocol { operation, source });
                }
                JsonRpcMessage::Notification(notification) => {
                    if notification.method == "window/logMessage" {
                        let severity = notification
                            .params
                            .as_ref()
                            .and_then(|params| params.get("type"))
                            .and_then(Value::as_i64);
                        notifications.push(NotificationMeta {
                            method: notification.method,
                            severity,
                        });
                        continue;
                    }
                    return Err(DownstreamError::Protocol {
                        operation,
                        source: LspError::UnexpectedMessage(format!(
                            "unsupported notification {}",
                            notification.method
                        )),
                    });
                }
                JsonRpcMessage::Request(request) => {
                    if phase == PendingPhase::Operation
                        && request.method == "workspace/workspaceFolders"
                    {
                        self.write_message(
                            "workspace/workspaceFolders response",
                            &JsonRpcMessage::Response(crate::protocol::workspace_folders_response(
                                request.id,
                                root_uri,
                                folder_name,
                            )),
                        )?;
                        continue;
                    }
                    return Err(DownstreamError::Protocol {
                        operation,
                        source: LspError::UnexpectedMessage(format!(
                            "unsupported request {}",
                            request.method
                        )),
                    });
                }
            }
        }
    }

    fn wait_for_exit(
        &mut self,
        operation: &'static str,
        timeout: Duration,
    ) -> Result<ExitStatus, DownstreamError> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(status) = self.try_reap()? {
                return Ok(status);
            }
            if Instant::now() >= deadline {
                return Err(DownstreamError::Timeout {
                    operation,
                    diagnostics: DiagnosticSummary::default(),
                });
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn try_reap(&mut self) -> Result<Option<ExitStatus>, DownstreamError> {
        match self.child.try_wait() {
            Ok(Some(status)) => {
                self.reaped = true;
                Ok(Some(status))
            }
            Ok(None) => Ok(None),
            Err(source) => Err(DownstreamError::Io {
                operation: "wait",
                source,
            }),
        }
    }

    fn join_workers(&mut self) -> DiagnosticSummary {
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
        while self.reader_rx.try_recv().is_ok() {}

        match self.stderr.take() {
            Some(handle) => diagnostics::join_stderr(handle).unwrap_or_default(),
            None => DiagnosticSummary::default(),
        }
    }

    fn fail_closed(mut self, error: DownstreamError) -> DownstreamError {
        self.stdin.take();
        let diagnostics = self.force_reap_and_join();
        enrich_error(error, diagnostics)
    }

    fn force_reap_and_join(&mut self) -> DiagnosticSummary {
        if !self.reaped {
            match self.child.try_wait() {
                Ok(Some(_)) => self.reaped = true,
                Ok(None) => {
                    let _ = self.child.kill();
                    let deadline = Instant::now() + self.limits.force_shutdown_timeout;
                    loop {
                        match self.child.try_wait() {
                            Ok(Some(_)) => {
                                self.reaped = true;
                                break;
                            }
                            Ok(None) if Instant::now() < deadline => {
                                thread::sleep(Duration::from_millis(10));
                            }
                            _ => break,
                        }
                    }
                    if !self.reaped {
                        let _ = self.child.wait();
                        self.reaped = true;
                    }
                }
                Err(_) => {
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                    self.reaped = true;
                }
            }
        }
        self.join_workers()
    }
}

impl Drop for DownstreamSession {
    fn drop(&mut self) {
        self.stdin.take();
        if !self.reaped {
            let _ = self.child.kill();
            let _ = self.child.wait();
            self.reaped = true;
        }
        let _ = self.join_workers();
    }
}

fn read_stdout<R: Read>(mut stdout: R, limits: FrameLimits, tx: mpsc::Sender<ReaderEvent>) {
    loop {
        match frame::decode_frame(&mut stdout, limits) {
            Ok(body) => {
                if tx.send(ReaderEvent::Message(body)).is_err() {
                    return;
                }
            }
            Err(FrameError::UnexpectedEof { .. }) => {
                let _ = tx.send(ReaderEvent::Eof);
                return;
            }
            Err(FrameError::Io(error)) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
                let _ = tx.send(ReaderEvent::Eof);
                return;
            }
            Err(error) => {
                let _ = tx.send(ReaderEvent::Failed(error));
                return;
            }
        }
    }
}

fn parse_protocol(operation: &'static str, body: &[u8]) -> Result<JsonRpcMessage, DownstreamError> {
    let value: Value =
        serde_json::from_slice(body).map_err(|source| DownstreamError::Protocol {
            operation,
            source: LspError::InvalidJson(source),
        })?;
    match value.get("jsonrpc").and_then(Value::as_str) {
        Some("2.0") => {}
        _ => {
            return Err(DownstreamError::Protocol {
                operation,
                source: LspError::InvalidJsonRpcVersion,
            });
        }
    }
    parse_message(body).map_err(|source| DownstreamError::Protocol { operation, source })
}

fn enrich_error(error: DownstreamError, diagnostics: DiagnosticSummary) -> DownstreamError {
    match error {
        DownstreamError::ChildExited {
            operation, status, ..
        } => DownstreamError::ChildExited {
            operation,
            status,
            diagnostics,
        },
        DownstreamError::Cleanup {
            operation, message, ..
        } => DownstreamError::Cleanup {
            operation,
            message,
            diagnostics,
        },
        DownstreamError::Timeout { operation, .. } => DownstreamError::Timeout {
            operation,
            diagnostics,
        },
        DownstreamError::UnsupportedCapability { message, .. } => {
            DownstreamError::UnsupportedCapability {
                message,
                diagnostics,
            }
        }
        // Keep protocol/frame/io identity after cleanup so callers can match failures.
        other => other,
    }
}
