//! Internal library surface for mango-lsp.
//!
//! Integration tests exercise the downstream STDIO lifecycle through this
//! crate. Public items here are not a stable product API.

pub mod diagnostics;
pub mod frame;
pub mod lifecycle;
pub mod protocol;

pub use diagnostics::{DiagnosticSummary, DiagnosticsError};
pub use frame::{FrameError, FrameLimits, decode_frame, encode_frame};
pub use lifecycle::{
    ChildCommand, DownstreamError, DownstreamLimits, DownstreamSession, InitializeResult,
    LifecycleOutcome,
};
pub use protocol::{
    JsonRpcId, JsonRpcMessage, JsonRpcVersion, LspError, NotificationMessage, RequestMessage,
    ResponseError, ResponseMessage,
};
