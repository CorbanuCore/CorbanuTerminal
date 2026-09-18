# RETURN — owner-brokerfix-116

Allocation digest: `98e7bd86553b8a3c2b912d05f36c68e8fd12b3cce5c802cab363e4b495742bbf`.
Claim: `467c38f3-070a-4f08-9965-17d9f0673113`. Runtime: gpt-6-astra / high.
Base: `64fa3474b899957e19fbf5705f886d19d3652b42`.
The frozen brief was read first after its SHA-256 matched
`599d252d78e0ace306439cbd431a2291e5ed950fbdb681b792594926a500d108`.

Bounded repair of the assigned offline qualification infrastructure. Product
citation: **Internal delivery control — TO BUILD**, “durable event dispatch,
acknowledgments and watchdog”; “authorizes the bootstrap and bounded live
qualification, not premature product sprint resumption, new product scope,
main/release or financial actions.” The existing broker authorization, upstream,
credential source, routing and tool behavior are preserved. No product feature,
plan, sprint, live operation or qualification is implemented by this repair.

## Findings and smallest repairs

1. **P2 — read tolerance and misleading unfinished evidence.** Separate
   60-second connection setup from 180-second actionable headers/nonstream reads
   and 600-second SSE event-idle deadlines. The latter matches
   `codex-rs/model-provider-info/src/lib.rs` and the per-event timeout in
   `codex-rs/codex-api/src/sse/responses.rs`. A small raw-reader adapter applies
   the remaining event deadline before every socket read, including HTTP chunk
   framing. Only complete SSE data events renew it; partial data and comments
   cannot keep a stalled event alive. Bytes still relay incrementally and
   unchanged. A bounded event observer detects upstream completion/failure.
   Admission is followed by a joined terminal receipt: `relay_completed` or
   `truncated`, the latter with a fixed `failure_code`. Clean HTTP EOF without
   `response.completed` is also truncation. This is the smallest repair that
   separates connection/read tolerance, enforces an actual event deadline, and
   makes post-admission failure visible without replacing the HTTP stack or
   buffering a whole turn.

2. **P3 — model/effort provenance.** Extract `model` and `reasoning.effort`
   from upstream `response.created.response`, or the upstream JSON response
   object for nonstream/compact. These populate authoritative `model`/`effort`.
   Client declarations exist only as `client_ids["body.model"]` and
   `client_ids["body.reasoning.effort"]`. `resolution_match` gives separate
   true/false comparisons; null means the upstream value is unavailable or
   redacted. Missing values never fall back to the client. This uses objects
   already parsed and adds only comparison fields; it does not invent a model
   resolver, rewrite requests or add another upstream call.

3. **P3 — unauthenticated journal growth.** Authenticate before entering the
   normal turn-record path. Wrong/missing bearers update a single padded
   1024-byte JSONL record for this broker instance, holding a serialized count,
   first/last processing times and the broker instance ID. Subsequent updates
   rewrite that slot under the existing lock/flock and fsync; authenticated
   turn records continue to append after it. A disconnect while sending the
   refusal cannot fall back to a per-attempt append. Count saturation at uint64
   max is explicitly marked as a lower bound. This is the smallest bounded-space
   change that retains durable counts and timestamps without another database,
   sidecar, background flush, lossy sampling or time-based growth limit.
   It bounds disk allocation, not CPU/IO consumption from a loopback flood.

## New and changed tests

All names below are methods of `test_qualification_broker.BrokerTests`.

| New test | What it proves |
| --- | --- |
| `test_real_gap_longer_than_old_timeout_completes` | Real loopback 61-second silence, with production 60/180/600 constants, completes byte-exactly and records relay completion. No fake clock or reduced timeout in this case. |
| `test_exceeded_event_idle_tolerance_is_truncation` | A 400 ms gap exceeds the test-scaled 150 ms idle deadline; downstream framing is incomplete and a joined truncation receipt names the timeout. |
| `test_deadline_renews_per_data_event_not_per_connection` | Five events 100 ms apart complete with a 300 ms deadline even though the entire connection exceeds 300 ms. |
| `test_comments_and_partial_events_do_not_renew_deadline` | Frequent comment heartbeats and partial data do not mask a missing complete data event. |
| `test_http_chunk_header_trickle_cannot_extend_event_deadline` | Trickle bytes inside HTTP chunk-size framing also cannot postpone the event deadline. |
| `test_clean_eof_and_upstream_failure_are_not_success` | Clean HTTP closure without completion and an upstream failed event both produce explicit terminal failure evidence. |
| `test_upstream_resolution_agreement_and_divergence` | Matching and differing model/effort values are authoritative from upstream with accurate comparisons and client provenance, for SSE and JSON. |
| `test_missing_or_sensitive_upstream_resolution_never_uses_client_claim` | Absent or synthetic-secret-valued upstream resolution remains null/redacted and never adopts client claims. |
| `test_unauthenticated_flood_has_fixed_space_exact_count_and_recovery` | 128 refusals occupy exactly 1024 bytes, preserve exact count/times, never read credentials or contact upstream, and allow authenticated success afterward; the 129th updates the original slot without damaging later turn records. |
| `test_concurrent_unauthenticated_refusals_do_not_lose_counts` | Four concurrent callers produce 64 counted refusals in one fixed slot with no upstream requests. |
| `test_preauth_disconnect_does_not_append_per_attempt_failure` | Eight disconnected refusal deliveries still use one slot and count all eight, without per-attempt error appends. |
| `test_refusal_slot_write_failure_latches_before_upstream` | A short aggregate write latches evidence failure and blocks subsequent upstream submission. |
| `test_terminal_evidence_failure_never_records_relay_success` | A terminal-receipt write failure closes downstream framing and latches the broker; admission remains explicitly unfinished, never completion evidence. |

