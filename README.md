# mango-lsp

`mango-lsp` is a planned CLI-first supervisor and agent-oriented gateway for
multiple Language Server Protocol (LSP) servers. It will own server processes,
route and aggregate useful operations, and expose a smaller, versioned interface
than unrestricted LSP access. MangoStudio will be its primary graphical
consumer, but the project and its release cycle are independent.

There is no Rust implementation yet. The repository currently contains the
planning and continuity structure that governs the first implementation stage.

## Start here

A new implementation session should read, in order:

1. [`AGENTS.md`](AGENTS.md) for repository working rules.
2. [`docs/STATUS.md`](docs/STATUS.md) for the only currently authorized stage
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

The initial delivery path is deliberately short: establish a portable Rust CLI
baseline, prove a deterministic downstream STDIO/LSP lifecycle with a test
double, and then expose a small configuration-backed vertical flow. The exact
shape of later stages must be revised from evidence rather than detailed in
advance.

All repository content is written in English unless a documented interoperability
or localization case requires otherwise.
