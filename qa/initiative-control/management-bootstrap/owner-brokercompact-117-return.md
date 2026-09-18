# RETURN — owner-brokercompact-117

Allocation digest: `7b8c5ab2ea05c5dbbcea4827a40386cd28085f5996056cbbb11183da61fc527a`.
Claim: `83b33226-2206-4973-ad53-911691aefdb8`. Runtime: gpt-6-astra / high.
Base: `fb1d2e8c387f27d62fbe9c89e629b34fec123317` (verified; initial tree clean).
The brief was read first, after `shasum -a 256` matched
`495ccb2b1a71bda0161929b63962cfb2ae9a9bc709d73295033fef10122ad1ce`.

Bounded repair of offline qualification infrastructure. Product citation:
**Internal delivery control — TO BUILD**, `docs/corbanu-product-spec.md:488-497`:
“durable event dispatch, acknowledgments and watchdog”; “authorizes the bootstrap
and bounded live qualification, not premature product sprint resumption, new
product scope, main/release or financial actions.” No product initiative or
sprint implementation is included.

## Findings and smallest repairs

1. **P2 — unary/compact response deadline.** Replace the 180-second read setting
   with the pinned client's 2400-second compact response budget. Reuse the raw
   deadline reader and install it before `HTTPResponse.begin()` parses the
   status/headers, through the connection's response factory. One monotonic
   deadline starts before connection setup and survives status, headers, body
   and HTTP framing; bytes do not renew a unary response's budget. Once SSE
   headers identify a stream, the same reader switches to the existing
   600-second per-data-event idle budget. Before response type is known, both
   routes use the conservative 2400-second ceiling; this also permits a JSON
   response regardless of the inbound stream flag. An expired unary budget
   yields HTTP 502 and one `turn` receipt with `outcome: transport_failed`,
   never `admitted` or `relay_completed`. No new dependency, authorization,
   credential source, routing, response transformation or evidence schema.

   Client source citations below refer to the pinned base above:

   | Deadline | Value and source |
   | --- | --- |
   | Connection establishment | Existing **60s broker cap**, unchanged. `codex-rs/http-client/src/client_builder.rs:85-88` defines connection-only timeout semantics; lines 312-321 leave the client's HTTP connect override unset. There is **no client source for a 60s default**; this is explicitly the existing harness policy, not a claimed client default. It does not constrain response silence. |
   | Initial response / unary reads | **2400s total** = 600 × 4. `codex-rs/model-provider-info/src/lib.rs:36,1087-1091` supplies the 600s default. `codex-rs/core/src/client.rs:209-211` defines multiplier 4 and full-response semantics; lines 1179-1191 calculate/pass the compact timeout. `codex-rs/codex-api/src/endpoint/compact.rs:46-57` makes a unary POST with `req.timeout`. `codex-rs/http-client/src/transport.rs:69-70,117-120` applies it and consumes the body. Applying the same ceiling before either route's response type is known is a broker policy, not a claim that the streaming endpoint has a 2400s client header timeout. |
   | SSE data-event idle | **600s**, unchanged. `codex-rs/model-provider-info/src/lib.rs:36,1087-1091`; `codex-rs/codex-api/src/sse/responses.rs:555-557` renews the timeout around each next event. Existing comment, partial-event and HTTP-framing trickle guards remain. |

   The old 180s value comes from the client's *actionable stream silence*
   setting (`codex-rs/model-provider-info/src/lib.rs:37,842-844,1094-1098`),
   which is not a compact or general response-read timeout. It is no longer
   used for these broker reads. The separate existing downstream request-read
   socket limit of 60s is unchanged; it is also broker policy, not an inference
   response deadline. These constants target the pinned default provider;
   custom timeout overrides are not discovered from live profiles.

   New cases:
   - `test_nonstream_compact_silent_past_old_180_seconds_completes`: actual
     loopback unary POST without a `stream` field, **181 real seconds** of no
     response bytes before status, then byte-exact JSON and relay completion.
     The production 2400s budget is not patched for this case.
   - `test_nonstream_status_headers_and_body_share_response_budget`: both
     allowed routes complete with independent pauses in all three read phases.
   - `test_exceeded_nonstream_deadline_records_transport_failure`: both routes
     exceed a shortened 0.3s budget at status, headers, body, or cumulatively
     across three 0.12s delays. Every case records transport failure, without a
     success receipt. The shortened failure tests avoid a 40-minute wait;
     they do not claim a real 2400-second overrun execution.

