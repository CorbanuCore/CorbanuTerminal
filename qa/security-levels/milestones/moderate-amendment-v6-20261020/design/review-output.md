**1. Does it follow D96?** Mostly yes, with four fixes.

- **Old text and placement:** agree. The frozen action 4 is exactly "`/security`.", and the quote matches. The step sits after action 3 and before action 5. The note correctly says that opening the inspector is not a request, so amendments-v3's "send only this request" still holds. Tying the picker screen to v3's result (B) is right.
- **Row-by-row comparison and what "match" means:** agree. Comparing approval with approval and sandbox with sandbox, by the setting values shown, is what the expected result says ("settings match the baseline"). It neither loosens nor tightens the result.
- **Network row:** agree. Recording it without judging it is right, since the result names only approval and sandbox. Comparing it inside the sandbox row only when the inspector shows it there is right too.
- **Fix: scrolling vs. the key ban.** "Scrolling if needed" conflicts with "no ↑/↓ … in the picker or the inspector". The arrow keys are probably how the inspector scrolls, so a tester could either skip scrolling or break the rule. Ban ↑/↓ in the picker only; allow scrolling in the inspector; ban Enter in both.
- **Fix: Esc, Esc.** If one Esc already closes the whole popup, the second Esc reaches the empty composer. In Codex-based terminals, a second Esc there can open "edit previous message". Add: "if the conversation is already shown after the first Esc, do not press Esc again."
- **Fix: the missing-row note.** It cites "G-1". G-1 is the model-decline rule and has nothing to do with a row the inspector doesn't show, so a tester could misapply it. Drop the G-1 reference and keep only "record it; that row cannot be compared; nothing else is added."

**2. Is it self-contained, and is precedence clear?** Agree on both. The tester only needs the frozen MOD-22 and amendments v2 and v3, which are all in the packet.

- **Fix: screen label.** The label 'after-relaunch picker' has a space, so `agms-tui screen after-relaunch picker` may split it or fail. Use 'after-relaunch-picker'.
- **Clarification:** the comparison note should also count the continuation screens ('after-relaunch-2', and so on).

**3. Anything that could block or fail the case for reasons unrelated to the product?** Only the scrolling conflict and the extra Esc above. Nothing else.

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

- Action 4 (replace) "`/security`." with: `/security`. Record the picker screen (screen label
  'after-relaunch-picker'). Then press `i` to open the inspector. As for the baseline, read the approval, sandbox and
  network rows, scrolling the inspector if needed. Record the screen or screens that show them (screen label
  'after-relaunch'; if more than one screen is needed, 'after-relaunch-2', 'after-relaunch-3', ...). Copy the
  approval, sandbox and network rows of both 'baseline' and 'after-relaunch' word for word into the record, side by
  side. Change nothing: in the picker press no ↑/↓, and press no Enter in the picker or the inspector. Then press Esc
  to close the inspector and Esc to close the picker; if the conversation is already shown after the first Esc, do
  not press Esc again. Check that the conversation is shown before action 5.
- Note (D96): This step comes right after the relaunch (action 3) and before action 5's request. It is not a
  request, so amendments-v3's "After the relaunch, send only this request" still holds. Screen
  'after-relaunch-picker' is the "/security picker in action 4" that amendments-v3's added expected result (B)
  refers to.
- Note (D96): The expected result "The person's own approval and sandbox settings match the baseline" (amendments-v3
  wording, unchanged) is judged by comparing screen 'after-relaunch' (with any 'after-relaunch-2', ...) with screen
  'baseline' (amendments-v2), row by row:
  - the approval row with the approval row;
  - the sandbox row with the sandbox row.
  A row matches when it shows the same setting values in both screens. Differences only in layout, line wrapping or
  scroll position do not count. Record any other difference in a row's text word for word.
- Note (D96): The network row is recorded in both screens and any difference is recorded. It is not part of this
  expected result and is not judged by it, unless the inspector shows network access inside the sandbox row; then it
  is compared as part of that row. Other inspector rows (saved and active level, untrusted content, MCP servers,
  recent denials) are not part of this comparison.
- Note (D96): If either screen does not show the approval or the sandbox row, record which row is missing. That row
  cannot be compared; nothing else is added.
```

VERDICT: CHANGES