Changed existing tests:

- `test_stream_is_incremental_exact_and_joined_to_upstream`: fixture prefix now
  contains upstream model/effort; retains the pre-completion incremental-byte
  assertion and checks the additional terminal receipt.
- `test_tool_call_round_trip_preserves_call_and_output`: filters admission
  rows from terminal rows when checking distinct upstream response joins.
- `test_wrong_and_absent_bearer_are_distinct_refusals`: checks the bounded
  pre-authentication aggregate instead of an individual turn refusal.
- `test_serve_cli_uses_only_explicit_credential_path`: expects startup,
  admission and terminal receipts.
- Shared fake-upstream/request helpers supply resolved metadata and controlled
  event gaps, early EOF, failed events and framing trickles. Existing compact,
  credential, redaction, proxy, routing, journal and ownership checks remain.

## Evidence record shape

Schema is now **2**, including startup. Startup retains the existing
`schema, kind, recorded_at, broker` shape. Each authenticated attempt retains
the old turn-record keys and adds `resolution_match`; client-declared model
and effort move into the existing provenance-prefixed `client_ids` map.

For an admitted attempt there is a `kind: turn`, `outcome: admitted` receipt
before response bytes, followed by a `kind: turn_end` receipt with the same
attempt ID, broker/upstream joins and request hash. Terminal outcome is
`relay_completed` or `truncated`; truncation adds `failure_code`.
A matching terminal receipt is required: admission alone is unfinished evidence.
If a final client framing write fails after recording relay completion, a
later truncation receipt supersedes it. Relay completion concerns broker
delivery; it does not prove client acceptance, successful tools or qualification.

A process crash or unwritable journal can prevent terminal evidence. No storage
scheme can durably record a failure into unavailable storage: such admission
must remain **unfinished/unqualified**, not be counted as success. Journal
write failures latch subsequent forwarding closed. This limitation is tested
explicitly rather than reported as successful truncation logging.

Illustrative terminal receipt, with invented nonsecret values only:

```json
{
  "schema": 2,
  "kind": "turn_end",
  "broker": {
    "pid": 1234,
    "process_started": "Thu Sep 17 12:00:00 2026",
    "ppid": 1200,
    "uid": 501,
    "instance_id": "00000000-0000-4000-8000-000000000001",
    "module_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
    "socket": {"host": "127.0.0.1", "port": 28443}
  },
  "attempt_id": "00000000-0000-4000-8000-000000000002",
  "received_at": "2026-09-17T19:00:01+00:00",
  "upstream_headers_at": "2026-09-17T19:00:02+00:00",
  "recorded_at": "2026-09-17T19:10:03+00:00",
  "path": "/v1/responses",
  "inbound_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
  "model": "resolved-model",
  "effort": "medium",
  "client_ids": {
    "body.model": "fixture-model",
    "body.reasoning.effort": "high",
    "header.session-id": "session_fixture"
  },
  "upstream_request_id": "req_upstream_1",
  "upstream_response_id": "resp_upstream_1",
  "upstream_status": 200,
  "resolution_match": {"model": false, "effort": false},
  "outcome": "truncated",
  "failure_code": "upstream_stream_idle_timeout"
}
```

The unauthenticated aggregate has the following exact field shape. Its JSON is
space-padded to 1023 bytes plus newline. Only this row is mutable, so its physical
position does not imply its last-update order. Each restarted broker instance
gets its own slot; existing instance records remain intact.

```json
{
  "schema": 2,
  "kind": "preauth_refusals",
  "broker_instance_id": "00000000-0000-4000-8000-000000000001",
  "outcome": "synthetic_bearer_required",
  "count": 129,
  "count_saturated": false,
  "first_at": "2026-09-17T19:00:01+00:00",
  "last_at": "2026-09-17T19:00:05+00:00"
}
```

