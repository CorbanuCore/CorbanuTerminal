# Handoff: bind stage-one memory requests to inherited, live security policy

Status: accepted adjacent follow-up for **PF-30-S02 / public policy binding**,
not implemented or qualified. The integration coordinator accepted this
disposition after Fable review 3; it is not a clean review or completion of
PF-30-S01. Protected memory remains unqualified.

This handoff is routine QA documentation. Any implementation changes the
security boundary and requires an explicitly allocated product-initiative
sprint and its own exact file scope. The root coordinator owns plan/index
linkage; this document does not allocate another lane or authorize edits.

Product specification, **Non-negotiable controls**: “Classify instruction intent
and provenance before external content can influence tools or financial actions.”
Plan: `docs/plans/active/p0-security-levels.md`, PF-30. Follow-up record:
`docs/sprints/current/p0-security-levels/pf-30-s02-persistent-taint-and-memory.md`.
PF-35 detector qualification remains separate; never manufacture an Allow verdict
to make this path appear functional.

## Verified path and source provenance

Inspected at source candidate `e592cf75a`, recorded in `fable-review-3.json`.
The data flow is:

1. `codex-rs/app-server/src/request_processors/turn_processor.rs:643` calls
   `start_memories_startup_task` after a submitted turn with input.
2. `codex-rs/memories/write/src/start.rs` skips ephemeral sessions, disabled
   `MemoryTool`, non-root agents and unavailable state DB, then starts the
   asynchronous pipeline with an `Arc<Config>` captured at startup.
3. `codex-rs/memories/write/src/phase1.rs:298` loads rollout items and serializes
   them; the user message embeds rollout contents through
   `build_stage_one_input_message`, then calls `stream_stage_one_prompt` at 323.
4. `codex-rs/memories/write/src/runtime.rs:251` independently constructs
   `ModelClient::new`, creates a client session and streams that prompt.
5. Core's new constructor defaults to Permissive and has no policy binding.
   The internal session's `with_ingress_policy` hook is `pub(crate)` and is not
   applied to this out-of-crate client. The memory trace-summarize endpoint guard
   is a different path; it does not cover this stage-one request.

Local Git provenance, not a claim about who introduced a security regression:

| Lines | Blamed commit and recorded author | Meaning |
| --- | --- | --- |
| Runtime construction and phase-one rollout/prompt path | `431ebeaef70ebb08bba9221d7e0803c1b80d3a04`, jif-oai, 2026-04-28, `feat: split memories part 2 (#19860)` | Existing memory path before the round-five allocation. |
| App-server startup invocation | `771a4e74ac319c3d8379c62c7caa3aec1ad53382`, Eric Traut, 2026-05-20, `Add thread/settings/update app-server API (#23502)` | Existing orchestration path. |
| Core constructor's new default admission floor | `9c53c0a033bd073d1ae4127b9e8e0b8da9e7d84b`, round-five provenance lane | Introduced by this lane to preserve original Permissive behavior. |
| Core constructor's unbound policy field | `abb97e2540f408eaf9eb04139b5f1479a74e1e12`, round-five provenance lane | Added live-policy binding for allocated internal callers, not this separate worker. |

`git diff 4f263ca73..1e237cd90 -- codex-rs/memories/write
codex-rs/app-server/src/request_processors/turn_processor.rs` is empty. Therefore
the memory data path itself was not added or changed by this lane; the new
admission boundary left it uncovered. Commit subjects contain historical PR
numbers, but PR authors, merger identities and automation triggers were not
independently retrieved. The two existing-path commits record GitHub as
committer; do not substitute that for a verified human merger identity.

## Minimum policy-binding contract to decide before coding

- Obtain a **host-owned binding from the owning thread/session**, carrying the
  correct thread identity and access to its inherited/live effective policy.
  Prefer a narrow session-derived client factory or opaque read-only binding;
  the API shape is a coordinator decision, not prescribed by this handoff.
