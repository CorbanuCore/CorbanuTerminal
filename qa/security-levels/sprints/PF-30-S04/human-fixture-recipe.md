# Human memory checks 21–23: preparation recipe

## Current answer

The operator fixture is now implemented, reviewed and qualified against the
combined RTX product `b12e32db3`. Use the [current operator guide](human-fixture/README.md)
and its pinned runner, not the historical commands below. Human checks21–23
are ready for assisted testing but are not marked accepted.

## Historical preparation record — superseded by the qualified operator guide

**The qualified binary is ready; a paused operator fixture is not yet implemented.**
Do not tick checks 21–23 by launching ordinary chat or replaying saved captures.
This preparation did not build, edit product/repository source, start services,
launch a UI, or invoke a reviewer.

Exact qualified RTX runtime: `6a6bb029d8f3e0c16653ce335d252f45b4d7326f`.
Binary: `/home/travis/security-round5/evidence/integration/6a6bb029d/candidate/codex`.
SHA256: `90d6a1f7f72c5397ff858583c038b2615c8fb034f57a890d6595d6b98afccd4f`.

## Runnable now: verify candidate and inspect existing synthetic proof

On RTX as travis, these commands are read-only and require no provider credentials:

```bash
memory_evidence=/home/travis/security-round5/evidence/integration/6a6bb029d
sha256sum "$memory_evidence/candidate/codex"
"$memory_evidence/candidate/codex" --version
head -v -n 50 "$memory_evidence"/memory-tmux/*outcomes.txt
head -v -n 50 "$memory_evidence"/memory-tmux/*input-events.txt
head -v -n 50 "$memory_evidence"/memory-tmux/*worker.txt
head -v -n 50 "$memory_evidence"/memory-tmux/*restart.txt
```

Expected automated proof: Permissive requests/output **1/1**; Moderate **0/0**;
Aggressive **0/0**; exit during pending Permissive extraction **1/0**. Restart
adds no canary request. This is supporting machine evidence, not human acceptance.

The existing automatic test can be rerun by the coordinator in a reserved RTX
build slot with the standard shared lock/fresh TMPDIR. Its exact selector is:

```bash
export CORBANU_TMUX_REQUIRED=1
export CARGO_BIN_EXE_codex=/home/travis/security-round5/evidence/integration/6a6bb029d/candidate/codex
# Set CORBANU_MEMORY_EVIDENCE to a new per-run directory before invoking.
just test -p codex-tui --test all -E 'test(tmux_memory_worker_policy)' --retries 0 --test-threads 1
```

This command is **not an operator-session launcher**. It is documented here,
not executed by this preparation task.

## Why existing utilities cannot supply the requested paused session

- `tui/tests/suite/memory_stage_one_policy.rs` creates an in-process WireMock
  server and TempDir home, seeds an eligible SQLite-backed historical rollout,
  types the foreground prompt, validates counts, types `/exit`, restarts,
  types `/status` and exits. All four cases finish in about 11 seconds.
- A `CORBANU_MEMORY_EVIDENCE` path saves results; it does not retain or pause a
  session. The failure-only `home.keep()` retains data, not the fake server.
- `support/tmux.rs` owns a private socket, session and cleanup watchdog. Drop
  terminates only its recorded fixture processes; successful tests cannot leave
  a usable session behind. There is no public attachment hint/pause facility.
- Current pending response is delayed only 30 seconds, too short for an
  unhurried operator test; default nextest timeout is 60 seconds.
- Existing actual-key memory testing does **not** perform `/model` switching.
  `startup_tests::pf_30_s04_live_provider_refresh_preserves_consolidation_pair`
  proves a real two-endpoint worker regression through Core settings APIs, not
  through an operator model picker. Do not conflate those proofs.

Do not suspend the test process: that also suspends its fake provider. Do not
disable the cleanup watchdog or attach to unrelated user TMUX sessions.

## Smallest proposed test-only support (requires coordinator allocation)

Add one explicitly ignored/manual fixture test, reusing existing typed TMUX,
WireMock, SQLite seeding and synthetic response utilities. No product code,
memory configuration API, DB schema, credentials or archived sprint changes.

Proposed paths:

1. `codex-rs/tui/tests/suite/memory_human_fixture.rs`: one manual entry point,
   three selectable cases (`startup`, `provider-switch`, `pending-exit`). Copy
   the small eligible-history seed pattern from the existing memory test; do
   not refactor that archived test merely for this helper.
2. `codex-rs/tui/tests/suite/mod.rs`: coordinator-owned Unix-only registration.
3. `codex-rs/tui/tests/support/tmux.rs`: small typed attachment hint accessor
   carrying the actual private socket path and session name. Obtain the socket
   from the owned server, not global TMUX discovery. Keep Drop/watchdog intact.
