# Independent review of the PF-84-S06 acceptance

**Verdict: AGREE WITH CHANGES.** I agree with the conclusions: accept slice A behind the flag, and don't accept slice B until #570 and #571 are fixed. Several verdict rows still overstate their evidence, and one gap affects how much of #571 is S06's fault.

## Findings

1. **B7 is overstated, and #570 is wider than the report says (blocking for the B verdicts).**
   - The README says the API-key account `key` under `forced_login_method="chatgpt"` was "refused at thread start with recovery text", citing oa-cli. oa-cli shows something else: `ChatGPT login is required, but an API key is currently being used. Logging out.` That is the default login's check, set off by the `CODEX_API_KEY` decoy. A clean refusal appears only in oa-cli2 and in oa-forced case 2, and both of those deleted `auth.json`.
   - oa-forced case 7 is not just a side effect. A named API-key session failed on the default login's workspace: `current credentials belong to ws-main. Logging out.`, then `[exit 1]`.
   - So under login restrictions, the default login and environment keys decide whether a named session runs. That contradicts B4 ("never read") and weakens B9.
   - B7 should be FAIL/partial, tied to #570.

2. **#571 has no control, so it isn't attributed to S06 (should-fix).** The V5 control (a Z.AI session using the default account, then `/model` → OpenAI) aborted (`pick: /GPT-6 Luna/ not found`) and was never re-run. There was no baseline run either. Without it, we can't tell whether the missing Authorization header is S06's or inherited. #573 (ChatGPT sign-in disabled when the session runs on a non-OpenAI provider) may share the cause. Re-run V5, and the same route on the baseline build where it applies.

3. **The tui-17 `auth.json` check is broken (should-fix).** The capture shows `auth.json before/after:  /` and `ls: /auth.json: No such file or directory`, because the path variable was empty. The note in brackets ("absent … both before and after") is an assertion, not evidence. Fix the check or remove the claim.

4. **A decoy has the same value as a test profile (should-fix).**
   - The decoy is `AWS_PROFILE=other`. Accounts `alt` and `tuialt` both use profile `other`, so `AKIDOTHERPROFILE` can't tell "used the account's profile" apart from "used `AWS_PROFILE`".
   - The CLI runs are still sound, because `work` tells them apart. The restart proof in A9 / tui-18 (`AKIDOTHERPROFILE`) is not.
   - Repeat it with a third profile, or change the decoy.

5. **C2 is overstated (nit).** The snapshot excludes `sessions/`, `tmp/`, `*.sqlite*` and `*.lock` (see `snap()` in carry.sh). So it isn't "every non-log file". Reword.

6. **A4 is overstated (nit).** The `ghost`, `[provider_accounts]` `ghost` and `keyacct` refusals say "is not configured…". They don't contain "Corbanu never falls back". Also, `keyacct` is an account whose add was refused, not a broken profile.

7. **The leak scan misses the decoy values (should-fix).** It doesn't search for `sk-envdecoy-`, `sk-codexenv-`, `fake-env-session`, `fake-webid-token`, or the `CODEX_ACCESS_TOKEN` JWT. That JWT's header is `eyJhbGciOiAibm9uZSJ9`, not the scanned `…Is`. The TUI ran with `RUST_LOG=info`, which makes this gap matter more. Add them and re-run.

8. **Coverage is weak in several places (should-fix).** These are untested and not listed as gaps:
   - **TUI add rules (gate B3):** a TUI add must never overwrite an existing account, and a refused add must revoke its tokens.
   - **Unreadable sign-ins:** logout and remove should still work on an unreadable store.
   - **Resume:** resuming a thread recorded on a named Bedrock or OpenAI account.
   - **Same-provider spawns:** a spawned agent with a named ChatGPT `account` under restrictions (the gate covers "sessions and spawned agents").
   - **Fingerprint (gate B5):** whether it stays the same after a refresh.
   - **Rename hint:** the message when `[provider_accounts]` still names the old name.
   - **D3:** Aggressive mode still needing approval to switch accounts.
   - **Flag off in the TUI:** C1 was tested only from the CLI.
   - **TUI-only behaviour:** hiding ruled-out add methods, and the same-user warning in the TUI.

   List each one as not exercised.

9. **Auxiliary request credentials aren't analysed (should-fix).**
   - In named sessions, `/backend-api/codex/plugins/featured` is sent with no Authorization header (0 of 17 requests in mock-b). Default ChatGPT sessions send the token (8 of 8 in mock-c).
   - `plugins/suggested` does use the named token.
   - MCP `ps/mcp` is unauthenticated even for the default session, because the mock answers 404. So "connectors use the named account" can't be checked here.
   - Record this as an observation, compare it with the gate's known limits, and say connector auth was not verifiable.

