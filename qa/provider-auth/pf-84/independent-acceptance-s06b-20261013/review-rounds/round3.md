# Independent review of the PF-84-S06 slice B re-acceptance (2026-10-13)

**Verdict: AGREE WITH CHANGES**

I read only the allowed files: the report folder, the sprint, `s06b-gate.md`, `docs/provider-accounts.md`, the plan's PF-84 decisions table, the 10-11 README/REVIEW, and the #555 `design.json` and `original-proposal.md`. I did not open `codex-rs/**`, root `scripts/**`, PR diffs or `.codex-work`. I modified no files. As a safety check I ran a pattern scan of this folder (Z.AI-style `hex32.alnum16`, long `sk-…`, `sk-ant-`, `AKIA…`). It found no real-looking key. The only `sk-` hits are the documented fake canaries.

Most rows match their captures. I confirmed these:

- **#570:** in `b-forced.txt`, CAND keeps `auth.json` in cases 1, 2, 5 and 7 (lines 10, 26, 51, 76). BASE deletes it in 1b, 2b, 5b and 7b (lines 19, 33, 60, 83).
- **#571:** `tB-04:33` and `tB-05:45` show `chatgpt tuiwork`. mock-t has no `/responses` request without credentials. All 64 credential-less requests across the mock logs are `plugins/featured`.
- **#572:** `tB-02:20–25` and `tB-03:28,36,56`.
- **#573:** `tB-10-b` and `t1-setup.txt`.
- **The z4 re-runs:** they discriminate as claimed. FK-01–05, FK-08, FK-09, RS-04 and PR-01b answered on `work`. FK-06 and PR-01a/c/d got Z.AI's 401.
- **GH-01:** the round-2 blocker is resolved. `555r3-GH01-config.txt:13` records `zai = "ghost"` for that exact run, and `555r3-GH01-ghost-resume.txt:40,48` shows `ghostok` on `account work`.
- **Count:** the 22-blocker count and the "19 pass" arithmetic are correct.

## Findings

### Blocking

1. **C6 says the leak scan was "the final run after all captures". It was not** (`README.md:104`).
   - `captures/leak-scan.txt` was written at 02:22:10. Every round-2 capture came later: `tB-19-*` at 02:34, `555r3-GH01-*` and `tB-10-i` at 02:34–02:35, and `tB-20-*` at 02:40–02:42.
   - Those runs used the real Z.AI key in z4 (`work`), z5 and z6, and the t1/t2 homes. They wrote new rollouts, logs and state DBs.
   - Clean-up at 09:44Z (02:44 MST) deleted those homes. So the round-2 home state was never scanned and now can't be.
   - My pattern scan of the folder found nothing, but it can't stand in for the real-value scan.
   - **Fix:**
     - Re-run `leak_run.py` over `captures/` and the surviving `logs/` with the real keys, fetched at use time.
     - Restate C6 as "PASS for everything up to 02:22. The round-2 captures were re-scanned. The round-2 home state was deleted unscanned."
     - Until then, C6 can't be an unqualified PASS of a security criterion.

### Should-fix

