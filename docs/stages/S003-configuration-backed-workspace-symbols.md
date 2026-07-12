# S003: Configuration-backed workspace symbol flow

This file is the complete contract for the third executable stage. It is
executable only while `docs/STATUS.md` names S003 as the authorized stage;
`docs/STATUS.md` also owns progress.

Adding this contract does not authorize implementation. Before the authorization
transition, the maintainer must accept the proposed configuration and external
operation boundaries below and record decisions resolving Q-004 and Q-007 in
`docs/PROJECT.md`. Q-009 resolves from the implementation evidence required by
this stage.

## Minimum context

Read:

- `AGENTS.md` and `docs/STATUS.md` in full;
- the product definition; all conceptual boundaries and architectural
  principles; decisions D-001 through D-013; questions Q-004, Q-007, and Q-009;
  risks R-001 through R-009; and the incremental strategy in
  `docs/PROJECT.md`;
- the Base Protocol, lifecycle messages, initialize parameters and capabilities,
  URI, Position, Range, Location, SymbolInformation, WorkspaceSymbol,
  `workspace/symbol`, `workspace/workspaceFolders`, and `window/logMessage`
  sections of the official [LSP 3.18 specification][lsp-3-18], limited to the
  behavior named by this contract;
- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `src/`, `tests/`,
  `.github/workflows/ci.yml`, and the S002 outcome and revision evidence in
  `docs/STATUS.md`.

No external conversation, installed language server, credentials, MangoStudio
checkout, or unpublished plan is required.

[lsp-3-18]: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/

## Local problem

S002 proves that an explicitly constructed direct child can complete a bounded
initialize-to-exit lifecycle, including failure cleanup. The product CLI still
does nothing beyond bootstrap argument handling. It cannot load a server
definition, establish a workspace, run an agent-useful request, distinguish
machine output from operational logs, or report a structured failure.

The current lifecycle is also intentionally closed over one hard-coded
interaction: it shuts down immediately after `initialized` and expects the next
frame to be the pending response. The first product flow must extend that proof
without jumping to generic protocol dispatch, document synchronization, multiple
servers, automatic configuration discovery, or a permanent service schema.

## Objective and central promise

Deliver one bounded, configuration-backed CLI operation:

```text
mango-lsp workspace-symbols --config <FILE> --workspace <DIR> --query <TEXT>
```

Given an explicitly selected versioned TOML file describing one server, the
command validates all inputs before launch, starts that server directly in the
requested workspace, performs the LSP initialize and `workspace/symbol`
sequence, normalizes the response into a versioned JSON result on stdout, emits
versioned JSON Lines lifecycle events on stderr, and returns only after graceful
shutdown or bounded forced cleanup has reaped the direct child.

The PR's central demonstration is:

> On Linux, macOS, and Windows, the real `mango-lsp` binary loads an explicit
> one-server configuration, queries a deterministic fake workspace symbol,
> emits one normalized JSON result and separate redacted lifecycle events, and
> leaves no direct child running on success or failure.

This is the first public CLI data boundary, but it is deliberately one operation
and one configured server. It does not expose raw downstream LSP or stabilize a
daemon transport, Rust library API, multi-server policy, or generated client.

## Scope

### Included

- A `workspace-symbols` subcommand with required `--config`, `--workspace`, and
  `--query` options. The query must be valid UTF-8, non-empty, and bounded before
  it is sent to a child.
- This minimal TOML configuration schema:

  ```toml
  schema_version = 1

  [server]
  id = "fixture"
  command = "/absolute/path/to/language-server"
  args = ["--stdio"]
  ```

  `args` may be omitted and defaults to an empty list. Every other field is
  required. Unknown fields, duplicate fields, an unsupported schema version,
  an invalid server ID, an empty command, excessive file/string/collection
  sizes, and malformed TOML are errors. Empty literal arguments remain valid.
- Explicit configuration selection only. S003 performs no filename search,
  parent-directory walk, environment-variable override, layer merge, or
  fallback. Relative `--config` and `--workspace` values are resolved from the
  caller's current directory.
- Direct command semantics. `command` is one executable token, `args` are
  literal argument tokens, and neither is parsed by a shell or subject to
  quoting, interpolation, globbing, variable expansion, or response-file
  expansion by mango-lsp. A relative command containing a path separator is
  resolved against the configuration file's directory; a bare command name is
  resolved by the operating system using the inherited process environment.
