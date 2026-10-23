# Independent review of the PF-84-S06 slice B re-acceptance (2026-10-13)

**Verdict: DISAGREE**

I can't agree with the report, because `README.md` does not exist. At 01:54:43 MST the folder held only `captures/` (last write 01:42:21) and `scripts/` (last write 01:32). So there are no verdict tables or recommendation to check. Below I audited the captures and scripts directly and give the verdict each criterion's evidence supports. Any README written later has to match these or explain why not. I did not read `codex-rs/**`, root `scripts/**`, PR diffs or file contents under `.codex-work`. I did list the directory names under `.codex-work/s06bacc/homes` once; nothing below relies on that. I used `git log --oneline` titles only to place BASE (`82b1b2cf32`): it is after #543/#557 (slices A and B) and before #561/#566 (the #555 fixes).

## Findings

1. **Blocking: there is no README, so nothing can be audited.**
   - There are no verdicts, method or safety statement, no "Not exercised" list and no recommendation.
   - When it is written, its verdicts should not be higher than the table at the end of this review.

2. **Blocking: the #555 test homes don't match the frozen design, so the "only X is charged" checks can't detect the failure they are meant to catch.**
   - The frozen design gives H-ZAI three accounts "each with its own key" and asks that "provider-side usage for each key should agree" (`fix-555-20261012/code-blind/original-proposal.md:9,13`).
   - The run gave every Z.AI account the same real key (`scripts/z_setup.sh:6`: "all named zai accounts hold the same real key"). The default account got that same key through `ZAI_API_KEY` (`captures/tui-invocations.txt`, every `zt` launch says "real default key"). The Linux run did the same (`scripts/linux_glitch.sh:22`).
   - So a request sent with the wrong account's key would still succeed. The only evidence left is Corbanu's own `/status` and `/cost` attribution. That is the same kind of self-report that #571 showed can be wrong.
   - Affected: FK-01–04, FK-06, FK-08, FK-09, GH-01, MSG-02, PR-01, the D3 spawn (tB-14) and every Linux case.
   - The design change has no recorded disposition. Policy says the implementing agent can't drop part of a frozen case on its own.
   - RS-01, plus RS-04's o1 picker leg (resume only, no turn), do discriminate: o1's only Z.AI credential is `work`, and the TUI starts with no `ZAI_API_KEY` (`rs_suite.sh:18–20`, `nt`).
   - Fix: re-run the charge checks with fake keys for `personal` and `default`, so a wrong route fails with 401. The alternative is to label those checks "Corbanu attribution only".

3. **Blocking for an unqualified #555 handoff: the executor isolation required by the September 12 amendment isn't shown.**
   - Scripts run from inside the repository worktree (`env.sh:7`), reuse the 10-11 acceptance's mock and leak scan (`mock.py:2`, `leakscan.py:2`), and use a BASE build.
   - They fetch real credentials from the operator's vault (`env.sh:13`, `leak_run.py:6–13`), reach the network (real APIs, `ssh glitch`), and no negative access probes are recorded.
   - The report must say the isolation was instruction-only and must not claim the code-blind execution gate passed.

4. **Should-fix: ESC-03 pairs captures from two different runs, and the draft was never tested.**
   - `fk_suite.sh:17` saved ESC03-a, but `msg_suite.sh:12` later overwrote it from a different session in home z3. The current `555-ESC03-a` file (01:21) shows that session's picker ("first"/"ping").
   - `555-ESC03-b` (00:57) is from the FK session. It shows two identical `/status` cards, with no picker from that run surviving.
   - No draft was typed before `/resume`, so "the draft is still there" is untested.
   - Verdict: **PARTIAL**.

5. **Should-fix: raw attempts were overwritten without disclosure.**
   - `rs_fixups.sh:2`: "first captures were empty (snap line-count bug)". Nine cases were re-run.
   - The first FK/PR batch, on thread `01a13438-…`, was replaced by a second batch (`tui-invocations.txt`, lines 1–8 against 9–17).
   - ESC01-a (`rs_suite.sh` run) and ESC01-b (`rs_fixups` run) come from different processes.
   - Policy requires keeping raw attempts. At minimum, list each re-run and its reason.
   - This also explains the unexplained `Z.AI · account default · 1 request` in `555-FK09-spawn-and-cost.txt:196`: it is probably the discarded first-batch PR01d run. That needs to be stated.

6. **Should-fix: RS-04 is only partly exercised.**
   - In o1, after Enter in the picker, `/status` was run with no turn (`rs_fixups.sh:9`). Its token totals match RS-01's exactly.
   - So "the same as RS-01" covers routing on resume but not the charge. Verdict: **PARTIAL**.

7. **Should-fix: PR-02 is not a PASS.**
   - The design expects the Z.AI session to keep using `work` and to report `openai:x` clearly.
   - The candidate refuses the launch (`555-PR02-a-launch.txt`: "account `x` of provider `openai` is not signed in", EXITED-1).
   - That is clearly reported, but the session never runs. Record it as a deviation for product authority to accept.

