# S002: Bounded downstream STDIO lifecycle

This file is the complete review candidate for the second executable stage. It
is executable only while `docs/STATUS.md` names S002 as the authorized stage;
authoring or linking this contract does not authorize implementation.

## Minimum context

Read:

- `AGENTS.md` and `docs/STATUS.md` in full;
- the product definition; supervision and downstream-protocol conceptual
  boundaries; architectural principles; decisions D-001, D-002, D-003, D-005,
  D-007 through D-011; questions Q-005 and Q-006; risks R-001 through R-006;
  and the incremental strategy in `docs/PROJECT.md`;
- the Base Protocol, request/response definitions, and Lifecycle Messages in the
  official [LSP 3.18 specification][lsp-3-18],
  limited to the behavior named by this contract;
- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `src/`, `tests/`,
  `.github/workflows/ci.yml`, and the S001 evidence recorded in
  `docs/STATUS.md`.

No external conversation, installed language server, credentials, or
unpublished plan is required.

[lsp-3-18]: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/

## Local problem

S001 proves that the repository builds and runs a portable Rust CLI, but the
application cannot yet own a downstream process or exchange one protocol
message. There is no evidence that it can frame byte-oriented LSP messages,
match a response to its request, drain a child's diagnostic stream without
deadlocking, enforce resource and time bounds, or reap the child on normal and
failure paths.

Adding configuration or a consumer-facing operation now would combine those
unproven process and protocol failure modes with public product decisions. This
stage isolates the downstream lifecycle first and makes a project-owned fake
server the deterministic acceptance oracle.

## Objective and central promise

Prove one bounded, direct-child LSP lifecycle over STDIO. Given an explicitly
constructed child command, the implementation must spawn it without a shell,
exchange a correctly framed and correlated `initialize` interaction, send the
specification-ordered `initialized`, `shutdown`, and `exit` messages, drain
stderr concurrently, and return only after the child has been reaped. Malformed
or stalled behavior must produce a useful bounded failure and the same cleanup
guarantee.

The PR's central demonstration is:

> On Linux, macOS, and Windows, a deterministic fake language server completes
> one framed LSP lifecycle, while protocol errors, backpressure, stalls, and
> early exits fail predictably without leaving the direct child running.

This is an internal downstream proof. It does not expose a user operation or
stabilize a public Rust or service API.

## Scope

### Included

- Project-owned LSP STDIO framing for the required ASCII headers, CRLF header
  terminator, UTF-8 JSON body, and byte-counted `Content-Length`.
- Incremental reads that tolerate headers and bodies split across multiple I/O
  operations and enforce explicit header and body size limits before retaining
  untrusted data.
- The minimum JSON-RPC/LSP message representation needed for requests,
  responses, notifications, response errors, and the lifecycle sequence in
  this stage.
- Direct child-process ownership using piped stdin, stdout, and stderr, with no
  shell interpretation and no detached process.
- A lifecycle state model sufficient to enforce:
  `initialize` request and correlated response, one `initialized` notification,
  `shutdown` request and correlated response, then one `exit` notification.
- Caller-configurable operation and shutdown bounds, followed by forced direct
  child termination and a wait/reap when graceful completion is no longer
  possible.
- Independent stderr draining with bounded retained diagnostics and an explicit
  indication when retained output was truncated.
- A deterministic, project-owned fake server with modes that prove the normal
  lifecycle, fragmented protocol output, diagnostic backpressure, malformed or
  oversized input, mismatched response IDs, early exit, and refusal to shut
  down.
- Focused unit and process-level regression tests. The fake must be test-only,
  offline, cross-platform, and must not become a user-visible CLI command or a
  release artifact.
- Any CI timeout or test invocation changes needed to make the new lifecycle
  gate finite and visible on Linux, macOS, and Windows.
- Resolution of Q-005 and Q-006 in `docs/PROJECT.md`, including the evidence and
  consequences of any runtime, process, JSON, or LSP dependency selected.
- The contributor and state-documentation updates needed to reproduce and audit
  the proof.

### Excluded

- Configuration files, command discovery, environment inheritance policy,
  workspace discovery, or accepting an executable path from a user-facing
  boundary.
- A new CLI subcommand, agent-facing operation, daemon transport, MangoStudio
  integration, public service envelope, or stable public Rust library API.
- Document synchronization, file paths or URIs, language features, capability
  negotiation beyond the minimal initialize result, server-to-client requests,
  dynamic registration, cancellation, progress, or multiple concurrent
  in-flight requests.
- Restart policy, health checking, pooling, multiple servers, routing,
  aggregation, or resource fairness.
