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
#[derive(Debug, Clone)]
pub struct ChildCommand {
    pub program: std::path::PathBuf,
    pub args: Vec<String>,
}

impl ChildCommand {
    #[must_use]
    pub fn new(program: impl Into<std::path::PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
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
            Self::Timeout { operation } => write!(f, "{operation} exceeded its configured bound"),
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
    pub fn spawn(command: ChildCommand, limits: DownstreamLimits) -> Result<Self, DownstreamError> {
        let mut child = Command::new(&command.program)
            .args(&command.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(DownstreamError::Spawn)?;

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
                let diagnostics = self.join_workers();
                return Err(DownstreamError::ChildExited {
                    operation: "graceful exit",
                    status: Some(exit_status),
                    diagnostics,
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
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(status) = self.try_reap()? {
                let diagnostics = self.join_workers();
                return Err(DownstreamError::ChildExited {
                    operation,
                    status: Some(status),
                    diagnostics,
                });
            }

            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(DownstreamError::Timeout { operation });
            }

            let slice = remaining.min(Duration::from_millis(50));
            match self.reader_rx.recv_timeout(slice) {
                Ok(ReaderEvent::Message(body)) => return Ok(body),
                Ok(ReaderEvent::Failed(source)) => {
                    return Err(DownstreamError::Frame { operation, source });
                }
                Ok(ReaderEvent::Eof) => {
                    let status = self.try_reap()?;
                    let diagnostics = self.join_workers();
                    return Err(DownstreamError::ChildExited {
                        operation,
                        status,
                        diagnostics,
                    });
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    let status = self.try_reap()?;
                    let diagnostics = self.join_workers();
                    return Err(DownstreamError::ChildExited {
                        operation,
                        status,
                        diagnostics,
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
                return Err(DownstreamError::Timeout { operation });
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
        DownstreamError::Timeout { operation } => DownstreamError::Cleanup {
            operation,
            message: format!("{operation} timed out; direct child was terminated and reaped"),
            diagnostics,
        },
        // Keep protocol/frame/io identity after cleanup so callers can match failures.
        other => other,
    }
}
