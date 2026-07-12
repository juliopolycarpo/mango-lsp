use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Arg, Command};
use mango_lsp::config::ConfigLimits;
use mango_lsp::lifecycle::DownstreamLimits;
use mango_lsp::operation::{WorkspaceSymbolsRequest, run_workspace_symbols};

fn main() -> ExitCode {
    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            Command::new("workspace-symbols")
                .about("Query workspace symbols through one explicitly configured language server")
                .arg(
                    Arg::new("config")
                        .long("config")
                        .value_name("FILE")
                        .required(true)
                        .help("Explicit TOML configuration file describing one server"),
                )
                .arg(
                    Arg::new("workspace")
                        .long("workspace")
                        .value_name("DIR")
                        .required(true)
                        .help("Existing workspace directory used as the child working directory"),
                )
                .arg(
                    Arg::new("query")
                        .long("query")
                        .value_name("TEXT")
                        .required(true)
                        .help("Non-empty UTF-8 symbol query sent to workspace/symbol"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("workspace-symbols", sub)) => {
            let request = WorkspaceSymbolsRequest {
                config: PathBuf::from(sub.get_one::<String>("config").expect("required --config")),
                workspace: PathBuf::from(
                    sub.get_one::<String>("workspace")
                        .expect("required --workspace"),
                ),
                query: sub
                    .get_one::<String>("query")
                    .expect("required --query")
                    .clone(),
            };
            let mut stdout = std::io::stdout().lock();
            let mut stderr = std::io::stderr().lock();
            run_workspace_symbols(
                request,
                DownstreamLimits::default(),
                ConfigLimits::default(),
                &mut stdout,
                &mut stderr,
            )
        }
        // Clap rejects unknown subcommands itself; only "no subcommand" remains.
        _ => ExitCode::SUCCESS,
    }
}