- Descendant process-tree containment or termination. S002 must own and reap the
  direct child; broader process-tree guarantees require explicit platform design
  and later evidence.
- Public logging or tracing schema, redaction policy, persistence, or telemetry.
  Test-visible bounded stderr capture is an internal diagnostic seam only.
- A real language server as a required test oracle, compatibility claims for a
  named server, or any public-network access in tests.
- Complete JSON-RPC or LSP 3.18 conformance, generated protocol schemas, a
  TypeScript client, release automation, packaging, publication, or licensing.
- Speculative protocol dispatch, generic plugin abstractions, or a permanent
  multi-crate architecture built for later stages.

## Preconditions and dependencies

- S001 is present on `main` at squash commit `40c8d0f` with its local and
  three-operating-system evidence recorded in `docs/STATUS.md`.
- Before implementation begins, `docs/STATUS.md` must explicitly name S002 as
  the authorized stage and link this contract. Its current review-candidate
  state is not authorization.
- Work begins on a writable S002 branch with unrelated user changes preserved.
- The pinned Rust 1.97.0 toolchain and the existing Linux, macOS, and Windows CI
  matrix remain available.
- Registry access may fetch reviewed build dependencies. The built product and
  all tests must run offline, without a real language server or another runtime
  installed.

If a precondition is false, record the mismatch in the active-stage working
record before requesting the narrowest maintainer action. Do not weaken a
platform or cleanup claim to bypass unavailable evidence.

## Frozen decisions

- D-001, D-002, D-003, D-005, and D-007 through D-011 remain in force.
- S002 owns one direct child communicating over STDIO. Do not introduce sockets,
  named pipes, a daemon, shell execution, or MangoStudio coupling.
- For the behavior under test, follow the official LSP base framing and lifecycle
  order: `Content-Length` counts body bytes; the `initialize` request is first;
  `initialized` follows its response; `shutdown` receives a response before
  `exit`; and a normal server exits successfully after that sequence.
- An absent `Content-Type` means UTF-8. Accept the specified `utf-8` charset and
  the legacy `utf8` spelling recommended for compatibility; reject other
  encodings within this stage's subset.
- All reads, retained diagnostics, request waits, graceful shutdown waits, and
  forced-cleanup waits must have explicit finite bounds. A timeout error is not
  complete until the direct child is terminated when necessary and reaped.
- Server stdout is protocol-only. Child stderr must never be parsed as LSP or
  copied into a protocol stream.
- The fake server is the mandatory acceptance oracle and may deliberately be
  hostile. Production services, public network access, wall-clock sleeps used
  as synchronization, and installed language servers are not acceptable gates.
- The existing `mango-lsp --help`, `--version`, and invalid-option behavior must
  remain intact. Test-fixture behavior must not be reachable through the shipped
  CLI.
- Do not claim broad LSP 3.18 support from this lifecycle subset.

## Implementation discretion

The implementing agent may choose:

- synchronous threads and channels or a maintained async runtime, after
  comparing process support, cancellation behavior, timer semantics,
  cross-platform behavior, dependency cost, license, and maintenance for Q-005;
- a maintained JSON-RPC/LSP type crate, focused serialization dependencies, or
  project-owned minimal types, after comparing protocol coverage, validation
  control, dependency surface, license, and compatibility for Q-006;
- module boundaries and whether the existing root package gains an internal
  library target. Public visibility needed by integration tests does not by
  itself make an API stable;
- the test-only fake's source and Cargo target layout, provided it is built
  reproducibly by the normal test command and is not exposed as product
  behavior;
- request ID representation, internal lifecycle states, error types, and
  diagnostic summary shape, provided errors identify the failed operation and
  retain useful bounded child context;
- exact header, body, diagnostic-retention, and time limits. Defaults must be
  documented at the owning boundary, small limits must be injectable in tests,
  and an outer test or CI watchdog must prevent a regression from hanging the
  gate indefinitely;
- whether protocol encoding and decoding share one codec type or separate
  responsibilities, provided byte framing can be tested independently from
  process ownership.

Dependencies must solve a present S002 need. Prefer pure Rust dependencies that
support the pinned toolchain. Keep feature sets narrow, retain the committed
lockfile, and do not add native libraries, build scripts, or an external runtime
without escalation.

## Escalation and recording triggers

Ask the maintainer, with a recommendation, before:

- exposing a new CLI surface, public service schema, stable Rust API, or
  configuration boundary;
- adding unsafe code, a native dependency, a build script, shell execution, a
  runtime companion, or a durable multi-crate workspace;
- changing the direct-child-only guarantee into descendant process-tree
  management, detaching a child, or adopting platform-specific process policy;
