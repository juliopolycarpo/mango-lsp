# Project status

Last updated: 2026-07-12

This file is the only source of truth for project execution state, stage
authorization, active-stage progress, the near-term sequence, and discovered
opportunities. A stage mentioned in the roadmap is not authorized unless the
snapshot below links its complete contract.

## Current snapshot

| Field | Value |
| --- | --- |
| Phase | Post-vertical planning checkpoint |
| Implementation state | S003 delivered the first configuration-backed `workspace-symbols` vertical flow on a dedicated PR branch. Local and three-OS CI gates are green; the PR remains open for maintainer review (do not merge until approved). |
| Authorized stage | **None — planning checkpoint** |
| Contract | Completed S003: [`docs/stages/S003-configuration-backed-workspace-symbols.md`](stages/S003-configuration-backed-workspace-symbols.md). No next stage is authorized. |
| Progress | S003 implementation is complete in [PR #6](https://github.com/juliopolycarpo/mango-lsp/pull/6) on branch `feat/s003-workspace-symbols-vertical-flow` and awaits maintainer review (do not merge until approved). Q-009 resolved as D-016; clap `string` feature and TOML/URI deps recorded as D-017/D-018. |
| Working branch or worktree | `feat/s003-workspace-symbols-vertical-flow` (implementation PR; merge only after explicit maintainer approval) |
| Last coherent checkpoint | Local and three-OS CI complete on `9cb5b8d`: [Actions run 29210426310](https://github.com/juliopolycarpo/mango-lsp/actions/runs/29210426310) — format/lint pass; Ubuntu/macOS `vertical_flow` 12/12 + full suite pass; Windows `vertical_flow` 13/13 (includes Windows drive URI test) + full suite pass. PR #6 MERGEABLE/CLEAN with no unresolved comments; merge still pending maintainer approval. |
| Remaining work | Maintainer review and merge of the S003 PR. After merge, revise later work from vertical-flow evidence at the planning checkpoint; do not start S004 without a reviewed contract. |
| Validation evidence | Local: `cargo test --all-targets --locked vertical_flow -- --nocapture` → 12 passed; full `cargo test --all-targets --locked` → 46 passed (19 unit + 3 CLI + 12 lifecycle + 12 vertical_flow); `cargo fmt --check`, `clippy -D warnings`, offline build/test, and CLI smokes passed. Three-OS CI ([Actions run 29210426310](https://github.com/juliopolycarpo/mango-lsp/actions/runs/29210426310) on `9cb5b8d`): format/lint pass; Ubuntu and macOS focused `vertical_flow` 12/12 and full suite pass; Windows focused `vertical_flow` 13/13 (includes Windows drive URI test) and full suite pass. PR #6 MERGEABLE/CLEAN; no unresolved comments; merge not claimed. |
| Blockers | None for implementation. Merge is gated on explicit maintainer approval. Q-011 remains required only before public distribution. |

S003's authorization prerequisites were satisfied before implementation. Completion
sets authorization to a planning checkpoint; S004 is not unblocked until a
maintainer reviews and authorizes a complete next-stage contract after merge.

## Near-term stage sequence

Only immediate outcomes are listed. Later decomposition must be revised after
the first vertical flow rather than expanded into a speculative backlog.

| ID | Outcome and observable gate | Depends on | State |
| --- | --- | --- | --- |
| P000 | Establish product, decision, state, stage, and handoff sources of truth. Gate: a clean session can identify authorized work and its objective checks without chat history. | None | Complete in the initial repository structure |
| S001 | Establish a reproducible Rust binary whose help, version, invalid-input behavior, tests, lint, formatting, and cross-platform CI are observable. | P000 | Complete in [PR #1](https://github.com/juliopolycarpo/mango-lsp/pull/1), squash commit `40c8d0f` |
| S002 | Prove a bounded downstream STDIO/LSP lifecycle with a deterministic fake: spawn, frame one interaction, correlate it, drain diagnostics, and shut down without orphaning the child. Exact API and crate boundaries remain open. | S001 | Complete in [PR #3](https://github.com/juliopolycarpo/mango-lsp/pull/3). Contract: [`docs/stages/S002-downstream-stdio-lifecycle.md`](stages/S002-downstream-stdio-lifecycle.md) |
| S003 | Complete the first configuration-backed vertical flow: explicit one-server TOML, supervised launch, one `workspace/symbol` interaction, normalized JSON result, redacted JSON Lines events, and controlled shutdown. | Revised S002; Q-004/Q-007 before authorization; Q-009 during implementation | **Implemented — awaiting maintainer PR review/merge**; [contract](stages/S003-configuration-backed-workspace-symbols.md) |
| Checkpoint | Review vertical-flow evidence before specifying multi-server routing, public schema generation, resilience hardening, packaging, or releases. | S003 merge | **Current authorization state** |

### Gate policy

- A stage is one PR-sized outcome that can be demonstrated independently.
- A stage is complete only when its behavioral acceptance criteria and mandatory
  validations have objective evidence and its state changes are included in the
  same PR.
- Completion does not authorize the next row. After S003, set the authorized
  stage to `None — planning checkpoint` unless a maintainer has reviewed and
  authorized a complete next-stage contract.
- If evidence invalidates the sequence, update this table and explain the
  deviation; do not force work through a stale plan.

## Active-stage working record

| Field | Current record |
| --- | --- |
| Stage | None — planning checkpoint after S003 implementation |
| Owner/session | S003 implementation owned by branch `feat/s003-workspace-symbols-vertical-flow` |
| Branch | `feat/s003-workspace-symbols-vertical-flow` |
| Last completed unit | S003 vertical flow: config/URI/query boundaries, one-operation session with interleaved `window/logMessage` and `workspace/workspaceFolders`, normalized envelope + redacted events, fake modes, `vertical_flow` tests, CI gate, D-016/D-017/D-018 |
| Next action | Maintainer reviews and approves the S003 PR; after merge, run the post-vertical checkpoint. Do not authorize or implement S004 implicitly. |
| Changed paths | `src/{main,lib,lifecycle,protocol,config,uri,output,symbols,operation}.rs`, `src/bin/mango_lsp_fake_server.rs`, `tests/vertical_flow.rs`, `tests/downstream_lifecycle.rs`, `Cargo.toml`, `Cargo.lock`, `.github/workflows/ci.yml`, `docs/{STATUS,PROJECT}.md`, `README.md` |
| Checks run | Local mandatory S003 gates passed (focused vertical_flow 12/12; full 46; fmt; clippy `-D warnings`; offline build/test; `--help`/`--version`/`workspace-symbols --help`; unknown option exit 2). Three-OS CI green on [Actions run 29210426310](https://github.com/juliopolycarpo/mango-lsp/actions/runs/29210426310) (`9cb5b8d`): format/lint; Ubuntu/macOS vertical_flow 12/12 + full suite; Windows vertical_flow 13/13 + full suite. |
| Failed or unavailable checks | None for S003 validation. Residual: cleanup remains direct-child only; CI is green pending maintainer merge of the open PR. |
| Open implementation decisions | None for S003. Q-003/Q-008/Q-010/Q-011 remain open for later stages. |
| Resume notes | Public CLI: `workspace-symbols --config --workspace --query`. Limits: config 64 KiB; server id 64 B; command 4 KiB; args 64×4 KiB; query 4 KiB; symbols 10_000; frame header 64 KiB / body 16 MiB; stderr retain 64 KiB; operation timeout 5s; force-shutdown 2s. Cleanup remains direct-child only. Fake is test-only. |

When a stage finishes, replace this record with its outcome and validation
evidence, move it into the history table, and leave the next active record empty
unless another complete contract is authorized.

## Stage history

| Stage | Outcome | Evidence | Material deviations |
| --- | --- | --- | --- |
| P000 | Established the minimal planning and continuity system and specified S001. | Repository documentation and its initial signed commit. | The bootstrap prompt was intentionally retired after its durable requirements were incorporated. |
| S001 | Established the root Rust 2024 application, deterministic bootstrap CLI behavior, real-binary integration tests, pinned toolchain/quality policy, and three-OS CI baseline. | All mandatory local commands passed with Rust 1.97.0; offline build/test passed; invalid option exited 2 with a stderr diagnostic; [Actions run 29190660631](https://github.com/juliopolycarpo/mango-lsp/actions/runs/29190660631) passed format/lint and Linux, macOS, and Windows check/build/test jobs, with 3 CLI tests executed on each OS. | None. An independent review evidence-timestamp finding was resolved by rerunning the complete final-tree validation suite. |
| S002 | Proved bounded direct-child STDIO LSP lifecycle: project-owned framing, minimal JSON-RPC types via serde_json, std process/thread supervision, concurrent stderr drain with truncation, forced cleanup/reap, and hostile fake-server acceptance tests. PR #4 subsequently corrected failure paths to preserve child diagnostics and defer pipe-worker joins until after termination/reap. | Original local and [three-OS CI evidence](https://github.com/juliopolycarpo/mango-lsp/actions/runs/29193519305): 11 lifecycle, 9 unit, and 3 CLI tests (23 total), plus fmt/check/clippy/offline gates. Revised `main` at `9f5692e`: [Actions run 29200218085](https://github.com/juliopolycarpo/mango-lsp/actions/runs/29200218085) passed quality and 12/12 focused lifecycle tests plus the 24-test complete suite on Ubuntu, macOS, and Windows, including diagnostic preservation, backpressure, and forced cleanup. Decisions D-012 and D-013. | Shipped a separate `mango-lsp-fake-server` test binary (not a product CLI subcommand). Packaging must exclude it before release (O-001). Descendant-inherited pipes remain outside the direct-child guarantee. |
| S003 | Delivered configuration-backed `workspace-symbols`: strict one-server TOML, workspace file URI, initialize/`workspace/symbol` session with interleaved supported messages, version 1 JSON stdout envelope, redacted JSON Lines stderr events, bounded error kinds/exits, fake modes, and `vertical_flow` process tests. | Local: focused `vertical_flow` 12/12; full suite 46; fmt/clippy/offline; CLI smokes. Decisions D-016 (Q-009), D-017 (clap `string`), D-018 (`toml` + `percent-encoding`). Three-OS CI: [Actions run 29210426310](https://github.com/juliopolycarpo/mango-lsp/actions/runs/29210426310) on `9cb5b8d` — format/lint pass; Ubuntu/macOS `vertical_flow` 12/12 + full suite pass; Windows `vertical_flow` 13/13 (includes Windows drive URI test) + full suite pass. PR #6 MERGEABLE/CLEAN, no unresolved comments; remains open for maintainer review (merge not claimed). | Timeout errors retain `Timeout` identity after cleanup (public kind `timeout`) instead of being rewritten to `Cleanup`. Clap feature set expanded with `string` (D-017). |

## Discovery and opportunity backlog

This is the source of truth for worthwhile findings outside the authorized stage.
An entry records an opportunity; it does not authorize implementation. Use IDs
`O-001`, `O-002`, and so on, and include the stage or review that discovered it.

| ID | Discovery or opportunity | Value and rationale | Source | Evaluate at | State |
| --- | --- | --- | --- | --- | --- |
| O-001 | Exclude `mango-lsp-fake-server` from release packaging and `cargo install` artifacts. | Keeps the fake out of distributed product surfaces while preserving the normal `cargo test --all-targets` build. | S002 | Release packaging / Q-011 | Open |
| O-002 | Consider feature-gating the fake binary once packaging exists. | Stronger guarantee than documentation alone that the fixture never ships. | S002 | Release packaging | Open |
| O-003 | Attempt graceful `shutdown`/`exit` after post-initialize application failures (for example unsupported capability) before forced cleanup. | May reduce noisy kills against well-behaved servers while preserving finite bounds. | S003 | Post-vertical resilience | Open |
| O-004 | Add injectable operation timeouts on the CLI or config for faster local failure-path tests without rewriting defaults. | Improves contributor ergonomics; not required for the frozen public contract. | S003 | Post-vertical DX | Open |

## Current deviations and blockers

S003 local and three-OS CI evidence is complete on the open implementation PR
([Actions run 29210426310](https://github.com/juliopolycarpo/mango-lsp/actions/runs/29210426310)).
CI is green pending maintainer merge. Authorization is a planning checkpoint;
no subsequent stage is authorized. Cleanup remains direct-child only. Q-011
remains deliberately open until public distribution.

## State transition checklist

At the end of an implementation stage:

1. Record the delivered outcome and exact validation evidence in this file.
2. Move durable decisions and cross-stage risks to `docs/PROJECT.md`.
3. Add deferred findings to the discovery backlog.
4. Explain accepted deviations from the stage contract.
5. Set `Authorized stage` to `None — planning checkpoint` unless the maintainer
   has explicitly authorized a complete next-stage contract.
6. Ensure the PR central promise matches the stage outcome, then stop.
