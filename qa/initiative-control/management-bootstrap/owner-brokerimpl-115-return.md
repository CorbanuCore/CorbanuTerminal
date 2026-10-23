# RETURN — owner-brokerimpl-115

Allocation digest: `a6c5ee74b364dc679d2f7ec939e2afcf1a6b19387b4f6343074e60798f685e32`.
Claim: `1fbebaac-72b9-484f-8b9f-e7e26211556b`. Runtime: gpt-6-astra / high.
Base: `a56fc4137c9de669967079a0a7715773f68dca54`.
Read the supplied brief first; SHA-256 matched
`bc3347612535d712a44fc7c7137b3a19f99f44d9c10c61af0aeac18267aca979`.

Routine offline QA infrastructure for the existing management-bootstrap recipe.
Product context: **Internal delivery control — TO BUILD**, “durable event
dispatch, acknowledgments and watchdog”; “authorizes the bootstrap and bounded
live qualification, not premature product sprint resumption.” This is the
frozen assignment's external qualification tool, not an activated product
authorization boundary. Plan/sprint changes are not part of this assignment.
Integrator acceptance of the internal-only classification/N/A remains distinct
from this implementation return.

## Files and why each part exists

| File | Lines | Purpose |
| --- | ---: | --- |
| [qualification_broker.py](qualification_broker.py) | 474 | Single standard-library broker module and port-owner CLI. |
| [test_qualification_broker.py](test_qualification_broker.py) | 446 | Single test module with 20 real-loopback HTTP/CLI tests. |

Final module SHA-256:
`3cfefd7a14ad148957958c9828b52259e1bd6ea2d003ac286e1f6c8b95455bb7`.
Final test-module SHA-256:
`71aa1f97844d79e95318527e9020a095033c8816a84e98770d48cd9cb3e809ec`.

The module contains these necessary parts:

- **CLI/configuration:** binds an explicit literal loopback address/port, fixes a
  single upstream `/v1` base, and accepts a credential **path** by argument or
  `BROKER_CREDENTIAL_FILE`. HTTPS uses Python's default certificate verification;
  HTTP upstreams must be literal loopback addresses for fixtures. It follows no
  redirects, environment proxies or alternate upstreams.
- **Admission/request forwarding:** only `POST /v1/responses` and
  `POST /v1/responses/compact`; exact
  `Authorization: Bearer corbanu-qualification-only`. Wrong/absent/duplicate
  authorization receives `401 synthetic_bearer_required`, distinct from missing
  upstream credentials, evidence failures and upstream refusal. Reads the
  credential afresh for each eligible request; never loads a user profile.
  Bodies are forwarded unchanged, including tools and tool outputs. This is not
  a tool executor. It replaces the upstream client request ID with a broker
  attempt UUID while retaining the supplied ID only under `client_ids`.
- **Evidence:** readiness check before upstream submission, then a locked
  append and fsync before successful bytes reach the client. An append/short
  write/fsync failure latches the broker closed for subsequent attempts.
  Only allowlisted identifier fields, model/effort and a SHA-256 body digest are
  recorded. Credential and marker values are excluded/redacted even if copied
  into an identifier. Raw request/response/error bodies, prompts and auth
  headers are never logged. Default HTTP logs and exception tracebacks are
  suppressed. Credential/evidence path or inode overlap is refused.
- **Response handling:** gets `x-request-id` exclusively from the upstream
  response header and the response ID from upstream `response.created` SSE or
  JSON `id`. Missing IDs fail closed. Buffers the initial SSE event (bounded),
  records admission, then relays entity bytes unchanged with regenerated HTTP
  chunk framing. Sticky `x-codex-turn-state` is preserved. Refused upstream
  bodies are replaced with a fixed broker error.
- **Identity/owner observation:** startup records PID, parent PID, UID, process
  start time, instance UUID, module hash and actual bound socket, both on stdout
  and in evidence. `port-owner` uses stock macOS `lsof` and `ps`, without
  reading process arguments or environments.