- dropping or weakening Linux, macOS, or Windows lifecycle evidence;
- making a real server or public network resource part of the acceptance gate;
- materially departing from the LSP base framing or lifecycle order named here.

Resolve Q-005 and Q-006 as accepted decisions in `docs/PROJECT.md`, even when the
result is standard-library process support or project-owned protocol types.
Record dependency versions only when version pinning itself is a durable policy;
otherwise record the selected boundary and rationale and leave exact resolution
to `Cargo.lock`. Add newly discovered cross-stage risks or architectural
questions to `docs/PROJECT.md`, useful excluded work to the discovery backlog in
`docs/STATUS.md`, and material contract deviations to the S002 history entry.

## Deliverables

The coherent change must provide:

- bounded LSP frame encoding and incremental decoding for the lifecycle subset;
- minimal JSON-RPC/LSP lifecycle messages and response correlation;
- direct-child spawn, concurrent pipe ownership, graceful shutdown, forced
  termination, and reap behavior;
- bounded stderr draining and a useful diagnostic summary;
- the deterministic fake server and its hostile modes;
- focused codec/state tests and process-level lifecycle acceptance tests whose
  names share the `downstream_lifecycle` prefix so the focused gate is
  reproducible;
- dependency and lockfile changes justified by Q-005 and Q-006, if any;
- CI coverage on Linux, macOS, and Windows with a finite job-level ceiling;
- updated contributor guidance, accepted decisions, and execution state;
- no generated build output, shipped fake-server command, or scratch fixture.

## Acceptance criteria

1. Frame encoding emits an ASCII `Content-Length` header whose value is the
   UTF-8 body byte length, followed by `\r\n\r\n` and exactly that body.
2. Frame decoding handles a header and body fragmented across reads, rejects a
   missing, invalid, duplicate, or conflicting `Content-Length`, treats an
   absent content type as UTF-8, accepts the `utf-8` and legacy `utf8` charset
   spellings, rejects other encodings, and rejects headers or bodies over
   configured limits before allocating or retaining the declared body size.
3. The normal fake is launched directly with piped stdin, stdout, and stderr. It
   observes a specification-valid minimal `initialize` request as the first
   message, returns a fragmented JSON-RPC 2.0 response with the same non-null ID,
   then observes exactly one `initialized`, one `shutdown` with a distinct ID,
   and one `exit` in specification order. The shutdown response is correlated
   before exit is sent.
4. A successful lifecycle returns the expected minimal initialize result only
   after the fake exits with status 0, the direct child has been reaped, pipe
   workers have finished, and no background task or thread remains detached.
5. A response with an unknown or mismatched ID cannot satisfy the pending
   request. The operation returns a specific correlation/protocol error and
   still cleans up and reaps the direct child.
6. Before responding, a fake mode writes at least 4 MiB to stderr, exceeding the
   configured retained-diagnostic limit. The lifecycle still completes without
   pipe deadlock; retained diagnostics stay within the limit and report
   truncation.
7. Missing or malformed framing, invalid JSON, an oversized declared body,
   unexpected EOF, and an early child exit with any status each produce a
   bounded, non-panicking error that identifies the failed operation and
   includes available bounded child status or diagnostics.
8. A fake that withholds a response or ignores graceful shutdown causes the
   configured bound to expire. The implementation forcibly terminates the
   direct child when needed, waits for it, joins pipe workers, and returns within
   the test's outer deadline.
9. Process-level tests use only the project-owned fake, do not invoke a shell,
   do not contact the network, do not depend on sleep timing for ordering, and
   run the substantive normal, backpressure, protocol-failure, and forced-cleanup
   cases on Linux, macOS, and Windows without platform no-ops or ignored tests.
10. Existing CLI integration tests and bootstrap behavior remain unchanged, and
    no fake-server or lifecycle control is exposed as a user-facing command.
11. Formatting, checking, Clippy, all tests, and an offline build and test pass
    under the pinned toolchain and lockfile. CI has a finite timeout and passes
    the complete test suite on all three operating-system families.
12. `docs/PROJECT.md` resolves Q-005 and Q-006 with evidence-backed decisions,
    and `docs/STATUS.md` records the exact outcome, local and CI evidence,
    deviations, discoveries, and next authorization state.

## Mandatory validation

Run the focused gate first from the repository root and inspect its output to
confirm that it executes nonzero substantive normal, backpressure,
protocol-failure, and forced-cleanup tests:

```text
cargo test --all-targets --locked downstream_lifecycle -- --nocapture
```

Then run the complete local pre-commit gates:

```text
cargo metadata --no-deps --format-version 1
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo build --locked --offline
cargo test --all-targets --locked --offline
cargo run --locked -- --help
cargo run --locked -- --version
```

Run the built binary with a deliberately unknown option and record its nonzero
exit status and useful stderr diagnostic to preserve the S001 boundary.

Before merge, CI must pass on Linux, macOS, and Windows. Inspect every OS job log
and record the count of executed lifecycle tests, including the backpressure and
forced-cleanup cases; a green job in which they were filtered, ignored, or
platform-disabled is not evidence. Record the workflow run URL and the finite
job timeout. If any live OS gate is unavailable, label that platform claim
unverified and do not describe S002 as fully validated.

## Review focus

An independent adversarial review should try to falsify the central promise,
with emphasis on:

- `Content-Length` computed from Rust characters or JSON text length instead of
  UTF-8 bytes;
- a decoder that uses `read_to_end`, assumes one read per message, accepts
  ambiguous lengths, or allocates before checking its limit;
- stdout logging, test-harness output, or stderr bytes contaminating protocol
  framing;
- sequential pipe handling that can block on a full stderr pipe;
- retained diagnostics that grow without bound or silently truncate;
- accepting the next response without checking its JSON-RPC version and ID;
- lifecycle messages sent out of order or `exit` sent before the shutdown
  response;
- a timeout path that drops a child handle, kills without waiting, leaks pipe
  tasks, or reports success after forced cleanup;
- detached threads/tasks, cleanup that depends on destructor timing, or races
  hidden by sleeps and generous CI timing;
- Unix-only signal or path assumptions concealed by Windows no-op tests;
- shell command construction or a test-only fake exposed through the product
  binary;
- a broad runtime, protocol crate, library API, or crate split not justified by
  current evidence;
- claims of LSP conformance, process-tree cleanup, or real-server compatibility
  beyond what the tests demonstrate;
- stale state records or validation claims not backed by current output.

Apply valid findings and rerun every affected focused and broad gate before
handoff.

## Improvement latitude and scope guard

The agent may include low-risk codec cases, error-context improvements, focused
test utilities, and CI timeout corrections discovered while proving the same
bounded lifecycle. It may centralize limits or cleanup ownership when the
implementation already needs one authoritative boundary.

Do not add a generic JSON-RPC router, all LSP message types, restart machinery,
configuration, tracing infrastructure, release tooling, or speculative public
abstractions. Record a concrete opportunity and its expected value in the
`docs/STATUS.md` discovery backlog instead. Escalate if a valuable improvement
would change a frozen decision or materially enlarge the PR.

## Handoff and state update

Before handoff:

1. Resolve Q-005 and Q-006 in `docs/PROJECT.md` and list each dependency added,
   its selected features, why it is needed now, and any standalone-binary or
   MSRV consequence.
2. Update the current snapshot, active working record, stage history,
   deviations, blockers, and discovery backlog in `docs/STATUS.md`.
3. Record the configured bounds and exact local command results, including the
   focused and full test counts, offline evidence, direct invalid-input smoke,
   and live CI URLs and per-OS lifecycle counts.
4. Set `Authorized stage` to `None — planning checkpoint`. S003 must receive a
   separate complete contract, review, and explicit authorization.
5. State explicitly that S002 guarantees direct-child cleanup only and identify
   any unverified descendant process-tree or real-server behavior.
6. Ensure a clean session can reproduce the fake-server evidence and identify
   every remaining decision without the implementation conversation.

If work stops incomplete, do not perform the completion transition. Update the
active-stage working record with the branch, last coherent checkpoint, changed
paths, remaining work, actual check results, failures, and blockers.

## PR boundary

Framing, response correlation, concurrent diagnostic draining, process cleanup,
and the deterministic fake belong in one PR because none independently proves
that the application can complete a bounded downstream lifecycle. The observable
demonstration is the cross-platform fake-server process test, supported by
focused codec and state tests.

Commits may separate protocol primitives from process lifecycle and acceptance
fixtures when each checkpoint remains coherent, but every commit must serve the
same proof. Configuration, public operations, real-server compatibility,
restart policy, descendant process trees, broad protocol routing, and S003
design must be deferred.

## Stopping rule

Stop when every acceptance criterion has current evidence, focused and broad
local gates pass, the substantive lifecycle suite has passed on Linux, macOS,
and Windows or missing evidence is explicitly reported, Q-005 and Q-006 and the
state records are updated, and the S002 PR is ready for maintainer review. Do not
start configuration work, expose an agent-facing operation, draft S003 in
detail, or claim support beyond the direct-child lifecycle proof.
