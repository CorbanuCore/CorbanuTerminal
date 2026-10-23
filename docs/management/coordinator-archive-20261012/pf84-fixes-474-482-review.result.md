I found one blocking issue, in #485. #486 can be approved as it is. #489 and #490 need small fixes. This was a read-only review: I built nothing and ran no tests, so every claim below comes from reading the code. Line numbers refer to the combined branch `tmp/pf84-fixes-combined`.

## PR #485: throttle fixes (#476, #482)

**Blocking**

- **B1. A busy database now delays every request on the default API-key account by about 10–15 s.**
  - `provider_request_key` (`core/src/session/turn.rs:3229-3242`) calls `adopt_legacy_provider_request_key` on every request.
  - That function (`state/src/runtime/provider_requests.rs:637-683`) opens a write transaction and retries for up to 10 s, plus SQLite's 5 s busy timeout.
  - It never checks `memory.paused()` or `provider_request_state_db_unready`, and it never pauses the database itself when it fails.
  - A failed adoption is not added to `adopted_legacy_keys`, so the next request waits again.
  - The result: while another process holds the write lock, every request on an environment-key account pays the full wait. This brings back the stall #351 fixed (the other throttle steps stop waiting for 60 s after one failure).
  - On a read-only database it also logs a warning on every request.
  - Fix: check the pause and feed failures into it, or run adoption in the background or with a short wait. Also mark the key as attempted after a non-busy error.

**Should-fix**

- **S1. Several 429s from one burst escalate the cooldown as if they were consecutive.**
  - `UnleasedRow::apply` (`provider_requests.rs:760`) adds one to `consecutive_429_count` for every 429, even when a cooldown is already running.
  - Unleased requests are not serialized, so N requests in flight that all get a 429 push the cooldown from 30 s to 60 s, 120 s and then 5 min from a single burst. The leased path could not do this, because only one lease holder records at a time.
  - Fix: when a cooldown is already running, keep the count and only extend the end time. No test covers this case.
- **S2. The reverse case: a leased result can still clear a cooldown set by an unleased 429.**
  - A leased success sets `cooldown_until_ms = 0` (`provider_requests.rs:303`); so does a leased non-429 failure (around line 350). The in-memory copy does the same (`provider_request_fallback.rs:217,247`).
  - So a lease taken just before a human session's 429, which then succeeds, ends that cooldown early.
  - Unleased results themselves are fine on this point: they never shorten a cooldown (`apply` takes the max and returns `None` while cooling, line 773) and never touch a lease.
- **S3. Some requests still never record a 429 or check the cooldown.** These paths call the model directly, outside `try_run_sampling_request`:
  - inline compaction (`compact.rs:780`)
  - remote compaction (`compact_remote_v2.rs:351`, `client.rs:1462`)
  - the turn-completion check (`turn_completion.rs:255`)
  - memory stage one (`memory_stage_one.rs:634`)

  The QA note's claim that "Every request now records it" is therefore wrong. Either fix the paths or narrow the wording.
- **S4. #482 is incomplete: unsalted, key-derived values can stay in the database.**
  - The migration's `DELETE` (`migrations/0053…sql:13-16`) only removes rows with no active cooldown or lease when it runs. Any key-derived row with an active cooldown or lease survives.
  - Those rows are adopted only if the same key is used again. Rows for keys that are retired or rotated keep the unsalted fingerprint forever.
  - An older build sharing the database (allowed by `ignore_missing: true`) keeps writing `env:` rows. The new build adopts only once per process, so those rows persist.
  - Fix: on each adoption, also delete any idle rows matching `key_fingerprint LIKE 'env:%'`.
  - Logs already stored by older builds may also hold the old `env:…:<hash>` values until log retention removes them.
- **S5. A type holding the raw API key derives `Debug`.**
  - `ProviderKeyIdentity::EnvSecret { secret: String }` (`turn.rs:3255-3259`) derives `Debug` and is not zeroized.
  - Nothing prints it today, but one `?identity` in a log line would leak the key. Redact `Debug` and wrap the secret in `Zeroizing`.

**Checked and correct**

- The `LIKE 'env:%'` pattern does not match the new `env-salted:` keys.
- The time units are right: milliseconds compared with `strftime('%s')*1000`.
- Adoption does `UPDATE OR IGNORE`, then `DELETE`, in one write transaction. When both a legacy and a salted row exist, the salted row wins.
- The salt is created with `INSERT OR IGNORE` and read back, so concurrent processes are safe. A failed read is not cached by the `OnceCell`, so it retries.
- The `env-salted:<ENV>:unavailable` fallback holds no secret.
- Skipping failures with no HTTP status is correct: interrupts and network errors say nothing about the key. Usage-limit 429s also map to no status and are skipped, which is the right outcome.
- The interrupt path writes to memory first. `or_cancel` picks randomly when the token is already cancelled, so a 429 that coincides with Esc may go unrecorded. That is acceptable.

**Nits**

- The comment "The DB's cooldown is at least the one recorded here" (`provider_request_fallback.rs:594`) is not always true. The in-memory count can be higher than the database's, so the database's cooldown can be shorter. That is harmless, because the database decides whenever it answers, but the comment should say so.
- Migration 0053 shares numbering with upstream Codex, so a future upstream merge could collide with it.
- The extra database write after each request is acceptable. One fewer write: skip a success when the row is already at count 0 with no cooldown.

**Tests**

- The core `provider_throttle` suite test would fail without the fix (a second request would reach the server). The fallback tests are good.
- Missing: the burst-escalation case (S1), the migration `DELETE`, adoption when a salted row already exists, adoption while the database is busy or read-only (B1), and the end-to-end salted environment-key path in `provider_request_key`.

