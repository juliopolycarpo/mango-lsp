# Project status

Last updated: 2026-07-12

This file is the only source of truth for project execution state, stage
authorization, active-stage progress, the near-term sequence, and discovered
opportunities. A stage mentioned in the roadmap is not authorized unless the
snapshot below links its complete contract.

## Current snapshot

| Field | Value |
| --- | --- |
| Phase | First vertical flow planning |
| Implementation state | S002 delivered and revised a bounded direct-child STDIO/LSP lifecycle with framing, correlation, stderr draining, forced cleanup, and a deterministic fake server. A complete S003 contract is now available for review; no S003 implementation is authorized. |
| Authorized stage | **None — planning checkpoint** |
| Contract | Proposed S003: [`docs/stages/S003-configuration-backed-workspace-symbols.md`](stages/S003-configuration-backed-workspace-symbols.md), complete draft pending maintainer review and explicit authorization. Completed S002 remains at [`docs/stages/S002-downstream-stdio-lifecycle.md`](stages/S002-downstream-stdio-lifecycle.md). |
| Progress | S002 was delivered in [PR #3](https://github.com/juliopolycarpo/mango-lsp/pull/3) and its failure paths were corrected in [PR #4](https://github.com/juliopolycarpo/mango-lsp/pull/4) at `9f5692e`. The S003 contract now specifies an explicit one-server TOML configuration, a `workspace-symbols` CLI operation, a normalized JSON result, redacted JSON Lines events, and deterministic three-OS evidence; implementation has not started. |
| Working branch or worktree | No implementation branch is authorized. The S003 contract is a planning artifact only until its review and authorization transition are recorded here. |
| Last coherent checkpoint | Revised S002 behavior is on `main` with current cross-platform evidence, and the complete S003 draft is recoverable from this repository. Q-005/Q-006 remain resolved as D-012/D-013. |
| Remaining work | Review the S003 contract, accept or revise its public boundaries, record decisions resolving Q-004 and Q-007, and explicitly authorize S003 before any implementation. Q-009 resolves from S003 implementation evidence. |
| Validation evidence | Revised S002 [Actions run 29200218085](https://github.com/juliopolycarpo/mango-lsp/actions/runs/29200218085) passed format/lint plus focused and full tests on Ubuntu, macOS, and Windows. Job logs show 12/12 focused `downstream_lifecycle_*` tests on every OS, including the PR #4 diagnostic-preservation regression; the complete suite passed 24 tests per OS. S002 guarantees direct-child cleanup only; descendant process-tree and real-server behavior remain unverified. |
| Blockers | None for planning. Q-004 and Q-007 are deliberate authorization prerequisites; Q-011 remains required only before public distribution. |

S002's completion dependency for S003 is satisfied by the squash-merge of
[PR #3](https://github.com/juliopolycarpo/mango-lsp/pull/3) and the follow-up
failure-path correction in [PR #4](https://github.com/juliopolycarpo/mango-lsp/pull/4).
S003 is dependency-unblocked and has a complete draft contract, but it still
requires maintainer review, Q-004/Q-007 decisions, and explicit authorization
before implementation.

## Near-term stage sequence

Only immediate outcomes are listed. Later decomposition must be revised after
the first vertical flow rather than expanded into a speculative backlog.

| ID | Outcome and observable gate | Depends on | State |
| --- | --- | --- | --- |
| P000 | Establish product, decision, state, stage, and handoff sources of truth. Gate: a clean session can identify authorized work and its objective checks without chat history. | None | Complete in the initial repository structure |
| S001 | Establish a reproducible Rust binary whose help, version, invalid-input behavior, tests, lint, formatting, and cross-platform CI are observable. | P000 | Complete in [PR #1](https://github.com/juliopolycarpo/mango-lsp/pull/1), squash commit `40c8d0f` |
| S002 | Prove a bounded downstream STDIO/LSP lifecycle with a deterministic fake: spawn, frame one interaction, correlate it, drain diagnostics, and shut down without orphaning the child. Exact API and crate boundaries remain open. | S001 | Complete in [PR #3](https://github.com/juliopolycarpo/mango-lsp/pull/3). Contract: [`docs/stages/S002-downstream-stdio-lifecycle.md`](stages/S002-downstream-stdio-lifecycle.md) |
| S003 | Complete the first configuration-backed vertical flow: explicit one-server TOML, supervised launch, one `workspace/symbol` interaction, normalized JSON result, redacted JSON Lines events, and controlled shutdown. | Revised S002; Q-004/Q-007 before authorization; Q-009 during implementation | [Complete draft contract](stages/S003-configuration-backed-workspace-symbols.md) pending review and decision recording; **not authorized** |
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
| Stage | None — planning checkpoint; S002 complete and revised; S003 contract drafted |
| Owner/session | No active implementation stage |
| Branch | None for implementation; the contract may be reviewed on a documentation branch without authorizing S003. |
| Last completed unit | Revised S002 framing, protocol, lifecycle session, fake server, 12 process-level `downstream_lifecycle_*` tests, D-012/D-013, and three-OS CI evidence at `9f5692e` |
| Next action | Review [`docs/stages/S003-configuration-backed-workspace-symbols.md`](stages/S003-configuration-backed-workspace-symbols.md), resolve Q-004/Q-007 if its proposed boundaries are accepted, and record a separate explicit authorization transition. Do not implement S003 before then. |
| Changed paths | Planning artifact only: `docs/stages/S003-configuration-backed-workspace-symbols.md` and `docs/STATUS.md`. No S003 implementation paths are active. |
| Checks run | S003 planning artifact: `git diff --check`, local linked-file existence checks, `cargo metadata --no-deps --format-version 1`, and `cargo fmt --all -- --check` passed; `cargo test --all-targets --locked` passed 24 tests (9 unit, 3 CLI, 12 lifecycle). Revised S002 CI: [run 29200218085](https://github.com/juliopolycarpo/mango-lsp/actions/runs/29200218085) passed format/lint and all test jobs; focused lifecycle suite passed 12/12 on Ubuntu, macOS, and Windows, and the complete suite passed 24 tests per OS. Workflow bounds remain 10 minutes for quality and 20 minutes per test job. |
| Failed or unavailable checks | None. |
| Open implementation decisions | Q-004 and Q-007 must be resolved before S003 authorization; Q-009 resolves from S003 evidence. Q-003 remains open until release planning. |
| Resume notes | S003 proposes `workspace-symbols --config <FILE> --workspace <DIR> --query <TEXT>`, explicit version 1 one-server TOML, one JSON stdout envelope, and redacted JSON Lines stderr events. Existing defaults remain: header 64 KiB, body 16 MiB, stderr retention 64 KiB, operation timeout 5s, force-shutdown 2s. Fake binary is test infrastructure only; cleanup remains direct-child only. |

When a stage finishes, replace this record with its outcome and validation
evidence, move it into the history table, and leave the next active record empty
unless another complete contract is authorized.

## Stage history

| Stage | Outcome | Evidence | Material deviations |
| --- | --- | --- | --- |
| P000 | Established the minimal planning and continuity system and specified S001. | Repository documentation and its initial signed commit. | The bootstrap prompt was intentionally retired after its durable requirements were incorporated. |
| S001 | Established the root Rust 2024 application, deterministic bootstrap CLI behavior, real-binary integration tests, pinned toolchain/quality policy, and three-OS CI baseline. | All mandatory local commands passed with Rust 1.97.0; offline build/test passed; invalid option exited 2 with a stderr diagnostic; [Actions run 29190660631](https://github.com/juliopolycarpo/mango-lsp/actions/runs/29190660631) passed format/lint and Linux, macOS, and Windows check/build/test jobs, with 3 CLI tests executed on each OS. | None. An independent review evidence-timestamp finding was resolved by rerunning the complete final-tree validation suite. |
| S002 | Proved bounded direct-child STDIO LSP lifecycle: project-owned framing, minimal JSON-RPC types via serde_json, std process/thread supervision, concurrent stderr drain with truncation, forced cleanup/reap, and hostile fake-server acceptance tests. PR #4 subsequently corrected failure paths to preserve child diagnostics and defer pipe-worker joins until after termination/reap. | Original local and [three-OS CI evidence](https://github.com/juliopolycarpo/mango-lsp/actions/runs/29193519305): 11 lifecycle, 9 unit, and 3 CLI tests (23 total), plus fmt/check/clippy/offline gates. Revised `main` at `9f5692e`: [Actions run 29200218085](https://github.com/juliopolycarpo/mango-lsp/actions/runs/29200218085) passed quality and 12/12 focused lifecycle tests plus the 24-test complete suite on Ubuntu, macOS, and Windows, including diagnostic preservation, backpressure, and forced cleanup. Decisions D-012 and D-013. | Shipped a separate `mango-lsp-fake-server` test binary (not a product CLI subcommand). Packaging must exclude it before release (O-001). Descendant-inherited pipes remain outside the direct-child guarantee. |

## Discovery and opportunity backlog

This is the source of truth for worthwhile findings outside the authorized stage.
An entry records an opportunity; it does not authorize implementation. Use IDs
`O-001`, `O-002`, and so on, and include the stage or review that discovered it.

| ID | Discovery or opportunity | Value and rationale | Source | Evaluate at | State |
| --- | --- | --- | --- | --- | --- |
| O-001 | Exclude `mango-lsp-fake-server` from release packaging and `cargo install` artifacts. | Keeps the fake out of distributed product surfaces while preserving the normal `cargo test --all-targets` build. | S002 | Release packaging / Q-011 | Open |
| O-002 | Consider feature-gating the fake binary once packaging exists. | Stronger guarantee than documentation alone that the fixture never ships. | S002 | Release packaging | Open |

## Current deviations and blockers

None. Revised S002 three-OS CI evidence is complete. S003 remains a planning
artifact until its public boundaries are reviewed, Q-004/Q-007 are resolved,
and authorization is explicit. Q-011 remains deliberately open until public
distribution.

## State transition checklist

At the end of an implementation stage:

1. Record the delivered outcome and exact validation evidence in this file.
2. Move durable decisions and cross-stage risks to `docs/PROJECT.md`.
3. Add deferred findings to the discovery backlog.
4. Explain accepted deviations from the stage contract.
5. Set the authorized stage to `None — planning checkpoint` unless the maintainer
   has explicitly authorized a complete next-stage contract.
6. Ensure the PR central promise matches the stage outcome, then stop.
