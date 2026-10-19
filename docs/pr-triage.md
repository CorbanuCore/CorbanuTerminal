# Pull request triage — 2026-10-19

**Recommended actions: merge 3 documentation PRs, resolve blockers on 10 PRs,
and close 2 superseded or unavailable additions.** The table below covers all
15 PR numbers in the requested review; the request's count of 14 was incorrect.
Other open PRs are outside this review.

Reviewed against main `78dd811d6e3718140c938ad4839968d9995fd5ac`.
Final review on 2026-10-19 UTC reconfirmed all 15 heads and both target refs.
PR #130 targets `fix/vercel-server-state-sticky-fallback-main`, whose live head
was `bf7e27e29bc1eeff5b3a1e8a31b0aa611ecb89cc`; its recommendation uses that target.

These recommendations describe the recorded heads and targets. Recheck changed
heads before acting. The document does not itself merge, close or submit a
GitHub review on any PR.

- **merge-ready:** no blocking issue found in the narrow documentation change;
  normal maintainer review and repository merge controls still apply.
- **needs-changes:** a source, integration or validation blocker remains.
  For #101 and #103 this is specifically a validation hold: no defect in the
  updated dependency is established, and their sampled CI failures are inherited
  or infrastructural.
- **close:** no useful merge remains for the proposed change in its current form.
  Closure does not erase authorship or preclude a future, newly scoped change.

