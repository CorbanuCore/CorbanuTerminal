All six issues (#474, #475, #476, #480, #481, #482) are fixed in four PRs. All four are merged with every check green, and all six issues are closed with evidence comments.

**What was wrong and what changed**

| PR | Issues | Merge commit |
|---|---|---|
| #485 | #476, #482 | `fd8b751b2d` |
| #486 | #474 | `a8c10de4e0` |
| #489 | #480, #481 | `d467af6ea9` |
| #490 | #475 | `8ad3bbb6a8` |

- **#476 (throttle never recorded):** only requests holding a lease recorded their result. Leases are taken only by spawned sub-agents sending large uncached requests, so no `exec` or TUI request ever recorded a 429.
  - Every turn request now records its result. A 429 holds that key back in every session on the home for the provider's `Retry-After`.
  - No result shortens an active cooldown, and a burst of 429s counts once.
  - The lease trigger is unchanged and is now written down in the S05 gate.
- **#482 (unsalted default key):** the default account's throttle key is now salted per home, with the salt kept in the state DB. That way the request path never opens or creates a vault.
  - A migration deletes idle unsalted rows.
  - An unsalted row with an active cooldown or lease moves to the salted key the next time that key is used, so the throttle state is kept.
- **#474 (vault rewrite):** I took the "don't touch the vault" option.
  - Reading a fingerprint never writes; the salt is now created only when an account is added or renamed.
  - Trade-off: a vault whose accounts all predate S05 records those accounts by name only until its next `corbanu account add` or rename. The gate says so.
- **#480:** `corbanu exec` now prints the same account line and `corbanu account add` recovery as the TUI. On `exec resume` it uses the account the thread actually runs on.
- **#481:** a turn that ends on an error now gives the child an "errored" status, so spawn wait reports the 401 instead of "Completed".
- **#475:** a 401 or 403 before any response is now recorded as a refusal. `/cost` shows it as refused with no charge and zero tokens, with no "usage incomplete", "at least" total or "check the bill".
  - This moves the cost ledger to a new format (5), but only when the first refusal is written. After that, an older developer-accounting build sharing the home stops reading the ledger until it is updated, the same as with the named-account format.

**Gate evidence**
- **Focused tests**, with and without developer-accounting, on the final branches:
  - codex-state: all 416 pass.
  - codex-core throttle, account and accounting filters: all pass both ways.
  - codex-exec: 148 pass.
  - codex-vault: 71 pass.
  - codex-tui provider-health and `/cost` filters: all pass.
  - The new core 429 test fails on the old code.
- **Linux clippy `-D warnings` on glitch (`-j10`):** clean both ways on the four branches merged together.
- **Independent Opus 5.5 High review** (read-only, installed `corbanu exec`):
  - #486 approved as is; #489 and #490 approved with small fixes; #485 got "request changes".
  - The one blocking item: a busy state DB could delay every default-key request by 10–15 s. Legacy-row migration is now tried once per key and process, and pauses the DB on failure like the other steps.
  - I fixed the other recommended items: burst escalation, a leased success shortening a cooldown, leftover unsalted rows, the raw key in a `Debug` type, a missing exec wiring test, and refused requests with no price showing as "no price".
  - The fixes were not re-reviewed.
- **tmux runs** with the fresh debug binary, disposable homes and `CORBANU_TEST_NO_NATIVE_KEYRING=1`, rerun after the review fixes:
  - **Mock 429 server (#476/#482):** account `b`'s second request was blocked locally for about 120 s without reaching the server, while `a` and the default account answered pong. The same default key got a different salted key in each home, and a planted unsalted row moved over with its cooldown, leaving no unsalted value in the DB.
  - **GLM 5.3 Flash with a fake-key account (#480/#481/#475/#474):** exec printed the account line; the spawned child showed "Error - 401"; `/cost` showed "1 attempt refused by the provider (HTTP 401) — no charge"; `local.age` was byte-identical across requests.
  - A scan found the real key in 0 files.
- Captures and scripts are in `.codex-work/workers-20261002/pf84-fixes-474-482-evidence/`.

**Still open**
- Only the turn's own model requests check and record the throttle. Compaction, the turn-completion check and memory extraction still bypass it, as before. The gate says so.
- Z.AI's built-in transport retries a 429 five times within a few seconds, ignoring `Retry-After`; the cooldown starts only after those retries. This was not changed.
- Named accounts are still throttled by name rather than by key fingerprint, as the S05 acceptance noted. That is a product decision for Travis.
- The macOS credential canary failed three times on #489 and #490 in `pf_27_s04_pf_27_s01_externally_terminated_broker_is_detected_and_cleaned_up` (network-proxy, code I didn't touch). It passed on rerun. It looks like a timing flake but has no issue filed yet.

Cleanup is done: the worktree, local branches, Mac and glitch build folders, and the scratch homes are removed.