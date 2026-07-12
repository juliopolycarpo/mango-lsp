# S001: Rust CLI foundation

This file is the complete contract for the first executable stage. It is
executable only while `docs/STATUS.md` names S001 as the authorized stage;
`docs/STATUS.md` also owns progress.

## Minimum context

Read:

- `AGENTS.md` and `docs/STATUS.md` in full;
- the product definition, conceptual boundaries, architectural principles,
  decisions D-001 through D-009, questions Q-001 through Q-003 and Q-011, and
  risks R-003 and R-008 in `docs/PROJECT.md`;
- the repository root and any toolchain or CI files that exist when execution
  begins.

No external conversation or unpublished plan is required.

## Local problem

The repository has no Rust package, executable, test harness, toolchain contract,
or automated platform gate. Starting protocol or process-supervision work now
would mix foundational build decisions with behavior that has materially
different failure modes. Contributors also lack a deterministic command that
proves the intended binary can be built and launched.

## Objective and central promise

Establish a reproducible, cross-platform Rust CLI baseline. From a clean clone, a
contributor must be able to use the declared toolchain to format, lint, build,
test, and launch a binary named `mango-lsp`. The binary must expose deterministic
help, version, and invalid-input behavior, and CI must exercise the baseline on
Linux, macOS, and Windows.

The PR's central demonstration is:

> The repository produces a tested `mango-lsp` executable with a documented and
> automated Rust quality baseline on all three major operating-system families.

This stage creates executable value without pretending to implement LSP.

## Scope

### Included

- A root Cargo package with a binary target named `mango-lsp` and a committed
  lockfile appropriate for an application.
- A checked-in Rust toolchain policy, formatter behavior, and lint baseline.
- Minimal CLI behavior:
  - `mango-lsp --help` exits successfully and identifies the program;
  - `mango-lsp --version` exits successfully and reports the package version;
  - an unknown option exits unsuccessfully and writes a useful diagnostic to
    stderr.
- Process-level integration tests for those three behaviors. Tests must launch
  the built binary and must not use the network or a real language server.
- CI configuration that runs the appropriate format, lint, build/check, and test
  gates across Linux, macOS, and Windows.
- A minimal contributor quick start in `README.md` once the commands exist.
- The `.gitignore` entries and package metadata required by this Rust
  application.
- Updates to `docs/PROJECT.md` and `docs/STATUS.md` required by the decisions and
  handoff rules below.

### Excluded

- Configuration parsing or discovery.
- JSON-RPC types, LSP framing, server capabilities, request routing, or protocol
  schemas.
- Child-process supervision, async runtimes, logging frameworks, health checks,
  restart policy, or shutdown orchestration.
- MangoStudio integration, TypeScript generation, a TUI, daemon transports, or
  a public service protocol.
- Release workflows, installers, binary cross-compilation, package publication,
  signing, provenance, or a support guarantee for a final architecture matrix.
- A speculative multi-crate workspace or reusable library API. A library target
  is allowed only if a concrete testable responsibility requires it and the PR
  explains why a binary-only package was insufficient.
- Choosing or adding a license without maintainer approval.

Do not add placeholder modules for excluded features.

## Preconditions and dependencies

- P000 is complete and `docs/STATUS.md` authorizes S001.
- Work begins on a writable stage branch with unrelated user changes preserved.
- A Rust toolchain can be installed or is available locally. Registry access may
  be used to fetch explicitly selected build dependencies, but runtime and tests
  must be offline and deterministic.
- GitHub-hosted CI can be configured. If no remote or Actions environment exists,
  validate the workflow structure locally where possible and report live
  cross-platform execution as unverified; cross-platform CI remains a pre-merge
  gate once a GitHub PR exists.

If a precondition is false, record it in the active-stage working record before
asking for the narrowest necessary maintainer action.

## Frozen decisions

- D-001, D-002, D-003, D-004, D-005, D-006, D-007, D-008, and D-009 remain in
  force.
- The Cargo package and official binary are named `mango-lsp`.
- The stage establishes one root package, not a permanent crate architecture.
- CLI output implemented here is a bootstrap interface, not the versioned public
  service protocol from D-007.
- CI must reveal operating-system assumptions; platform-specific no-op tests do
  not satisfy the matrix gate.

## Implementation discretion

The implementing agent may choose:

- the Rust edition, pinned toolchain policy, and MSRV after checking current Rust
  support and recording the rationale as the resolution of Q-001;
- a maintained CLI parser or a small project-owned parser after comparing API
  clarity, dependency footprint, license, maintenance, and expected near-term
  command growth for Q-002;
- whether a focused test helper dependency is justified;
- exact help wording and manifest version, provided the acceptance contract is
  deterministic and version output derives from package metadata rather than a
  duplicate literal;
- CI job layout and caching, provided all required gates and OS families remain
  visible;
- the smallest lint policy that catches defects without adding unrelated style
  ceremony.

Dependencies must solve a present stage need. Pin third-party CI actions to an
immutable commit SHA and annotate the corresponding release tag for review.
Avoid native dependencies and build scripts unless evidence shows they are
necessary.

## Escalation and recording triggers

Ask the maintainer, with a recommendation, before:

- selecting a project license or enabling publication;
- changing the package or binary name;
- introducing a native dependency, external runtime, multi-crate workspace, or
  public library API;
- dropping Linux, macOS, or Windows from the CI evidence;
- adding release or deployment behavior.

