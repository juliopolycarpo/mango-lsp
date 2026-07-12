//! Explicit one-server TOML configuration loading and validation.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::lifecycle::ChildCommand;

/// Finite bounds for configuration parsing and command construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigLimits {
    /// Maximum configuration file size in bytes.
    pub max_file_bytes: usize,
    /// Maximum length of `server.id`.
    pub max_server_id_bytes: usize,
    /// Maximum length of `server.command`.
    pub max_command_bytes: usize,
    /// Maximum number of literal arguments.
    pub max_args: usize,
    /// Maximum length of one argument token.
    pub max_arg_bytes: usize,
}

impl Default for ConfigLimits {
    fn default() -> Self {
        Self {
            max_file_bytes: 64 * 1024,
            max_server_id_bytes: 64,
            max_command_bytes: 4 * 1024,
            max_args: 64,
            max_arg_bytes: 4 * 1024,
        }
    }
}

/// Validated server definition ready for direct spawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    pub id: String,
    pub command: ChildCommand,
}

/// Failures while loading or validating configuration.
#[derive(Debug)]
pub enum ConfigError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Oversized {
        path: PathBuf,
        size: usize,
        limit: usize,
    },
    Parse {
        path: PathBuf,
        message: String,
    },
    Invalid {
        message: String,
    },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(
                    f,
                    "failed to read configuration {}: {source}",
                    path.display()
                )
            }
            Self::Oversized { path, size, limit } => write!(
                f,
                "configuration {} is {size} bytes, exceeding limit of {limit}",
                path.display()
            ),
            Self::Parse { path, message } => {
                write!(f, "configuration {} is invalid: {message}", path.display())
            }
            Self::Invalid { message } => write!(f, "invalid configuration: {message}"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    schema_version: u32,
    server: RawServer,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawServer {
    id: String,
    command: String,
    #[serde(default)]
    args: Vec<String>,
}

/// Load and strictly validate an explicitly selected configuration file.
pub fn load_server_config(
    config_path: &Path,
    limits: ConfigLimits,
) -> Result<ServerConfig, ConfigError> {
    let absolute = if config_path.is_absolute() {
        config_path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|source| ConfigError::Io {
                path: config_path.to_path_buf(),
                source,
            })?
            .join(config_path)
    };

    let metadata = fs::metadata(&absolute).map_err(|source| ConfigError::Io {
        path: absolute.clone(),
        source,
    })?;
    let size = usize::try_from(metadata.len()).unwrap_or(usize::MAX);
    if size > limits.max_file_bytes {
        return Err(ConfigError::Oversized {
            path: absolute,
            size,
            limit: limits.max_file_bytes,
        });
    }

    let text = fs::read_to_string(&absolute).map_err(|source| ConfigError::Io {
        path: absolute.clone(),
        source,
    })?;
    if text.len() > limits.max_file_bytes {
        return Err(ConfigError::Oversized {
            path: absolute,
            size: text.len(),
            limit: limits.max_file_bytes,
        });
    }

    let raw: RawConfig = toml::from_str(&text).map_err(|error| ConfigError::Parse {
        path: absolute.clone(),
        message: error.to_string(),
    })?;

    if raw.schema_version != 1 {
        return Err(ConfigError::Invalid {
            message: format!(
                "unsupported schema_version {}; only version 1 is accepted",
                raw.schema_version
            ),
        });
    }

    validate_server_id(&raw.server.id, limits.max_server_id_bytes)?;
    if raw.server.command.is_empty() {
        return Err(ConfigError::Invalid {
            message: "server.command must not be empty".to_owned(),
        });
    }
    if raw.server.command.len() > limits.max_command_bytes {
        return Err(ConfigError::Invalid {
            message: format!(
                "server.command exceeds limit of {} bytes",
                limits.max_command_bytes
            ),
        });
    }
    if raw.server.args.len() > limits.max_args {
        return Err(ConfigError::Invalid {
            message: format!(
                "server.args has {} entries, exceeding limit of {}",
                raw.server.args.len(),
                limits.max_args
            ),
        });
    }
    for (index, arg) in raw.server.args.iter().enumerate() {
        if arg.len() > limits.max_arg_bytes {
            return Err(ConfigError::Invalid {
                message: format!(
                    "server.args[{index}] exceeds limit of {} bytes",
                    limits.max_arg_bytes
                ),
            });
        }
    }

    let program = resolve_command(&raw.server.command, &absolute)?;
    Ok(ServerConfig {
        id: raw.server.id,
        command: ChildCommand::new(program).args(raw.server.args),
    })
}

