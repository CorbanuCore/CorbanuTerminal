# RETURN — owner-brokerauth-118

Allocation digest: `fe79886245e6ffe812f2d505314676a77781cc02fb877a5f35a1c5a0a4c7464f`.
Claim: `ab0c5dc8-a00a-40e9-a5b3-353b421d7a02`. Runtime: gpt-6-astra / high.
Base verified: `7f9f4697fd47056d95e527e240ddc15315bbe82a`; initial tree clean.
Read the frozen brief first after SHA-256 verification matched
`98aadd6824bfb900e139bc8f9ea48958685200a616ae2a04f3c9e42c958aadd8`.

## Source facts established before implementation

[Full source-cited findings](owner-brokerauth-118-source-facts.md) cover stored
format, bearer/account identity, URL/path, originator/User-Agent, session/beta
headers, body/compression, expiration and refresh. Key conclusions:

- Subscription bearer is `tokens.access_token`; `tokens.account_id` supplies
  ChatGPT-Account-ID (`codex-rs/login/src/auth/manager.rs:554-582`;
  `codex-rs/model-provider/src/bearer_auth_provider.rs:32-46`). ID token supplies
  the conditional FedRAMP claim, not bearer authentication.
- Default subscription base is `https://chatgpt.com/backend-api/codex`, rather
  than API-key `/v1` (`codex-rs/model-provider-info/src/lib.rs:1003-1022`). Append
  `responses` or `responses/compact` (`codex-rs/codex-api/src/endpoint/responses.rs:117`;
  `codex-rs/codex-api/src/endpoint/compact.rs:35`).
- Originator defaults to `codex_cli_rs`; User-Agent is constructed by the client
  (`codex-rs/login/src/auth/default_client.rs:40,63-80,160-176,360-365`). Session
  and thread headers come from Responses metadata; optional subagent, turn and
  beta feature headers retain their existing derivation. HTTP has no mandatory
  WebSocket OpenAI-Beta header (`codex-rs/core/src/client.rs:2381-2409,2261-2287`).
  These are source-emitted headers; source alone cannot establish backend necessity.
- Access-token lifetime is server-defined by JWT `exp`, not a fixed source TTL.
  Proactive refresh begins within five minutes of expiry; eight days is only
  the client's fallback refresh interval when exp cannot be obtained
  (`codex-rs/login/src/auth/manager.rs:227-228,3200-3223`). Broker requires an
  explicit future exp instead of assuming validity from last_refresh.
- Refresh is JSON POST to `https://auth.openai.com/oauth/token`, client_id
  `app_EMoamEEZ73f0CkXaXp7hrann`, grant_type `refresh_token`, refresh_token
  (`codex-rs/login/src/auth/manager.rs:1903-1942,2000-2027`). Native client
  reloads matching-account credentials, then refreshes on 401, persists token
  rotation and reloads cache (`manager.rs:2290-2338,2991-3027,3288-3308`). Broker
  rotation deliberately remains memory-only to honor the supplied-file read-only
  constraint.
- The configured synthetic API-key worker's Responses JSON can pass unchanged;
  no auth-specific body rewrite is needed (`codex-rs/core/src/client.rs:1697-1765`).
  Native subscription compression is optional; synthetic API-key requests remain
  uncompressed (`client.rs:2965-2975`). No product-code change is needed for this
  HTTP-only route.

## Change and preserved guarantees

Only the existing QA broker/test module changes, plus this allocation's evidence.
`--auth-mode subscription` selects the supplied auth.json format and backend-api
path mapping; default `api-key` behavior remains. There is no profile lookup,
keychain access, credential write, environment-auth fallback, redirect following
or proxy discovery. CLI refresh authority is fixed to OpenAI; the constructor
accepts only literal-loopback HTTP fixture overrides.

The broker reads the source at each authenticated use, validates token shape and
future exp, builds bearer/account/FedRAMP headers from it, preserves incoming
originator/User-Agent, and substitutes a default originator only when absent.
A serialized memory cache retains OAuth rotations while the source file is
unchanged. Near-expiry tokens refresh before inference; a 401 permits one
same-account reload/refresh and retry before admission. Refresh failure latches
that source generation closed. Expired credentials refuse; a stream whose
credential expires truncates without refresh or replay. Unknown exp or missing
refresh material refuses. JWT parsing checks expiration/identity, not cryptographic
signatures; the upstream remains the bearer-token verifier.

