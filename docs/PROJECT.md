# Project definition and decision record

This document is the durable source of truth for the product definition,
architectural boundaries, accepted decisions, open questions, and known project
risks. `docs/STATUS.md` owns mutable execution state; stage contracts own local
scope and gates.

## Product definition

`mango-lsp` is an independent, CLI-first Rust tool that supervises, routes, and
aggregates multiple language servers for coding agents. It mediates downstream
LSP rather than exposing every server directly and without policy. It will expose
a public, versioned interface usable by MangoStudio and other consumers while
remaining operable as a standalone command-line program.

### Goals

- Start, monitor, restart, and stop configured language-server processes
  predictably.
- Implement correct LSP/JSON-RPC framing, lifecycle, request correlation,
  capability handling, routing, and failure reporting.
- Offer agent-oriented operations with bounded behavior and structured results.
- Keep configuration declarative, reviewable, and independent of MangoStudio's
  database.
- Produce structured diagnostics without contaminating protocol streams.
- Ship as a standalone, multi-architecture binary with deliberate Windows,
  macOS, and Unix behavior.
- Maintain a versioned external protocol whose evolution can be tested and
  documented.

### Non-goals

- A standalone desktop application or a second graphical product surface.
- Reimplementing language servers or language semantics.
- Requiring MangoStudio or its database to run the CLI.
- Network daemon transports in the first vertical flow.
- Stabilizing the complete public protocol, routing policy, crate graph, or
  dependency stack before a small end-to-end flow provides evidence.
- Treating a possible operational TUI as a primary interface.

## Conceptual boundaries

These are responsibility boundaries, not a prescribed crate layout.

1. **Agent-facing control plane.** CLI and future public-protocol inputs are
   validated at this boundary and translated into bounded operations. Raw,
   unrestricted downstream LSP is not the product interface.
2. **Configuration.** Declarative configuration describes servers and policy.
   Configuration that names executables or arguments is untrusted input at a
   process-execution boundary.
3. **Supervision.** Process ownership, health, restart policy, backpressure,
   shutdown, and cleanup are distinct from message semantics. Process-tree and
   pipe behavior must be made explicit per operating system.
4. **Downstream protocol.** LSP lifecycle and JSON-RPC semantics are distinct
   from byte framing. Server stdout carries protocol bytes; diagnostics and
   project logs must never corrupt it.
5. **Integration.** MangoStudio consumes a versioned external interface. It does
   not own `mango-lsp` state, implementation, or release cadence.

## Architectural principles

- Prove a thin, observable vertical path before designing a broad abstraction
  hierarchy.
- Keep public contracts narrower and more stable than internal implementation.
- Separate framing, protocol semantics, process supervision, and consumer-facing
  policy where their failure modes differ, without forcing a crate per concern.
- Prefer deterministic fake servers for required tests; add controlled real-server
  smoke tests only as supplementary evidence.
- Treat malformed frames, server output, configuration, paths, capabilities,
  and subprocess behavior as untrusted boundary data.
- Make cancellation, timeouts, backpressure, crash recovery, and shutdown
  observable rather than relying on best effort.
- Design and test platform differences at the process boundary early. Do not
  scatter operating-system conditionals through protocol logic.
- Delay irreversible dependency and schema choices until the stage that can
  evaluate them against a running path.
- Use objective compiler, test, smoke, and cross-platform results as gates. A
  plausible implementation or a compiling stub is not completion.

## Accepted decisions

Accepted decisions are not reopened implicitly. To change one, add a decision
that names the superseded record, rationale, consequences, and migration impact.

