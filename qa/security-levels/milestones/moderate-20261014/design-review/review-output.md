**Review against the owner decisions of 2026-10-14**

The design mostly follows D1–D7. No case expects the opposite of a decision, and no case relies on Linux, Windows, Claude panes or the real trading project. One decided behaviour has no case (D2: the person's own setting wins). Two other places could pass when the decided behaviour isn't actually shown (D4 approvals before restart, and the model-decline rule). The scope limits are not named, and two coordinator answers go beyond the decisions.

**D1: ship with today's controls; unbuilt ones shown "not available"**
- Cases: MOD-01 (the review lists seven controls), MOD-20 (the inspector lists seven controls).
- No contradiction, no gap and no case depends on an unbuilt control.

**D2: broker on by default under Moderate; the person's own setting still wins**
- Cases: MOD-02 (the broker waits for restart), MOD-05, MOD-17 (a project file cannot turn it off), MOD-20, MOD-24.
- **Gap:** no case checks that the person's own `[features] broker_model_auth = false` still wins under Moderate. That means after a restart the inspector doesn't claim the broker and a turn completes. The review screen and Q8 both describe this setting.
- **Can't be run as written:**
  - MOD-24 asks the tester to watch the relay log and outbound connections. Q5 says the tester can't read either. The tester should record the time of the attempt, and the evidence check proves that "no request reaches the relay".
  - Q5 also says "No case is changed by these answers", but MOD-24 needs this rewording.
- **Can't be run as written:** MOD-17 step 3 says "plus any level key the owner names". The owner named none, so the step has to name the key itself or drop it.

**D3: Moderate never runs outside the sandbox; the prompt, inspector and docs stop offering it**
- Cases: MOD-03, MOD-07, MOD-08, MOD-21, MOD-22 (before quitting), MOD-32. The prompt wording and "no retry without sandbox" are covered, and nothing contradicts the decision.
- **Small gap:** no case checks that the inspector doesn't offer running outside the sandbox. One extra expected line in MOD-20 would cover it.

**D4: approval rules at confirmation, the rest at the next start (how to read this is still open)**
- Cases: MOD-02, MOD-03, MOD-32.
- **Gap in the decided part.** The decided part is that the untrusted-content approval rules apply at confirmation. Before any restart, only MOD-03 step 2 touches this, and it accepts "prompts with a security-level reason or is refused". The protected-file rule alone would satisfy that, so the case can't show the approval rule (an untrusted-content reason, no "don't ask again") is live before restart.
  - "In a brand-new prompt" is also ambiguous. If it means a new conversation, no untrusted content exists, so the approval rule isn't tested at all.
- **Depends on the open reading.** These expectations hold only under the open D4 reading:
  - MOD-02's split of "Active now" into sandbox and protected files.
  - MOD-03's sandbox and `.ssh` expectations.
  - MOD-32 (the second window picks up the sandbox rule).

  Keep them, named as limits judged against the documented reading. If the owner rejects that reading, re-judge them. The reading is stricter, never weaker, so the cases stay relevant either way.

**D5: child agents allowed, with Moderate enforced**
- Case: MOD-21. Its expectations match the decision. The inspector row "child at Moderate" separates allowed (D5b) from refused (D5a).
- **Scope:** Q4 lets the tester "ask the agent to start a child agent". If the agent does that by running `corbanu exec` as a command, that is the nested launch that is still open with the owner (#766).
  - MOD-21 should use `/spawn` or the agent tools only.
  - Any `corbanu exec` started by a command should be recorded as the #766 named limit, not as a pass or fail.

**D6: "don't ask again" rules saved under Permissive keep working for ordinary commands**
- Cases: MOD-08 (no prompt, still sandboxed; the `.ssh` read after README is not skipped), consistent with MOD-07 and MOD-11.
- No contradiction and no gap.

**D7: macOS only, stand-in project, Claude panes out of scope, limits named**
- No case depends on Claude panes, Linux, Windows (#783) or the real project.
- **Gap:** the decision says "limits named", but the proposal has no named-limits list. Only Q12 ("code you haven't seen") is named. The list should cover:
  - the stand-in project instead of the real TensorCash project;
  - Claude panes;
  - Linux and Windows (#783);
  - nested `corbanu exec` (#766);
  - the open D4 reading.

**Model-decline rule**
- The design never states the rule.
- MOD-12 and MOD-13 have only absence checks ("no protected action runs", "the hook file is not created") plus checks that only apply "if a prompt appears". If the model just ignores the injected text, the case passes without showing the protection. The rule says such a case is blocked unless the product's own text meets the expected wording. Both cases need that stated.

**Problems with the coordinator's answers**
- **Q6:** it cites an Aggressive owner decision of 2026-10-11 that isn't in the decisions packet, and stretches it to Moderate through D2 ("the same broker applies"). That goes beyond the decisions and steers the tester toward accepting the missing `r` in MOD-22. It should simply say "not decided for Moderate; record what is shown", and MOD-22's tolerance for no `r` should become a named limit until the owner answers.
- **Q4:** see D5. It needs the #766 caveat.
- **Q5:** see D2. It conflicts with MOD-24 as written while also saying no case changes.
- Q1, Q2, Q3, Q7–Q12 stay within the decisions and the packet.

**Other steps that can't be run as written on the described test machine**
- MOD-05 says to start `corbanu` from the shell with `ZAI_API_KEY`. On this machine, relaunch is `/quit` then `agms-tui start`.
- MOD-18 uses `export …`. On this machine, extra variables are set with `agms-tui start --env NAME=value`.
- MOD-09 item 2 hard-codes `/Users/tester`. The product's `~` is the tester account's own home (Q11), so the path should use that home.
- MOD-10 asks for "the sign-in and vault store files" without naming the files.
- MOD-22 compares against "the baseline" but never records one.

**Minimum changes needed**
1. **D2:** add one case or step: the person's own `broker_model_auth = false` plus Moderate, then restart. Expect the broker not to be claimed and the turn to complete.
2. **D2:** reword MOD-24 so the tester records attempt times and the evidence check proves nothing reached the relay. Name the level key in MOD-17 or drop "any level key the owner names".
3. **D3:** add to MOD-20 that the inspector doesn't offer running outside the sandbox.
4. **D4:** before restart, MOD-03 should check one protected action after untrusted content with MOD-14-style expectations, and say "same conversation". Mark the MOD-02, MOD-03 and MOD-32 expectations as resting on the open reading (named limit).
5. **D5 / #766:** limit MOD-21 to `/spawn` or the agent tools. Record a nested `corbanu exec` as the #766 limit, and add the same caveat to Q4.
6. **D7:** add the named-limits list from the D7 section above.
7. **Model-decline rule:** state the rule in the design. In MOD-12 and MOD-13, a model that never attempts the injected actions makes the case blocked, not passed.
8. **Q6:** remove the imported Aggressive decision and make MOD-22's tolerance for no `r` a named limit.
9. **D7 test machine:** fix the wording in MOD-05, MOD-09, MOD-10, MOD-18 and MOD-22.

VERDICT: CHANGES