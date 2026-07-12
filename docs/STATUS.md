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
| Implementation state | S001 delivered a reproducible, tested Rust CLI baseline. |
| Authorized stage | **None — planning checkpoint** |
| Contract | No authorized contract. The complete S002 review candidate is [`docs/stages/S002-downstream-stdio-lifecycle.md`](stages/S002-downstream-stdio-lifecycle.md); completed S001 remains at [`docs/stages/S001-rust-cli-foundation.md`](stages/S001-rust-cli-foundation.md). |
| Progress | S001 was squash-merged through [PR #1](https://github.com/juliopolycarpo/mango-lsp/pull/1) as `40c8d0f`; the S002 contract is ready for maintainer review but is not authorized. |
| Working branch or worktree | No implementation branch; this documentation change prepares the S002 review candidate. |
| Last coherent checkpoint | S001 is on `main`, and the complete S002 downstream-lifecycle contract has been authored from the merged baseline and current project decisions. |
| Remaining work | Review the S002 contract, revise it if needed, then explicitly authorize S002 in this file before creating an implementation branch. |
| Validation evidence | Final-tree mandatory gates and a separate offline build/test passed locally on Linux with Rust 1.97.0. Three real-binary CLI tests passed locally and in each OS job in [Actions run 29190660631](https://github.com/juliopolycarpo/mango-lsp/actions/runs/29190660631); logs were inspected and showed 3 passed, 0 failed, and 0 ignored on Linux, macOS, and Windows. Direct invalid-input smoke exited 2 with a useful stderr diagnostic. |
| Blockers | None. Q-005 and Q-006 are intentionally resolved from S002 implementation evidence; Q-011 remains required only before public distribution. |

No implementation stage is currently authorized. The S002 review candidate is
complete, but implementation must wait for maintainer review and an explicit
authorization transition in this file.

## Near-term stage sequence

Only immediate outcomes are listed. Later decomposition must be revised after
the first vertical flow rather than expanded into a speculative backlog.

| ID | Outcome and observable gate | Depends on | State |
| --- | --- | --- | --- |
| P000 | Establish product, decision, state, stage, and handoff sources of truth. Gate: a clean session can identify authorized work and its objective checks without chat history. | None | Complete in the initial repository structure |
| S001 | Establish a reproducible Rust binary whose help, version, invalid-input behavior, tests, lint, formatting, and cross-platform CI are observable. | P000 | Complete in [PR #1](https://github.com/juliopolycarpo/mango-lsp/pull/1), squash commit `40c8d0f` |
| S002 | Prove a bounded downstream STDIO/LSP lifecycle with a deterministic fake: spawn, frame one interaction, correlate it, drain diagnostics, and shut down without orphaning the child. Exact API and crate boundaries remain open. | S001 | [Complete contract drafted](stages/S002-downstream-stdio-lifecycle.md); awaiting review and explicit authorization |
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
| Stage | None |
| Owner/session | — |
| Branch | — |
| Last completed unit | S001 on `main` at `40c8d0f`; see stage history and PR #1 |
| Next action | Review [`docs/stages/S002-downstream-stdio-lifecycle.md`](stages/S002-downstream-stdio-lifecycle.md), then explicitly authorize it here before implementation. |
| Changed paths | None for an active stage |
| Checks run | None for an active stage; completed S001 evidence is recorded above and in stage history. |
| Failed or unavailable checks | None |
| Open implementation decisions | Q-005 and Q-006 are reserved for evidence-driven resolution during an authorized S002; Q-003 remains intentionally open until release planning. |
| Resume notes | No implementation is authorized. S002 has a complete review-candidate contract but has not started. |

When a stage finishes, replace this record with its outcome and validation
evidence, move it into the history table, and leave the next active record empty
unless another complete contract is authorized.

## Stage history

| Stage | Outcome | Evidence | Material deviations |
| --- | --- | --- | --- |
| P000 | Established the minimal planning and continuity system and specified S001. | Repository documentation and its initial signed commit. | The bootstrap prompt was intentionally retired after its durable requirements were incorporated. |
| S001 | Established the root Rust 2024 application, deterministic bootstrap CLI behavior, real-binary integration tests, pinned toolchain/quality policy, and three-OS CI baseline. | All mandatory local commands passed with Rust 1.97.0; offline build/test passed; invalid option exited 2 with a stderr diagnostic; [Actions run 29190660631](https://github.com/juliopolycarpo/mango-lsp/actions/runs/29190660631) passed format/lint and Linux, macOS, and Windows check/build/test jobs, with 3 CLI tests executed on each OS. | None. An independent review evidence-timestamp finding was resolved by rerunning the complete final-tree validation suite. |

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

None. Q-011 remains deliberately open until public distribution and does not
block review or execution of S002 while the package remains unpublished.

## State transition checklist

At the end of an implementation stage:

1. Record the delivered outcome and exact validation evidence in this file.
2. Move durable decisions and cross-stage risks to `docs/PROJECT.md`.
3. Add deferred findings to the discovery backlog.
4. Explain accepted deviations from the stage contract.
5. Set the authorized stage to `None — planning checkpoint` unless the maintainer
   has explicitly authorized a complete next-stage contract.
6. Ensure the PR central promise matches the stage outcome, then stop.
