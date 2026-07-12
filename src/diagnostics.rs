//! Bounded stderr capture for a supervised child process.

use std::io::{self, Read};
use std::thread::{self, JoinHandle};

/// Retained stderr bytes plus truncation metadata.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiagnosticSummary {
    pub bytes: Vec<u8>,
    pub truncated: bool,
    pub total_observed: usize,
}

impl DiagnosticSummary {
    #[must_use]
    pub fn lossy_text(&self) -> String {
        String::from_utf8_lossy(&self.bytes).into_owned()
    }
}

/// Failures while draining or joining the stderr worker.
#[derive(Debug)]
pub enum DiagnosticsError {
    Io(io::Error),
    Join(String),
}

impl std::fmt::Display for DiagnosticsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "stderr drain I/O error: {error}"),
            Self::Join(message) => write!(f, "stderr worker join failed: {message}"),
        }
    }
}

impl std::error::Error for DiagnosticsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

/// Spawn a worker that drains `reader` while retaining at most `retain_limit` bytes.
pub fn spawn_stderr_drainer<R>(
    mut reader: R,
    retain_limit: usize,
) -> JoinHandle<Result<DiagnosticSummary, io::Error>>
where
    R: Read + Send + 'static,
{
    thread::Builder::new()
        .name("mango-lsp-stderr-drain".to_owned())
        .spawn(move || {
            let mut summary = DiagnosticSummary::default();
            let mut buffer = [0_u8; 8192];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        summary.total_observed = summary.total_observed.saturating_add(n);
                        let remaining = retain_limit.saturating_sub(summary.bytes.len());
                        if remaining > 0 {
                            let take = remaining.min(n);
                            summary.bytes.extend_from_slice(&buffer[..take]);
                        }
                    }
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                    Err(error) => return Err(error),
                }
            }
            if summary.total_observed > summary.bytes.len() {
                summary.truncated = true;
            }
            Ok(summary)
        })
        .expect("stderr drain thread should spawn")
}

/// Join the stderr worker, mapping panic payloads into an error.
pub fn join_stderr(
    handle: JoinHandle<Result<DiagnosticSummary, io::Error>>,
) -> Result<DiagnosticSummary, DiagnosticsError> {
    match handle.join() {
        Ok(Ok(summary)) => Ok(summary),
        Ok(Err(error)) => Err(DiagnosticsError::Io(error)),
        Err(payload) => Err(DiagnosticsError::Join(format!("{payload:?}"))),
    }
}