- Expose neither trusted controller mutation nor a caller-forgeable principal,
  envelope, serialized policy or “allow” flag. Bindings must not migrate across
  unrelated threads. A provider replacement within the same thread must retain
  its policy/admission identity.
- Calculate the effective level as at least the maximum of configured intent
  and current inherited policy. A startup `Config` snapshot or public setter for
  `config.security_level` alone is insufficient: queued work can outlive policy
  changes, and inherited policy can be stronger than the local configuration.
- Missing, uninitialized, stale-identity or unavailable policy must fail closed
  before rollout context reaches the provider. Do not silently fall back to the
  constructor's Permissive default.
- Re-evaluate at the actual network-dispatch boundary, including subsequent jobs
  and retries/new attempts after a policy change. Specify cancellation behavior
  for in-flight work explicitly; do not promise retrieval of frames already
  sent. A denied job must not be recorded as a successful memory extraction.
- Until complete persisted-source lineage, segmentation and real screening
  capability delivery are available, reject unscreened stage-one input under
  protected policy. Keep existing Permissive formatting and successful behavior
  unchanged. Merely tagging the serialized rollout as a user message is not
  admission, and one admitted fragment cannot cover mixed or legacy raw input.
- Keep diagnostics bounded and free of rollout text, secrets and credentials.
  Specify how denial is reported to the worker/orchestrator so repeated jobs do
  not masquerade as successful extraction or endlessly retry a policy denial.

Before allocating implementation, inspect direct ModelClient constructors within
the newly approved worker scope and its provider/retry lifecycle. Do not silently
expand into all detached clients, phase-two architecture or a new public policy
schema. Those require their own disposition and allocation.

## Required regression matrix

Use fake providers with recorded requests and synthetic canaries, not production
credentials or real user rollouts. Test the actual worker entry/stream path, not
only an isolated Core helper.

| Scenario | Required evidence |
| --- | --- |
| Permissive root, feature enabled | Existing stage-one prompt/output behavior succeeds; compare captured wire shape and persisted successful result. |
| Configured Moderate and Aggressive | Zero provider requests containing rollout input; stable policy-denial outcome, no successful result persisted. |
| Permissive config, stronger inherited policy | Same refusal; demonstrates binding is more than a config-only floor. |
| Live increase after job enqueue but before dispatch | Synthetic canary never leaves the worker. Repeat across subsequent jobs and a retry/new attempt. |
| Unavailable/uninitialized binding or wrong thread | Refusal before network; no fallback, no cross-thread registry or authority sharing. |
| Same-thread provider/model replacement | Policy and source identity remain bound; protected denial cannot be escaped by transport/provider choice. |
| Authorized downgrade while configured floor stays stronger | Maximum-floor semantics hold; no implicit weakening from the live snapshot alone. |
| Legacy, mixed-source or forged-authority rollout | Unscreened protected input rejects; preserved human quotes/tool text cannot mint approval or trust. Future positive screening proof must bind exact sources/content and be labeled fixture versus production. |
| Policy denial, cancellation and restart | No false successful summary, no secret-bearing error, no uncontrolled denial retry; resumed worker obtains the owning session's current binding. |
| Disabled feature, ephemeral/non-root session, missing state DB | Existing skip behavior remains intact; no unintended new model traffic. |

Run scoped fix/full formatting before final tests on the RTX host, serialize
builds, and use fresh per-run TMPDIR outside the contaminated shared directory.
Include an actual-key TMUX startup/turn flow in an isolated synthetic test home
with memory enabled, observing the applicable success/denial/recovery behavior
and fake-provider request recording. Core/unit tests and structured exec are
supporting evidence, not a substitute for that user-facing interaction.

No new tests, build or review were run for this documentation-only handoff.
Existing round-five results and their limitations are in `qualification.md`.
Three of the five authorized provenance reviews were used; no fourth invocation
is authorized by this document. The coordinator decides any future review budget
and final qualification. No human acceptance, benchmark or release pass is implied.
