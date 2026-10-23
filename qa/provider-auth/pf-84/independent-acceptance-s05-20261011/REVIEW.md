<!-- Reviewer: installed `corbanu exec` (/Users/Neo/.local/bin/corbanu), -m claude-opus-5-5-plan, model_provider=claude-plan, model_reasoning_effort=high, -s read-only; code-blind prompt (allowed files listed; codex-rs/**, scripts/**, diffs, worker logs and other reviews excluded). It reviewed the first README draft. Output saved verbatim below; executor dispositions follow. -->

**Verdict: ACCEPT WITH CHANGES.**

The core S05 claims hold up. Named attempts record `{name, fingerprint}` at ledger format 4. Default-only ledgers keep their bytes and format. Flag-off behaviour matches the baseline. For Z.AI, the logging proxy shows the two accounts reconciling exactly. I recomputed the sums from the proxy log:
- coordinator: 9 replies, 79,880 tokens (78,662 prompt + 1,218 output, 63,424 cached);
- default worker: 4,659 + 4,761 = 9,420;
- work: 4,616 + 4,616 = 9,232.

The h1 rows also add up: work 4,616 + 7,425 + 7,417 + 7,614 = 27,072, and default 7,425 + 39,488 = 46,913.

Several verdicts are worded more strongly than the captures support, some gate claims were not tested, and the recommendation is not qualified.

## Findings

1. **Medium · Row 5 (reconciles "exactly with provider-reported usage") · the built-in routes are not independent.**
   - Evidence: only h2 uses a wire-level source, and that is the custom provider `zproxy`. For h1 (built-in Z.AI, Kimi) and hc (Claude Plan, OpenAI), "provider-reported" means the rollout's `token_count`. The candidate writes that file itself, so it is a self-consistency check, not an independent one.
   - Change: split row 5 into two claims:
     - independent wire match: Z.AI through `zproxy` only;
     - rollout equals ledger: built-in Z.AI, Kimi and Claude Plan.

     Optionally rerun built-in Z.AI through `mitmmock.py` in forwarding mode to get an independent figure for the built-in route.

2. **Medium · Row 5 · the interrupted attempt is called "matching", but the proxy cannot see it.**
   - Evidence: the README does not quote the script's verdict. `h2-proxy-reconcile.txt` prints `PROXY vs LEDGER exact per thread: MISMATCH` (10 attempts vs 9 replies).
   - In `usageproxy.py`, `forward()` calls `log(rec)` only after it has streamed the whole reply to the client. If the client aborts (Esc), the write fails first and nothing is logged.
   - So whatever the provider billed for that request is unknown. "Its tokens match" is only true because both sides are empty.
   - Change: quote the MISMATCH and explain it. Make the proxy log on exception, recording status, bytes relayed and any usage seen. Rerun the Esc case.

3. **Medium · Recommendation · it does not follow from the verdicts.**
   - Evidence: row 8 is PARTIAL against a sprint Done item that is ticked ("account B's 429 never blocks A"), and row 16 is NOT VERIFIABLE.
   - AGENTS.md says out-of-scope or not-verifiable dispositions need product-authority acceptance. The prior REVIEW (item 7) set the same precedent.
   - Change: reword to "limited acceptance behind the flag, conditional on Travis accepting the row 8 and row 16 dispositions and #474/#475/#476". Ask the lane owner to qualify the throttle Done item until #476 is resolved.

4. **Medium · Row 8 is mis-scoped, and #476 is under-evidenced.**
   - The plan says leases are "keyed by (provider, account fingerprint)", and the sprint says to "make that the account fingerprint for every auth kind". The candidate keys by `account:<name>` instead (gate review note 5). The README does not flag this departure from the plan.
   - "Baseline included" has no `provider_request_state` dump for the baseline. Only `mitm-probe-baseline.jsonl` exists.
   - The built-in route retried b's 429 five times in about 3 s, although the mock sent `Retry-After: 120` (`mock-cand-zai.jsonl`, `mitm-probe-baseline.jsonl`, `mitm-probe-z1302.jsonl`). This is not reported.
   - Change:
     - cite the name-vs-fingerprint keying as a product decision that needs Travis's acceptance;
     - add the baseline state dump;
     - add the Retry-After evidence to #476 and grade it at least Medium. It looks pre-existing, but it blocks a ticked S05 claim.

5. **Medium · Missing criteria from the gate and plan.** None of these were exercised or given a recorded disposition:
   - switching account mid-session in the TUI ("rebound on every turn"; "a sampling refuses a rebound mode on another account");
   - resuming a named-account thread and checking that later attempts keep the account;
   - `/compact` on a named account (review item 6c says no compaction test was written, so functional acceptance is the only coverage);
   - the 2 s fingerprint timeout and the name-only fallback;
   - exports, which row 13 claims are clean but were never produced;
   - rate-limit snapshots, which the sprint mandate covers but the gate defers to S06.

   Change: add the switch, resume and compaction cases. List the rest as N/A with reasons for product sign-off. Row 16 (Claude panes bridge) should be attempted or marked BLOCKED with a reason, not left NOT VERIFIABLE with no explanation.

6. **Medium · #474 grading and coverage.**
   - Evidence: the vault grows from 621 to 737 bytes on the first named use. This contradicts the gate's claim that the read "never creates a vault, key or salt" (`vault-write-on-first-named-use.txt`).
   - The vault is security-sensitive persistent state. Two cases are untested:
     - two workers making their first named request at the same time on a pre-S05 vault (risk of a race, a lost update, or divergent fingerprints);
     - the baseline using `--account work` after the rewrite. Only the baseline's default path ran, and that takes its key from the environment.
   - Change: raise #474 to Medium unless product says otherwise, and add both cases.

7. **Low–Medium · Row 14 ("matches the docs") is over-claimed.**
   - `docs/provider-accounts.md` says "A rejected key names the account and its `corbanu account add` recovery".
   - Every `fake` 401 captured is a bare "unexpected status 401 Unauthorized: Authentication Failed, url: … request id: …", with no account name and no recovery hint (`exec-h1-workers.txt`, `providers.txt`, `release-build.txt`).
   - Change: mark row 14 PARTIAL and file a doc-or-behaviour issue against S03. The README did not file this.

8. **Low · Unfiled defects listed only as observations.**
   - The spawn wait reports the 401 child as "Completed" (`tui-h1-spawn.txt`), for S03.
   - Sessions on `--account work` still show `via zai default` in the footer while `/status` says `account work` (`tui-h1-session-work.txt`, `tui-release-work.txt`). The prior REVIEW item 3 flagged the same thing. Link the existing issue or file one.
   - The default throttle key is an unsalted sha256 prefix of the key, which allows correlation across machines. It is pre-existing, but the README should file it rather than leave it as an observation.
   - `/cost` says "Enter open", but Enter does not open the top rows (`tui-h2-cost-enter.txt`).
   - State a severity for #475 and #476 in the README; it gives one only for #474.

9. **Low · Row 10 says the inputs were identical, but they weren't.**
   - In `tui-flagoff-cand.txt` the submitted prompt is "with exactly the word pong…" ("Reply" was dropped). The baseline got the full prompt.
   - `send_line` in `tmuxlib.sh` presses Enter even after both composer checks fail.
   - Change: make a failed composer check abort the run, rerun the candidate's flag-off session, and disclose the dropped keystrokes.

10. **Low · Wrong or imprecise numbers and citations.**
    - Row 1: `9ef7707e5fe8` is Z.AI `work` in h1 only. Kimi `alt` is `5a4334eda677` and Claude Plan `cp` is `bbf4a6e822ee`.
    - Row 5: "every /cost row equals the ledger sum" should cite `tui-h1-session-work.txt`, which shows the final state (work 19,458 + 7,614). `tui-h1-cost-overview-nav.txt` was captured before the last two threads ran.
    - Row 15: disclose that the first reconcile pass printed DIFF (Claude total 0; OpenAI prewarm), and that the script was then changed to normalise Anthropic usage and exclude the prewarm. The "provider reported" 9,465 prewarm tokens are unverified.
    - Row 12: cite the mock's per-account labels (b → 429, a → 200, default → 200) as the proof that each account's request carries that account's own key. The real-key proxy can't show this, because `work` and `default` share a bearer hash.

11. **Low · Method and hygiene.**
    - The TLS-intercepting mock logs about 4 TLS failures per process (`UNKNOWN_CA` / `BAD_CERTIFICATE`), which means connections to other hosts are failing. These are unexplained. Log the CONNECT host and disclose them.
    - The `README.md` links to a `REVIEW.md` that does not exist yet.
    - Real keys came from `corbanu.bak-20261010`, which reads the operator vault (this is disclosed). No owner authorisation is recorded, and the helper version is not recorded either.
    - Keyed tmux servers keep the key in their environment through `sleep 600`. Record that those servers were killed.
    - `env.sh` hard-codes `/Users/Neo/...` paths.

12. **Low · Secrets.**
    - I found no secret values in the evidence. The leak scan found 0 real-key hits outside the expected `auth.json`, and the synthetic canaries appear only in the scripts.
    - However, `auth_h` in `h2-proxy-requests.jsonl` is a 10-hex unsalted sha256 prefix of the real Z.AI key. The README itself points out that it matches the state DB's throttle key, which makes it a cross-machine correlator.
    - Change: replace it with an HMAC using a per-run salt in committed evidence, or drop it.
---

## Executor dispositions

| # | Disposition |
| --- | --- |
| 1 | **Done.** Row 5 is split. 5a (independent wire) now also covers the **built-in** Z.AI route: `mitmmock.py` in forwarding mode, h6, 9 threads with ALL MATCH. 5b is the rollout self-consistency check (h1, hc). |
| 2 | **Done.** The h2 `MISMATCH` is quoted and explained. The proxy now logs abandoned replies (`client_aborted`, bytes, usage seen). The Esc case was rerun on h6: `200/client-aborted`, no usage relayed, ledger attempt with no usage, tokens and attempt count match. README states that provider billing for an aborted reply is unknowable from the client. |
| 3 | **Done.** The recommendation is now "limited acceptance behind the flag", conditional on Travis accepting rows 8 and 17–19 and the follow-up issues. It asks the lane owner to qualify the ticked throttle item. |
| 4 | **Done.** Name-vs-fingerprint keying is flagged as a product decision (row 8 and #476). A baseline state dump and the Retry-After evidence (5 retries in about 3.5 s, both builds) are in `throttle-retry-after-base-vs-cand.txt`. #476 is raised to Medium. |
| 5 | **Done.** Added: row 10, covering the session switch (`/providers` → "Use for this session" starts a new session; no user path rebinds a running thread), resume and `/compact`, all wire-verified. Row 17 (2 s fingerprint timeout) is NOT VERIFIABLE with its reason. Row 18 (rate-limit snapshots) is N/A, deferred to S06 by the gate, and needs Travis's acceptance. Row 19 (Claude panes bridge) is NOT VERIFIABLE with its reason. "Exports" was removed from row 14. |
| 6 | **Partly done.** Added a concurrent first-use test (one consistent fingerprint, vault rewritten once) and the baseline on `--account work` after the rewrite (works). No divergence or loss was seen, so #474 stays **Low**. Both results are in a comment on #474. |
| 7 | **Done.** Row 15 is PARTIAL. The TUI names the rejected account and its recovery; `exec` prints a bare 401: filed as #480 (low, S03 area). |
| 8 | **Mostly done.** Filed #481 (spawn wait reports a 401 child as "Completed") and #482 (unsalted default throttle key). The footer `via zai default` is **not a defect**: it is the reasoning effort, as shown in the S01/S02 acceptance (`tui-status-line-effort-not-account.txt`); `/status` names the account. "Enter open" on the top `/cost` rows is recorded as a UX note in row 6 and not filed. Every issue now has a severity. |
| 9 | **Done.** `send_line` now aborts without Enter if the composer never shows the text. The candidate's flag-off session was rerun with the full prompt, and the dropped keystrokes are disclosed in Method. |
| 10 | **Done.** Fingerprints are listed per provider. Row 6 cites the final `tui-h1-session-work.txt` state. The first reconcile pass's DIFF (Anthropic dialect, OpenAI prewarm) is disclosed, and the raw output is kept in `providers.txt`. The prewarm's 9,465 tokens are excluded and not claimed as verified. Row 13 cites the mock's per-label log as per-account key proof. |
| 11 | **Done.** The TLS failures were the candidate's other hosts (`github.com`, `chatgpt.com`); they are now refused and logged via `MITM_HOSTS`. REVIEW.md exists. The owner authorisation (task brief) and helper version (`corbanu 0.1.48`) are recorded. Keyed tmux servers were all killed. Hard-coded paths are disclosed. |
| 12 | **Done.** The committed h2 proxy captures replace the unsalted `auth_h` with pseudonyms (`bearer-A/B`). The proxy now uses an HMAC under a per-run random salt (h6 log). |

Not re-reviewed after the changes; the dispositions above are the executor's.
