//! Cross-platform file URI construction for workspace paths.

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use std::path::{Component, Path};

/// Encode path characters that are not safe unencoded in a file URI path.
///
/// Leaves unreserved characters and common path separators alone; encodes
/// spaces, non-ASCII UTF-8 sequences, and other reserved characters.
const FILE_URI_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

/// Convert an absolute filesystem path into a `file:` URI.
///
/// Unix paths become `file:///…`. Windows drive paths become `file:///C:/…`.
pub fn path_to_file_uri(path: &Path) -> Result<String, String> {
    if !path.is_absolute() {
        return Err(format!(
            "workspace path must be absolute before URI encoding: {}",
            path.display()
        ));
    }

    #[cfg(windows)]
    {
        windows_path_to_file_uri(path)
    }
    #[cfg(not(windows))]
    {
        unix_path_to_file_uri(path)
    }
}

#[cfg(not(windows))]
fn unix_path_to_file_uri(path: &Path) -> Result<String, String> {
    let mut uri = String::from("file://");
    for component in path.components() {
        match component {
            Component::RootDir => uri.push('/'),
            Component::Normal(part) => {
                if !uri.ends_with('/') {
                    uri.push('/');
                }
                let owned = part.to_string_lossy().into_owned();
                let encoded = utf8_percent_encode(&owned, FILE_URI_ENCODE_SET);
                uri.push_str(&encoded.to_string());
            }
            Component::CurDir | Component::ParentDir | Component::Prefix(_) => {
                return Err(format!(
                    "unsupported path component while encoding URI: {}",
                    path.display()
                ));
            }
        }
    }
    if uri == "file://" {
        uri.push('/');
    }
    Ok(uri)
}

#[cfg(windows)]
fn windows_path_to_file_uri(path: &Path) -> Result<String, String> {
    use std::path::Prefix;

    let mut components = path.components();
    let Some(Component::Prefix(prefix)) = components.next() else {
        return Err(format!(
            "Windows workspace path missing drive or UNC prefix: {}",
            path.display()
        ));
    };

    let mut uri = String::from("file://");
    match prefix.kind() {
        Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => {
            uri.push('/');
            uri.push(letter.to_ascii_uppercase() as char);
            uri.push(':');
        }
        Prefix::UNC(server, share) | Prefix::VerbatimUNC(server, share) => {
            uri.push('/');
            uri.push('/');
            uri.push_str(
                &utf8_percent_encode(&server.to_string_lossy(), FILE_URI_ENCODE_SET).to_string(),
            );
            uri.push('/');
            uri.push_str(
                &utf8_percent_encode(&share.to_string_lossy(), FILE_URI_ENCODE_SET).to_string(),
            );
        }
        other => {
            return Err(format!(
                "unsupported Windows path prefix {other:?} for {}",
                path.display()
            ));
        }
    }

    for component in components {
        match component {
            Component::RootDir => {}
            Component::Normal(part) => {
                uri.push('/');
                uri.push_str(
                    &utf8_percent_encode(&part.to_string_lossy(), FILE_URI_ENCODE_SET).to_string(),
                );
            }
            Component::CurDir | Component::ParentDir | Component::Prefix(_) => {
                return Err(format!(
                    "unsupported path component while encoding URI: {}",
                    path.display()
                ));
            }
        }
    }
    Ok(uri)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn unix_style_path_encodes_spaces() {
        let path = PathBuf::from("/tmp/my workspace/src");
        let uri = path_to_file_uri(&path).unwrap();
        assert_eq!(uri, "file:///tmp/my%20workspace/src");
    }

    #[test]
    fn unix_style_path_encodes_non_ascii() {
        let path = PathBuf::from("/tmp/café");
        let uri = path_to_file_uri(&path).unwrap();
        assert_eq!(uri, "file:///tmp/caf%C3%A9");
    }
}
