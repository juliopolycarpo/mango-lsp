//! LSP STDIO framing: ASCII headers, CRLF terminator, byte-counted body.

use std::io::{self, Read, Write};

/// Configurable limits for untrusted frame decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameLimits {
    /// Maximum accepted header block size, including the terminating blank line.
    pub max_header_bytes: usize,
    /// Maximum accepted body size declared by `Content-Length`.
    pub max_body_bytes: usize,
}

impl Default for FrameLimits {
    fn default() -> Self {
        Self {
            max_header_bytes: 64 * 1024,
            max_body_bytes: 16 * 1024 * 1024,
        }
    }
}

/// Errors produced while encoding or decoding LSP frames.
#[derive(Debug)]
pub enum FrameError {
    Io(io::Error),
    MissingContentLength,
    InvalidContentLength(String),
    DuplicateContentLength,
    ConflictingContentLength { first: usize, second: usize },
    UnsupportedCharset(String),
    HeaderTooLarge { limit: usize },
    BodyTooLarge { declared: usize, limit: usize },
    UnexpectedEof { context: &'static str },
    InvalidHeaderLine(String),
    InvalidHeaderEncoding(std::str::Utf8Error),
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "frame I/O error: {error}"),
            Self::MissingContentLength => write!(f, "missing Content-Length header"),
            Self::InvalidContentLength(value) => {
                write!(f, "invalid Content-Length value: {value}")
            }
            Self::DuplicateContentLength => write!(f, "duplicate Content-Length header"),
            Self::ConflictingContentLength { first, second } => {
                write!(f, "conflicting Content-Length values: {first} and {second}")
            }
            Self::UnsupportedCharset(charset) => {
                write!(f, "unsupported Content-Type charset: {charset}")
            }
            Self::HeaderTooLarge { limit } => {
                write!(f, "LSP header exceeded limit of {limit} bytes")
            }
            Self::BodyTooLarge { declared, limit } => write!(
                f,
                "Content-Length {declared} exceeds body limit of {limit} bytes"
            ),
            Self::UnexpectedEof { context } => {
                write!(f, "unexpected EOF while reading {context}")
            }
            Self::InvalidHeaderLine(line) => write!(f, "invalid LSP header line: {line}"),
            Self::InvalidHeaderEncoding(error) => {
                write!(f, "LSP header was not valid UTF-8: {error}")
            }
        }
    }
}

