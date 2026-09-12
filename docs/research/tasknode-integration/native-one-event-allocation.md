# PF-80-S01 next increment — bound one-event request engine

Manager allocation, September 11 local / September 12 UTC. Same active feature
PF-80 and sprint PF-80-S01; **Internal delivery control — TO BUILD**, “Use
sequential sprints per initiative”. Native preparation `2b281975a` is reviewed
and integrated in receiving baseline `c33d47f6ccd00a64fcb05a057472d8ca9f0139d4`.
Both native crates passed 244 tests there, plus 129 Python/governance regressions
and two cross-language goldens. This allocates code, not live authority.

## Exact scope and boundary

One Astra High worker uses the existing Task Node checkout/branch recorded in
the active plan, clean-fast-forwarded to the reviewed allocation commit. Parent
records actual launch HEAD, serializes registration and owns independent review.
Literal writes:

- `codex-rs/tasknode-session/src/delivery_send.rs`
- `codex-rs/tasknode-session/src/delivery_send_tests.rs`
- `codex-rs/tasknode-session/src/lib.rs`: only `#[cfg(test)] mod delivery_send;`
- `qa/initiative-control/pf-80-s01/native-one-event/`

Implement a private request/response engine using the accepted preparation
module and a narrow one-event fixture transport. The entire new module is
test-only. No production transport implementation, live preflight loader,
credential resolver, CLI/TUI entry, outbox, retry scheduler, enrollment or API
key loader. No edits to delivery_goal, Client, tracker, recovery, dependencies,
BUILD, lockfiles or shared plans. Target 250–400 non-test lines; hard bound 500
non-test / 800 total changed lines. Stop and re-slice rather than omit validation.

This deliberately proves request/receipt behavior before connecting actual
authority. It is not a live-capable sender or proof of profile credential origin.
The next outer adapter must reuse native profile resolution and Client after
explicit allocation; completing fixtures must not turn model-supplied facts
into production permission.

## Source seams and retained contracts

Read native `SessionScope`, `ActiveSession::is_expired_at`, `Client::for_session`,
`Client::identity` and `Client::request_blocking`; retain them unchanged.
Client identity fences origin/account/token rotation but not profile, which
must be checked separately. A freely constructed ActiveSession plus chosen
SessionScope does not prove credential provenance. Missing/invalid expiry is
not expired under the existing API; it is also not live validity evidence.
`resolve_scoped` may poll or mutate pending credentials: never call it here.
Tracker capture generates new identity and sync processes a batch: neither fits.

First-party Task Node pin `40d2df72710a644f33a2b30831061e8265716db0` and reviewed
[contract](native-contract-and-decisions.md) remain the authority. Future
transport maps to `Client::request_blocking(POST,
"/api/terminal/tasknode/campaign-tracker/events", ...)`, off the async runtime.
Its redirect refusal and 45-second timeout are retained, not immediate cancel or
nondelivery guarantees. Wire shape is `{event, apiKey}`; approved full-payload
digest covers only `{event}`. Never hash/log/export credentials with the event.
Tracker status verifies account/enrollment only, not entitlement/ownership;
replay/tombstone responses do not establish current preflight evidence.
Do not invent server fields or a new permission/token/receipt protocol.

## Required behavior

Use a non-Clone, one-attempt context and explicit injected time/current identity.
All authority evidence is synthetic, fixture-only construction in this slice:

1. Compare explicit profile (including default), nonempty account, normalized
   approved HTTPS origin, and actual Client identity from a synthetic session.
   No anonymous/default fallback or ambient lookup.
2. Recompute preparation for the selected ID, full Python-canonical payload
   digest, sprint/workspace and exact tasks. Reject stale observation in this
   slice. Preserve values and the unverified-live-authority advisory blocker;
   never treat Preparation itself as permission.
3. Require separately modeled identity, enrollment, ownership, supported task
   lifecycle, entitlement, explicit one-event authorization and posting facts,
   bound to the same invocation/profile/account/origin/client identity/event.
   Missing, unknown or mismatched evidence makes zero fixture calls. No
   production constructor or public booleans-to-authority API.
4. Recheck cancellation, time/expiry, mapping/digest and identity immediately
   before exchange. Consume the attempt first; no batch or retry parameter.
   The narrow fixture operation attaches only a synthetic API key at the fixed
   endpoint and records one exact payload. Never invoke real Client networking.
5. Success requires 2xx, explicit `ok: true`, exact event ID and
   `summaryState: "not_applicable"`, with unchanged profile/client fence.
   Deleted/tombstoned, malformed, mismatched, unknown or pending-summary results
   are held, never task acceptance/completion. Return bounded typed outcomes,
   not raw server bodies, messages or credential-bearing Debug output.

Cancel before exchange means not attempted. Cancellation after start, timeout,
transport failure or changed identity before receipt consumption means outcome
unknown/held, retaining original event ID/digest. No rollback or safe-retry
claim. A 401 points to existing profile relink; no automatic relink/resend.
In-memory consumption is not durable exactly-once delivery. Later restart
recovery needs separately allocated lookup/bookkeeping and human same-payload
retry approval. No signed receipt format, TTL or new auth store is introduced.

## Verification and handoff

Synthetic in-memory tests: success/exact payload, default versus named/wrong
profile, missing/wrong account/origin, rotation before/after exchange, expiry,
missing/changed preflight facts, ID/digest/mapping/staleness, cancel before/during,
second-use rejection, timeout, 401/402/403/409/429/5xx/redirect, missing ok,
wrong ID/deletion/unknown summary, and credential canaries absent from outcomes.
No real credentials, external network or live target reads. Existing isolated
loopback Client regression may run as part of the unmodified full crate suite;
it is not a live Task Node call. New tests use no sockets or server subprocesses.

Follow native formatting policy: invoke `just fmt` with writes constrained to
allocated Rust files; record any out-of-scope Python/environment failure and
use scoped rustfmt if necessary. Do not rewrite inherited Python or repair the
toolchain. Run pinned preinstalled 1.95.0, offline:

```text
just test -p codex-tasknode-session delivery_send
just test -p codex-tasknode-session delivery_goal
just test -p codex-tasknode-session
```

Run both governance checkers and whitespace checks. Record actual nonzero
counts, candidate/file hashes, limitations and the normal-build OFF boundary.
Return uncommitted; parent owns one independent Astra High review plus bounded
corrections and final combined-tree tests. No review-budget reset.

Upstream: canonical `https://github.com/openai/codex.git`, verified common ancestor
`413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`, cached upstream
`1d74c3ba1ee98be2025ab066dcc3fd654fe8a3b6`; newer observed tip not integrated.
Only product-owned tasknode-session code and one test registration change;
retain Client/profile/history/permission/cancellation contracts. Existing Bazel
source glob registers sibling tests; use literals, no external include fixtures.
No upstream upgrade, TUI, live-repository, benchmark or human pass is claimed.

## Later human test readiness

Before live activation, the manager must assemble the pinned candidate, actual
supported controls, named setup operator/tester, exact native profile/account/
HTTPS origin/workspace/tasks, ownership/lifecycle/visibility and entitlement
evidence, separately authorized enrollment, approved API-key source, exact one
new event ID/full digest and recovery procedure. Reconcile prior answers rather
than reasking them; never request seed material or copy broad human credentials.
The current fixture slice needs none of those live decisions to run. Whole PF80
completion, PF79/PF81 activation, main landing and dashboard cutover stay gated.
