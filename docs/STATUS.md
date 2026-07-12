# Project status

Last updated: 2026-07-12

This file is the only source of truth for project execution state, stage
authorization, active-stage progress, the near-term sequence, and discovered
opportunities. A stage mentioned in the roadmap is not authorized unless the
snapshot below links its complete contract.

## Current snapshot

| Field | Value |
| --- | --- |
| Phase | Foundation |
| Implementation state | S002 delivered a bounded direct-child STDIO/LSP lifecycle with framing, correlation, stderr draining, forced cleanup, and a deterministic fake server. |
| Authorized stage | **None — planning checkpoint** |
| Contract | Completed S002 remains at [`docs/stages/S002-downstream-stdio-lifecycle.md`](stages/S002-downstream-stdio-lifecycle.md). S003 is not authorized. |
| Progress | S002 is implemented on `feat/s002-downstream-stdio-lifecycle`. Local mandatory gates passed under Rust 1.97.0. CI evidence is recorded when the implementation PR completes three-OS validation. |
| Working branch or worktree | `feat/s002-downstream-stdio-lifecycle` |
| Last coherent checkpoint | Local focused and full test suites pass; Q-005/Q-006 resolved as D-012/D-013. |
| Remaining work | Maintainer review and squash-merge of the S002 PR; then draft/review/authorize a complete S003 contract before any configuration-backed vertical work. |
| Validation evidence | See active-stage working record and stage history. S002 guarantees direct-child cleanup only; descendant process-tree and real-server behavior are unverified. |
| Blockers | None for S002 local proof. Public CI run URL and per-OS lifecycle counts are filled during PR babysitting. Q-011 remains required only before public distribution. |

S002's dependency for S003 is satisfied once this stage is squash-merged. S003
still requires a complete contract, review, and explicit authorization before
implementation.

## Near-term stage sequence

Only immediate outcomes are listed. Later decomposition must be revised after
the first vertical flow rather than expanded into a speculative backlog.

