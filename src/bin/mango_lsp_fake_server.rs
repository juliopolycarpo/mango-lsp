//! Deterministic hostile/compliant fake language server for S002 tests.
//!
//! This binary is a test fixture. It is not part of the product CLI surface.

use std::io::{self, Read, Write};
use std::process::ExitCode;

use mango_lsp::frame::{FrameLimits, decode_frame, encode_frame};
use mango_lsp::protocol::{
    JsonRpcId, JsonRpcMessage, JsonRpcVersion, NotificationMessage, RequestMessage,
    ResponseMessage, parse_message,
};
use serde_json::json;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Fragmented,
    StderrFlood,
    BadJsonrpc,
    MismatchedId,
    MalformedFrame,
    OversizedBody,
    EarlyExit,
    StderrThenExit,
    HangShutdown,
    StallInitialize,
}

fn main() -> ExitCode {
    let mode = match parse_mode(std::env::args().nth(1).as_deref()) {
        Ok(mode) => mode,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };

    match run(mode) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("fake server failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn parse_mode(raw: Option<&str>) -> Result<Mode, String> {
    match raw.unwrap_or("normal") {
        "normal" => Ok(Mode::Normal),
        "fragmented" => Ok(Mode::Fragmented),
        "stderr-flood" => Ok(Mode::StderrFlood),
        "bad-jsonrpc" => Ok(Mode::BadJsonrpc),
        "mismatched-id" => Ok(Mode::MismatchedId),
        "malformed-frame" => Ok(Mode::MalformedFrame),
        "oversized-body" => Ok(Mode::OversizedBody),
        "early-exit" => Ok(Mode::EarlyExit),
        "stderr-then-exit" => Ok(Mode::StderrThenExit),
        "hang-shutdown" => Ok(Mode::HangShutdown),
        "stall-initialize" => Ok(Mode::StallInitialize),
        other => Err(format!("unknown fake-server mode: {other}")),
    }
}

fn run(mode: Mode) -> io::Result<()> {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    let limits = FrameLimits {
        max_header_bytes: 64 * 1024,
        max_body_bytes: 16 * 1024 * 1024,
    };

    if mode == Mode::StallInitialize {
        // Read forever without responding so the supervisor hits its bound.
        let mut sink = Vec::new();
        let _ = stdin.read_to_end(&mut sink);
        return Ok(());
    }

    if mode == Mode::EarlyExit {
        return Ok(());
    }

    if mode == Mode::MalformedFrame {
        let _initialize = read_request(&mut stdin, limits, "initialize")?;
        stdout.write_all(b"Content-Length: not-a-number\r\n\r\n")?;
        stdout.flush()?;
        return Ok(());
    }

    if mode == Mode::OversizedBody {
        let _initialize = read_request(&mut stdin, limits, "initialize")?;
        // Declare a body larger than the test-configured decoder limit.
        stdout.write_all(b"Content-Length: 1048576\r\n\r\n")?;
        stdout.flush()?;
        return Ok(());
    }

    let initialize = read_request(&mut stdin, limits, "initialize")?;
    if mode == Mode::StderrThenExit {
        // Consume the request first so the supervisor's write deterministically
        // succeeds, then die with only a stderr trace and no response.
        stderr.write_all(b"stderr-then-exit: simulated crash before initialize response\n")?;
        stderr.flush()?;
        return Ok(());
    }

    if mode == Mode::StderrFlood {
        // Exceed the default 64 KiB retention with at least 4 MiB of stderr.
        let chunk = vec![b'x'; 64 * 1024];
        for _ in 0..(4 * 1024 * 1024 / chunk.len()) {
            stderr.write_all(&chunk)?;
        }
        stderr.flush()?;
    }

    let initialize_id = initialize.id.clone();
    match mode {
        Mode::BadJsonrpc => {
            // Bypass typed serialization so jsonrpc can be wrong on purpose.
            write_raw(
                &mut stdout,
                &json!({
                    "jsonrpc": "1.0",
                    "id": id_to_json(&initialize_id),
                    "result": { "capabilities": {}, "serverInfo": { "name": "fake" } }
                }),
                false,
            )?;
        }
        Mode::MismatchedId => {
            write_response(
                &mut stdout,
                ResponseMessage {
                    jsonrpc: JsonRpcVersion::V2,
                    id: Some(JsonRpcId::number(9_999)),
                    result: Some(json!({
                        "capabilities": {},
                        "serverInfo": { "name": "fake" }
                    })),
                    error: None,
                },
                false,
            )?;
        }
        Mode::Fragmented => {
            write_response(
                &mut stdout,
                ResponseMessage {
                    jsonrpc: JsonRpcVersion::V2,
                    id: Some(initialize_id),
                    result: Some(json!({
                        "capabilities": {},
                        "serverInfo": { "name": "fake-fragmented" }
                    })),
                    error: None,
                },
                true,
            )?;
        }
        _ => {
            write_response(
                &mut stdout,
                ResponseMessage {
                    jsonrpc: JsonRpcVersion::V2,
                    id: Some(initialize_id),
                    result: Some(json!({
                        "capabilities": {},
                        "serverInfo": { "name": "fake" }
                    })),
                    error: None,
                },
                false,
            )?;
        }
    }

    if matches!(
        mode,
        Mode::BadJsonrpc | Mode::MismatchedId | Mode::MalformedFrame | Mode::OversizedBody
    ) {
        // Hostile protocol modes still need to stay alive until the supervisor cleans up.
        let mut sink = Vec::new();
        let _ = stdin.read_to_end(&mut sink);
        return Ok(());
    }

    let initialized = read_notification(&mut stdin, limits, "initialized")?;
    assert_eq!(initialized.method, "initialized");

    let shutdown = read_request(&mut stdin, limits, "shutdown")?;
    if mode == Mode::HangShutdown {
        // Acknowledge nothing and ignore exit so forced cleanup is required.
        let mut sink = Vec::new();
        let _ = stdin.read_to_end(&mut sink);
        // Park until killed.
        loop {
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
    }

    write_response(
        &mut stdout,
        ResponseMessage {
            jsonrpc: JsonRpcVersion::V2,
            id: Some(shutdown.id),
            result: Some(serde_json::Value::Null),
            error: None,
        },
        false,
    )?;

    let exit = read_notification(&mut stdin, limits, "exit")?;
    assert_eq!(exit.method, "exit");
    Ok(())
}

fn read_request(
    stdin: &mut impl Read,
    limits: FrameLimits,
    expected_method: &str,
) -> io::Result<RequestMessage> {
    let body = decode_frame(stdin, limits).map_err(to_io)?;
    match parse_message(&body).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))? {
        JsonRpcMessage::Request(request) if request.method == expected_method => Ok(request),
        other => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("expected {expected_method} request, got {other:?}"),
        )),
    }
}