The tests supply only invented credentials and a fake upstream in the same
process. They exercise incremental SSE before upstream completion, exact bytes,
a two-request function-call/output round trip, compaction and nonstream JSON,
credential refusal/missing/unreadable/rotation, evidence failure before and
after upstream contact, a latched short write, upstream refusal/missing or
secret-valued IDs, route/framing refusal, proxy/redirect refusal, metadata
redaction, startup identity, real CLI startup and ownership, and a different
same-UID process taking the released broker port. They assert that invented
client IDs cannot become the upstream join fields.

Source basis: `codex-rs/codex-api/src/endpoint/responses.rs` and `compact.rs`
supply the two endpoint suffixes; `requests/headers.rs` supplies session/thread
headers; `core/src/responses_metadata.rs` supplies the per-turn metadata IDs.
`core/src/client.rs::responses_request_compression` leaves this API-key lane
uncompressed. Compressed/chunked request bodies are refused explicitly.

## Evidence shape

The file is JSON Lines. Startup writes one record with exactly
`schema, kind, recorded_at, broker`; the `broker` object has the same shape as
in the example below. Every Responses request attempt handled by admission has
one `kind: turn` admission/refusal record, when the evidence store is writable. Tool continuation
and retries are separate records, joined by the supplied session/turn IDs.
This is an admission receipt, **not a completed-turn claim**.

Example only: all values below are invented nonsecret values; the all-zero
digests illustrate the exact field shape rather than representing executed work.

```json
{
  "schema": 1,
  "kind": "turn",
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
  "recorded_at": "2026-09-17T19:00:02.100000+00:00",
  "path": "/v1/responses",
  "inbound_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
  "model": "fixture-model",
  "effort": "high",
  "client_ids": {
    "header.session-id": "session_fixture",
    "header.thread-id": "thread_fixture",
    "header.x-client-request-id": "client_invented_request",
    "client_metadata.turn_id": "turn_fixture",
    "turn_metadata.turn_id": "turn_fixture",
    "metadata.conversation_id": "conversation_fixture",
    "body.previous_response_id": "resp_upstream_previous"
  },
  "upstream_request_id": "req_upstream_1",
  "upstream_response_id": "resp_upstream_1",
  "upstream_status": 200,
  "outcome": "admitted"
}
```

Identity fields are repeated so each receipt can join directly to its listener.
`client_ids` preserves field provenance via its key prefix; only supplied,
bounded identifier values appear. Refusals retain the same shape with unknown
fields `null`, empty `client_ids` where unavailable, and a fixed refusal code
as `outcome`. Missing credentials/evidence never trigger a fallback.

Successful admission is fsynced before delivery. A crash between upstream
submission and the append can leave a billed upstream attempt with no receipt,
but cannot produce a served success. An evidence failure can itself prevent a
refusal record; it never permits unlogged success. A post-admission disconnect
does not rewrite admission as completion: use the upstream/client transcript
join for that. Preserve any partial evidence line and reconcile before restart
after a write failure. These limits are explicit in the module; there is no
claim of transactional completion across a remote provider and a local file.

## Invocation and the port-owner control

Examples are instructions for later authorized provisioning; not executed here
against a guest or real provider:

```sh
python3 qualification_broker.py serve \
  --host 127.0.0.1 --port 28443 \
  --upstream https://api.openai.com/v1 \
  --credential-file /MANAGER-ONLY/approved-credential \
  --evidence /MANAGER-ONLY/run-evidence.jsonl

python3 qualification_broker.py port-owner --host 127.0.0.1 --port 28443
```

Use `BROKER_CREDENTIAL_FILE` instead of `--credential-file` if desired; that
environment variable holds a path, never the credential value. The public
synthetic bearer constant is the value to author in the guest fixture auth file.

`port-owner` proves which **visible** processes were listening on that port at
observation time, including their PID/start time, UID, command name and socket
addresses. It includes wildcard and other-address listeners on the same port.
Errors or disappearance during inspection are unknown, not absence. It does not
prove continuous ownership, hidden-process absence, executable provenance,
tunnel destination, provider traversal, egress isolation or hostile same-UID
confinement. The broker's self-reported module digest is not OS attestation.