Record the Q-001 resolution as an accepted decision in `docs/PROJECT.md`. Record
the Q-002 resolution there only if it establishes a durable CLI dependency or
policy; a no-dependency local detail can be explained in the PR. Add newly found
cross-stage risk or architecture questions to the appropriate project registry.
Add useful excluded work to the discovery backlog in `docs/STATUS.md`.

## Deliverables

The coherent change must provide:

- the buildable Rust application and lockfile;
- declared toolchain, format, and lint behavior;
- help, version, and invalid-input implementation;
- deterministic process-level CLI tests;
- Linux, macOS, and Windows CI gates;
- updated contributor commands and project/state records;
- no generated build output or scratch files.

## Acceptance criteria

1. Cargo metadata reports one package named `mango-lsp` with a binary target
   named `mango-lsp`.
2. A clean build using the checked-in toolchain and lockfile succeeds without
   undocumented system dependencies.
3. `mango-lsp --help` exits 0, writes non-empty usage text, and contains the
   program name.
4. `mango-lsp --version` exits 0 and reports the same version as Cargo package
   metadata without maintaining a second version constant.
5. An unknown option exits nonzero, emits a useful diagnostic on stderr, and does
   not panic.
6. Integration tests launch the real built binary and verify criteria 3 through
   5 without network access or installed language servers.
7. Formatting is clean, compilation/checking succeeds for all targets, Clippy
   reports no warnings under the declared policy, and all tests pass.
8. CI represents Linux, macOS, and Windows and runs tests on each. Formatting and
   lint may be deduplicated to one runner if compilation and tests still expose
   platform-specific failures on every OS.
9. `README.md` gives commands that match the checked-in toolchain and verified
   workflow.
10. No LSP, configuration, supervision, logging, release, or future-stage stub is
    present.
11. `docs/PROJECT.md` records durable toolchain/dependency decisions and
    `docs/STATUS.md` records the outcome, exact validation evidence, deviations,
    and next authorization state.

## Mandatory validation

Run these local pre-commit gates from the repository root, adapting only flags
that the checked-in Cargo version demonstrably does not support and documenting
any adaptation:

```text
cargo metadata --no-deps --format-version 1
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo run --locked -- --help
cargo run --locked -- --version
```

Run the implemented binary with a deliberately unknown option and record its
nonzero exit status and stderr behavior. The integration test is the repeatable
gate; a shell transcript supplements it.

Before merge, the CI matrix must pass on Linux, macOS, and Windows. Inspect the
job logs to confirm tests ran and were not skipped. If live CI is unavailable at
handoff, label the cross-platform claim unverified and do not describe S001 as
fully validated.

## Review focus

An independent review should try to falsify the central promise, with emphasis
on:

- a version string duplicated or drifting from Cargo metadata;
- tests that exercise a helper instead of the packaged binary;
- platform branches that make Windows or Unix tests vacuous;
- help or error output accidentally written to the wrong stream;
- CI actions referenced by mutable tags;
- undeclared native/system requirements or unnecessary dependencies;
- a lockfile or toolchain file that CI silently ignores;
- warnings suppressed globally rather than fixed;
- early modules, APIs, or abstractions for excluded future work;
- repository docs or status that claim checks not actually run.

Apply valid findings and rerun affected gates before handoff.

## Improvement latitude and scope guard

The agent may include low-risk corrections to repository instructions, links,
or validation wording discovered while exercising this stage, when they make the
same Rust baseline easier to reproduce. It may centralize a repeated build
setting when the implementation actually needs it.

Do not add generic task runners, release tooling, dependency-audit systems,
benchmarks, coverage services, or future protocol scaffolding merely because
they may be useful later. Record a concrete opportunity and its expected value
in `docs/STATUS.md` for later evaluation.

## Handoff and state update

Before handoff:

1. Resolve and record Q-001 and, when durable, Q-002.
2. Update the current snapshot, working record, history, deviations, blockers,
   and validation evidence in `docs/STATUS.md`.
3. Set `Authorized stage` to `None — planning checkpoint`; S002 must receive a
   separate complete contract and maintainer authorization.
4. Record the exact local commands and results plus live CI URLs or the explicit
   absence of live cross-platform evidence.
5. List dependencies introduced and why each is needed now.
6. Ensure a clean session can reproduce the commands from `README.md` and audit
   the PR without the implementation conversation.

If work stops incomplete, do not perform the completion transition. Instead,
update the active-stage working record with the branch, last coherent checkpoint,
changed paths, remaining work, actual check results, failures, and blockers.

## PR boundary

The Cargo package, CLI contract, process-level tests, toolchain policy, and CI
matrix are one change because together they make one claim independently
verifiable: `mango-lsp` is a real, reproducible cross-platform executable
baseline. Splitting CI or tests into later PRs would leave the initial binary's
promise unverified; adding protocol or supervision would introduce separate
behavior and risk that reviewers cannot judge from the same gate.

Commits may separate mechanical project creation from tests/CI if that improves
review, but all commits must serve this promise and the PR must be reviewable as
one unit.

## Stopping rule

Stop when every acceptance criterion has evidence, required local checks pass,
cross-platform CI has either passed or is explicitly reported as the remaining
pre-merge gate, the state and decision records are updated, and the S001 change
is ready for maintainer review. Do not implement STDIO framing, spawn a language
server, draft S002 in detail, or begin the first vertical flow.