Every prior guarantee is retained:

1. **Durable receipt before serving:** evidence opened before credential/network
   work, append/fsync admission before successful content; failure blocks serving.
2. **Upstream-only join fields:** x-request-id and response.id come exclusively
   from the accepted upstream response. Retry success joins the successful attempt;
   worker IDs stay under client_ids, and outbound X-Client-Request-Id stays the
   broker attempt UUID.
3. **Upstream-attested model and effort:** only upstream response fields populate
   model/effort, with separate declaration comparisons. Missing attestation stays
   null; the broker does not promote client claims into attestation.
4. **Terminal completion or truncation receipts:** relay_completed still requires
   terminal SSE completion (or complete unary body) and a durable terminal receipt;
   stream errors, EOF and expiration produce truncated. Crash/journal failure may
   leave an unfinished admission; that remains incomplete evidence, never success.
5. **Bounded pre-authentication aggregate:** the same single 1024-byte aggregate,
   exact count/saturation and failure latch apply before any credential read.
6. **Aligned deadlines:** 60s connection cap, 2400s total initial/unary response
   budget and 600s SSE data-event idle budget remain; OAuth gets an additional
   60s ceiling inside the existing turn budget. Retry does not reset the turn
   budget. Comments/partial events/HTTP framing cannot extend event deadlines.
7. **Credential secrecy:** fixed public errors, no upstream error-body relay,
   HTTP logging or exception tracebacks; credential fields, encoded JWT components
   and decoded identity strings are excluded from evidence. Successful response
   bodies remain opaque byte-preserving relays, as before; this is not a general
   secret-content classifier for model output.

## Tests and evidence

Read `docs/development/test-isolation.md` before any test. Created a fresh
disposable venv under `env -i` from `scripts/initiative_control/requirements.txt`,
using the local pinned wheels with `--no-index`. See
[venv log](owner-brokerauth-118-venv.txt) and
[disposable root](owner-brokerauth-118-test-root.txt). Test commands run from the
repository root, empty inherited environment, fixed PATH, disposable HOME/TMPDIR,
all three profile aliases pinned to the disposable profile,
`CORBANU_TEST_NO_NATIVE_KEYRING=1`, `PYTHONDONTWRITEBYTECODE=1`. Full control
suite uses its own disposable HOME/TMPDIR/profile and
`PYTHONPATH=scripts/initiative_control`. Broker fixtures reject off-loopback
connections; all credentials and identity claims are invented.

New subscription tests (17 methods; the original 36 API-key tests remain):

