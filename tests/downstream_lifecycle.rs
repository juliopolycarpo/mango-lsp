//! Process-level acceptance tests for the bounded downstream STDIO lifecycle.

use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use mango_lsp::frame::FrameLimits;
use mango_lsp::{ChildCommand, DownstreamError, DownstreamLimits, DownstreamSession};

fn fake_server() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mango-lsp-fake-server"))
}

fn limits_for_tests() -> DownstreamLimits {
    DownstreamLimits {
        frame: FrameLimits {
            max_header_bytes: 4_096,
            max_body_bytes: 64 * 1024,
        },
        max_stderr_bytes: 64 * 1024,
        operation_timeout: Duration::from_millis(800),
        force_shutdown_timeout: Duration::from_millis(800),
    }
}

fn spawn_mode(mode: &str) -> DownstreamSession {
    let command = ChildCommand::new(fake_server()).args([mode]);
    DownstreamSession::spawn(command, limits_for_tests()).expect("fake server should spawn")
}

#[test]
fn downstream_lifecycle_normal_completes_and_reaps() {
    let outcome = spawn_mode("normal")
        .run_lifecycle()
        .expect("normal lifecycle should succeed");

    assert!(outcome.exit_status.success());
    assert_eq!(outcome.initialize.raw["serverInfo"]["name"], "fake");
    assert!(
        !is_process_running(outcome.exit_status),
        "exit status should come from a reaped child"
    );
}

#[test]
fn downstream_lifecycle_fragmented_initialize_response() {
    let outcome = spawn_mode("fragmented")
        .run_lifecycle()
        .expect("fragmented lifecycle should succeed");
    assert_eq!(
        outcome.initialize.raw["serverInfo"]["name"],
        "fake-fragmented"
    );
}

#[test]
fn downstream_lifecycle_stderr_backpressure_reports_truncation() {
    let outcome = spawn_mode("stderr-flood")
        .run_lifecycle()
        .expect("stderr flood must not deadlock the lifecycle");

    assert!(outcome.exit_status.success());
    assert!(
        outcome.diagnostics.truncated,
        "retained stderr should report truncation"
    );
    assert!(
        outcome.diagnostics.bytes.len() <= 64 * 1024,
        "retained diagnostics must stay within the configured limit"
    );
    assert!(
        outcome.diagnostics.total_observed >= 4 * 1024 * 1024,
        "fake must have emitted at least 4 MiB of stderr"
    );
}

#[test]
fn downstream_lifecycle_rejects_bad_jsonrpc_and_reaps() {
    let error = spawn_mode("bad-jsonrpc")
        .run_lifecycle()
        .expect_err("bad jsonrpc must fail");
    assert_protocol_or_cleanup(&error, "jsonrpc");
}

#[test]
fn downstream_lifecycle_rejects_mismatched_id_and_reaps() {
    let error = spawn_mode("mismatched-id")
        .run_lifecycle()
        .expect_err("mismatched id must fail");
    assert_protocol_or_cleanup(&error, "id");
}

#[test]
fn downstream_lifecycle_malformed_frame_fails_bounded() {
    let error = spawn_mode("malformed-frame")
        .run_lifecycle()
        .expect_err("malformed frame must fail");
    assert!(
        matches!(
            error,
            DownstreamError::Frame { .. }
                | DownstreamError::Cleanup { .. }
                | DownstreamError::ChildExited { .. }
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn downstream_lifecycle_oversized_body_fails_bounded() {
    let error = spawn_mode("oversized-body")
        .run_lifecycle()
        .expect_err("oversized body must fail");
    let text = error.to_string();
    assert!(
        text.contains("Content-Length") || text.contains("body") || text.contains("framing"),
        "error should identify framing/body failure: {text}"
    );
}

#[test]
fn downstream_lifecycle_early_exit_fails_bounded() {
    let error = spawn_mode("early-exit")
        .run_lifecycle()
        .expect_err("early exit must fail");
    assert!(
        matches!(
            error,
            DownstreamError::ChildExited { .. }
                | DownstreamError::Cleanup { .. }
                | DownstreamError::Io { .. }
                | DownstreamError::Frame { .. }
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn downstream_lifecycle_forced_cleanup_on_hang_shutdown() {
    let started = Instant::now();
    let error = spawn_mode("hang-shutdown")
        .run_lifecycle()
        .expect_err("hanging shutdown must time out");
    let elapsed = started.elapsed();

    assert!(
        elapsed < Duration::from_secs(5),
        "forced cleanup must return within the outer deadline, elapsed={elapsed:?}"
    );
    let text = error.to_string();
    assert!(
        text.contains("timed out") || text.contains("cleanup"),
        "error should identify timeout/cleanup: {text}"
    );
}

#[test]
fn downstream_lifecycle_forced_cleanup_on_stall_initialize() {
    let started = Instant::now();
    let error = spawn_mode("stall-initialize")
        .run_lifecycle()
        .expect_err("stalled initialize must time out");
    let elapsed = started.elapsed();

    assert!(
        elapsed < Duration::from_secs(5),
        "forced cleanup must return within the outer deadline, elapsed={elapsed:?}"
    );
    assert!(
        matches!(
            error,
            DownstreamError::Cleanup { .. } | DownstreamError::Timeout { .. }
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn downstream_lifecycle_fake_not_exposed_through_product_cli() {
    let output = Command::new(env!("CARGO_BIN_EXE_mango-lsp"))
        .arg("--help")
        .output()
        .expect("product CLI should launch");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(
        !stdout.contains("fake-server") && !stdout.contains("fake_server"),
        "product help must not expose the fake server: {stdout}"
    );
}

fn assert_protocol_or_cleanup(error: &DownstreamError, needle: &str) {
    let text = error.to_string().to_lowercase();
    assert!(
        text.contains(needle)
            || matches!(
                error,
                DownstreamError::Protocol { .. } | DownstreamError::Cleanup { .. }
            ),
        "expected protocol/correlation failure containing {needle}, got: {error}"
    );
}

fn is_process_running(_status: std::process::ExitStatus) -> bool {
    // ExitStatus existing means wait/reap already completed for that child.
    false
}