- A validated, canonical existing workspace directory used as the child working
  directory and encoded as one file URI for LSP initialization. The initialize
  request supplies the current mango-lsp process ID, `rootUri`, one
  `workspaceFolders` entry, and only the client capabilities actually supported
  by this flow.
- Parent environment inheritance for the child, with no configuration fields or
  CLI options for adding, replacing, removing, or interpolating environment
  variables in this stage. Configuration content is treated as trusted only for
  the explicitly selected invocation and remains untrusted for parsing and
  process construction.
- Static capability inspection. The flow sends `workspace/symbol` only when the
  initialize result advertises `workspaceSymbolProvider` as supported. It does
  not advertise dynamic registration, partial results, or workspace-symbol
  resolve support.
- One in-flight `workspace/symbol` request with the exact validated query, a
  distinct correlated ID, and the existing finite operation bound. The narrow
  receive loop handles `window/logMessage` while waiting for initialize or the
  operation response and, after initialization, handles the specifically
  supported `workspace/workspaceFolders` request. Neither may satisfy or
  displace a pending response.
- Typed validation for the complete response forms supported here:
  `SymbolInformation[]`, `WorkspaceSymbol[]`, or `null`. Because resolve support
  is not advertised, every returned symbol must contain a location with a
  concrete range. Partial results and unresolved locations are rejected.
- A smaller normalized symbol representation containing `name`, a documented
  lower-snake-case `kind`, nullable `container_name`, and a location with the
  server-provided URI plus zero-based start/end line and character positions.
  Server order is preserved. Tags, arbitrary `data`, deprecated fields, partial
  result tokens, and other downstream payload are not exposed.
- Exactly one compact JSON object followed by one newline on stdout for every
  successfully parsed `workspace-symbols` invocation, including configuration,
  workspace, spawn, protocol, timeout, and cleanup failures. The version 1
  envelope is:

  ```json
  {
    "schema_version": 1,
    "operation": "workspace_symbols",
    "status": "ok",
    "server": "fixture",
    "result": {
      "symbols": [
        {
          "name": "Widget",
          "kind": "class",
          "container_name": null,
          "location": {
            "uri": "file:///workspace/src/widget.rs",
            "range": {
              "start": { "line": 0, "character": 0 },
              "end": { "line": 0, "character": 6 }
            }
          }
        }
      ]
    },
    "error": null
  }
  ```

  A failure uses `status: "error"`, `result: null`, and an error object with a
  bounded stable `kind`, a useful human-readable `message`, and `cleanup` set to
  `not_required`, `completed`, or `failed`. Error kinds are exactly
  `configuration`, `workspace`, `spawn`, `unsupported_capability`, `protocol`,
  `downstream`, `timeout`, `cleanup`, or `output`. `server` is null when no valid
  server ID was available. No raw JSON-RPC object appears in this envelope.
  Required version 1 fields and their meanings are stable; additive fields may
  be introduced only when consumers can safely ignore them. A post-spawn error
  envelope is not emitted until its cleanup attempt finishes, so `cleanup`
  reports observed state rather than intent.
- Exit status 0 only after a successful result and reaped child; status 2 for a
  parsed invocation rejected at the CLI/configuration/workspace boundary; and
  status 1 for downstream, protocol, timeout, output, or cleanup failure.
  Clap-owned help, version, and syntax errors retain their existing behavior and
  are not wrapped in the JSON envelope.
- Version 1 JSON Lines events on stderr. Every line is one compact JSON object
  with at least `schema_version`, `level`, `event`, and `operation`; events after
  valid configuration also include the server ID. The bounded event names are
  `operation_started`, `child_started`, `downstream_notification`,
  `child_stopped`, `operation_succeeded`, and `operation_failed`. A successful
  terminal event includes symbol count. Child stop includes observed and
  retained diagnostic byte counts plus truncation status. A failure terminal
  event includes the same error kind and cleanup state as the result envelope.
  Required version 1 fields and meanings are stable; additive fields must be
  safe for consumers to ignore.
