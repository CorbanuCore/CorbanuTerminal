# RETURN — owner-brokerbound-119

Allocation digest: `daff61b0d8d6ccf6f26a4717f3b1be08e2e5a6eda0a8598bfb4e5a7ee6800de7`.
Claim: `e5c90681-fcd6-42b8-8c0c-864ae4225475`. Runtime: gpt-6-astra / high.
The frozen brief was read first after SHA-256 verification matched
`a3fd39388c434053e63696904e9d7965e23c56328b277c7ae658d06e0b7d0284`.
Initial checkout was clean at assigned base
`989f23eeb43bccfda5360398ad59062b78fa23aa`.

## Findings and changes

1. **P1, refresh amplification:** replaced the request-local-only bound with a
   serialized process-lifetime budget for each observed credential generation.
   The generation is a private in-memory SHA-256 of the canonical access token,
   refresh token, ID token and optional account ID. File formatting, last_refresh,
   unknown metadata, and replaying a previously observed generation cannot reset
   its budget. One attempt is spent before contacting OAuth, regardless of
   success/failure. A generated rotation is also marked spent and shares the
   source generation's budget; externally saving it cannot earn another grant.
   Successful rotations remain usable for ordinary inference.
   A refresh failure returns `subscription_refresh_failed` on that attempt;
   any later use of that failed generation refuses as
   **`subscription_refresh_exhausted`**. A further refresh need or second
   upstream 401 also latches this terminal refusal, recorded as its own journal
   outcome, HTTP 503. Later requests to a latched generation make no inference
   or refresh calls. A process-wide **300-second monotonic cooldown** additionally
   separates refresh attempts across externally supplied generations; refusal is
   `subscription_refresh_cooldown`, also journaled, without spending the new
   generation's budget. Deadline-bounded locking, no replay after admission,
   memory-only rotation and read-only credential files remain.
   The budget is per broker process, not restart-persistent; the worker cannot
   reset it through HTTP. Host process restart/file ownership stays external.
2. **P2, claim substring filtering:** removed recursive registration of arbitrary
   token-object strings and decoded claims. Substring secrets now consist only
   of full access/refresh/ID tokens and JWT payload/signature components with a
   **32-character minimum**. This is a length floor, not an entropy claim.
   Full credential fields shorter than 32 characters match exactly; public JWT
   header components are excluded. Explicit account ID, subject and email
   identities are a separate exact-match-only private set. There is no recursive
   claim blacklist. The same matcher guards request identifiers, upstream
   identifiers/model/effort, response routing headers and evidence scrubbing.
   Regression cases use `pro`, `high` and `fixture` in decoded claims and
   extra token-object strings: the model `gpt-pro-fixture` reaches upstream;
   attested model/effort remain intact with true comparisons; exact and embedded
   claim-probing identifiers all receive 200 and retain their intended values.
   Exact private identity/full credential exclusions still have negative tests.
   This remains a bounded metadata gate, not a classifier of opaque model output.
3. **P2, inference authority:** subscription HTTPS now accepts only the exact
   `https://chatgpt.com/backend-api/codex` URL, as strict as the existing refresh
   URL pin. Wrong host with the correct path, suffix-spoof host, alternate port
   and non-loopback HTTP are covered in
   `test_subscription_configuration_does_not_accept_other_bases_or_refresh_hosts`.
   Constructor refusal precedes credential reads/inference. Literal-loopback HTTP
   fixtures remain supported. The existing API-key upstream contract remains.
4. **P3, audit identity:** startup identity now includes `auth_mode` plus upstream
   `scheme`, `host`, `port` and `base_path`. Turn records retain the same
   broker identity, making subscription/API-key evidence distinguishable.
   No credential path, token, account ID or credential-generation hash is added.
   Both auth modes have startup identity assertions.
5. **P3, account identity:** missing/null account_id is accepted and the account
   header omitted, including after refresh, matching the pinned client's optional
   header at `codex-rs/model-provider/src/bearer_auth_provider.rs:38-42`.
   Malformed present values still refuse. Account consistency is checked when a
   supplied account is known; worker-supplied account headers never substitute.
6. **Headers:** the allowlist now preserves supplied window ID, opaque attestation
   and responses-lite headers on Responses and compact, without minting them.
   The loopback suite verifies exact values, omission when absent, unchanged request
   bodies and absence of attestation values from the journal.
   [Source-cited analysis](owner-brokerbound-119-source-facts.md) distinguishes:
   - window ID: native compatibility/correlation header; forward the client's ID;
   - attestation: optional host-generated value, gated off for synthetic API-key
     auth; client paths explicitly permit omission. Forward only a supplied value.
     This broker cannot create a valid attestation if the real server requires one;
   - responses-lite: `true` for model metadata selecting lite body semantics;
     preserve the header/body pair. Normal non-lite requests omit it.

   Real server enforcement and the precise response to a missing header are not
   established by client source or fake HTTP tests. No live backend probe is
   authorized here. A mandatory real attestation requirement would block this
   synthetic-key fronting arrangement with the current broker. No such requirement
   has been proved, and backend acceptance is not claimed.

## Validation

Read `docs/development/test-isolation.md` before tests. Built a fresh disposable
venv under `env -i` from `scripts/initiative_control/requirements.txt`, using only
the pinned local wheels with `--no-index`.
[Environment log](owner-brokerbound-119-venv.txt);
[disposable root](owner-brokerbound-119-test-root.txt).
Test commands run from the repository root with fixed PATH, disposable HOME and
TMPDIR, all three profile aliases pinned to disposable state,
`CORBANU_TEST_NO_NATIVE_KEYRING=1`, and `PYTHONDONTWRITEBYTECODE=1`.
The control suite receives a separate disposable HOME/TMPDIR/profile and
`PYTHONPATH=scripts/initiative_control`. Broker networking is loopback-only and
all credentials/identities are invented.