**Verdict: request changes** (B1), then fix S1, S4 and S5.

## PR #486: vault salt (#474)

- The fingerprint read now only reads. `account_fingerprint_salt` uses only `secrets.get`, and a missing salt returns `None` (`vault/src/provider_accounts.rs:506`).
- The salt is written in the same batch as the account write in `store_provider_account`, which covers create, the CLI `account add`, and the token refresh in `login/.../provider_key_vault.rs`. Rename also adds it in its batch. Remove correctly does not.
- I found no other vault write reachable from a read path.
- Both callers treat `None` as "record by name only" (`core/src/accounting.rs:764`, `tui/src/provider_named_accounts.rs:146`).
- Once a salt exists, fingerprints stay stable; the test checks that later writes keep it.
- **Nit:** in a vault with no salt yet, the same account is recorded by name only until the next account write, then with a fingerprint. In `/cost` it could appear as two rows. This only affects vaults that never ran the earlier salt-on-read build.
- **Nit:** an account entry added by hand through `/vault` never creates the salt.
- The test snapshots the whole vault directory, so it would fail without the fix.

**Verdict: approve.**

## PR #489: rejected account and errored status (#480, #481)

**#480 (exec message)**

- Resume is correct: `ThreadResumeResponse.provider_account` is always set, including `<provider>:default` (`core/src/session/session.rs:244`), and `default` gives no extra line.
- `--account` reaches `model_providers[..].account` through config (`core/src/config/mod.rs:4283`), so new threads get the message too.
- The Claude Plan and auth-command recovery wording matches the TUI's, because both now share `named_account_rejection_message`.
- JSON mode is untouched: the trait method's default does nothing.
- **Should-fix (test gap):** only the helper is tested. Nothing covers the wiring, i.e. `process_server_notification` on a 401 turn and the resume `set_provider_account` call.

**#481 (status mapping)** — the change is in `core/src/agent/status.rs`.

- V2 parent notifications already reported `Errored` from `terminal_error` (`core/src/session/mod.rs:2302`), so they are unchanged.
- What changes:
  - V1 `wait` and its collab tool-call status now show `Failed` (`wait.rs:231`).
  - `close_agent` and residency treat `Errored` and `Completed` the same way, so they are unaffected.
  - App-server enrichment now shows a spawned child as `SystemError` instead of `Idle` until its next turn (`app-server/.../thread_enrichment.rs:63`). This is visible to clients and intended, but it belongs in the PR notes.
- **Nit:** `Errored` drops `last_agent_message`, so a child that wrote a partial answer before a terminal error loses it in the status.
- **Nit:** a withheld `TurnComplete` clears `error` (`security/disclosure_gate.rs:334`), so that rare path still reports `Completed(None)`.
- The test would fail without the fix.

**Verdict: approve with fixes** (add a wiring test).

## PR #490: refused attempts in cost (#475)

**Checked and correct**

- Every write path checks the ledger format. `append_on_connection` takes the highest format across the batch and the attempt, so a refusal upgrades the ledger to format 5 in the same transaction.
- Late import (`accounting_late_import.rs`) writes only compact aggregates and checks formats for price snapshots only. Refusals pass through `quote_observations` as zero usage, so this is safe.
- `skip_serializing_if` keeps the bytes of every existing patch and usage record; the store test checks this.
- Replay requires a refusal to be the only observation and to state nothing else (`accounting_types.rs:233-249`).
- Transport retries are safe. Unauthorized recovery builds a new `ResponseEvidence` on each pass of its loop (`client.rs:3148,3762,3984`), so binding the refused attempt cannot block the next attempt from being recorded.

**Should-fix**

- **S6. A refused attempt with no price record shows as "had no price" instead of $0.**
  - `quote_observations_under` prices zero counts only when a price snapshot was selected: `(Some(0), _) if selected.is_some()` (`accounting_pricing.rs:466`). Otherwise each bucket is marked missing a rate.
  - So on a provider with no declared pricing, a 401 shows "1 attempt had no price" and the day's total becomes unknown. `has_bill` only hides the bill hint.
  - Fix: price every bucket of a refused attempt at exactly $0. No recorded quote changes, because refusals only exist from format 5.
  - Add a test for this case; the current tests always bind a price.
- **S7. One 401 makes the ledger unreadable for older builds.** It moves the ledger to format 5, so any older build sharing the home refuses the whole ledger. This follows the existing format rules but is easy to trigger now. State it in the release notes.

**Nits**

- Once old records are compacted after the 90-day detail retention, the "refused" note disappears and the attempt shows as a plain 0-token request.
- Replay accepts any 4xx except 429, but only 401 and 403 are ever written. Either tighten the check or keep it as deliberate leeway.
- Is a 401 or 403 ever billed?
  - Before this change, a 401/403 reached the ledger as "incomplete usage" with no observation.
  - With `TransportError::Http`, the status line is the whole response and no usage was streamed, so recording zero is sound for OpenAI, Anthropic, Z.AI and Kimi.
  - The one doubt is gateways such as OpenRouter that return 403 for moderation-flagged input; their billing for that case is unconfirmed, and a short note in the code would help.

**TUI wording**: it is clear, and the test rules out misleading words such as "incomplete", "at least" and "check".

**Tests**: the three recovery suites and the store test changed from expecting no observation to expecting a refusal, so they would fail without the fix.

**Verdict: approve with fixes** (S6).

## Verdicts

| PR | Verdict |
| --- | --- |
| #485 throttle | Request changes (B1), plus S1, S4, S5 |
| #486 vault salt | Approve |
| #489 rejected account / errored status | Approve with fixes (wiring test) |
| #490 cost refused | Approve with fixes (S6) |