2. **B4 says "never read", but #658 shows the default login being read** (`README.md:89, 185–194`).
   - B4's wording is "The default login is never read or changed by a named account". The PASS rests only on `auth.json` staying byte-identical, which shows "not changed".
   - `tB-19-named-start-default-token-traffic.txt:2–12` shows a session started on named `work` sending the default `tuidefault` token for `ps/plugins/installed` (34), `ps/plugins/list` (5), `ps/plugins/suggested` (5), `plugins/featured`, `wham/settings/user` and `wham/rate-limit-reset-credits`.
   - That contradicts `docs/provider-accounts.md:162–170` ("never read … by a named account"; the session's account is used for "connectors, plugin installs, usage"). It also bears on gate `s06b-gate.md:133` ("Plugin installs read the active sign-in per use") and the sprint's checked Done item (`pf-84-s06…md:64`).
   - **Fix:**
     - Label B4 "PASS for *not changed*; *never read* contradicted for side requests (#658)".
     - Cross-reference #658 from B4.
     - Say in the recommendation that #658 may affect a checked Done item and a gate claim, not only a pre-flag-removal list. Plugin *listings* may not be *installs*, so state which reading you take.

3. **C7's Aggressive failure is overstated** (`README.md:105, 288–289`).
   - The Aggressive row says "every turn warns", and line 289 says accounting "drops every request there".
   - The evidence is 2 clean turns per build (`tB-20-*`), plus the degraded-state runs.
   - The warning itself says "one or more model requests" (`tB-20-cand-aggressive-default.txt:27`). The log shows an "admit attempt" failing and then "later requests in this turn are sent unrecorded" (`tB-20-*-accounting-log.txt:2–5`).
   - `/status` Account usage is still populated in those same captures (line 47), and `/cost` was never captured under Aggressive.
   - **Fix:** say "in every Aggressive turn tested (2 per build, default and named), at least one request went unrecorded; `/cost` under Aggressive not captured".

4. **The round-2 evidence can't be traced or reproduced from the folder.**
   - **Launch lines are missing.** `captures/tui-invocations.txt` (02:14, 112 lines) ends at the manual GH-01 run. It has no launch lines for the r3 GH-01 run, `tB-19`, or the `tB-20` CAND/BASE runs. Method (`README.md:69–70`) points to this file as the launch record.
   - **The #658 raw log isn't kept.** `captures/mock/mock-t.jsonl` (02:14) predates `tB-19`. So the #658 counts rest on a summary with no raw mock lines in the folder. `logs/` still exists, so the file can be refreshed.
   - **Reproduce is out of date** (`README.md:336–341`):
     - It lists `fk_suite2.sh`, whose GH-01 step produces the *discarded* result. Round 2's closing note on this is still unaddressed.
     - It omits `r3_suite.sh`.
     - It stops at "`tB-01` … `tB-16`", although `tB-17`–`tB-20` exist.

5. **The connector-traffic bullets contradict each other and `tB-19`** (`README.md:181–183, 195–197`).
   - Line 183 says "`ps/plugins/suggested` does use the named token". In `tB-19:6–7` it is 5 default against 2 named, so "partly" is the accurate word.
   - Line 197 says "`plugins/featured` is sent with no credentials in named-account homes". `tB-19:3` shows it with `tuidefault` in t2, a named-account home that also has a default ChatGPT login.
   - **Fix:** distinguish homes with and without a default ChatGPT login.

6. **The D3 row leaves out the state the *Cancel* leg ran in** (`README.md:106`).
   - `tB-14-D3-c:21–22` ran in "stored level unreadable … Aggressive enforced". The row doesn't say so; only Re-runs (line 223) does.
   - Under the plan's D3 row (#428, `unified-provider-auth.md:740`), that state counts as Aggressive, so the leg is valid. The row should still say so explicitly.
   - The "approvals stay on" claim (line 243) can cite `tB-14-D3-a:29`: "Profile corbanu-aggressive (workspace, untrusted)". Today it has no capture behind it.

7. **The recommendation and limits miss some items** (`README.md:269–277, 286`).
   - "Every other slice B criterion passes again" skips the NOT VERIFIABLE and not-exercised items: B-live, the B1 keychain, B3's browser flow, and B7's spawned agents, fork and daemon. Name them.
   - The tested artifacts were **debug builds copied without companions**. `tB-04:24` and `tB-05:36` show "Code Mode is unavailable because failed to spawn code-mode host". AGENTS.md requires the exact packaged candidate. Add this to the limited-testing statement, next to instruction-only isolation.

8. **Two universal claims rest on sampled checks.**
   - `README.md:43–44`: "Nothing in #604/#610/#561/#566 changed flag-off behaviour against BASE". Reword to "no flag-off difference was observed in the C1/C2/REG-01 checks".
   - `README.md:148` (PR-01): "the 401 names the default credential". `555r2-PR01d…:27` names no account ("Z.AI (zai) credential was rejected. Open /providers…"). Say it shows the unnamed-credential message.

### Nits

9. **A10** (`README.md:97`): stray ",." after "`GLM 5.2 via zai-anthropic`". Doubled spaces remain in the FK-06 (`:128`) and RS-04 (`:135`) cells ("both spellings:  ", "without a turn:  "). This is left over from round 2 #18.
10. **C7 citation placement** (`README.md:105`): `555-FK09` is cited after the OpenAI `default`/`fake` rows, but it shows only Z.AI rows (`:194–198`). Those OpenAI rows are in `tB-11:85–88`. Move the FK09 citation next to the Z.AI rows.
11. **ESC-03** (`README.md:146`): "`/resume` can only be typed into the composer, so a draft can't be kept" is the executor's inference. Record it as a design ambiguity for disposition, not as fact.
    - **ESC-01/02:** the captures show only `EXITED-0`, so "terminal restored" wasn't observed. A lingering onboarding screen would have kept the process alive, so a regression of that kind would have been caught.
12. **In-TUI rejection wording.** In z4 the named-account 401 inside the TUI points to `corbanu account add zai personal` (`555r2-FK06…:28`, `PR01a:26`). The unnamed one points to `/providers` (`PR01d:27`). This is outside MSG-01's missing-account scope, but it fits the #609/#588 wording family. Worth one line under Other observations.
13. **`tB-16:28`**: in the allowed `k2` block, the mock identified the bearer as `UNKNOWN`. That rules out the default login and the known decoys, but doesn't positively identify `k2`'s key. Phrase B4 accordingly.
14. **Clean-up** (`README.md:317–326`): the local `rm`/`ls` and the glitch `ls -d` are asserted, with no captured output. A two-line capture would close this.

### Round-1 and round-2 disposition check

| Item | Status |
| --- | --- |
| R1 #1–#15, #17 | Addressed, either directly or through R2's dispositions. R1 #17 (clean-up) is now recorded; see nit 14. |
| R1 #16 / R2 leak-scan timing | It ran after the round-1 captures, but not after the round-2 captures. See finding 1. |
| R2 #1 (recommendation) | Addressed: 22 blockers, conditional acceptance, code-blind item kept open. See finding 7 for the gaps. |
| R2 #2 (GH-01) | Addressed by `555r3-*`. The Reproduce part is not (finding 4). |
| R2 #3 ("D" labels) | Addressed. |
| R2 #4 (C7) | Log quote, degraded state, `tB-04` citation, scope wording and `tB-18-b` addressed. A new overstatement remains (finding 3). |
| R2 #5 (B1 TUI) | Addressed (`tB-10-i`). |
| R2 #6 (connector counts, default-token traffic) | Addressed: 35, plus #658 filed. The inconsistencies are in finding 5, and the raw log is missing (finding 4). |
| R2 #7 (B4 BASE controls) | Addressed. |
| R2 #8 (REG-02) | Addressed: NOT MET, disposition requested, mask blind spot disclosed. |
| R2 #9 (D3 approvals-off) | Listed in Not exercised. See finding 6. |
| R2 #10 (clean-up) | Done and recorded (nit 14). |
| R2 #11 (raw attempts) | Addressed. |
| R2 #12 (PR-02 source) | Addressed (`docs/provider-accounts.md:36–38`). |
| R2 #13–#17 | Addressed. |
| R2 #18 | Mostly addressed; the doubled spaces remain (nit 9). |

## Closing

The captures support the report's core conclusions:

- the four defects that held slice B (#570, #571, #572, #573) are fixed on `53c02c9301`;
- per-account storage, refresh write-back, env-key isolation and fail-closed refusal hold;
- most #555 routing is now checked at the provider rather than through Corbanu's own attribution.

The recommendation to accept slice B behind the flag, under an explicit limited-testing agreement, is reasonable. Before this README becomes the accepted record:

- correct the C6 leak-scan claim and re-scan the round-2 captures;
- qualify B4's "never read" against #658, and the Aggressive half of C7;
- refresh `tui-invocations.txt`, `mock-t.jsonl` and the Reproduce steps so the round-2 evidence can be traced;
- add the debug-build limit to the limited-testing statement.

Any human-test handoff should carry these limits forward: instruction-only isolation, debug builds, and the not-exercised list.