| ID | Date | Decision and rationale | Consequence |
| --- | --- | --- | --- |
| D-001 | 2026-07-12 | The primary implementation and official CLI use Rust, providing a single systems-language toolchain and strong compile-time safety. | The first executable stage establishes a Rust baseline; alternatives may be used only for generated clients or test tooling when justified. |
| D-002 | 2026-07-12 | Distribution targets a standalone, multi-architecture binary. | Native dependencies, runtime companions, and dynamic libraries require explicit justification; packaging follows the vertical proof. |
| D-003 | 2026-07-12 | The repository, state, process, and release cycle are independent from MangoStudio. | MangoStudio integration occurs through a versioned boundary and cannot be a prerequisite for CLI operation. |
| D-004 | 2026-07-12 | The product is CLI-first; MangoStudio is the primary graphical integration; no mango-lsp desktop application is planned. | A small future TUI may support operations but must not become a second product surface. |
| D-005 | 2026-07-12 | Initial server communication uses STDIO. | Unix sockets and Windows named pipes remain possible later and must not shape the first vertical flow. |
| D-006 | 2026-07-12 | Configuration is declarative and independent of the MangoStudio database. | Configuration format and discovery remain open, but database coupling is excluded. |
| D-007 | 2026-07-12 | External consumers will receive a public, versioned protocol. | Stabilization waits for vertical evidence; schemas and a TypeScript client may be generated if that proves useful. |
| D-008 | 2026-07-12 | Work proceeds as sequential, reviewable stages in clean contexts. | Only one stage is authorized at a time, repository state replaces chat memory, and repository-modifying work is not parallelized. |
| D-009 | 2026-07-12 | English is the repository's default language. | Any non-English content needs a documented interoperability, localization, or test reason. |
| D-010 | 2026-07-12 | The project uses Rust 2024 and pins the contributor and CI toolchain to Rust 1.97.0, which is also the initial MSRV. Pinning the current stable toolchain and its formatter and linter components makes clean-clone results reproducible; compatibility with older compilers is not yet a product requirement. | `rust-toolchain.toml`, Cargo metadata, local gates, and CI use the same exact toolchain. Raising the MSRV requires an explicit decision update and validation. |
| D-011 | 2026-07-12 | The CLI boundary uses clap 4.6.1's builder API with only its `std`, `help`, `usage`, and `error-context` features. Its maintained help and diagnostic behavior fits expected command growth better than a project-owned parser, while avoiding derive macros, terminal styling, suggestions, native dependencies, and duplicate version constants. | Bootstrap and future CLI arguments should use this boundary while the dependency remains justified; the public service protocol remains separate. |

## Open questions

An open question is not a decision. Resolve it only when a stage has evidence and
needs the answer; add the resulting decision above.

| ID | Question | Decision point |
| --- | --- | --- |
| Q-003 | What operating-system and architecture matrix is required for releases? | Start evidence in S001 CI; freeze before release automation. |
| Q-004 | What configuration format, discovery rule, precedence, and validation model should be public? | Before the configuration-backed vertical flow. |
| Q-005 | Which async runtime, process APIs, and shutdown model best satisfy cross-platform supervision? | During the downstream lifecycle proof, after small experiments if needed. |
| Q-006 | Which LSP/JSON-RPC types or libraries should be reused, and which framing behavior should remain project-owned? | During the downstream lifecycle proof; evaluate protocol coverage, control, maintenance, and license. |
| Q-007 | What is the smallest useful agent-facing operation and external protocol envelope? | Before exposing the first configuration-backed vertical flow. |
| Q-008 | How are multiple servers selected, routed, aggregated, and reconciled when capabilities conflict? | After one-server lifecycle evidence; not required for the first vertical flow. |
| Q-009 | What logging/tracing schema, redaction policy, and stdout/stderr contract should be public? | During the first observable supervised flow. |
| Q-010 | Which protocol schemas and clients should be generated, and by which toolchain? | After the external envelope is exercised and versioning rules are defined. |
| Q-011 | Which open-source license and package-publication metadata apply? | Maintainer decision before public distribution; not a blocker for S001 if publication remains disabled. |

## Known risks

| ID | Risk | Early control or evidence |
| --- | --- | --- |
| R-001 | A language server can emit malformed lengths, oversized messages, invalid JSON, unsolicited responses, or protocol text on stderr/stdout. | Bound allocations, parse incrementally, keep logs separate, and test malformed fake-server cases before real-server reliance. |
| R-002 | Cancellation, request IDs, dynamic registration, and lifecycle ordering can race or leave requests unresolved. | Model lifecycle states explicitly and use deterministic transcript tests with controlled scheduling. |
| R-003 | Child shutdown and process-tree cleanup differ materially across Unix and Windows. | Put platform behavior behind a narrow supervision boundary and require CI evidence on all three major OS families early. |
| R-004 | Pipe backpressure or an unread stderr stream can deadlock the supervisor. | Exercise bounded high-volume fake output and independently drain required streams before claiming supervision is robust. |
| R-005 | A public protocol designed before a vertical flow may expose downstream LSP details and become costly to evolve. | Keep early surfaces explicitly narrow or experimental; version only behavior demonstrated end to end. |
| R-006 | Tests using installed real servers can be nondeterministic, slow, or unavailable and can conceal unsupported variants. | Make a project-owned fake server the acceptance oracle; use pinned real servers only for supplementary compatibility smoke tests. |
| R-007 | Declarative server commands create a code-execution and trust boundary. | Define configuration ownership and validation before adding discovery, remote configuration, or automatic execution. |
| R-008 | Path, URI, encoding, and case-sensitivity differences can corrupt routing across platforms. | Add cross-platform fixtures when paths first cross the external or downstream protocol boundary. |
| R-009 | Multiple servers can create resource starvation, ambiguous results, or incompatible capabilities. | Prove bounded one-server behavior first, then define routing and fairness with explicit limits and observability. |