- A default redaction boundary: never log the query, command, arguments,
  environment, configuration contents, raw JSON-RPC payloads, raw child stderr,
  or raw `window/logMessage` text. Explicit config/workspace paths may appear in
  a boundary error that identifies an unreadable or invalid user-selected path.
  Downstream diagnostics and log notifications are represented only by bounded
  counts, truncation metadata, source, and severity.
- Normal specification-ordered `shutdown`/response/`exit`, with the stdout
  success envelope emitted only after the child is reaped and pipe workers are
  joined. Every error path retains the S002 direct-child termination, reap,
  bounded diagnostic drain, and useful error identity guarantees.
- Extensions to the project-owned fake server for workspace initialization,
  capability declaration, an interleaved supported notification/request, one
  deterministic symbol result, unsupported capability, malformed result,
  JSON-RPC error, diagnostic-redaction sentinel, stall, and shutdown failure
  modes.
- Unit and real-binary integration tests. All process-level S003 tests share the
  `vertical_flow` prefix, use temporary explicit configuration/workspace inputs,
  and make the fake the deterministic offline acceptance oracle.
- Contributor documentation, accepted-decision records, state transitions, and
  any finite CI invocation or timeout changes needed to make the vertical gate
  visible on Linux, macOS, and Windows.

### Excluded

- Implicit configuration discovery, conventional default filenames, multiple
  files or profiles, merge precedence, include directives, remote configuration,
  environment interpolation, command strings interpreted by a shell, secrets,
  initialization options, or per-server environment mutation.
- Multiple configured or running servers, server selection, routing,
  aggregation, restart, health checks, pooling, resource fairness, background
  persistence, or a long-running supervisor.
- Document open/change/close synchronization, hover, definitions, references,
  diagnostics, completion, file watching, workspace edits, or access to symbol
  locations on disk.
- Generic JSON-RPC dispatch, multiple concurrent client requests, cancellation,
  progress or partial-result tokens, dynamic registration, workspace-symbol
  resolve, arbitrary server-to-client requests, or arbitrary notifications.
- Raw LSP passthrough, a generic request CLI, full SymbolKind or LSP 3.18
  conformance claims, generated schemas or TypeScript clients, MangoStudio
  integration, sockets, named pipes, or a daemon protocol.
- User-selectable output or log formats, pretty printing, color, timestamps,
  verbosity controls, persistence, telemetry, tracing spans, or emission of raw
  downstream diagnostics. These may be reconsidered after the first evidence.
- Descendant process-tree containment or termination. S003 preserves S002's
  direct-child-only guarantee and must not broaden that claim without explicit
  platform design and evidence.
- A real language server as a required oracle, compatibility claims for a named
  server, public-network access in tests, release packaging, publication,
  licensing, or changelog work.
- A stable public Rust library API, speculative crate split, async runtime, or
  native dependency added only for anticipated later concurrency.

## Preconditions and dependencies

- S002 is present on `main` as squash commit `7720278`, followed by the
  failure-path correction `9f5692e` from PR #4. The revised three-operating-system
  evidence is recorded in `docs/STATUS.md`.
- This contract has been reviewed by the maintainer, Q-004 and Q-007 have
  accepted decisions in `docs/PROJECT.md` matching the versioned TOML and
  `workspace-symbols` boundaries above, and `docs/STATUS.md` explicitly names
  S003 as the authorized stage and links this file.
- Work begins on a writable S003 implementation branch with unrelated
  user-owned changes preserved.
- The pinned Rust 1.97.0 toolchain and the Linux, macOS, and Windows CI matrix
  remain available.
- Registry access may fetch reviewed pure-Rust build dependencies. The built
  product and complete test suite must run offline after dependencies are
  fetched, without another runtime or installed language server.

If a precondition is false, record the mismatch in the active-stage working
record before requesting the narrowest maintainer action. Do not silently alter
the public schema, relax redaction, bypass the explicit configuration boundary,
or weaken a platform or cleanup claim.

## Frozen decisions

- D-001 through D-013 remain in force. In particular, the CLI remains
  independent of MangoStudio, downstream transport remains STDIO, process
  ownership remains standard-library based, and the project-owned protocol
  subset expands only as required by this flow.
- When S003 is authorized, the command name, required option names, TOML
  `schema_version = 1` one-server shape, no-discovery rule, direct command
  semantics, version 1 result envelope, JSON Lines stderr boundary, and default
  redaction rules in this contract are stage-frozen public behavior.
