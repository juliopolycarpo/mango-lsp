# Repository instructions

## Authority and scope

These instructions apply to every change in this repository. A maintainer's
explicit current request may change the authorized work, but the same change
must update `docs/STATUS.md` so the repository remains sufficient for the next
session. Chat history, an issue, or a roadmap item is not a substitute for that
state update.

Work on one implementation stage at a time. Do not run parallel agents that
modify the repository. A separate, read-only review in a clean context is useful
when risk warrants it, but its findings must be reconciled by the stage owner.

English is the default for source, documentation, tests, fixtures, configuration,
scripts, user-facing text, commits, and PRs. A necessary non-English fixture must
document why it exists in English.

## Required session intake

Before editing:

1. Run `git status --short` and preserve user-owned changes.
2. Read `docs/STATUS.md` in full.
3. Read the authorized stage contract in full.
4. Read the project sections and decision records named by that contract.
5. Check the nearby code, tests, scripts, and repository conventions affected by
   the stage.
6. Verify the stage preconditions. Record a material mismatch before proceeding.

If no stage is authorized, limit work to planning, diagnosis, or review explicitly
requested by the maintainer. Do not begin the next roadmap item implicitly.

## Executing a stage

Treat the stage contract as an outcome contract, not a mechanical recipe. Honor
its central promise, included and excluded scope, frozen decisions, acceptance
criteria, mandatory validations, and stopping rule. Within those boundaries,
choose the clearest solution supported by repository evidence.

Implement adjacent, low-risk improvements only when they directly strengthen the
stage promise. Record a valuable but out-of-scope opportunity in the discovery
backlog in `docs/STATUS.md`; do not hide it in the diff. Escalate before changing
a public interface, security model, data format, accepted product decision,
dependency strategy with lasting impact, or other boundary identified by the
stage.

Prefer deterministic tests and project-owned fakes at external process and
network boundaries. Unit and integration tests must not contact production
services, depend on public network access, use real credentials, or rely on an
uncontrolled language server. A real-server smoke test may supplement, but not
replace, deterministic acceptance evidence.

Do not use compilation as a reason to add stubs, silent fallbacks, skipped tests,
or placeholder behavior. Correct the implementation or record a blocker.

## State and decision recording

`docs/STATUS.md` is the only source of truth for authorization and progress.
While a stage is incomplete, keep its working record resumable: branch or
worktree, last coherent checkpoint, remaining work, validation evidence, and
blockers. Do not claim a validation passed without command output from the
current implementation.

Record durable product or architecture choices in the decision registry in
`docs/PROJECT.md`. Add a new record when superseding a decision instead of
silently rewriting history. Keep unresolved choices in its open-question
registry. Keep stage-local reasoning in the stage contract or PR rather than
turning every implementation detail into a project decision.

At completion, update `docs/STATUS.md` in the implementation PR with the stage
outcome and evidence. Set the next authorized stage to `None` unless its complete
contract has been separately reviewed and explicitly authorized. Do not author
and start the next stage as part of finishing the current one.

## Validation and review

Run the stage's targeted validations first, followed by the broader gates its
risk justifies. Before handoff:

1. Inspect `git status --short` and the complete diff.
2. Confirm every acceptance criterion using recorded evidence.
3. Check for unrelated changes, generated debris, secrets, unsafe process or
   protocol handling, platform assumptions, missing tests, and stale links.
4. Review the diff from the perspective of an unfamiliar maintainer and, for
   protocol or process-lifecycle changes, seek an independent adversarial review
   when available.
5. Apply valid findings and rerun affected validations.

If a required check cannot run, state the reason and the strongest remaining
evidence. An unavailable mandatory gate is not a pass.

## Commits and pull requests

Use a writable, non-protected branch for implementation work. Preserve a linear
history and never include unrelated user changes.

Every commit must:

- follow Conventional Commits;
- have an imperative subject of at most 72 characters;
- include a useful body explaining what and why;
- use DCO and GPG signing through `git commit -s -S`.

A stage may use several logical commits, but one PR must deliver one coherent,
independently observable promise. Do not split by file or internal layer when the
pieces cannot be verified independently. Conversely, defer changes that do not
serve the central promise.

Before opening a PR, obtain any authorization required by the maintainer or
repository ownership rules. Its description must state:

- the central promise and observable demonstration;
- included scope and explicit deferrals;
- validation commands and results, including platform coverage;
- decisions, discoveries, risks, and deviations;
- the resulting `docs/STATUS.md` transition.

Never merge, release, publish, or create other consequential external effects
without explicit authorization. Do not edit a generated changelog manually.

## Handoff and stopping rule

End a stage with a self-contained handoff containing changed files and behavior,
commit and PR references when present, exact validation results, remaining risks
or unverified claims, and worthwhile deferred opportunities. Ensure the working
record in `docs/STATUS.md` lets a clean session resume without chat history.

Stop after the authorized stage is implemented, validated, reviewed, and handed
off. Do not start the next roadmap item.