2. **P3 — misleading refusal test name.** Rename to
   `test_absent_wrong_and_credential_bearers_share_fixed_refusal_aggregate`.
   It now explicitly asserts exact counts 1/2/3, a single aggregate beside
   startup, and exactly 1024 bytes of growth for absent, wrong and
   credential-valued synthetic bearers; all receive the same refusal with no
   upstream request. Per-reason counters are not added: they have little
   forensic value for these uniformly unauthorized inputs and are unnecessary
   for this repair. Existing aggregate counts, timestamps and saturation
   semantics remain; no caller-controlled reason keys or secret comparisons
   are introduced.

3. **P3 — verification narrative.** Amend the original round-116 first-run
   bullet in [its return](owner-brokerfix-116-return.md): the 32-to-33 test
   increase included adding
   `test_terminal_evidence_failure_never_records_relay_success` between runs,
   alongside the closed-connection fixture fix. That test executed exactly
   once in round 116, in its final run. Round 117 reruns it as part of the
   complete broker module; original raw round-116 logs are unchanged.

## Verification

Read `docs/development/test-isolation.md` before testing. Build a fresh
venv under `env -i` from `scripts/initiative_control/requirements.txt`, using
only the pinned local wheels with `--no-index`. See
[venv log](owner-brokercompact-117-venv.txt) and
[disposable root](owner-brokercompact-117-test-root.txt).
Run from the repository root with an empty inherited environment, fixed PATH,
disposable HOME/TMPDIR and all three profile aliases pinned to the disposable
profile, `CORBANU_TEST_NO_NATIVE_KEYRING=1`, `PYTHONDONTWRITEBYTECODE=1`.
The full suite uses separate disposable HOME/TMPDIR/profile directories and
`PYTHONPATH=scripts/initiative_control`. Broker fixtures deny off-loopback
connections and use only invented credentials.

Commands, in required order:

```text
python -B -m unittest discover -v -s qa/initiative-control/management-bootstrap -p test_qualification_broker.py
python -B -m unittest discover -v -s scripts/initiative_control -p 'test_*.py'
```

- [Broker module, first attempt](owner-brokercompact-117-focused-first.txt):
  **36 passed**, **250.480s**, exit **0**; **0 failures**, **0 errors**,
  **0 skips**. Failure/error names: **none**. Includes all 33 prior cases
  (one renamed) plus three new deadline cases. The unchanged terminal-evidence
  test passed here, its second recorded execution across rounds 116 and 117.
- [Full control suite, first attempt](owner-brokercompact-117-suite.txt):
  **880 passed**, **515.094s**, exit **0**; **0 failures**, **0 errors**,
  **0 skips**. Failure/error names: **none**.
- Combined final coverage: **916 unique passing tests**. No round-117 test
  retries or first-attempt failures. Existing fixture publication/status
  messages in the raw suite log describe synthetic fixtures, not live actions.

No source/test changes followed the focused run. Source SHA-256:
`12f1fe833ba03341c412409709e12c342edf3ebdee4bc2dab0d38320f10eb03e`;
test SHA-256:
`04e248292b05d47b0a0b0fe1eb82e2a2666242514540ab020a0c0683e5828e24`.
Original round-116 failed and successful raw logs remain unchanged. Round 117
has no failed broker attempt to retry or replace.

## Changed files

All paths below are under `qa/initiative-control/management-bootstrap/`.

| File | Final lines | Delta from base |
| --- | ---: | ---: |
| [qualification_broker.py](qualification_broker.py) | 642 | +36 / -11 |
| [test_qualification_broker.py](test_qualification_broker.py) | 700 | +75 / -4 |
| [owner-brokerfix-116-return.md](owner-brokerfix-116-return.md) | 249 | +4 / -0 |
| [owner-brokercompact-117-focused-first.txt](owner-brokercompact-117-focused-first.txt) | 41 | new |
| [owner-brokercompact-117-suite.txt](owner-brokercompact-117-suite.txt) | 939 | new |
| [owner-brokercompact-117-test-root.txt](owner-brokercompact-117-test-root.txt) | 1 | new |
| [owner-brokercompact-117-venv.txt](owner-brokercompact-117-venv.txt) | 7 | new |
| [owner-brokercompact-117-return.md](owner-brokercompact-117-return.md) | 145 | new |

## Scope and handoff

No VM contact, live journal/schedule/transport/coordinator operation, live
preflight, qualification, native credential access, push or release was run.
Only assigned QA infrastructure/evidence files changed. No formatter was run;
`git diff --check` checks whitespace without rewriting other files.
This offline infrastructure repair is not a user-facing product change;
interactive, code-blind functional and live-repository acceptance are N/A for
this implementation stage, subject to integrator acceptance. Later exact-package
broker/guest functional qualification remains a separate mandatory gate; this
return does not claim qualification, human acceptance or release readiness.