- Configuration selection is explicit consent to launch that file's one
  command for this invocation. It is not consent to discover another file,
  interpret shell syntax, expand values, execute more than one server, or retain
  the command after exit.
- Stdout contains only the single result envelope for a parsed operation.
  Mango-lsp logs and child stderr never appear there. Stderr contains only
  mango-lsp's JSON Lines events for this subcommand; downstream protocol stdout
  and raw stderr are never forwarded.
- The workspace is one existing local directory. The flow neither reads its
  files nor accepts a prebuilt URI from the caller. URI construction must be
  correct for Unix paths, Windows drive paths, spaces, and non-ASCII text.
- The client advertises only capabilities it implements. A missing or false
  static workspace-symbol capability is a structured unsupported-capability
  failure, not permission to send the request optimistically.
- LSP request IDs remain correlated. Supported interleaved server messages are
  handled by method and cannot satisfy the pending operation. An unsupported
  request or notification causes a bounded protocol failure and cleanup; it
  does not trigger speculative generic behavior.
- A normal success uses graceful lifecycle ordering and is not observable on
  stdout before reap. A failure may force-kill the direct child, but completion
  still requires bounded cleanup and must report whether cleanup succeeded.
- The fake remains test infrastructure only and must not become a product
  subcommand or intended release artifact.
- Existing `--help`, `--version`, and unknown-option behavior remain compatible.
  Do not claim multi-server supervision, broad LSP support, real-server
  compatibility, or descendant cleanup from this stage.

## Implementation discretion

The implementing agent may choose:

- internal module, type, and error boundaries, including how to reshape the
  S002 hard-coded lifecycle into a reusable one-operation session. Integration
  test visibility does not make the Rust API stable;
- a maintained pure-Rust TOML parser and, if evidence justifies it, a focused
  path/URI conversion dependency compatible with D-002 and D-010. Compare
  maintenance, transitive surface, unsafe/native/build-script use, license, and
  locked offline behavior before selection;
- exact finite limits for configuration bytes, string lengths, argument count,
  query bytes, event messages, and returned symbols, provided defaults are
  documented at their owning boundaries and smaller limits are injectable in
  tests;
- internal symbol types and the mapping from the LSP SymbolKind range to the
  public lower-snake-case names, provided all advertised/accepted values have a
  deterministic mapping and unknown values cannot leak as raw payload;
- how the narrow receive loop represents the pending response, supported
  notification, and supported workspace-folders request, provided correlation
  and message ordering remain explicit and independently tested;
- how to attempt graceful shutdown after a post-initialize application error
  before falling back to the existing forced-cleanup path, provided the total
  behavior stays finite and preserves the original error identity;
- compact JSON serialization and event-writing helpers, provided stdout/stderr
  separation, one-result semantics, required fields, and redaction are enforced
  at one authoritative boundary;
- whether integration tests construct TOML text directly or through a named
  fixture helper, provided temporary paths are cross-platform and no absolute
  developer path is committed;
- CI timeout adjustments supported by observed execution time. Every new bound
  must remain finite and substantive tests must not be filtered or disabled on
  any supported operating system.

Do not add a broad logging/tracing framework, async runtime, LSP type universe,
or configuration abstraction merely to avoid writing the small boundaries this
stage actually exercises.

## Escalation and recording triggers

Ask the maintainer, with a recommendation, before:

- changing the subcommand, TOML schema, discovery/precedence rule, command
  resolution, stdout envelope, JSON Lines fields, exit-status contract, or
  redaction policy specified here;
- adding automatic or remote configuration, shell execution, environment
  expansion/mutation, secret storage, initialization options, file reads, or
  execution of more than one configured command;
- exposing raw LSP, raw child stderr, raw server log text, command arguments, the
  query, or environment content in a public output or log;
- adding a generic server-message dispatcher, another agent-facing operation,
  multi-server routing, a daemon transport, a stable Rust API, a generated
  client, or MangoStudio coupling;
- adding unsafe code, a native dependency, a build script, runtime companion,
  async runtime, or durable multi-crate workspace;
- changing direct-child cleanup into descendant process-tree policy, detaching a
  child, or adopting platform-specific process guarantees;
- dropping or weakening Linux, macOS, or Windows vertical-flow evidence, or
  making an installed server or network service a mandatory gate.