The round-114 P3 is addressed by requiring identity at both endpoints:
keep the guest SSH forward alive for the negative control, record that its
owner remains the expected SSH process, stop/reject the host broker lane, record
host-port ownership and require inference failure under continuous external
egress observation, then restore and obtain a freshly joined positive control.
Record owner observations before/during/after each control. If the tunnel is
closed, its port is available for reuse: any replacement listener, owner
mismatch or observation gap makes that attempt inconclusive. A socket number
or an inference failure by itself is no longer accepted as broker-down proof.
The test actually observes a replacement PID on the broker's released port.
Actual control execution and continuous observations remain future qualification
work; this return does not claim that the guest control has passed.

## Verification

Read `docs/development/test-isolation.md` before testing.
The [disposable root](owner-brokerimpl-115-test-root.txt) was created under
`/private/tmp`; the venv was built under `env -i` from the pinned
`scripts/initiative_control/requirements.txt`. The
[installation log](owner-brokerimpl-115-venv.txt) records `--no-index` use of
pre-existing local wheels, with all three required versions installed.

Both test commands ran from the repository root under `env -i`, fresh HOME
and all three profile aliases, private TMPDIR, a fixed PATH,
`CORBANU_TEST_NO_NATIVE_KEYRING=1`, and `PYTHONDONTWRITEBYTECODE=1`.
The control suite used a separate disposable HOME/TMPDIR and
`PYTHONPATH=scripts/initiative_control`. The new tests guard in-process TCP
connections against non-loopback destinations; CLI subprocess fixtures use only
explicit loopback endpoints. No real credential or profile was used.

```text
python -B -m unittest discover -v -s qa/initiative-control/management-bootstrap -p test_qualification_broker.py
python -B -m unittest discover -v -s scripts/initiative_control -p 'test_*.py'
```

- [First broker run](owner-brokerimpl-115-focused-first.txt): 18 tests,
  17 passed, 1 failed, 0 errors/skips, 1.438s, exit 1. Failure:
  `test_qualification_broker.BrokerTests.test_stream_is_incremental_exact_and_joined_to_upstream`.
  This caught a duplicate case-variant client-request-ID header; removing the
  inherited header before inserting the broker UUID fixed it.
- [Second broker run](owner-brokerimpl-115-focused-second.txt): 18 passed,
  1.449s, exit 0; failure/error/skip names: none.
- [Expanded broker run](owner-brokerimpl-115-focused-final.txt): 19 passed,
  1.707s, exit 0; failure/error/skip names: none. Preserved as executed before
  the final latch-lock placement and CLI-startup test.
- [Final-tree broker run](owner-brokerimpl-115-focused-verified.txt): **20 passed**,
  **1.804s**, exit **0**; failures **0**, errors **0**, skips **0**.
  Failure names: **none**.
- [Control regression suite](owner-brokerimpl-115-suite.txt): **880 passed**,
  **512.226s**, exit **0**; failures **0**, errors **0**, skips **0**.
  Failure names: **none**. Existing control sources were unchanged throughout
  the run; the broker's final focused checks completed while this suite ran.

There are **900 unique passing tests** across the final broker and regression
runs. Earlier broker reruns are not additional unique coverage. The regression
log contains existing fixture publication messages and ResourceWarnings; they
are retained without treating fixture publication as live activity. No native
credential prompt or live-profile read was observed. Final source/test hashes
and line counts above match the tested files; whitespace and writable-scope
checks passed. All ten new files are inside the authorized QA directory.

No formatter, Rust test, VM contact, live preflight, qualification, commit or
push was performed. The change is confined to the allowed QA directory.
Independent functional/TUI/live-repository/benchmark evidence is N/A for this
offline infrastructure implementation; the later exact-package broker and
guest functional qualification gate remains required. No independent review,
human acceptance or qualification status is asserted.