| Test suffix | Proof |
| --- | --- |
| subscription_routes_exact_headers_body_and_joinable_receipts | Both real-loopback path translations, bearer/account contract, originator/User-Agent/session/thread headers, unchanged bodies/response bytes, request UUID replacement, no forged account/cookie/FedRAMP/beta forwarding, joined admission/completion and model/effort. |
| expired_missing_exp_and_malformed_credentials_refuse_without_upstream | Expired, unknown-exp and malformed material refuses before inference or refresh. |
| file_is_reread_revocation_and_replacement_are_observed | Removed file refuses after success; supplied replacement is used on the next request. |
| proactive_refresh_is_serialized_memory_only_and_source_is_unchanged | Five concurrent requests cause one proactive refresh, all complete, source bytes/mtime stay unchanged, initial/rotated secret material stays out of journal/logs. |
| 401_refresh_retries_before_admission_and_preserves_request_body | One OAuth exchange/retry on 401, same inference body, successful-upstream join, only one admission/completion pair, unchanged file. |
| mid_run_refresh_failure_refuses_then_latches_until_file_changes | After a successful request, a continuation 401 plus failed refresh serves only a fixed error; successors fail closed until explicit source replacement. |
| refresh_bad_status_redirect_invalid_payload_account_and_expiry_fail_closed | Redirect, 500, invalid/missing response, wrong-account and expired refresh responses never reach inference. |
| refresh_deadline_fails_closed_without_inference | Silent OAuth response exceeds a shortened deadline, produces refusal and no inference. |
| refresh_lock_wait_respects_total_turn_deadline | Credential-lock contention is bounded by the total turn deadline, without credential/network work. |
| no_refresh_token_refuses_proactive_refresh_without_network | Missing refresh material refuses for both near-expiry and otherwise valid access tokens. |
| second_401_is_not_retried_or_served | At most one auth refresh/retry; a second 401 cannot produce admission or upstream content. |
| expiration_during_stream_truncates_without_refresh_or_replay | Expiry after admission closes an incomplete stream with a truncation receipt, without replay. |
| durable_receipt_failure_serves_nothing_for_subscription | Upstream success cannot release content when admission evidence cannot be written. |
| no_credential_fields_components_or_identity_in_journal_logs_or_errors | Full token fields, encoded JWT parts and decoded identity strings stay out of receipts, logs and public error bodies even when supplied as metadata or upstream errors. |
| fedramp_flag_comes_from_stored_id_token_only | Conditional FedRAMP header comes from credential claims. |
| preauth_does_not_open_subscription_file_or_refresh_and_stays_bounded | Missing/wrong/credential-valued synthetic marker never loads auth or refreshes; aggregate remains exactly 1024 bytes. |
| subscription_configuration_does_not_accept_other_bases_or_refresh_hosts | Wrong base paths, arbitrary refresh hosts and URL query injection are refused. |

Execution ledger:

- [First development run](owner-brokerauth-118-focused-first.txt): **51 tests,
  251.289s, exit 1; 50 passed, 1 failure, 0 errors, 0 skips.** Failure:
  `SubscriptionTests.test_no_credential_fields_components_or_identity_in_journal_logs_or_errors`.
  The test incorrectly demanded HTTP 400 for an overlength token placed in an
  ID header; the existing bounded identifier allowlist omits that field and
  safely permits the request (HTTP 200). The corrected assertion distinguishes
  omitted IDs from allowlisted secret IDs and still scans all credential pieces.
  The raw failure is retained. While its long regression cases were running,
  code review also bounded refresh-lock waits, rejected already-exhausted
  network deadlines, and added the two timeout methods. The first run's main
  module loaded the initial implementation (51 tests); this is development
  evidence, **not final-tree qualification**. Its traceback source display may
  reflect concurrently updated line numbers. Additional post-run hardening
  refuses missing refresh material even outside the five-minute window.
- Initial loaded module hashes: source
  `46a4a766368895100e5b1637a507e800a5704143d425bee94db7038ae066a6d6`;
  tests `8f7be39303f576c915fe3391ddcb0f60629f9c5d794703ba49629d9fac888ee8`.
- [Final broker run](owner-brokerauth-118-focused-final.txt): **53 passed,
  251.641s, exit 0; 0 failures, 0 errors, 0 skips.** Failure/error names: none.
  Includes the original 36 API-key tests and 17 subscription tests. Both real
  silent-response regressions executed unchanged (61s SSE and 181s compact).
- [Full control suite, first attempt](owner-brokerauth-118-suite.txt): **880
  passed, 506.658s, exit 0; 0 failures, 0 errors, 0 skips.** Failure/error names:
  none. The seven nonfatal ResourceWarnings (HTTPError cleanup and unclosed
  SQLite fixture connections) match the prior round-117 suite log and remain
  in the raw evidence. Publication messages concern synthetic test fixtures.
- Final coverage: **933 unique passing tests** (53 broker + 880 control).
  Only the one named development-run test failure occurred; it is not erased
  or relabeled as a pass. No Rust tests, native credential prompts or real
  inference were involved.

Commands, in required order for final evidence:

```text
python -B -m unittest discover -v -s qa/initiative-control/management-bootstrap -p test_qualification_broker.py
python -B -m unittest discover -v -s scripts/initiative_control -p 'test_*.py'
```

No source/test edits followed the start of the final broker run. Final module
hashes: source `a00fc9a02ea6fbed85d764914e6c496bc66053b6d40d32c3abf2e366754d8267`;
tests `7633fdaa478be6648791ab2b1a2d20fdd00423c42d6a62ce67730265f5672707`.


