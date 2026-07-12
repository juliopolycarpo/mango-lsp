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
| Implementation state | S001 Rust CLI baseline implemented locally; live cross-platform CI is pending. |
| Authorized stage | **S001 — Rust CLI foundation** |
| Contract | [`docs/stages/S001-rust-cli-foundation.md`](stages/S001-rust-cli-foundation.md) |
| Progress | Implementation, local validation, and independent review complete; live CI pending |
| Working branch or worktree | `feat/s001-rust-cli-foundation` |
| Last coherent checkpoint | The binary, process-level tests, toolchain policy, CI workflow, and contributor documentation pass all mandatory local gates. |
| Remaining work | Open the PR, verify Linux/macOS/Windows CI logs, then record the completion transition. |
| Validation evidence | Final-tree mandatory gates and a separate offline build/test passed on Linux with Rust 1.97.0; 3 process-level CLI tests passed. Direct invalid-input smoke exited 2 with a useful stderr diagnostic. Independent review found no implementation defect after its evidence-timestamp finding was reconciled by a full rerun. Live cross-platform CI is not yet verified. |
| Blockers | None. Q-011 (license) must be resolved before public distribution, not before S001. |

The next implementation session may execute S001. It must not begin S002.

## Near-term stage sequence

Only immediate outcomes are listed. Later decomposition must be revised after
the first vertical flow rather than expanded into a speculative backlog.

| ID | Outcome and observable gate | Depends on | State |
| --- | --- | --- | --- |
| P000 | Establish product, decision, state, stage, and handoff sources of truth. Gate: a clean session can identify authorized work and its objective checks without chat history. | None | Complete in the initial repository structure |
| S001 | Establish a reproducible Rust binary whose help, version, invalid-input behavior, tests, lint, formatting, and cross-platform CI are observable. | P000 | **Authorized; fully specified** |
| S002 | Prove a bounded downstream STDIO/LSP lifecycle with a deterministic fake: spawn, frame one interaction, correlate it, drain diagnostics, and shut down without orphaning the child. Exact API and crate boundaries remain open. | S001 | Planned outline; not authorized |
| S003 | Complete the first configuration-backed vertical flow: CLI startup, minimal declarative configuration, supervised server launch, one useful LSP interaction, structured result and logs, and controlled shutdown. | S002 and decisions needed from Q-004, Q-007, and Q-009 | Planned outline; not authorized |
| Checkpoint | Review vertical-flow evidence before specifying multi-server routing, public schema generation, resilience hardening, packaging, or releases. | S003 | Not authorized |

### Gate policy

- A stage is one PR-sized outcome that can be demonstrated independently.
- A stage is complete only when its behavioral acceptance criteria and mandatory
  validations have objective evidence and its state changes are included in the
  same PR.
- Completion does not authorize the next row. After S001, set the authorized
  stage to `None — planning checkpoint` unless a maintainer has reviewed and
  authorized a complete S002 contract.
- If evidence invalidates the sequence, update this table and explain the
  deviation; do not force work through a stale plan.

## Active-stage working record

Update these fields whenever work must survive a handoff or context reset. Keep
them factual and point to commits, paths, or exact command results where useful.

| Field | Current record |
| --- | --- |
| Stage | S001 |
| Owner/session | Codex implementation session, 2026-07-12 |
| Branch | `feat/s001-rust-cli-foundation` |
| Last completed unit | S001 implementation and all mandatory local validation |
| Next action | Commit and open the PR, then inspect every live CI matrix job before completing the stage record. |
| Changed paths | `Cargo.toml`, `Cargo.lock`, toolchain/format/lint policy, `src/main.rs`, `tests/cli.rs`, `.github/workflows/ci.yml`, `.gitignore`, `README.md`, `docs/PROJECT.md`, and this record |
| Checks run | Final tree: `cargo metadata --no-deps --format-version 1`; `cargo fmt --all -- --check`; `cargo check --all-targets --locked`; `cargo clippy --all-targets --locked -- -D warnings`; `cargo test --all-targets --locked` (3 passed); `cargo build --locked`; help/version runs; invalid option smoke (exit 2). Separate `--offline` build and test also passed. |
| Failed or unavailable checks | Initial check/Clippy/test run exposed and then resolved test error E0382. Live GitHub CI has not run yet. |
| Open implementation decisions | None; Q-001 and Q-002 resolved by D-010 and D-011. Q-003 remains intentionally open until release planning. |
| Resume notes | Local work is coherent, validated, and independently reviewed. Do not claim S001 complete until the PR's Linux, macOS, and Windows jobs pass and logs confirm tests ran. |

When a stage finishes, replace this record with its outcome and validation
evidence, move it into the history table, and leave the next active record empty
unless another complete contract is authorized.

## Stage history

| Stage | Outcome | Evidence | Material deviations |
| --- | --- | --- | --- |
| P000 | Established the minimal planning and continuity system and specified S001. | Repository documentation and its initial signed commit. | The bootstrap prompt was intentionally retired after its durable requirements were incorporated. |

## Discovery and opportunity backlog

This is the source of truth for worthwhile findings outside the authorized stage.
An entry records an opportunity; it does not authorize implementation. Use IDs
`O-001`, `O-002`, and so on, and include the stage or review that discovered it.

| ID | Discovery or opportunity | Value and rationale | Source | Evaluate at | State |
| --- | --- | --- | --- | --- | --- |
| _None_ | No deferred implementation opportunities have been recorded. | — | P000 | — | — |

Remove the `_None_` row when adding the first real entry. Resolve an entry by
linking the decision, stage, issue, or reason for rejection; do not silently
delete it.

## Current deviations and blockers

None. The license question is deliberately open and non-blocking for S001 because
that stage must not publish a package or release.

## State transition checklist

At the end of an implementation stage:

1. Record the delivered outcome and exact validation evidence in this file.
2. Move durable decisions and cross-stage risks to `docs/PROJECT.md`.
3. Add deferred findings to the discovery backlog.
4. Explain accepted deviations from the stage contract.
5. Set the authorized stage to `None — planning checkpoint` unless the maintainer
   has explicitly authorized a complete next-stage contract.
6. Ensure the PR central promise matches the stage outcome, then stop.