| PR                                                              | Recommendation | One-line reason                                                                                                                                         |
| --------------------------------------------------------------- | -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [#28](https://github.com/CorbanuCore/CorbanuTerminal/pull/28)   | needs-changes  | The billing draft still has an explicit do-not-merge gate and conflicts in eight paths against current main.                                            |
| [#38](https://github.com/CorbanuCore/CorbanuTerminal/pull/38)   | needs-changes  | Starting --relink saves terminal_token: None before replacement authorization succeeds, discarding the working session; five paths also conflict.       |
| [#88](https://github.com/CorbanuCore/CorbanuTerminal/pull/88)   | needs-changes  | The quarantined OAuth/payload draft mixes installer changes and conflicts in eight paths; separate and rebase it before focused qualification.          |
| [#94](https://github.com/CorbanuCore/CorbanuTerminal/pull/94)   | needs-changes  | The 1,095-file upstream sync conflicts in 118 paths against current main and still has failed required CI.                                              |
| [#101](https://github.com/CorbanuCore/CorbanuTerminal/pull/101) | needs-changes  | The semver metadata is present; failed CI exposes stale path-crate facts, so rebase onto current main and rerun required checks.                        |
| [#102](https://github.com/CorbanuCore/CorbanuTerminal/pull/102) | needs-changes  | SDK 0.32.1 is mixed with OpenTelemetry API/exporters 0.31, producing incompatible Context and LoggerProvider types in codex-otel.                       |
| [#103](https://github.com/CorbanuCore/CorbanuTerminal/pull/103) | needs-changes  | The Bazel lock repair is present, but required CI remains failed; refresh against current main and obtain passing validation.                           |
| [#104](https://github.com/CorbanuCore/CorbanuTerminal/pull/104) | needs-changes  | The time Cargo.lock update omits MODULE.bazel.lock regeneration, and the hosted Bazel log confirms the stale-lock failure.                              |
| [#105](https://github.com/CorbanuCore/CorbanuTerminal/pull/105) | needs-changes  | HMAC 0.13 uses digest 0.11 while workspace SHA-256 remains on digest 0.10, breaking `Hmac<Sha256>` in cloud-config.                                     |
| [#112](https://github.com/CorbanuCore/CorbanuTerminal/pull/112) | needs-changes  | The MCP 401 handling and recovery test conflict with current main; resolve both paths and rerun the affected authentication tests.                      |
| [#113](https://github.com/CorbanuCore/CorbanuTerminal/pull/113) | close          | Ox Alpha has no advertised OpenRouter provider endpoints and is absent from the current model list, so adding it as selectable is obsolete.             |
| [#130](https://github.com/CorbanuCore/CorbanuTerminal/pull/130) | close          | Its nine argument comments are already in target-branch commit bf7e27e29; merging this PR into that branch produces no tree change.                     |
| [#136](https://github.com/CorbanuCore/CorbanuTerminal/pull/136) | merge-ready    | The documentation-only lifecycle preserves account scope, separates initial evidence from verification, and requires explicit reward confirmation.      |
| [#137](https://github.com/CorbanuCore/CorbanuTerminal/pull/137) | merge-ready    | The documentation-only Windows diagnostics match the installer guard, use inspection commands, and disclose the lack of Windows execution.              |
| [#139](https://github.com/CorbanuCore/CorbanuTerminal/pull/139) | merge-ready    | The proposed wallet-linkage document matches the status renderer, distinguishes Task Node from Solana, and requires authoritative account verification. |

## Suggested order

1. Review the documentation-only changes #136, #137 and #139 for merge.
2. Close #130 as absorbed into its target and #113 as currently unavailable.
3. Update #101 and #103 with current main and rerun required CI; regenerate
   Bazel metadata for #104. These have no identified dependency API defect.
4. Repair the concrete session-loss/type errors in #38, #102 and #105.
   Resolve and qualify the broader integration work in #28, #88, #94 and #112.

## Evidence and next actions

### #28 — billing token buckets and reported cost

Head `e5ddf72293301386d8859093d809a505856801b7`; 46 changed files.
The description explicitly gates merging, and the PR is still a draft.
The change adds a cache-creation token field through transport, persistence and
presentation records and records OpenRouter cost. Main already has
`cache_write_input_tokens`; reconciliation needs to preserve the current
accounting contract instead of adding a competing bucket.

A local three-way merge reports eight conflicted paths, including
`codex-api/src/endpoint/anthropic_messages.rs`, the Chat Completions adapter,
`codex-api/src/common.rs`, and provider plumbing. Keep the gate in place.
Have the billing owner reconcile the remaining work with current accounting,
resolve the conflicts, and supply focused stream-usage and persistence evidence.
This review does not claim that every feature of the draft has been superseded.

### #38 — shared Task Node client and headless link

Head `98c9a96cc2738f908a6f28680d0a42595e69b2a9`; 12 changed files.
In [start_link_command at the reviewed head](https://github.com/CorbanuCore/CorbanuTerminal/blob/98c9a96cc2738f908a6f28680d0a42595e69b2a9/codex-rs/cli/src/tasknode_cmd.rs#L767),
the `--relink` path accepts an existing session, starts authorization, then
saves a replacement record with no active token before polling succeeds.
A cancelled, rejected or abandoned replacement therefore loses the working
local session. The two earlier P2 recovery findings were fixed; this remaining
session-loss defect is separate and agrees with the maintainer's hold comment.

Keep active and pending state separate and promote a validated replacement
atomically. Reconcile the extraction with current `tasknode-session` code,
resolve five conflicted paths, and verify cancellation, rejection, timeout,
concurrent attempts and successful promotion without real-account mutations.

### #88 — Anthropic/OAuth preservation draft

Head `ea5f08734310ccb86d227efa68a3059b16f37769`; 14 changed files.
The PR explicitly preserves unreviewed WIP. Its complete PR diff includes
release/installer changes as well as OAuth recovery and payload handling.
The OAuth changes invoke the Claude CLI and restore a prior credential file
on unsuccessful refresh; those paths need isolated credential qualification.

There are eight conflicted paths, including OAuth, client code and installers.
Split or explicitly account for the mixed scope, rebase, then qualify the
resulting candidate's OAuth recovery, payload limits and affected installer
behavior. No production credentials or inference were exercised for this review.

### #94 — upstream 0.147.0 integration

Head `b9579894f58f381f43a84bcb4310dd130ef4fb69`; 1,095 changed files.
The description and committed upgrade note cover a broad protocol, permission,
provider, TUI, schema and release-tooling merge. Historical smoke evidence is
not proof that it integrates safely with today's main.

The current merge simulation reports **118 conflicted paths**, replacing the
44-path count in the older September review. Conflicts include workflow files,
Cargo.lock, generated protocol schemas and runtime code. Required CI is failed.
Stage integration onto the recorded current base, resolve the source and
generated artifacts together, and qualify that integrated tree before merge.

This is a blocking triage assessment, not a new exhaustive security audit of
all 1,095 files. The complete changed-file inventory and local diff were
collected; GitHub's combined diff endpoint rejected this PR's size, so the
recorded base/head Git diff was used instead.

### #101 — semver is blocked by inherited validation failures

Head `b836fdf18d31ff492e25c702cf1c544d041bf979`; only Cargo.lock changes.
It merges textually into main. The
[hosted Bazel log](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/31391384278/job/93463709405)
reports an out-of-date lock. Comparing its computed facts with the PR's lock
shows that `semver_1.0.28` facts are already present and identical.

The exact missing facts in that log are `path-absolutize_3.1.1` and
`path-dedot_3.1.1`; current main already contains both. Do not attribute this
failure to the semver bump or demand a nonexistent semver metadata addition.
Rebase onto current main and rerun required validation. This is a validation
hold, not an identified semver compatibility defect.

### #104 — time needs matching Bazel metadata

Head `130c4c2a8ec50d17655c00967ae9fd11241fb9a5`; only Cargo.lock changes.
The update merges textually, but `MODULE.bazel.lock` lacks the new time 0.3.55
facts at both the PR head and recorded main. The
[hosted Bazel log](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/32591735834/job/97076532533)
confirms the stale-lock failure.

Regenerate and commit the Bazel lock using the repository workflow, then rerun
CI. This is a build-metadata finding; it does not establish a defect in time
0.3.55 itself.

### #102 — coordinated OpenTelemetry upgrade needed

Head `d92b955f1bc4d0f97d5974b34fb4c7c6ebf4fcdb`; two changed files.
The workspace SDK becomes 0.32.1, but the directly used API, OTLP exporter and
tracing appender remain 0.31. The lock graph contains both API/SDK generations.
The [packaging build](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/31391472635/job/93463996559)
fails in `otel/src/provider.rs`: `on_start` has an incompatible `Context`
type and the new `SdkLoggerProvider` does not implement the old
`LoggerProvider` trait. This corroborates the source-level incompatibility.

Upgrade the interacting telemetry crates and adapters together, regenerate both
locks, and run affected telemetry tests. A textually clean merge is insufficient.

### #103 — repaired lock, incomplete merge qualification

Head `07642f8715308d4b3f04fbe9e59679c8f087c7f5`; two changed files.
This head **includes the Bazel lock repair** for insta 1.48.0 and console
0.16.4. It merges textually into current main; the earlier missing-lock
finding is resolved.

The required CI aggregate remains failed. The downloaded
[Bazel job](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/35906111615/job/107334109203)
fails fetching a zlib package with HTTP 404, which is infrastructure failure,
not evidence that insta is defective. The maintainer also documented inherited
lint/build failures in [the existing diagnosis](https://github.com/CorbanuCore/CorbanuTerminal/pull/103#issuecomment-5816574937).
Refresh against current main and obtain passing required validation. The
recommendation is a validation hold; no new insta code defect is asserted.

### #105 — HMAC and SHA-256 trait generations disagree

Head `513b2b83828ed5ca7fbbfb85f54172cd630f944a`; three changed files.
The PR changes HMAC to 0.13 without updating workspace `sha2 = "0.10"`.
HMAC 0.13 depends on digest 0.11; SHA-256 from sha2 0.10 implements digest 0.10
traits. The [packaging build](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/31391738149/job/93464846884)
reports E0277 at `cloud-config/src/cache.rs:221` for `Hmac<Sha256>`,
including incompatible `CoreProxy` and `BlockSizeUser` bounds.

Migrate the affected HMAC/SHA call sites and dependency versions together,
regenerate both locks, and verify cache-signature and authentication behavior.

### #112 — retain the fix, resolve the current integration conflict

Head `b8400099a4387376a593c65e4643d7dabb370b7a`; seven changed files.
The change excludes sibling upstream global configuration, classifies
challenge-less HTTP 401 responses and shortens authentication errors.
Two current-main conflicts remain:
`codex-rs/rmcp-client/src/http_client_adapter.rs` and
`codex-rs/rmcp-client/tests/streamable_http_recovery.rs`.

Resolve those against current transport/recovery behavior, then rerun the
foreign-home, challenge/no-challenge 401, token-expiry and recovery tests.
The review does not equate these integration conflicts with proof that the
original authentication fix is incorrect.

### #113 — unavailable model route

Head `371cc46a4c8c9b78730df9223a2516b06453d6d6`; 11 changed files.
The PR makes `stealth/ox-alpha` visible and eligible for orchestration with
free pricing. On 2026-10-19, OpenRouter's [model list](https://openrouter.ai/api/v1/models)
contained no matching ID, while its
[endpoint metadata](https://openrouter.ai/api/v1/models/stealth/ox-alpha/endpoints)
returned model metadata with an empty `endpoints` array.

Close this addition rather than expose a currently unadvertised route.
If the provider restores it, recheck availability, pricing and capabilities in
a new current-base change. No authenticated generation call was made, and the
review does not claim permanent retirement. The separate model-detail 404 is
not used as the deciding proof.

### #130 — fix already absorbed into its target

Head `fd202c4943b4f66c0b5effd61e631ad9624f4697`; two changed files.
The PR targets #126's branch, not main. The live target commit
[bf7e27e29](https://github.com/CorbanuCore/CorbanuTerminal/commit/bf7e27e29bc1eeff5b3a1e8a31b0aa611ecb89cc)
contains the same nine comments and wrapping changes. Both affected source
files match, and a merge simulation produces exactly the target's existing
tree. Close as absorbed, retaining credit for the contribution. This does not
claim that #126 is merged into main.

### #136, #137 and #139 — documentation recommendations

- **#136**, head `9926bcbd95f428a1776745740de45dbf0f166afa`: only 75 lines
  of Task Node lifecycle guidance are added. Command/flag structure matches the
  current CLI; the examples require preserving inherited account scope,
  checking action permissions, answering the actual verifier request and
  inspecting the reward outcome. Inspection after an uncertain write prevents
  blind duplicate submissions. No live task mutation was used to test examples.
- **#137**, head `618a5d43924cb01576efcab4fceefa748607de81`: only README
  navigation and the diagnostics guide change. The installation paths and
  `Ensure-Junction` ownership guard match current source; relative links
  resolve. `Get-Command -All` precedence and `Get-Item -LiteralPath -Force`
  agree with [Microsoft's command documentation](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/get-command)
  and [item documentation](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.management/get-item).
  Native Windows execution remains explicitly unperformed.
- **#139**, head `62088d596cba673bef6ed4f7d9a4d6e09b6dc00b`: only a proposed
  root specification is added. Current `tasknode_status_lines` still emits
  the account, GitHub and full linked-wallet values described. The document
  requires checking the task owner's authenticated server association; copied
  text or a screenshot alone is not key-possession proof. Merging this proposal
  does not deploy its wording. Newcomer usability and negative-case validation
  remain adoption work and are not claimed as passed.

All three merge cleanly into the recorded main. Their available head check
lists contain only a skipped CLA job, so this review does not claim a passing
runtime CI campaign for these documentation-only changes.

## Reproduce the merge findings

Fetch the pinned commits before running these commands. A conflict result from
`git merge-tree` exits 1; it is the expected review observation, not a completed
merge. These commands leave the checkout and index unchanged.

For #94, the output lists 118 conflicted paths before the merge messages:

```sh
git merge-tree --write-tree --name-only \
  78dd811d6e3718140c938ad4839968d9995fd5ac \
  b9579894f58f381f43a84bcb4310dd130ef4fb69
```

For #130, the first output line equals the target tree printed by the second
command, showing that merging into the actual target adds no tree change:

```sh
git merge-tree --write-tree --name-only \
  bf7e27e29bc1eeff5b3a1e8a31b0aa611ecb89cc \
  fd202c4943b4f66c0b5effd61e631ad9624f4697
git rev-parse bf7e27e29bc1eeff5b3a1e8a31b0aa611ecb89cc^{tree}
```

## Review method and limits

Descriptions, full changed-file inventories, diffs, available comments/reviews
and head checks were collected for all 15 PRs. Exact heads were fetched and
compared with the API inventory. Local `git merge-tree --write-tree --name-only`
checks used the recorded current main, with an additional check of #130's
actual target. Selected hosted logs were read to distinguish specific compiler
and lockfile failures from unrelated infrastructure failures.

Source inspection and merge simulation do not establish runtime correctness.
No full Rust, TUI, Windows or release qualification campaign was executed;
the blocked implementation PRs require their own updated evidence. Historical
author test claims are attributed to their PRs, not claimed as tests run here.

This change is routine review documentation. It changes no product behavior,
authorization boundary, persistent state or external interface. Product plans,
implementation sprints, code-blind functional execution, live-repository
release tests and release sign-off are not applicable to this document.