## Remaining real proof and smallest turn

No real credential, subscription backend, VM, tunnel, pinned guest binary or
qualification was exercised. Offline success does not prove subscription/model
entitlement, server acceptance of the exact binary's body/headers, actual token
TTL/rotation policy, real upstream request/response IDs or model/effort
attestation, or successful display in the TUI. Source inspection is of the
assigned checkout; the manager must retain the binary/source provenance link.
The known missing code-mode host remains outside this change.

Smallest authorized follow-up: use the manager's existing pinned candidate,
fresh isolated profile, exact OpenAI provider/model/effort pins and synthetic
marker through the already-owned reverse tunnel; start this host broker with
subscription mode and the manager-supplied credential path, then send one TUI
prompt, text and Enter separately: **"Reply exactly BROKER_ROUTE_OK. Do not use
tools."** Require the visible answer plus startup identity, durable admission,
real upstream x-request-id/response.id, attested requested model/effort, and
matching relay_completed. No invented or merely client-declared attestation
counts. This proves the basic inference route only; refresh, compaction,
tools and broker-down/no-fallback controls need their separate real cases.

Operational limitation: refresh-token rotations are retained only in memory.
Restarting after rotation requires the credential owner to supply current tokens;
the broker never persists them. It does not coordinate refresh with another
process using that file. A live owner/refresh contention failure must remain a
failure, not be retried by copying credentials or relaxing these rules.

## Scope and policy

This is an explicitly allocated authentication-capable **QA infrastructure
extension** under the existing PF-80 management bootstrap, not a shipped Terminal
feature or qualification approval. Conservatively, the root policy's
product-initiative classification applies to the credential boundary; no new
product goal or initiative is added. Product citation: **Internal delivery control
— TO BUILD**, `docs/corbanu-product-spec.md:488-497`: "durable event dispatch,
acknowledgments and watchdog"; "authorizes the bootstrap and bounded live
qualification, not premature product sprint resumption, new product scope,
main/release or financial actions."

Active plan: `docs/plans/active/initiative-delivery-control.md`; current sprint:
PF-80-S01, `in_progress`. The frozen manager allocation supplies this worker's
exact worktree/base/write scope. Plan/sprint bookkeeping is manager-owned and
outside the writable scope: their older coordinates are not claimed to equal
this allocation's base. `python3 docs/sprints/check.py` passed (115 current,
127 archived); that does not prove those allocation coordinates agree.

This is an internal offline implementation return. True-TUI, live-repository and
code-blind functional acceptance are not performed here; stage N/A remains for
integrator acceptance, with later exact-package broker/guest qualification still
mandatory. No shipped-feature documentation or release/benchmark claim is made.
No VM contact, live journal/schedule/transport/coordinator operation, native
credential access, real credential read/copy, push, formatter sweep or release.

## Changed files

All files below are under `qa/initiative-control/management-bootstrap/`.

| File | Final lines | Delta from base |
| --- | ---: | ---: |
| [qualification_broker.py](qualification_broker.py) | 855 | +252 / -39 |
| [test_qualification_broker.py](test_qualification_broker.py) | 1047 | +347 / -0 |
| [owner-brokerauth-118-source-facts.md](owner-brokerauth-118-source-facts.md) | 13 | new |
| [owner-brokerauth-118-focused-first.txt](owner-brokerauth-118-focused-first.txt) | 65 | new |
| [owner-brokerauth-118-focused-final.txt](owner-brokerauth-118-focused-final.txt) | 58 | new |
| [owner-brokerauth-118-suite.txt](owner-brokerauth-118-suite.txt) | 939 | new |
| [owner-brokerauth-118-test-root.txt](owner-brokerauth-118-test-root.txt) | 1 | new |
| [owner-brokerauth-118-venv.txt](owner-brokerauth-118-venv.txt) | 7 | new |
| [owner-brokerauth-118-return.md](owner-brokerauth-118-return.md) | 245 | new |

No commit or push was made. Final `git status --short` contains only these
assigned paths; `git diff --check` passes without rewriting any file.