fn validate_server_id(id: &str, max_bytes: usize) -> Result<(), ConfigError> {
    if id.is_empty() {
        return Err(ConfigError::Invalid {
            message: "server.id must not be empty".to_owned(),
        });
    }
    if id.len() > max_bytes {
        return Err(ConfigError::Invalid {
            message: format!("server.id exceeds limit of {max_bytes} bytes"),
        });
    }
    if !id
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
    {
        return Err(ConfigError::Invalid {
            message: "server.id may contain only ASCII lowercase letters, digits, '-' and '_'"
                .to_owned(),
        });
    }
    Ok(())
}

fn resolve_command(command: &str, config_path: &Path) -> Result<PathBuf, ConfigError> {
    let path = Path::new(command);
    let has_separator = command.contains('/') || command.contains('\\');
    if has_separator && path.is_relative() {
        let config_dir = config_path.parent().ok_or_else(|| ConfigError::Invalid {
            message: "configuration path has no parent directory for relative command resolution"
                .to_owned(),
        })?;
        Ok(config_dir.join(path))
    } else {
        Ok(path.to_path_buf())
    }
}

/// Maximum accepted `--query` length in bytes.
pub const MAX_QUERY_BYTES: usize = 4 * 1024;

/// Validate the workspace-symbols query before any child launch.
pub fn validate_query(query: &str) -> Result<(), ConfigError> {
    if query.is_empty() {
        return Err(ConfigError::Invalid {
            message: "query must be non-empty".to_owned(),
        });
    }
    if query.len() > MAX_QUERY_BYTES {
        return Err(ConfigError::Invalid {
            message: format!("query exceeds limit of {MAX_QUERY_BYTES} bytes"),
        });
    }
    Ok(())
}

/// Resolve and validate an existing workspace directory.
pub fn resolve_workspace(workspace: &Path) -> Result<(PathBuf, String), ConfigError> {
    let absolute = if workspace.is_absolute() {
        workspace.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|source| ConfigError::Io {
                path: workspace.to_path_buf(),
                source,
            })?
            .join(workspace)
    };

    let metadata = fs::metadata(&absolute).map_err(|source| ConfigError::Io {
        path: absolute.clone(),
        source,
    })?;
    if !metadata.is_dir() {
        return Err(ConfigError::Invalid {
            message: format!("workspace is not a directory: {}", absolute.display()),
        });
    }

    let canonical = fs::canonicalize(&absolute).map_err(|source| ConfigError::Io {
        path: absolute.clone(),
        source,
    })?;
    let uri = crate::uri::path_to_file_uri(&canonical)
        .map_err(|message| ConfigError::Invalid { message })?;
    Ok((canonical, uri))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir(PathBuf);
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn loads_valid_config_and_defaults_args() {
        let root = std::env::temp_dir().join(format!("mango-lsp-cfg-ok-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let _guard = TempDir(root.clone());
        let path = root.join("config.toml");
        fs::write(
            &path,
            r#"
schema_version = 1
[server]
id = "fixture"
command = "/usr/bin/true"
"#,
        )
        .unwrap();

        let config = load_server_config(&path, ConfigLimits::default()).unwrap();
        assert_eq!(config.id, "fixture");
        assert_eq!(config.command.program, PathBuf::from("/usr/bin/true"));
        assert!(config.command.args.is_empty());
    }

    #[test]
    fn rejects_unknown_fields() {
        let root = std::env::temp_dir().join(format!("mango-lsp-cfg-unk-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let _guard = TempDir(root.clone());
        let path = root.join("config.toml");
        fs::write(
            &path,
            r#"
schema_version = 1
extra = 1
[server]
id = "fixture"
command = "/usr/bin/true"
"#,
        )
        .unwrap();
        let error = load_server_config(&path, ConfigLimits::default()).unwrap_err();
        assert!(error.to_string().contains("invalid") || error.to_string().contains("unknown"));
    }

    #[test]
    fn rejects_invalid_server_id() {
        let root = std::env::temp_dir().join(format!("mango-lsp-cfg-id-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let _guard = TempDir(root.clone());
        let path = root.join("config.toml");
        fs::write(
            &path,
            r#"
schema_version = 1
[server]
id = "Bad ID"
command = "/usr/bin/true"
"#,
        )
        .unwrap();
        let error = load_server_config(&path, ConfigLimits::default()).unwrap_err();
        assert!(error.to_string().contains("server.id"));
    }

    #[test]
    fn resolves_relative_command_against_config_dir() {
        let root = std::env::temp_dir().join(format!("mango-lsp-cfg-rel-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let _guard = TempDir(root.clone());
        let path = root.join("config.toml");
        fs::write(
            &path,
            r#"
schema_version = 1
[server]
id = "fixture"
command = "./bin/server"
args = ["--stdio"]
"#,
        )
        .unwrap();
        let config = load_server_config(&path, ConfigLimits::default()).unwrap();
        assert_eq!(config.command.program, root.join("bin/server"));
        assert_eq!(config.command.args, vec!["--stdio".to_owned()]);
    }
}
