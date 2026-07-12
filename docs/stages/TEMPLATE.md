# SNNN: Outcome-oriented stage title

This file is a stage contract, not a progress tracker. `docs/STATUS.md` alone
controls whether the stage is authorized and records its live progress. Copy this
template only when the next stage is ready to be specified, replace every
placeholder with repository-specific content, and delete sections that truly do
not apply.

## Minimum context

List the smallest set of project sections, decisions, code, tests, and prior
evidence the implementer must read. Do not require every project document by
default.

## Local problem

Describe the concrete missing or incorrect behavior and the evidence that makes
this stage timely.

## Objective and central promise

State one observable outcome in language suitable for the PR title and opening
paragraph. Explain how a reviewer can demonstrate it independently.

## Scope

### Included

Name the behavior and supporting artifacts that jointly deliver the promise.

### Excluded

Name tempting adjacent work that must be deferred. Explain any boundary that
could otherwise be interpreted several ways.

## Preconditions and dependencies

List required prior stage outcomes, repository state, tools, fixtures, or human
decisions. Say how to respond if a precondition is false.

## Frozen decisions

Reference decision IDs from `docs/PROJECT.md`; do not copy their full text. Add
stage-specific constraints that the implementer may not change silently.

## Implementation discretion

Identify choices the implementer may make after inspecting evidence. Give
evaluation criteria rather than prescribing libraries or internal shapes without
a reason.

## Escalation and recording triggers

List choices that need maintainer input and discoveries that must become project
decisions, risks, deviations, or backlog entries. Include a recommendation when
escalating.

## Deliverables

List the code, tests, documentation, fixtures, and state updates needed for the
central promise. Avoid a file-by-file checklist unless paths are part of the
contract.

## Acceptance criteria

Use behavioral, falsifiable statements. Include failure behavior and relevant
platforms, not only a successful compile.

## Mandatory validation

Provide exact local commands or procedures and separate pre-commit gates from
CI or pre-merge gates. State what evidence to capture. Do not make an
uncontrolled external service a mandatory oracle.

## Review focus

Identify likely false positives, shortcuts, platform assumptions, security
boundaries, and failure modes for an independent or adversarial reviewer.

## Improvement latitude and scope guard

Define related low-risk improvements that may be included and where to record
valuable work that would expand the stage. Thinking critically is required;
silent scope expansion is not allowed.

## Handoff and state update

Specify which `docs/STATUS.md` fields, decisions, risks, discoveries, and evidence
must change. A clean session must be able to resume an incomplete stage or audit
a completed one without chat history.

## PR boundary

Explain why these deliverables form one independently verifiable change, which
changes belong in its commits, and which changes must be deferred. State the PR's
central demonstration.

## Stopping rule

Define the exact point at which the agent must hand off and stop. Never authorize
the following stage implicitly.
