# mango-lsp

`mango-lsp` is a planned CLI-first supervisor and agent-oriented gateway for
multiple Language Server Protocol (LSP) servers. It will own server processes,
route and aggregate useful operations, and expose a smaller, versioned interface
than unrestricted LSP access. MangoStudio will be its primary graphical
consumer, but the project and its release cycle are independent.

The repository produces a `mango-lsp` executable with a configuration-backed
`workspace-symbols` operation, plus an internal library that owns one
direct-child LSP session over STDIO. A deterministic `mango-lsp-fake-server`
binary exists only as test infrastructure and is not a product command.

## Contributor quick start

Install [rustup](https://rustup.rs/) and clone the repository. The checked-in
toolchain file selects Rust 1.97.0 and installs the required formatter and
linter components automatically.

```console
cargo build --locked
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked vertical_flow -- --nocapture
cargo test --all-targets --locked
cargo run --locked -- --help
cargo run --locked -- --version
cargo run --locked -- workspace-symbols --help
```

Builds and tests do not require a language server, credentials, or network
access after Cargo has fetched the locked dependencies. The focused
`vertical_flow` filter exercises the S003 fake-server proof, including
redaction, unsupported capability, timeout, and cleanup cases.

### Example configuration

```toml
schema_version = 1

[server]
id = "fixture"
command = "/absolute/path/to/language-server"
args = ["--stdio"]
```

```console
cargo run --locked -- workspace-symbols \
  --config ./path/to/config.toml \
  --workspace ./path/to/workspace \
  --query Widget
```

A successful invocation writes one version 1 JSON result object to stdout and
redacted JSON Lines lifecycle events to stderr. Configuration selection is
explicit only: mango-lsp does not search for default files or walk parents.

## Start here

A new implementation session should read, in order:

1. [`AGENTS.md`](AGENTS.md) for repository working rules.
2. [`docs/STATUS.md`](docs/STATUS.md) for the currently authorized stage, if any,
   and its live progress.
3. The stage contract linked from `docs/STATUS.md`.
4. Only the sections of [`docs/PROJECT.md`](docs/PROJECT.md) referenced by the
   stage contract, plus any nearby decisions needed for the work.

If `docs/STATUS.md` does not name an authorized stage, implementation must stop
until a stage is specified and authorized. A roadmap entry is not authorization.

## Sources of truth

| Information | Authoritative file |
| --- | --- |
| Product definition, boundaries, durable decisions, open questions, risks | [`docs/PROJECT.md`](docs/PROJECT.md) |
| Current state, active-stage progress, stage sequence, discovery backlog | [`docs/STATUS.md`](docs/STATUS.md) |
| Scope, deliverables, gates, and stopping rule for authorized work | The stage contract linked from `docs/STATUS.md` |
| Session, validation, commit, PR, and handoff rules | [`AGENTS.md`](AGENTS.md) |
| Required shape of a future stage | [`docs/stages/TEMPLATE.md`](docs/stages/TEMPLATE.md) |

These responsibilities are intentionally separate. Do not copy live status into
stage contracts or duplicate accepted decisions in multiple files.

## Current direction

The initial delivery path established a portable Rust CLI baseline, proved a
deterministic downstream STDIO/LSP lifecycle with a test double, and delivered
the first configuration-backed vertical flow. Later stages must be revised from
that evidence rather than detailed in advance.

All repository content is written in English unless a documented interoperability
or localization case requires otherwise.