The authorization change must resolve Q-004 and Q-007 as accepted decisions.
The implementation must resolve Q-009 with evidence for the actual stream,
schema, and redaction behavior. Record newly discovered cross-stage risks or
questions in `docs/PROJECT.md`, valuable excluded work in the discovery backlog
in `docs/STATUS.md`, and material deviations in the S003 history entry.

## Deliverables

The coherent change must provide:

- the `workspace-symbols` CLI boundary and documented version 1 TOML example;
- bounded explicit configuration loading, strict validation, direct command
  construction, workspace validation, and cross-platform file-URI conversion;
- a one-operation downstream session that initializes the configured workspace,
  checks static capability support, handles the two supported interleaved server
  messages, correlates `workspace/symbol`, and shuts down and reaps;
- typed workspace-symbol response validation and the normalized public symbol
  representation;
- one authoritative version 1 result-envelope writer and one authoritative
  redacted JSON Lines event writer with correct stdout/stderr ownership;
- stable bounded error kinds and documented exit statuses for parsed operations;
- fake-server modes and focused unit/process tests for success, interleaving,
  unsupported capability, invalid config/workspace, spawn failure, response
  error, malformed result, redaction, timeout, cleanup, and output separation;
- regression coverage proving the S002 lifecycle and CLI bootstrap behavior are
  preserved;
- a visible finite three-OS CI gate for substantive `vertical_flow` tests;
- dependency and lockfile changes justified by current needs, if any;
- Q-009's accepted decision, contributor guidance, and complete execution-state
  updates without generated output, scratch files, or shipped fake behavior.

## Acceptance criteria

1. `mango-lsp workspace-symbols --help` documents exactly the required
   `--config`, `--workspace`, and `--query` operation inputs. Existing top-level
   help/version and unknown-option behavior remain useful and non-panicking.
2. Only the explicitly supplied TOML file is read. A missing file, malformed or
   oversized TOML, wrong schema version, unknown/duplicate field, invalid ID,
   empty command, or exceeded configured bound fails before any child launch.
   An empty literal argument remains valid. No default file, parent path, or
   environment variable is consulted.
3. After successful Clap parsing, every configuration or workspace rejection
   emits exactly one valid version 1 error envelope on stdout, one or more valid
   redacted JSON Lines events on stderr, and exit status 2. A sentinel placed in
   command arguments or unrelated environment values is absent from both
   streams.
4. The selected command is spawned directly without a shell. Literal arguments
   arrive unchanged, the child's current directory is the canonical workspace,
   config-relative executable paths and bare command names follow the documented
   distinct rules, and config cannot modify the inherited environment.
5. The fake observes an initialize request first with mango-lsp's real process
   ID, a correctly encoded workspace `rootUri`, one matching workspace-folder
   URI/name, and no capabilities for dynamic registration, partial results, or
   workspace-symbol resolve. Paths containing spaces and non-ASCII characters,
   plus a Windows drive path in Windows CI, produce valid consistent file URIs.
6. If `workspaceSymbolProvider` is absent or false, mango-lsp does not send
   `workspace/symbol`. It emits an `unsupported_capability` error envelope,
   performs bounded cleanup, and exits 1.
7. In the normal fake flow, a `window/logMessage` notification arrives before
   the initialize response and a `workspace/workspaceFolders` request arrives
   while mango-lsp awaits the operation response. Mango-lsp records only
   redacted notification metadata, returns the configured single folder to the
   server request, sends exactly one `workspace/symbol` request with the
   caller's query and a distinct ID, and accepts only the correlated responses.
8. A valid `SymbolInformation[]` or fully located `WorkspaceSymbol[]` response
   becomes the documented normalized representation. The result preserves
   server order, maps SymbolKind deterministically, retains URI and zero-based
   range coordinates, represents an absent container as null, and excludes raw
   JSON-RPC, tags, arbitrary data, and other uncommitted fields. A null LSP result
   becomes an empty `symbols` array.
9. A JSON-RPC error, mismatched ID, unsupported message, non-array/non-null
   result, missing or invalid symbol field, unresolved location, invalid range,
   oversized response, unexpected EOF, early exit, or stalled response produces
   one bounded version 1 error envelope, exit status 1, and direct-child cleanup.
   No malformed downstream value is silently dropped to turn failure into
   success.