10. **A missed display issue (nit).** In tui-7, tui-8 and tui-9 the footer shows `GPT-5.4 via amazon-bedrock default`, while `/status` shows `· account work`. Add it to the display nits.

11. **A10 is only weakly reproduced (nit).** tui-20 shows only the first entry of each `/model` tab. Capture the full lists, and say whether the picker hides Bedrock even on the baseline.

12. **Method soundness is acceptable.**
    - Each fallback source answers with its own key ID. Because environment keys and the bearer token come first in the default chain, any fallback would show `AKIDENVDECOY` or `envbearer`. That makes "0 IMDS/ECS calls" a secondary signal, not the main detector.
    - The real-key runs are well designed: a fake named key gets a 401 while the real key sits in the environment.
    - Mock artefacts:
      - The `5h limit` on Bedrock comes from the mock sending `x-codex-*` headers.
      - Requests with no auth get 200, so the #571 turns look successful. Real OpenAI would return 401.
      - The README should say both.

13. **Safety: no real secrets found.** Captures and scripts contain only fake canaries (`sk-canary-*`, `sk-fake-*`, `AKID*`, `fakesecret-*`). Real keys are passed only through the vault helper into stdin or the process environment.

14. **REVIEW.md doesn't exist yet (nit).** The README links to it, so the link is broken.

## Severities

- **#570 — raise from medium to high.** It deletes the user's default credential (an API key may exist only in `auth.json`). It breaks the B4 contract ("never … logged out by a named account"). It also aborts named sessions (case 7). It's limited by needing restriction settings and the flag. It blocks slice B either way.
- **#571 — medium, agreed.** No secret leaks, but `/status` and `/cost` attribute the turns to the wrong account. Severity and ownership depend on finding 2.
- **#572 — low, agreed.** It's a UX gap with a misleading confirmation message.
- **#573 — low, agreed.** It's inherited; link it to #571.

The recommendation stands: accept slice A behind the flag, and hold slice B.

## Required README edits

1. **B7:** change to FAIL/partial. Cite oa-cli `Logging out.` and oa-forced cases 2 and 7, and widen #570's description.
2. **B4/B9:** note that the default login and `CODEX_API_KEY` affect a named session's outcome under login restrictions.
3. **B11/#571:** state that the V5 control and a baseline run are missing, or add them.
4. **B1/B11:** remove or fix the tui-17 `auth.json` claim.
5. **A9:** acknowledge that the `other` decoy and profile coincide, or re-run.
6. **Wording:** fix C2 ("non-log, non-session, non-DB files") and A4 (the refusal text, and `keyacct`).
7. **C6:** add the decoy values to the scan and re-run.
8. **Untested items:** add a "Not exercised" list covering finding 8, and add the auxiliary-auth observations from finding 9.
9. **Display nits:** add the footer showing `default`.
10. **Links:** fix the REVIEW.md link once it exists.
---

_The review above was written by one code-blind reviewer: the installed `corbanu exec -m claude-opus-5-5-plan -c
model_provider="claude-plan" -c model_reasoning_effort="high" -s read-only`, limited to this directory, the S06 sprint
record, the gates and the docs._

## Executor dispositions

1. **B7.** Done. Now PARTIAL, citing oa-cli "…Logging out." and oa-forced cases 2 and 7. #570 was widened in a comment
   and raised to high. B4 now says that the default login and env keys decide the outcome under restrictions.
2. **#571 controls.** Done. V5 (default account via env key) and V6 (default ChatGPT `auth.json` login) both send the
   default credential on the same switch (tui-13). So the defect is in S06's named ChatGPT path; #571 has a comment. The
   baseline V6 run did not reach a model because of a picker mismatch, and it is reported as not used.
3. **tui-17 check.** Done. The broken line was removed, and a direct check was appended (0 `auth.json` in t1 before V6
   created a default login).
4. **Decoy and profile overlap.** Done. The restart was repeated with profile `third`, which no decoy names: it signed
   with `AKIDTHIRDPROFILE` and showed `· account third` (tui-21).
5. **C2 wording.** Done.
6. **A4 wording.** Done.
7. **Leak scan.** Done. It was re-run with `sk-envdecoy-`, `sk-codexenv-`, `fake-env-session`, `fake-webid-token` and the
   `CODEX_ACCESS_TOKEN` JWT header added. Those values appear only in my scripts, fake `HOME` files and launch log.
8. **Coverage.** Each item is listed under "Not exercised".
9. **Auxiliary auth.** Recorded under "Not exercised": `plugins/featured` was unauthenticated in every named-account
   home (b1 17/17, t1 19/19), the default-only home sent its token (8/8), and connector auth is not verified.
10. **Footer.** Added to the display nits.
11. **A10.** Full `/model` lists captured (tui-21). The baseline picker was not checked; this is stated.
12. **Mock artefacts.** Added as their own section.
13. No action needed.
14. Done (this file).