Live readers must hold a shared flock when reading the mutable slot; stop the
broker before preserving the qualification artifact. A torn/failed write is
not valid evidence. No request body, arbitrary header, bearer, error payload,
prompt or real credential is included in either record shape.

## Verification

Read `docs/development/test-isolation.md` before any tests. Both commands run
from the repository root in a disposable venv built under `env -i` from
`scripts/initiative_control/requirements.txt`. Installation used the three
existing pinned local wheels with `--no-index`; see
[venv log](owner-brokerfix-116-venv.txt) and
[disposable root](owner-brokerfix-116-test-root.txt).
Tests run under `env -i` with disposable HOME, TMPDIR, all three profile aliases,
`CORBANU_TEST_NO_NATIVE_KEYRING=1`, `PYTHONDONTWRITEBYTECODE=1` and a fixed PATH.
The control suite has a separate disposable HOME/TMPDIR/profile and
`PYTHONPATH=scripts/initiative_control`. Broker network tests deny off-loopback
connections in-process; CLI fixtures have explicit loopback endpoints and only
synthetic credentials.

Commands, in gate order:

```text
python -B -m unittest discover -v -s qa/initiative-control/management-bootstrap -p test_qualification_broker.py
python -B -m unittest discover -v -s scripts/initiative_control -p 'test_*.py'
```

- [First broker run](owner-brokerfix-116-focused-first.txt): **32 tests**,
  **31 passed**, **0 failures**, **1 error**, **0 skips**, **65.732s**, exit **1**.
  Error: `test_qualification_broker.BrokerTests.test_preauth_disconnect_does_not_append_per_attempt_failure`.
  macOS returned `ConnectionResetError: [Errno 54]` when the refusal connection
  closed with unread request bytes; the assertion originally accepted only
  `RemoteDisconnected`. The fixture now accepts either closed-connection
  result and still asserts exact aggregate count and size. Raw traceback is
  retained. All other checks, including the real 61-second gap, passed.
- [Final broker run](owner-brokerfix-116-focused-final.txt): **33 passed**,
  **65.795s**, exit **0**; **0 failures**, **0 errors**, **0 skips**.
  Failure/error names: **none**.
- [Full control regression suite](owner-brokerfix-116-suite.txt): **880 passed**,
  **512.671s**, exit **0**; **0 failures**, **0 errors**, **0 skips**.
  Failure/error names: **none**. Final combined coverage: **913 unique passing
  tests**. The earlier broker attempt is preserved, not counted as extra coverage.

Existing suite fixture publication/status messages are retained verbatim; they
are not evidence of live publication or controller activation.

## Changed files

| File | Final lines | Diff (+/-) |
| --- | ---: | ---: |
| [qualification_broker.py](qualification_broker.py) | 617 | +167 / -24 |
| [test_qualification_broker.py](test_qualification_broker.py) | 629 | +192 / -9 |
| [owner-brokerfix-116-focused-first.txt](owner-brokerfix-116-focused-first.txt) | 61 | new |
| [owner-brokerfix-116-focused-final.txt](owner-brokerfix-116-focused-final.txt) | 38 | new |
| [owner-brokerfix-116-suite.txt](owner-brokerfix-116-suite.txt) | 965 | new |
| [owner-brokerfix-116-test-root.txt](owner-brokerfix-116-test-root.txt) | 1 | new |
| [owner-brokerfix-116-venv.txt](owner-brokerfix-116-venv.txt) | 7 | new |
| [owner-brokerfix-116-return.md](owner-brokerfix-116-return.md) | 245 | new |

Final tested SHA-256 values:

- Broker: `66c791fc3a0dac6006be0115271e2934aa5e6d404260e0ed1390f6d4be65b2b7`.
- Test module: `6afe4642f8406162a1db426834be1e636564e7c1b096f17c853cd1e4cb344ec3`.

Whitespace validation with `git diff --check` passed. Final `git status --short`
shows exactly these eight files, all inside the assigned QA directory; no path
outside scope moved. The two source/test hashes still match the final broker
run. No native credential prompt or live-profile read was observed.

No VM contact, live journal/schedule/transport/coordinator access, live preflight,
qualification, Rust tests, workspace formatter, commit or push was performed.
Only the assigned QA directory is edited. This return claims offline implementation
and automated evidence only. Independent functional/TUI/live-repository and
benchmark qualification are not exercised: there is no product interactive
handoff in this allocation. Integrator acceptance of this internal-only N/A
and later exact-package broker/guest functional qualification remain separate
gates; no independent approval, human acceptance or qualification is claimed.
