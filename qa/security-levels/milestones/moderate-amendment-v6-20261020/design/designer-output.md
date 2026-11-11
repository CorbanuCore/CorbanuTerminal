I've written amendment v6. It changes one thing in MOD-22: action 4 becomes `/security`, then `i` (the inspector), as D96 asks. A note says how the inspector screen is compared with the 'baseline' screen. No expected result and no rule changes.

**(a) Amendment v6**

```markdown
# Amendments to the Moderate design (amendments-v6, MOD-22 inspector step)

Applied on top of `design-cases.md` + `amendments-v2.md` (cited in later amendments as `amendments.md`) +
`amendments-v3.md` + `amendments-v4.md` + `amendments-v5.md`, for future runs of MOD-22 only. Precedence: this
amendment over amendments-v5, over amendments-v4, over amendments-v3, over amendments-v2, over the frozen cases. For
MOD-22 this amendment governs where it differs. Everything it does not mention stays as in force: the frozen text,
amendments-v2 (the 'baseline' screen in the starting conditions, action 3 and the 'restart now' named limit) and
amendments-v3 (action 5 and the expected results after the relaunch). Amendments v4 and v5 do not touch MOD-22.
No expected result of MOD-22 is replaced or added. Amendment v7 (D97: MOD-12, MOD-23, MOD-29) is separate. The two
do not overlap and neither depends on the other.

Decision label (listed in the integrator's record, not part of the tester's packet):
- **D96** (owner, 2026-10-20): "MOD-22 case text: add `/security`, `i` (inspector) after the relaunch step."

## Rules

- G-1, G-2, G-3, the named limits, R-1, R-2, R-3 and the rules of amendments-v4 and amendments-v5 are unchanged.
  Each still applies only to the cases it names. This amendment changes no rule.

## Case amendments

### MOD-22 (amended; D96)

- Action 4 (replace) "`/security`." with: `/security`. Record the picker screen (screen label 'after-relaunch
  picker'). Then press `i` to open the inspector. As for the baseline, read the approval, sandbox and network rows,
  scrolling if needed. Record the screen or screens that show them (screen label 'after-relaunch'; if more than one
  screen is needed, 'after-relaunch-2', 'after-relaunch-3', ...). Copy the approval, sandbox and network rows of
  both 'baseline' and 'after-relaunch' word for word into the record, side by side. Do not change anything in the
  picker or the inspector: no ↑/↓ and no Enter. Then press Esc, Esc to close the inspector and the picker. Check
  that the conversation is shown again before action 5.
- Note (D96): This step comes right after the relaunch (action 3) and before action 5's request. It is not a
  request, so amendments-v3's "After the relaunch, send only this request" still holds. The picker screen recorded
  first is the "/security picker in action 4" that amendments-v3's added expected result (B) refers to.
- Note (D96): The expected result "The person's own approval and sandbox settings match the baseline" (amendments-v3
  wording, unchanged) is judged by comparing screen 'after-relaunch' with screen 'baseline' (amendments-v2), row by
  row:
  - the approval row with the approval row;
  - the sandbox row with the sandbox row.
  A row matches when it shows the same setting values in both screens. Differences only in layout, line wrapping or
  scroll position do not count. Record any other difference in a row's text word for word.
- Note (D96): The network row is recorded in both screens and any difference is recorded. It is not part of this
  expected result and is not judged by it, unless the inspector shows network access inside the sandbox row; then it
  is compared as part of that row. Other inspector rows (saved and active level, untrusted content, MCP servers,
  recent denials) are not part of this comparison.
- Note (D96): If either screen does not show the approval or the sandbox row, record that. That row cannot be
  compared, and the expected result is judged as in force (G-1 and the existing judging); nothing else is added.
```

**(b) Changes**

| Case | Old text changed (quoted) | New text | Decision |
| --- | --- | --- | --- |
| MOD-22 | Action 4: "`/security`." | Record the picker ('after-relaunch picker'), press `i`, record the approval, sandbox and network rows ('after-relaunch'; more screens if scrolled), copy the rows of both 'baseline' and 'after-relaunch' word for word, change nothing in the picker or inspector, Esc, Esc, and check the conversation is shown before action 5 | D96 |
| MOD-22 | (none; note added) | The inspector step comes before action 5 and is not a request; the picker recorded first is the one v3's (B) result refers to | D96 |
| MOD-22 | (none; note added; the expected result "The person's own approval and sandbox settings match the baseline" is unchanged) | Compare 'after-relaunch' with 'baseline' row by row (approval, then sandbox); a row matches when its setting values are the same, ignoring layout, wrapping and scroll position | D96 |
| MOD-22 | (none; note added) | The network row is recorded but not judged, unless the inspector shows it inside the sandbox row; other rows are not compared | D96 |
| MOD-22 | (none; note added) | A row missing from either screen is recorded and judged as in force; nothing is added | D96 |

**(c) Ambiguities and how I resolved them**

1. **Where the step goes.** D96 says "after the relaunch step", which is action 3. Action 4 already opens `/security`, so I added `i` to action 4 instead of a new action. That avoids opening `/security` twice. The picker is recorded before `i` because amendments-v3's result (B) can be judged from "the /security picker in action 4".
2. **Before or after action 5.** Before, for four reasons:
   - D96 places it right after the relaunch, and the numbering already puts action 4 before action 5.
   - The 'baseline' screen was also taken before any request in its session, so the two screens are comparable.
   - Action 5 can bring an approval prompt the tester accepts, or a refusal. Reading the rows first shows "the person's own" settings as loaded at startup, not affected by anything in the session.
   - It does not conflict with v3's "send only this request", since opening the inspector is not a request.
3. **The network row.** The baseline records it, but the expected result names only approval and sandbox. Judging network would add an expectation, so it is recorded but not judged. The exception is when the inspector shows network access inside the sandbox row; then it is part of that row.
4. **What "match" means.** I compare the setting values each row shows; differences only in layout, wrapping or scroll position don't count. Other text differences are recorded word for word for the judge. The cases don't say whether a note that changes no value (for example, one naming a security level) counts as a mismatch. I didn't decide that, because deciding either way would tighten or loosen the result.
5. **The baseline procedure.** amendments-v2 already says to record the approval, sandbox and network rows, so its starting conditions are unchanged. If a row is missing from either screen, the existing judging applies (the Tart run's blocked outcome); no new rule is added.

DESIGN: COMPLETE