fn read_notification(
    stdin: &mut impl Read,
    limits: FrameLimits,
    expected_method: &str,
) -> io::Result<NotificationMessage> {
    let body = decode_frame(stdin, limits).map_err(to_io)?;
    match parse_message(&body).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))? {
        JsonRpcMessage::Notification(notification) if notification.method == expected_method => {
            Ok(notification)
        }
        other => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("expected {expected_method} notification, got {other:?}"),
        )),
    }
}

fn write_response(
    stdout: &mut impl Write,
    response: ResponseMessage,
    fragmented: bool,
) -> io::Result<()> {
    let body = serde_json::to_vec(&JsonRpcMessage::Response(response))
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    write_raw_bytes(stdout, &body, fragmented)
}

fn write_raw(
    stdout: &mut impl Write,
    value: &serde_json::Value,
    fragmented: bool,
) -> io::Result<()> {
    let body = serde_json::to_vec(value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    write_raw_bytes(stdout, &body, fragmented)
}

fn write_raw_bytes(stdout: &mut impl Write, body: &[u8], fragmented: bool) -> io::Result<()> {
    let frame = encode_frame(body);
    if fragmented {
        for chunk in frame.chunks(5) {
            stdout.write_all(chunk)?;
            stdout.flush()?;
        }
    } else {
        stdout.write_all(&frame)?;
        stdout.flush()?;
    }
    Ok(())
}

fn id_to_json(id: &JsonRpcId) -> serde_json::Value {
    match id {
        JsonRpcId::Number(value) => json!(*value),
        JsonRpcId::String(value) => json!(value),
    }
}

fn to_io(error: mango_lsp::FrameError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}