Required order:

```text
python -B -m unittest discover -v -s qa/initiative-control/management-bootstrap -p test_qualification_broker.py
python -B -m unittest discover -v -s scripts/initiative_control -p 'test_*.py'
```

- [Broker suite, first attempt](owner-brokerbound-119-focused-first.txt):
  **62 passed**, 251.854 seconds, exit 0. **0 failures, 0 errors, 0 skips**;
  failure/error names: none. Includes 36 existing API-key tests and 26 subscription
  tests (nine new methods). Both original real silent-response tests ran unchanged
  (181-second compact and 61-second SSE).
- [Full initiative-control suite, first attempt](owner-brokerbound-119-suite.txt):
  **880 tests, 878 passed, 2 failures, 0 errors, 0 skips**, 580.081 seconds, exit 1.
  Failure names:
  `test_fable_launcher.RealTmux.test_auth_failure_inactive_provider_and_stale_screen`
  (expected provider_auth_failure, observed timeout at auth_stage 6);
  `test_fable_launcher.RealTmux.test_forced_cleanup_stops_owned_descendant`
  (one owned PID observed, expected at least two). These files and all of
  `scripts/initiative_control/` are unchanged from the assigned base. Their
  fixture has a four-second startup/action budget. The unchanged passing replay
  below supports timing sensitivity; the first full run remains failed. Raw failure receipts
  and seven nonfatal HTTPError/SQLite ResourceWarnings remain in the log
  (nine ResourceWarning lines include two allocation-traceback advisory lines).
- [Unchanged isolated replay](owner-brokerbound-119-replay.txt): **2 passed**,
  26.776 seconds, exit 0; 0 failures, errors or skips; failure names: none.
  This supports timing sensitivity but does not erase or clear the failed full run.
- [Complete full-suite retry](owner-brokerbound-119-suite-retry.txt):
  **880 passed**, 521.691 seconds, exit 0; **0 failures, 0 errors, 0 skips**;
  failure/error names: none. Both first-run failing methods passed within this
  full run. Seven nonfatal HTTPError/SQLite ResourceWarnings are retained
  (nine ResourceWarning lines, including two advisory lines).
- Final coverage: **942 distinct passing tests** (62 broker + 880 control).
  First-attempt failures remain the two named above; no test expectations, fixture
  timeouts, source files or broker tests were changed to obtain replay/retry passes.
- No source or test edits followed the start of the broker run.
  Source SHA-256:
  `31ec9c8f1e2de7155034b91ac04ccf3286a0a1e936c22a90b2186ed257766130`.
  Test SHA-256:
  `d3fc9db58fd19f460165e8ce4c607b5e9a5d7de22400f7fd22cf5ce96a328e6d`.

## Scope and policy

This is the explicitly allocated offline credential-boundary revision within
PF-80 management-bootstrap QA infrastructure, conservatively **product-initiative**
work under the existing active plan. Product citation: **Internal delivery control
— TO BUILD**, `docs/corbanu-product-spec.md:488-497`: “durable event dispatch,
acknowledgments and watchdog”; “authorizes the bootstrap and bounded live
qualification, not premature product sprint resumption, new product scope,
main/release or financial actions.”

Plan: `docs/plans/active/initiative-delivery-control.md`; sprint: PF-80-S01,
`in_progress`. The explicit frozen worker allocation supplies this worktree/base
and write boundary. The manager owns plan/sprint bookkeeping; existing plan/sprint
base coordinates are older and are not claimed to match this allocation's base.
`python3 docs/sprints/check.py` passed (115 current, 127 archived).

Offline implementation return only. No guest package, TUI, VM, real credential,
live journal, schedule, transport, coordinator or qualification was exercised.
Internal-stage functional N/A is proposed for integrator acceptance; the later
independent exact-package broker/guest functional gate remains mandatory.
No human acceptance, release readiness, benchmark completion or real backend
acceptance is claimed. No Rust tests, workspace-wide formatter, commit or push.

## Changed files

All paths are under `qa/initiative-control/management-bootstrap/`.

| File | Final lines | Delta from assigned base |
| --- | ---: | ---: |
| [qualification_broker.py](qualification_broker.py) | 916 | +103 / -42 |
| [test_qualification_broker.py](test_qualification_broker.py) | 1221 | +180 / -6 |
| [owner-brokerbound-119-source-facts.md](owner-brokerbound-119-source-facts.md) | 105 | new |
| [owner-brokerbound-119-focused-first.txt](owner-brokerbound-119-focused-first.txt) | 67 | new |
| [owner-brokerbound-119-suite.txt](owner-brokerbound-119-suite.txt) | 960 | new |
| [owner-brokerbound-119-replay.txt](owner-brokerbound-119-replay.txt) | 7 | new |
| [owner-brokerbound-119-suite-retry.txt](owner-brokerbound-119-suite-retry.txt) | 939 | new |
| [owner-brokerbound-119-test-root.txt](owner-brokerbound-119-test-root.txt) | 1 | new |
| [owner-brokerbound-119-venv.txt](owner-brokerbound-119-venv.txt) | 7 | new |
| [owner-brokerbound-119-return.md](owner-brokerbound-119-return.md) | 179 | new |

Final `git diff --check` passes. `git status --short` contains only the ten files
listed above, all in the assigned QA directory. No commit or push was made.
