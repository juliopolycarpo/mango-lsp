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
| Progress | S002 is complete in [PR #3](https://github.com/juliopolycarpo/mango-lsp/pull/3). Local mandatory gates passed under Rust 1.97.0, and [Actions run 29193519305](https://github.com/juliopolycarpo/mango-lsp/actions/runs/29193519305) passed all quality and three-OS lifecycle gates. |
| Working branch or worktree | `main` after PR #3 squash-merge; no implementation branch is authorized. |
| Last coherent checkpoint | S002 implementation and cross-platform CI evidence are complete; Q-005/Q-006 are resolved as D-012/D-013. |
| Remaining work | Draft, review, and explicitly authorize a complete S003 contract before any configuration-backed vertical work. |
| Validation evidence | [Actions run 29193519305](https://github.com/juliopolycarpo/mango-lsp/actions/runs/29193519305): quality job passed with a 10-minute timeout; test jobs passed with 20-minute timeouts; 11 focused `downstream_lifecycle_*` tests passed on Ubuntu, macOS, and Windows, including stderr backpressure and both forced-cleanup cases. S002 guarantees direct-child cleanup only; descendant process-tree and real-server behavior are unverified. |
| Blockers | None. Q-011 remains required only before public distribution. |

S002's completion dependency for S003 is satisfied by the squash-merge of
[PR #3](https://github.com/juliopolycarpo/mango-lsp/pull/3). S003 is
dependency-unblocked but still requires a complete contract, review, and explicit
authorization before implementation.

## Near-term stage sequence

Only immediate outcomes are listed. Later decomposition must be revised after
the first vertical flow rather than expanded into a speculative backlog.

| ID | Outcome and observable gate | Depends on | State |
| --- | --- | --- | --- |
| P000 | Establish product, decision, state, stage, and handoff sources of truth. Gate: a clean session can identify authorized work and its objective checks without chat history. | None | Complete in the initial repository structure |
| S001 | Establish a reproducible Rust binary whose help, version, invalid-input behavior, tests, lint, formatting, and cross-platform CI are observable. | P000 | Complete in [PR #1](https://github.com/juliopolycarpo/mango-lsp/pull/1), squash commit `40c8d0f` |
| S002 | Prove a bounded downstream STDIO/LSP lifecycle with a deterministic fake: spawn, frame one interaction, correlate it, drain diagnostics, and shut down without orphaning the child. Exact API and crate boundaries remain open. | S001 | Complete in [PR #3](https://github.com/juliopolycarpo/mango-lsp/pull/3). Contract: [`docs/stages/S002-downstream-stdio-lifecycle.md`](stages/S002-downstream-stdio-lifecycle.md) |
| S003 | Complete the first configuration-backed vertical flow: CLI startup, minimal declarative configuration, supervised server launch, one useful LSP interaction, structured result and logs, and controlled shutdown. | S002 and decisions needed from Q-004, Q-007, and Q-009 | S002 dependency satisfied; dependency-unblocked, planned outline; **not authorized** |
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
| Stage | None — planning checkpoint; S002 complete |
| Owner/session | No active implementation stage |
| Branch | None |
| Last completed unit | S002 framing, protocol, lifecycle session, fake server, process-level `downstream_lifecycle_*` tests, D-012/D-013, and three-OS CI evidence |
| Next action | Plan and review a complete S003 contract. Do not implement S003 until it is explicitly authorized. |
| Changed paths | `src/lib.rs`, `src/frame.rs`, `src/protocol.rs`, `src/diagnostics.rs`, `src/lifecycle.rs`, `src/bin/mango_lsp_fake_server.rs`, `tests/downstream_lifecycle.rs`, `Cargo.toml`, `Cargo.lock`, `.github/workflows/ci.yml`, `README.md`, `docs/PROJECT.md`, `docs/STATUS.md` |
| Checks run | Local: `cargo test --all-targets --locked downstream_lifecycle -- --nocapture` → 11 passed; unit frame/protocol tests → 9 passed; full `cargo test --all-targets --locked` → 23 passed; fmt/check/clippy/offline and CLI gates passed. CI: [run 29193519305](https://github.com/juliopolycarpo/mango-lsp/actions/runs/29193519305) passed format/lint and focused plus full tests; focused `downstream_lifecycle_*` tests → Ubuntu 11 passed, macOS 11 passed, Windows 11 passed. Workflow bounds: 10 minutes for quality, 20 minutes for each test job. |
| Failed or unavailable checks | None. |
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
| S002 | Proved bounded direct-child STDIO LSP lifecycle: project-owned framing, minimal JSON-RPC types via serde_json, std process/thread supervision, concurrent stderr drain with truncation, forced cleanup/reap, and hostile fake-server acceptance tests. | Local: 11 `downstream_lifecycle_*` tests, 9 codec/protocol unit tests, 3 CLI tests (23 total); fmt/check/clippy/offline gates passed on Linux with Rust 1.97.0. CI: [Actions run 29193519305](https://github.com/juliopolycarpo/mango-lsp/actions/runs/29193519305) passed the 10-minute format/lint job and 20-minute test jobs; focused lifecycle suite passed 11/11 on Ubuntu, 11/11 on macOS, and 11/11 on Windows, including stderr backpressure and forced cleanup on stalled initialize and hung shutdown. Decisions D-012 and D-013. | Shipped a separate `mango-lsp-fake-server` test binary (not a product CLI subcommand). Packaging must exclude it before release (O-001). |

## Discovery and opportunity backlog

This is the source of truth for worthwhile findings outside the authorized stage.
An entry records an opportunity; it does not authorize implementation. Use IDs
`O-001`, `O-002`, and so on, and include the stage or review that discovered it.

| ID | Discovery or opportunity | Value and rationale | Source | Evaluate at | State |
| --- | --- | --- | --- | --- | --- |
| O-001 | Exclude `mango-lsp-fake-server` from release packaging and `cargo install` artifacts. | Keeps the fake out of distributed product surfaces while preserving the normal `cargo test --all-targets` build. | S002 | Release packaging / Q-011 | Open |
| O-002 | Consider feature-gating the fake binary once packaging exists. | Stronger guarantee than documentation alone that the fixture never ships. | S002 | Release packaging | Open |

## Current deviations and blockers

None. S002 local and three-OS CI evidence is complete. Q-011 remains
deliberately open until public distribution.

## State transition checklist

At the end of an implementation stage:

1. Record the delivered outcome and exact validation evidence in this file.
2. Move durable decisions and cross-stage risks to `docs/PROJECT.md`.
3. Add deferred findings to the discovery backlog.
4. Explain accepted deviations from the stage contract.
5. Set the authorized stage to `None — planning checkpoint` unless the maintainer
   has explicitly authorized a complete next-stage contract.
6. Ensure the PR central promise matches the stage outcome, then stop.
