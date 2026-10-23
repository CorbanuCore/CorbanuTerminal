I recommend Travis accept S05 behind the off-by-default flag, but with conditions. The usage numbers are right; the rate-limit half is unproven. The evidence is merged as PR #483.

I tested current `origin/main` `d93f5e639d` (the S05 merge, #469) with three builds: a developer-accounting debug build, a `--release` build, and the build just before S05 (`bddf6292c4`) for comparison. I used real Z.AI (GLM 5.3 Flash), Kimi k3, Claude Plan and OpenAI gpt-5.4, through the CLI and the TUI via tmux. The run was source-blind (I read no implementation code), but it is not the full isolated code-blind VM run, so it doesn't tick the sprint's handoff-checker box.

## Verdicts

| # | Criterion | Verdict | Evidence |
|---|---|---|---|
| 1 | Named attempts record `{name, fingerprint}` at ledger format 4 | PASS | Z.AI, Kimi and Claude Plan attempts carry the account; format goes to 4 on the first named request; `/providers` shows the same fingerprint. |
| 2 | Fingerprint is salted per home | PASS | The same key gives a different fingerprint in two homes; it changes when the key is replaced and returns when it's restored; no key value is stored. |
| 3 | Default work unchanged, ledger never upgraded | PASS | A home made by the old build: no format change, no account field, old rows byte-identical, config and vault untouched. |
| 4 | Older ledgers still read | PASS | The new build reads old ledgers. The old build, on an upgraded ledger, says accounting is off and still answers. |
| 5a | Two accounts reconcile exactly with what the provider reported | PASS | I logged Z.AI's own usage on the wire, on both the built-in and a custom route. All 9 built-in conversations match exactly (coordinator, spawned agents, a cancelled request, compaction, resume, account switch). |
| 5b | Ledger matches each conversation's own log (a self-consistency check, not an independent one) | PASS | Z.AI, Kimi default/`alt`, Claude Plan, OpenAI. |
| 6 | `/cost` groups by provider · account · model, here and in other conversations | PASS | Coordinator on `default`, spawned agents on `work` (pong) and `fake` (401), CLI workers; per-account pages and request links work. |
| 7 | A fake-key account records the failure and no spend | PASS with #475 | One request, no tokens, no spend. But `/cost` calls it "usage incomplete — check the bill" and never shows the 401. |
| 8 | Rate-limit state is per account; B's 429 never blocks A | **PARTIAL** | Rows are kept per account. But no 429 cooldown or request lease was ever recorded, on any provider or build, including the pre-S05 one. `Retry-After: 120` is ignored. So A isn't blocked only because nothing blocks anything (#476, medium). |
| 9 | `/status` names the account | PASS | Shows `· account default` or `· account work`, in debug and release. |
| 10 | Switch, resume and `/compact` keep the account | PASS | All three stay on `work`, confirmed on the wire. |
| 11 | Flag off: nothing changes | PASS | Same as the pre-S05 build. |
| 12 | Existing single-account home carries over untouched | PASS | |
| 13 | Missing or wrong account fails, never falls back | PASS | `ghost` is refused in the CLI, at TUI launch and at spawn. `fake` gets a 401 even with a valid default key present. |
| 14 | No secret in the database, logs or conversation files | PASS | Real keys found only in OpenAI's `auth.json`, where `login --with-api-key` always stores them. |
| 15 | Release build matches the docs | PARTIAL | `/status` shows the account and `/cost` doesn't exist, as documented. But `corbanu exec` prints a bare 401 where the docs promise the account name and recovery step (#480). |
| 16 | Kimi, Claude Plan and OpenAI work | PASS | |
| 17 | 2-second fingerprint timeout | NOT VERIFIABLE | Can't be triggered from outside. |
| 18 | Rate-limit snapshots carry the account | N/A | Deferred to S06 by the sprint's own gate. |
| 19 | Claude panes bridge records no account | NOT VERIFIABLE | Needs a Claude Code login, which the safety rules kept out of this run. |

## Issues filed
- **#474 (low):** the first named-account request on a vault created before S05 rewrites the vault once. The gate says that read never writes. Two workers doing it at once still agreed, so no damage was seen.
- **#475 (low):** a 401 shows in `/cost` as a possible cost instead of a failed request.
- **#476 (medium):** cooldown and lease never engage. Separately, accounts are keyed by name, not fingerprint as the plan says; Travis should decide whether that's acceptable.
- **#480 (low):** `exec` gives no account name or recovery hint on a 401.
- **#481 (low):** a spawned agent that got a 401 is reported as "Completed".
- **#482 (low, pre-existing):** the default account's rate-limit key is an unsalted hash of the API key, so it's the same on every machine.

## Review and PR
One code-blind reviewer (Opus 5.5 High, read-only) returned ACCEPT WITH CHANGES with 12 findings. I acted on all of them: added the wire check on the built-in route, re-ran the cancelled-request case, added the switch/resume/compaction tests, scrubbed hashes from the evidence, and qualified the recommendation. They weren't re-reviewed after those changes.

PR #483 (docs-only) merged as `767d315c88` after all checks passed, with the head commit pinned. Scratch homes, builds, the comparison worktree and my own worktree are removed.

## Recommendation for Travis
Accept S05 behind the flag, but only if you also accept these:
- the unproven rate-limit behaviour (#476);
- name-based rather than fingerprint-based account keys;
- the three untestable or deferred items (rows 17–19);
- the other follow-up issues (#474, #475, #480, #481, #482).

The usage side is solid: counts match Z.AI's own figures exactly per account, old data is untouched, and turning the flag off changes nothing. Don't rely on per-account rate limiting until #476 is explained or fixed. The sprint's "B's 429 never blocks A" box is ticked but isn't really proven yet.