10. A successful parsed invocation writes exactly one compact JSON object plus
    newline to stdout and no other bytes. Its required fields and normalized
    symbol content match the version 1 schema, it exits 0, and the envelope is
    not emitted until graceful shutdown has completed, the direct child is
    reaped, and pipe workers are joined.
11. Every stderr line from a parsed invocation is independent valid JSON with
    the required versioned event fields. Success events identify operation and
    child start/completion/stop; failure events identify the bounded error kind
    and cleanup outcome. Event ordering reflects actual lifecycle ordering and
    contains no human prose outside the JSON records.
12. A fake places distinct secret sentinels in its stderr and
    `window/logMessage` text while the configuration contains sentinels in an
    argument. The flow completes or fails as selected, retained byte/truncation
    counts remain useful, and none of the sentinel values appears in stdout or
    stderr. Raw server stdout is never forwarded.
13. Spawn failure, operation timeout, shutdown refusal, output-write failure
    where deterministically testable, and every post-spawn protocol error in the
    direct-child fake modes retain the useful primary error identity, report
    cleanup status, preserve bounded diagnostic metadata, terminate the direct
    child when needed, wait/reap, join workers, and return within the test's
    outer deadline. Descendant-inherited pipes remain outside this claim.
14. Process-level tests invoke the real product binary with temporary config and
    workspace paths, use only the project-owned fake, perform no shell or network
    access, use synchronization through protocol messages rather than sleep
    timing, and run substantive success, redaction, protocol-failure, timeout,
    and cleanup cases on Linux, macOS, and Windows without ignored tests or
    platform no-ops.
15. Formatting, checking, Clippy, all tests, and offline build/test pass under
    the pinned toolchain and lockfile. CI has finite timeouts and passes the
    complete suite on all three operating-system families.
16. `docs/PROJECT.md` contains accepted Q-004, Q-007, and Q-009 decisions matching
    shipped behavior, and `docs/STATUS.md` records the exact outcome, local and
    CI evidence, limits, deviations, discoveries, residual direct-child-only
    guarantee, and next authorization state.

## Mandatory validation

Run the focused gate first from the repository root and inspect its output to
confirm that it executes nonzero substantive real-binary success, redaction,
protocol-failure, timeout, and cleanup tests:

```text
cargo test --all-targets --locked vertical_flow -- --nocapture
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
cargo run --locked -- workspace-symbols --help
```

Run the built binary with a deliberately unknown top-level option and record its
nonzero exit status and useful stderr diagnostic. Inspect at least one successful
and one failing captured process result from the integration suite to confirm
that stdout contains exactly one valid envelope, every stderr line parses as a
valid event, exit statuses match the contract, sentinels are absent, and the
success result was emitted after the fake completed shutdown.

Before merge, CI must pass on Linux, macOS, and Windows. Inspect every OS job log
and record the count of executed `vertical_flow` tests, including success,
redaction, protocol-failure, timeout, and cleanup cases; a green job in which
they were filtered, ignored, or platform-disabled is not evidence. Record the
workflow run URL and finite job timeouts. If any live OS gate is unavailable,
label that platform claim unverified and do not describe S003 as fully
validated.

## Review focus

An independent adversarial review should try to falsify the central promise,
with emphasis on:

- a parser that accepts unknown/duplicate fields, reads an implicit config,
  merges environment values, or launches before complete validation;
- a command string split or passed through a shell, interpolation of arguments,
  config-relative paths resolved from the workspace, or undocumented PATH/CWD
  differences across operating systems;
- workspace file URIs that mishandle Windows drive letters, separators, spaces,
  percent encoding, non-ASCII text, canonicalization, or workspace-folder
  consistency;
- advertising capabilities mango-lsp does not implement, ignoring a false
  server capability, sending the operation before `initialized`, or accepting
  an unresolved workspace symbol without resolve support;
- a receive loop that treats the next frame as the response, loses the pending
  ID when a notification/request interleaves, accepts a mismatched response, or
  deadlocks on an unsupported server message;
- passing raw LSP through, silently dropping malformed symbols, leaking unknown
  fields, exposing numeric/downstream details outside the documented mapping, or
  changing result order without justification;