| ID | Outcome and observable gate | Depends on | State |
| --- | --- | --- | --- |
| P000 | Establish product, decision, state, stage, and handoff sources of truth. Gate: a clean session can identify authorized work and its objective checks without chat history. | None | Complete in the initial repository structure |
| S001 | Establish a reproducible Rust binary whose help, version, invalid-input behavior, tests, lint, formatting, and cross-platform CI are observable. | P000 | Complete in [PR #1](https://github.com/juliopolycarpo/mango-lsp/pull/1), squash commit `40c8d0f` |
| S002 | Prove a bounded downstream STDIO/LSP lifecycle with a deterministic fake: spawn, frame one interaction, correlate it, drain diagnostics, and shut down without orphaning the child. Exact API and crate boundaries remain open. | S001 | Implemented; awaiting squash-merge. Contract: [`docs/stages/S002-downstream-stdio-lifecycle.md`](stages/S002-downstream-stdio-lifecycle.md) |
| S003 | Complete the first configuration-backed vertical flow: CLI startup, minimal declarative configuration, supervised server launch, one useful LSP interaction, structured result and logs, and controlled shutdown. | S002 and decisions needed from Q-004, Q-007, and Q-009 | Dependency unblocked after S002 merge; planned outline; **not authorized** |
| Checkpoint | Review vertical-flow evidence before specifying multi-server routing, public schema generation, resilience hardening, packaging, or releases. | S003 | Not authorized |

### Gate policy

- A stage is one PR-sized outcome that can be demonstrated independently.
- A stage is complete only when its behavioral acceptance criteria and mandatory
  validations have objective evidence and its state changes are included in the
  same PR.
- Completion does not authorize the next row. After S002, set the authorized
  stage to `None — planning checkpoint` unless a maintainer has reviewed and
  authorized a complete S003 contract.
- If evidence invalidates the sequence, update this table and explain the
  deviation; do not force work through a stale plan.

## Active-stage working record

| Field | Current record |
| --- | --- |
| Stage | S002 — implementation complete locally; authorized stage cleared to planning checkpoint |
| Owner/session | S002 implementation session on `feat/s002-downstream-stdio-lifecycle` |
| Branch | `feat/s002-downstream-stdio-lifecycle` |
| Last completed unit | Framing, protocol, lifecycle session, fake server, process-level `downstream_lifecycle_*` tests, D-012/D-013, CI timeouts and focused lifecycle step |
| Next action | Open/babysit the S002 PR through green Linux/macOS/Windows CI, then maintainer squash-merge. Do not start S003 until a complete contract is authorized. |
| Changed paths | `src/lib.rs`, `src/frame.rs`, `src/protocol.rs`, `src/diagnostics.rs`, `src/lifecycle.rs`, `src/bin/mango_lsp_fake_server.rs`, `tests/downstream_lifecycle.rs`, `Cargo.toml`, `Cargo.lock`, `.github/workflows/ci.yml`, `README.md`, `docs/PROJECT.md`, `docs/STATUS.md` |
| Checks run | `cargo test --all-targets --locked downstream_lifecycle -- --nocapture` → 11 passed; unit frame/protocol tests → 9 passed; full `cargo test --all-targets --locked` → 23 passed; `cargo fmt --check`, `cargo check --all-targets --locked`, `cargo clippy --all-targets --locked -- -D warnings` passed; `cargo build --locked --offline` and `cargo test --all-targets --locked --offline` passed; `cargo run --locked --bin mango-lsp -- --help`/`--version` ok; unknown option exits 2 with stderr diagnostic. |
| Failed or unavailable checks | Live three-OS CI counts pending PR run. |
| Open implementation decisions | Q-005 → D-012; Q-006 → D-013. Q-003 remains open until release planning. |
| Resume notes | Default bounds: header 64 KiB, body 16 MiB, stderr retention 64 KiB, operation timeout 5s, force-shutdown 2s (tests inject tighter limits). Fake binary `mango-lsp-fake-server` is test infrastructure only. Direct-child cleanup only. |

When a stage finishes, replace this record with its outcome and validation
evidence, move it into the history table, and leave the next active record empty
unless another complete contract is authorized.

## Stage history

| Stage | Outcome | Evidence | Material deviations |
| --- | --- | --- | --- |
| P000 | Established the minimal planning and continuity system and specified S001. | Repository documentation and its initial signed commit. | The bootstrap prompt was intentionally retired after its durable requirements were incorporated. |
| S001 | Established the root Rust 2024 application, deterministic bootstrap CLI behavior, real-binary integration tests, pinned toolchain/quality policy, and three-OS CI baseline. | All mandatory local commands passed with Rust 1.97.0; offline build/test passed; invalid option exited 2 with a stderr diagnostic; [Actions run 29190660631](https://github.com/juliopolycarpo/mango-lsp/actions/runs/29190660631) passed format/lint and Linux, macOS, and Windows check/build/test jobs, with 3 CLI tests executed on each OS. | None. An independent review evidence-timestamp finding was resolved by rerunning the complete final-tree validation suite. |
| S002 | Proved bounded direct-child STDIO LSP lifecycle: project-owned framing, minimal JSON-RPC types via serde_json, std process/thread supervision, concurrent stderr drain with truncation, forced cleanup/reap, and hostile fake-server acceptance tests. | Local: 11 `downstream_lifecycle_*` tests, 9 codec/protocol unit tests, 3 CLI tests (23 total); fmt/check/clippy/offline gates passed on Linux with Rust 1.97.0. Decisions D-012 and D-013. CI URL and per-OS counts recorded at merge. | Shipped a separate `mango-lsp-fake-server` test binary (not a product CLI subcommand). Packaging must exclude it before release (O-001). |

## Discovery and opportunity backlog

This is the source of truth for worthwhile findings outside the authorized stage.
An entry records an opportunity; it does not authorize implementation. Use IDs
`O-001`, `O-002`, and so on, and include the stage or review that discovered it.

| ID | Discovery or opportunity | Value and rationale | Source | Evaluate at | State |
| --- | --- | --- | --- | --- | --- |
| O-001 | Exclude `mango-lsp-fake-server` from release packaging and `cargo install` artifacts. | Keeps the fake out of distributed product surfaces while preserving the normal `cargo test --all-targets` build. | S002 | Release packaging / Q-011 | Open |
| O-002 | Consider feature-gating the fake binary once packaging exists. | Stronger guarantee than documentation alone that the fixture never ships. | S002 | Release packaging | Open |

## Current deviations and blockers

None for the local S002 proof. Three-OS CI evidence is pending the implementation
PR. Q-011 remains deliberately open until public distribution.

## State transition checklist

At the end of an implementation stage:

1. Record the delivered outcome and exact validation evidence in this file.
2. Move durable decisions and cross-stage risks to `docs/PROJECT.md`.
3. Add deferred findings to the discovery backlog.
4. Explain accepted deviations from the stage contract.
5. Set the authorized stage to `None — planning checkpoint` unless the maintainer
   has explicitly authorized a complete next-stage contract.
6. Ensure the PR central promise matches the stage outcome, then stop.
