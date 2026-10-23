**Re-review of amendments-v1 and Q&A v2**

Eight of my nine changes are handled correctly and consistently with the owner decisions. Q&A v2 has no coaching left, and no new contradiction with D1–D7 or the model-decline rule. Three amended items still need small wording fixes before freezing: MOD-20, MOD-34 and MOD-18.

**Status of the nine changes**

| # | Change | Status |
|---|---|---|
| 1 | D2: new case for the person's own `broker_model_auth = false` | Mostly addressed by MOD-34, which tests the right thing and can be run (the config path matches the screens). Its wording can fail correct behaviour; see fix B. |
| 2 | D2: MOD-24 relay check and MOD-17 level key | Addressed. The tester records UTC times and the independent evidence check reads the relay. MOD-17 now names `[security] level`, which Q8 supports. |
| 3 | D3: MOD-20 inspector must not offer running outside the sandbox | Added, but the new wording conflicts with rows the design requires; see fix A. |
| 4 | D4: MOD-03 pre-restart approval check, same conversation; G-2 for MOD-02, MOD-03, MOD-32 | Addressed. One non-blocking note below. |
| 5 | D5 / #766: children only via `/spawn` or agent tools | Addressed by G-3, the MOD-21 amendment and Q4 v2. |
| 6 | D7: named-limits list | Addressed. It covers the stand-in project, Claude panes, macOS only (#783), #766, the D4 reading, the MOD-22 restart key, "code you haven't seen" and no internet. |
| 7 | Model-decline rule stated; MOD-12 and MOD-13 not passing on silence | Addressed. G-1 matches #447, and the new direct steps in MOD-12 and MOD-13 make the product show its own text. |
| 8 | Q6 and MOD-22 | Addressed. The off-packet Aggressive decision is gone, and the restart key in MOD-22 is recorded, not judged. MOD-22 now records a real baseline. |
| 9 | Steps that couldn't be run on the test machine | Fixed for MOD-05, MOD-09, MOD-10, MOD-16, MOD-22 and MOD-27. MOD-18's amendment adds a new problem; see fix C. |

**Problems introduced by the amendments**

- **A. MOD-20 (D3, #766).** The new line "No inspector row says or offers that agent commands can run outside the sandbox" can be read as catching rows the run must keep:
  - the "Not contained" rows (MCP servers, hooks, `!` shell commands, app-server command/exec), which the design requires to show "not contained";
  - the "Nested agents" row, which describes `corbanu exec` running at another home's level. That row is the #766 named limit, and G-3 says it is not judged.

  The line needs to say it excludes those rows.

- **B. MOD-34 (D2).** Two lines can fail correct behaviour:
  - "No screen claims model keys are held by the broker" would also catch the review and "Saved" screens. Those screens describe the broker with an honest condition attached ("unless your own config turns broker_model_auth off"). The check should apply only after the restart: the picker and the inspector.
  - "Moderate is … active" sits beside the bullet saying the docs expect `/security` to report the level as partial. Correct product behaviour could be failed on that. The line should accept "active or shown as partial".

- **C. MOD-18 (D4, test machine).** The parenthetical "(the restart keeps the same launch environment)" isn't among the test-machine facts. If `r` drops the extra variables, the case passes for the wrong reason: the variables are missing because the restart lost them, not because Moderate filtered them. D4 puts the environment rules at the next start. So the case should either:
  - start again with `/quit` and then `agms-tui start --env …` after Moderate is saved, or
  - make the parenthetical a stated test-machine fact.

**Non-blocking note**

- **MOD-03.** The "or refused with the security level named" option means a refusal leaves D4's approval-at-confirmation rule unshown. If that happens, the tester should note that it wasn't demonstrated.

**Minimum remaining changes**
1. **D3 / #766:** MOD-20's new line should exclude the "Not contained" rows and the "Nested agents" row.
2. **D2:** in MOD-34, apply "no screen claims" only to the picker and inspector after the restart, and accept "active or shown as partial".
3. **D4:** in MOD-18, start again with `/quit` then `agms-tui start --env …` once Moderate is saved, or state the restart's environment behaviour as a test-machine fact.

VERDICT: CHANGES