## Premises and consequences reviewed

No accepted premise is rejected for the first stage, but several require a
careful interpretation.

| Premise | Identified risk | Alternative | Recommendation | Blocks S001? |
| --- | --- | --- | --- | --- |
| Publish a versioned protocol. | Freezing an envelope before observing real lifecycle failures creates accidental compatibility debt. | Expose raw LSP or keep everything private indefinitely. | Preserve the versioned-protocol commitment, but delay its concrete schema until a deterministic vertical flow exists; never default to raw unrestricted LSP. | No. |
| Ship one standalone multi-architecture binary. | Choosing native dependencies early can make this promise expensive or impossible on a target. | Permit runtime companions or platform-specific products. | Treat standalone delivery as a dependency constraint now and defer packaging mechanics until the core path runs cross-platform. | No. |
| Support real language servers. | Making a third-party server the test oracle introduces version and environment drift. | Test only mocks or require a real server everywhere. | Use a behavior-rich fake for gates and later add a pinned real-server compatibility smoke; neither alone is sufficient for production confidence. | No. |
| Aggregate multiple servers. | Designing aggregation before one server is correctly supervised encourages speculative abstractions. | Build a generic multiplexer first. | Prove one-server lifecycle and framing, then use observed capability and routing conflicts to design aggregation. | No. |

## Incremental strategy

The project follows a short evidence ladder:

1. Establish a reproducible Rust CLI and cross-platform validation baseline.
2. Prove downstream STDIO framing and lifecycle against a deterministic fake.
3. Demonstrate a configuration-backed, observable CLI flow that starts a server,
   performs one useful LSP interaction, reports a structured result, and shuts
   down cleanly.
4. Reassess crate boundaries, public protocol shape, real-server compatibility,
   multi-server routing, and release architecture from that evidence.

Only step 1 is currently specified and authorized. `docs/STATUS.md` owns the
current sequence and gates.

## Why this repository structure

The repository keeps one stable project/decision record, one mutable state
manifest, one contract per executable stage, and one short repository instruction
file. This borrows the useful parts of decision logs and executable project
plans without requiring a separate ADR, issue, or status document for every
choice. The roadmap stays shallow because detailed distant plans would become
false authority before implementation evidence exists. External trackers may
link to these records, but they are not required to recover project state.

The one-shot bootstrap prompt is not retained: its session-specific commands
would become stale and compete with the sources of truth above. Its product
constraints, decision consequences, methodology, and first-stage requirements
are represented in the permanent records instead.

## Methodology

The planning model adapts lessons from Bun's official
[_Rewriting Bun in Rust_](https://bun.com/blog/bun-in-rust) account:

- serialize important context and constraints in the repository before coding;
- decompose work by observable outcomes and objective failure queues;
- de-risk a broad effort with a small trial;
- make compiler, smoke, conformance, and platform results the confidence source;
- review in a clean, adversarial context and apply the feedback;
- fix stage instructions or validation when they repeatedly produce bad output,
  instead of repeatedly patching the same symptom.

Bun's situation was a large mechanical rewrite with a language-independent
conformance suite and extensive parallel workflows. `mango-lsp` is greenfield,
small, serial, and human-supervised. It therefore does **not** copy the
all-at-once rewrite or parallel worktree loops. Each stage delivers one coherent
PR, a human may stop or redirect between stages, and only immediate work receives
detailed specification. Deterministic fakes provide the first conformance
evidence because no prior implementation or test suite exists.

## Maintaining this document

- Add accepted decisions with the next `D-` ID; do not reuse IDs.
- Add unresolved product or architecture choices with the next `Q-` ID.
- Add cross-stage risks with the next `R-` ID. Stage-local risks stay in the
  stage contract.
- Keep execution progress and opportunity backlog out of this document.
- If this registry becomes hard to navigate, split it only with links and one
  authoritative location per record.