4. `codex-rs/tui/tests/support/tmux_tests.rs`: accessor/cleanup regression.
5. `codex-rs/.config/nextest.toml`: coordinator-owned explicit manual profile,
   bounded about 10 minutes, zero retries. Normal automated profiles unchanged.

Fixture requirements:

- Ignored by default; explicit human opt-in and selected case required. Create
  a fresh disposable home; never accept an existing personal CODEX_HOME. Seed
  only `PF30S04_SYNTHETIC_ROLLOUT_CANARY`, two-hour-old metadata with nonempty
  preview/first_user_message, memory mode enabled, and completed DB backfill.
- Select the immutable candidate above, log its hash. Isolated CODEX_HOME and
  CORBANU_HOME, file-only synthetic auth, native keyring disabled; no real key.
- Two loopback fake Responses endpoints A/B with fixed fake keys, explicit
  provider IDs/names, zero request/stream retries. Reuse custom-provider config
  and picker pattern from `provider_management.rs` / `provider_convergence.rs`.
  Verify both appear under `/model` before handing off; no production endpoints.
- Keep mock servers, TempDir, DB and TMUX owners alive during a bounded async
  operator wait. Publish `ready.json`, an attach command and periodically updated
  `status.json` to a new evidence directory. Record endpoint, model, request kind,
  source-ID output count and bounded denial reason, never Authorization headers.
- For switch/exit cases, delay only the canary response for a visible 120-second
  window (foreground responses remain immediate). Publish the observed pending
  request and deadline before asking the operator to act. If missed, mark the
  case not exercised and rerun fresh; never quietly count it as passing.
- Do not synthesize the operator's `/model`, prompt or `/exit` keys. The fixture
  observes output/counts; it may relaunch the same owned home only after the
  operator exits and explicitly requests the restart step. Preserve exact
  source ID for DB checks; exclude unrelated newly created foreground histories.
- Explicit finish/cancel or total deadline exits the fixture and lets existing
  Drop/watchdog close its server/panes. Persist only synthetic evidence, then
  remove the disposable home by normal TempDir cleanup. No daemon left behind.

Suggested future command contract (does **not exist yet**):

```bash
CORBANU_MEMORY_HUMAN_CASE=startup CORBANU_TMUX_REQUIRED=1 \
  just test -p codex-tui --test all --profile human-memory \
  --run-ignored only -E 'test(memory_human_fixture)' --retries 0 --no-capture
```

The same command selects `provider-switch` or `pending-exit`. Operator opens a
second SSH terminal and runs the fixture's printed `tmux -S <owned-socket>
attach-session -t <owned-session>` command. Do not invent the socket/session name.
Compile/test-only helper under the shared build lock on RTX; release the build
lock before a long operator wait by using a frozen test artifact/archive under
the coordinator's approved runner. Never tie up the shared target for human time.

## Operator steps / expected proof

**21 — startup:** attach, then send `Run the synthetic foreground fixture.`
and Enter. Only then wait for fixture status to report canary request A=1 and
source output=1 while foreground remains responsive. Record both viewport and
fixture outcome. **Correct current humanTest order:** app-server
`turn_processor.rs:639–651` starts the memory task after a nonempty input turn;
launching alone is not a reliable trigger. Do not wait for completion before the
first harmless prompt.

**22 — provider switch:** new fixture; send first harmless prompt and wait for
`A canary pending` in status. Within the declared window, use `/model` to select
already-configured fake B, verify top/bottom indicators agree, and send a harmless
prompt. Routing log must identify the foreground request at B with its selected
model. The already-sent A extraction is not retractable; after replacement its
old binding must not successfully persist. Any fresh background request must
use its matching endpoint/model; no mixed pair. Record the selected model,
endpoint log and source-specific output/job outcome, not only the chat answer.

**23 — pending exit/restart:** new fixture; first harmless prompt triggers A
canary request with delayed response. While status says pending, type `/exit`.
On normal exit, inspect source output=0; request count can remain1. Explicitly
request same-home restart through fixture, attach new owned pane, run `/status`
only, then `/exit`. No hang/panic, no successful source output, no extra canary
request from status-only restart. Do not send a new prompt in this phase because
that would intentionally trigger another startup attempt subject to backoff.

## Recommendation / limits

Allocate the small manual-fixture support separately (roughly 2–4 hours including
cleanup/timeout and real-key operator rehearsal), then update human guidance to
the tested command. Until then leave 21–23 unticked/not exercised; check22 in
particular has no existing paused human route. No user product/security decision
is required—only coordinator approval of test-only scope and bounded runner.
No privileged setup or provider credentials are needed. Keep protected memory
inference disabled; these are Permissive regression checks, not protected-mode
qualification or a replacement for missing historical policy lineage.