- help, logs, child stdout/stderr, panic output, or diagnostics contaminating the
  one-object stdout contract; multiple envelopes; success emitted before child
  reap; or an error path that exits 0;
- query, command, argument, environment, config content, raw child stderr, raw
  `window/logMessage`, protocol payload, or secret sentinels appearing in result
  or event streams;
- unbounded config, query, result, diagnostic, error-message, or event retention;
  wall-clock sleeps hiding races; or an output failure that abandons cleanup;
- regression of the revised S002 failure path by joining pipe workers before
  termination/reap, overwriting real diagnostic metadata, or waiting forever on
  a live child or inherited descendant pipe;
- Unix-only fake paths, executable suffixes, signals, process checks, or URI
  assumptions concealed by Windows/macOS skips;
- a TOML/URI/logging/runtime dependency whose native code, build script, unsafe
  use, license, MSRV, or transitive cost conflicts with accepted decisions;
- stale decision/state records or claims of discovery, routing, LSP conformance,
  real-server compatibility, descendant cleanup, or schema coverage beyond the
  tests.

Apply valid findings and rerun every affected focused and broad gate before
handoff.

## Improvement latitude and scope guard

The agent may include low-risk strict-validation cases, more precise bounded
error kinds, small cross-platform URI fixtures, lifecycle-state simplification,
or event-writer centralization discovered while proving this exact flow. It may
improve the fake and shared process-test helpers when that directly strengthens
the same success, redaction, or cleanup invariant.

Do not add config discovery, multiple servers, another operation, file reads,
raw log modes, generic dispatch, restart machinery, tracing infrastructure,
schema generation, release tooling, or speculative public abstractions. Record
a concrete opportunity and its expected value in the `docs/STATUS.md` discovery
backlog instead. Escalate if a valuable improvement would change a frozen
boundary or materially enlarge the PR.

## Handoff and state update

Before handoff:

1. Confirm the authorization-time decisions resolving Q-004 and Q-007 match the
   implementation. Resolve Q-009 with the exact result/event stream ownership,
   schema, redaction, and compatibility consequences.
2. List every dependency added, selected features, why it is needed now, and any
   standalone-binary, unsafe/native/build-script, license, transitive, offline,
   or MSRV consequence.
3. Update the current snapshot, active working record, stage history,
   deviations, blockers, and discovery backlog in `docs/STATUS.md`.
4. Record the exact config/query/result/event/diagnostic/time bounds and local
   command results, including focused and full test counts, offline evidence,
   direct CLI smokes, and live CI URLs with per-OS `vertical_flow` counts.
5. Set `Authorized stage` to `None — planning checkpoint`. The post-vertical
   checkpoint must revise later work from evidence; no subsequent stage is
   implicitly authorized.
6. State explicitly that S003 proves one explicitly configured direct child and
   one workspace-symbol operation only. Identify unverified real-server,
   multi-server, descendant-process, discovery, document-sync, and integration
   behavior.
7. Ensure a clean session can reproduce the fake-backed CLI evidence and audit
   every public field and remaining decision without implementation chat.

If work stops incomplete, do not perform the completion transition. Update the
active-stage working record with the branch, last coherent checkpoint, changed
paths, remaining work, actual checks and captured stream evidence, failures, and
blockers.

## PR boundary

Configuration parsing, workspace construction, the one downstream operation,
normalized output, redacted events, and real-binary fake acceptance belong in
one PR because none alone demonstrates a useful, observable product flow. The
independent demonstration is one explicit CLI invocation whose result and logs
can be parsed, whose fake transcript can be inspected, and whose child is reaped
on all tested paths.

Commits may separate the strict configuration/public-envelope boundary from the
downstream operation and process-level acceptance fixtures when each checkpoint
remains coherent, but every commit must serve the same vertical promise.
Discovery, additional operations, multiple servers, general dispatch,
real-server compatibility, integration clients, and packaging must be deferred.

## Stopping rule

Stop when every acceptance criterion has current evidence, focused and broad
local gates pass, substantive `vertical_flow` tests pass on Linux, macOS, and
Windows or missing evidence is explicitly reported, Q-004/Q-007/Q-009 and state
records match the shipped boundary, and the S003 PR is ready for maintainer
review. Do not start the post-vertical checkpoint, add another operation, or
claim support beyond this explicit one-server workspace-symbol flow.