8. **Should-fix: two REG-01 legs prove less than they seem to.**
   - **ESC-01 flag-off:** this leg never reached the picker. Both builds open the Z.AI key-entry onboarding straight away (`555-REG01-cand-ESC01-a-picker.txt:18,24`, which is not a picker). Builds match, but the Esc-in-picker path wasn't exercised with the flag off.
   - **RS-01 flag-off:** both builds show a raw `color-eyre` report with `Location: tui/src/app.rs:1042` (`…/1037`) and "Backtrace omitted" (`555-REG01-*-RS01-a-launch.txt:6`). It's a parity match, but it's a poor refusal and it predates this change. Record it as an observation or issue.

9. **Should-fix: REG-06 (Linux) is a subset.**
   - `linux-reg06.txt` covers FK-05, FK-06, GH-01, GH-03, GH-02/MSG-03 and ESC-01: 6 of the 24 blocker cases.
   - The binary is identified only as `corbanu 0.1.48` (line 3), not by commit or SHA-256.
   - Verdict: **PARTIAL**. Record the binary's provenance.

10. **Should-fix: the advisory REG cases need dispositions.**
    - **REG-02:** no fragment scan was done. `b-forced.txt:136` shows `sk-canar***y-S6B`, a masked part of the fake default key printed by `login status`. That is inherited behaviour, but "any part of" a key is the design's wording.
    - **REG-03:** the footer word is the reasoning effort, not the account (`555-FK02…:89` shows "via zai high" with `model_reasoning_effort="high"`). The footer never names the account, so "both name the same account" is unmet. Record a UX follow-up.
    - **REG-04:** the numbers don't reconcile. `555-FK05-cli-fork.txt:40,42` shows 13.6K total but `work` 7,417.
    - **REG-05:** the `Account:` line appears in `555-RS01-b…:33` but is missing in RS04-b and RS05 (resume with no turn).

11. **Should-fix: D3 needs a product decision.**
    - `tB-13…:50`: under Aggressive, a user's *Use for this session* switches to `work` with no prompt. The sprint says "Under Aggressive, switching account still needs approval".
    - The spawn prompt (tB-14) works, but only *Allow once* was tested; *Cancel* wasn't.
    - Don't mark D3 as passing until product authority confirms that D3 covers only agent-initiated switches.

12. **Should-fix: C7 has an unrecorded accounting failure.**
    - `tB-14-D3-b-after-allow.txt:53` shows "Developer accounting could not record one or more model requests in this turn … /cost does not include them."
    - This happened on a spawn onto another account. Investigate the log and file an issue. C7 can't be a clean PASS.

13. **Should-fix: C1 compares against a BASE that already contains S06.**
    - The CLI and TUI parity checks (`carry-flagoff.txt:115–147`, `tB-12`) only show that nothing changed since `82b1b2cf32`, which already includes slices A and B. They don't show "unchanged from before S06".
    - Cite the 10-11 run for that, or re-run against `3a0a38bed2`.
    - The BASE label disagrees between files: "(main before #561)" in `carry-flagoff.txt:1`, `b-forced.txt:1` and `tB-16:16`, against "#561/#566/#604/#610" in `env.sh:5`.

14. **Should-fix: B4 has places where it could be overstated.**
    - The heading in `b-cli.txt:223` says "default login still used", but the request went out with the `CODEX_API_KEY` decoy (`codexenv`, lines 225–228), not `deflt`'s login.
    - b-forced case 3 (a default session deletes `auth.json`) has no BASE control.
    - In `tB-16`, the default login is deleted on both CAND and BASE. That is inherited behaviour and must not be counted as #570 evidence.
    - The solid #570 evidence is CAND keeping the default against BASE deleting it in cases 1b, 2b, 5b and 7b (`b-forced.txt:19,33,60,83`), plus case 10/10b with decoys.

15. **Should-fix: several items must be listed as not exercised.**
    - **Refused-add revocation:** the duplicate-name add was refused *before* sign-in. The mock shows one `devz` sign-in and 0 revokes (`mock/mock-t.jsonl`), so the revoke path was never reached.
    - **Unreadable stores:** logout and remove on an unreadable stored sign-in.
    - **Browser sign-in.**
    - **Same-account concurrent processes (B2):** not re-run.
    - **Spawned agents under login restrictions.**
    - **`corbanu fork` under a forced login,** and **the running app-server daemon path** (both claimed in `docs/provider-accounts.md`).
    - **`corbanu fork` with a missing `[provider_accounts]` entry** (docs line 72).
    - **A same-provider spawn on a named ChatGPT account.**
    - **Connector auth.** The MCP server answers 404, and `/backend-api/codex/plugins/featured` is unauthenticated in named/Z.AI sessions (35 in mock-t, 13 in mock-b1, 16 in mock-b4). A default ChatGPT home sends its token (6 of 6 in mock-c). This still contradicts the docs' "connectors, plugin installs … use it".