impl std::error::Error for FrameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::InvalidHeaderEncoding(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for FrameError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

/// Encode a body into an LSP STDIO frame.
///
/// `Content-Length` is the UTF-8 byte length of `body`.
pub fn encode_frame(body: &[u8]) -> Vec<u8> {
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    let mut frame = Vec::with_capacity(header.len() + body.len());
    frame.extend_from_slice(header.as_bytes());
    frame.extend_from_slice(body);
    frame
}

/// Write one framed message to `writer`.
pub fn write_frame<W: Write>(writer: &mut W, body: &[u8]) -> Result<(), FrameError> {
    writer.write_all(&encode_frame(body))?;
    writer.flush()?;
    Ok(())
}

/// Read one framed message from `reader`, enforcing `limits` before retaining the body.
pub fn decode_frame<R: Read>(reader: &mut R, limits: FrameLimits) -> Result<Vec<u8>, FrameError> {
    let header_bytes = read_headers(reader, limits.max_header_bytes)?;
    let content_length = parse_headers(&header_bytes)?;
    if content_length > limits.max_body_bytes {
        return Err(FrameError::BodyTooLarge {
            declared: content_length,
            limit: limits.max_body_bytes,
        });
    }

    let mut body = vec![0_u8; content_length];
    reader
        .read_exact(&mut body)
        .map_err(|error| map_read_exact(error, "body"))?;
    Ok(body)
}

fn map_read_exact(error: io::Error, context: &'static str) -> FrameError {
    if error.kind() == io::ErrorKind::UnexpectedEof {
        FrameError::UnexpectedEof { context }
    } else {
        FrameError::Io(error)
    }
}

fn read_headers<R: Read>(reader: &mut R, max_header_bytes: usize) -> Result<Vec<u8>, FrameError> {
    let mut header = Vec::new();
    let mut window = [0_u8; 4];
    let mut filled = 0_usize;

    loop {
        if header.len() >= max_header_bytes {
            return Err(FrameError::HeaderTooLarge {
                limit: max_header_bytes,
            });
        }

        let mut byte = [0_u8; 1];
        match reader.read(&mut byte)? {
            0 => {
                return Err(FrameError::UnexpectedEof { context: "headers" });
            }
            _ => {
                header.push(byte[0]);
                if filled < 4 {
                    window[filled] = byte[0];
                    filled += 1;
                } else {
                    window.copy_within(1..4, 0);
                    window[3] = byte[0];
                }

                if filled == 4 && window == *b"\r\n\r\n" {
                    return Ok(header);
                }
            }
        }
    }
}

fn parse_headers(header_bytes: &[u8]) -> Result<usize, FrameError> {
    let header_text =
        std::str::from_utf8(header_bytes).map_err(FrameError::InvalidHeaderEncoding)?;
    let mut content_length: Option<usize> = None;

    for raw_line in header_text.split("\r\n") {
        if raw_line.is_empty() {
            continue;
        }

        let Some((name, value)) = raw_line.split_once(':') else {
            return Err(FrameError::InvalidHeaderLine(raw_line.to_owned()));
        };

        let name = name.trim();
        let value = value.trim();
        if name.eq_ignore_ascii_case("Content-Length") {
            let parsed = value
                .parse::<usize>()
                .map_err(|_| FrameError::InvalidContentLength(value.to_owned()))?;
            match content_length {
                None => content_length = Some(parsed),
                Some(existing) if existing == parsed => {
                    return Err(FrameError::DuplicateContentLength);
                }
                Some(existing) => {
                    return Err(FrameError::ConflictingContentLength {
                        first: existing,
                        second: parsed,
                    });
                }
            }
        } else if name.eq_ignore_ascii_case("Content-Type") {
            validate_content_type(value)?;
        }
        // Unrecognized header fields are ignored within this stage's subset.
    }

    content_length.ok_or(FrameError::MissingContentLength)
}

fn validate_content_type(value: &str) -> Result<(), FrameError> {
    // Absent Content-Type means UTF-8. When present, accept utf-8 and legacy utf8.
    let mut charset = None;
    for part in value.split(';') {
        let part = part.trim();
        let Some((key, charset_value)) = part.split_once('=') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case("charset") {
            charset = Some(charset_value.trim().trim_matches('"'));
        }
    }

    match charset {
        None => Ok(()),
        Some(charset)
            if charset.eq_ignore_ascii_case("utf-8") || charset.eq_ignore_ascii_case("utf8") =>
        {
            Ok(())
        }
        Some(charset) => Err(FrameError::UnsupportedCharset(charset.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn encode_frame_uses_utf8_byte_length() {
        let body = "café".as_bytes();
        assert_eq!(body.len(), 5);
        let frame = encode_frame(body);
        assert!(frame.starts_with(b"Content-Length: 5\r\n\r\n"));
        assert_eq!(&frame[frame.len() - body.len()..], body);
    }

    #[test]
    fn decode_frame_handles_fragmented_reads() {
        let body = br#"{"jsonrpc":"2.0","id":1,"result":null}"#;
        let frame = encode_frame(body);
        let mut fragmented = FragmentedReader::new(frame, 3);
        let decoded = decode_frame(&mut fragmented, FrameLimits::default()).unwrap();
        assert_eq!(decoded, body);
    }

    #[test]
    fn decode_frame_rejects_body_over_limit_before_allocation_of_declared_size() {
        let frame = b"Content-Length: 1048576\r\n\r\n";
        let mut reader = Cursor::new(frame.as_slice());
        let error = decode_frame(
            &mut reader,
            FrameLimits {
                max_header_bytes: 1024,
                max_body_bytes: 64,
            },
        )
        .unwrap_err();
        match error {
            FrameError::BodyTooLarge {
                declared: 1_048_576,
                limit: 64,
            } => {}
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn decode_frame_accepts_legacy_utf8_charset() {
        let body = b"{}";
        let mut frame = Vec::new();
        frame.extend_from_slice(b"Content-Length: 2\r\n");
        frame.extend_from_slice(b"Content-Type: application/vscode-jsonrpc; charset=utf8\r\n\r\n");
        frame.extend_from_slice(body);
        let decoded = decode_frame(&mut Cursor::new(frame), FrameLimits::default()).unwrap();
        assert_eq!(decoded, body);
    }

    #[test]
    fn decode_frame_rejects_non_utf8_charset() {
        let frame = b"Content-Length: 2\r\nContent-Type: application/vscode-jsonrpc; charset=ascii\r\n\r\n{}";
        let error =
            decode_frame(&mut Cursor::new(frame.as_slice()), FrameLimits::default()).unwrap_err();
        assert!(matches!(error, FrameError::UnsupportedCharset(_)));
    }

    #[test]
    fn decode_frame_rejects_missing_content_length() {
        let frame = b"Content-Type: application/vscode-jsonrpc; charset=utf-8\r\n\r\n{}";
        let error =
            decode_frame(&mut Cursor::new(frame.as_slice()), FrameLimits::default()).unwrap_err();
        assert!(matches!(error, FrameError::MissingContentLength));
    }

    struct FragmentedReader {
        data: Vec<u8>,
        offset: usize,
        chunk: usize,
    }

    impl FragmentedReader {
        fn new(data: Vec<u8>, chunk: usize) -> Self {
            Self {
                data,
                offset: 0,
                chunk,
            }
        }
    }

    impl Read for FragmentedReader {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.offset >= self.data.len() {
                return Ok(0);
            }
            let end = (self.offset + self.chunk)
                .min(self.data.len())
                .min(self.offset + buf.len());
            let amount = end - self.offset;
            buf[..amount].copy_from_slice(&self.data[self.offset..end]);
            self.offset = end;
            Ok(amount)
        }
    }
}