16. **Should-fix: the C6 leak scan is incomplete.**
    - It ran at 01:38:10, before `tB-16` (01:41:20), `tB-01` (01:41:52) and the final copies of `tui-invocations.txt` and `mock/*` (01:42).
    - The named refresh-token needles (`leakscan.py:16–17`) miss `rt.work~2.`, `rt.deflt~2.`, `rt.play2~2.` and `rt.tuiwork~2.`. Those are the twin, dup, same-user re-login and twinz accounts.
    - It doesn't search for key fragments.
    - Re-run it after the final capture with these fixes.

17. **Safety: no real secret is in any file; the clean-up is incomplete (should-fix).**
    - A pattern scan of this folder found only fake canaries (`sk-canary-*`, `sk-fake-*`, `sk-envdecoy-*`, `sk-codexenv-*`).
    - Real keys entered only through the installed helper, into stdin, process environments or a tmux buffer deleted with `-d` (`msg_suite.sh`). Test binaries always ran with `CORBANU_TEST_NO_NATIVE_KEYRING=1`.
    - The real OpenAI, Z.AI, Kimi and Claude Plan credentials still sit in the disposable homes' vaults (r1, z1–z3, o1, n1). Each vault's fallback key file is next to it (layout in `b-cli.txt:67–70`). `real.sh` never removes r1. `linux_glitch.sh` leaves `~/corbanu-rtx/s06b-qa` in place until its next run. Shred these and record it.
    - The helper reading the operator's vault is allowed by vault policy, but it is the "real credential access" that finding 3 has to disclose.

18. **Nits.**
    - FK-07's refusal says "resumes on it" for a fork (`555-FK07-fork…:1`).
    - FK-03's edited branch shows no "Forked from" line (`555-FK03…:142` onwards).
    - `555-MSG01-b…:53` shows the composer as "Runs/review"; it looks like a stray keystroke.
    - Display nits from the 10-11 run are still present: V6 `/status` says "OpenAI API key configured" while the request used the ChatGPT login (`tB-15`); `tB-01` marks two providers "this session".
    - The CLI `account list` and `login status` show no fingerprint (`b-cli.txt:38–58`), so don't claim they do.

### Verdicts the captures support

| Item | Supportable verdict |
| --- | --- |
| B1 | PASS (keychain not verifiable; fix the scan, finding 16) |
| B2 | PASS: cross-process pickup and no reused-token rejections (`b-cli.txt:76–100`). Same-account concurrency not re-run. |
| B3 | PASS for CLI and TUI adds; refused-add revocation, unreadable stores and browser not exercised |
| B4 | PASS (with finding 14's caveats) |
| B5 | PASS (fingerprint stable across refresh in `tB-09`) |
| B6, B8, B9, B10 | PASS |
| B7 | PASS for session start; spawned agents, fork and daemon not exercised |
| B11 | PASS. #571 is fixed, and the mock now returns 401 to requests without credentials, so it can detect a regression (`tB-04`/`tB-05`). Connectors not verified. |
| B-live | NOT VERIFIABLE |
| C1 | PASS against BASE only (finding 13) |
| C2, C3, C4, C5 | PASS |
| C6 | INCOMPLETE |
| C7 | PASS with finding 12 |
| D3 | PARTIAL, product decision needed |
| A10 | PASS (#572) |
| #555: FK-05, FK-07, RS-01, RS-02, RS-03, GH-02, GH-03, MSG-01, MSG-03, MSG-04, ESC-01, ESC-02 | PASS |
| #555: FK-01–04, FK-06, FK-08, FK-09, GH-01, MSG-02, PR-01 | PASS on what `/status` shows; the charge check relies on Corbanu's own attribution (finding 2) |
| #555: RS-04, ESC-03, REG-06, REG-01 | PARTIAL |
| #555: ESC-04 | PASS, with no BASE comparison |
| #555: RS-05 | PASS (advisory) |
| #555: PR-02, REG-02–05 | Need a disposition |

The captures do support the targeted fixes: #570 (with BASE controls), #571 (with the mock now able to catch it), #572, and #573 (`tB-10-b`, `t1-setup.txt`). The core of slice B also looks sound at mock level. What the evidence doesn't yet support is an unqualified "slice B accepted" or "#555 cases passed". That needs:

- a README matching the verdicts above;
- the charge checks re-run with a distinct key per account, or plainly labelled as Corbanu attribution only;
- the isolation limit disclosed;
- the overwritten attempts and coverage gaps listed;
- D3 and PR-02 dispositions from product authority;
- the accounting warning investigated;
- the leak scan re-run;
- the real-key homes shredded.

Any handoff to human testing should carry these limits.
