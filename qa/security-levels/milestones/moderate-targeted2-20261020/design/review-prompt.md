You are an independent, read-only reviewer of a test-design amendment for a code-blind functional milestone run of
Corbanu Terminal ("Moderate ships", second targeted re-run). You have not seen the product's source or any test
results, and must not look for them. Work only from the files in the current directory; write nothing to disk.

- `decisions.md`: the owner's decisions this amendment must follow (verbatim) and the integrator's choices.
- `scope.md`: which 2 cases are re-run (MOD-15, MOD-28) and why.
- `design-cases.md`: the frozen cases; `amendments-v2.md`: the first run's amendments (the tester packet calls this
  file `amendments.md`); `amendments-v3.md`: the first targeted re-run's amendment, in force.
- `amendments-v4-draft.md`: amendment v4, written by a fresh read-only designer from these files
  (`designer-output.md` is its full reply, with a change table and the ambiguities it resolved). The integrator then
  made only the mechanical edits in `integrator-edits.diff` (the file name `amendments.md`; no reference to
  `decisions.md`, which testers will not have).
- `tools.md`: the test machine; `screens-v4/` (this candidate, see README.md) and `screens-v3/` (the previous
  candidate's design-time screens).
- `groups.json` and `standing-notes.json`: the tester groups (Q01 = MOD-15, Q02 = MOD-28) and per-group navigation
  notes (none).

The AI testers (GLM 5.3 Flash) will receive `design-cases.md`, `amendments.md` (= v2), `amendments-v3.md`,
`amendments-v4.md` (the final version of the draft), the first run's screens, screens-v3, screens-v4, neutral
navigation, and one prompt per group. They will not receive `decisions.md` or `scope.md`.

Your task:
1. For each rule and case section in `amendments-v4-draft.md`: does it follow the decisions exactly (D38 for MOD-15,
   D40 and the coordinator's brief for MOD-28), without loosening or tightening a case beyond what they allow, and
   without unrelated expectations? Is quoted old text exact? Check in particular: the R-2 change for MOD-15; where
   MOD-15's "message" may appear and that the model's reply does not count; the failed/blocked boundary for MOD-15;
   MOD-28's bounded search P1-P3 and its S2 control; the "accepted under D40" outcome and what is still judged in
   that branch; that the A(11) revoke wording is judged on screens-v4 06 when no session approval exists.
2. Is amendment v4 self-contained and unambiguous for a tester who has only the tester packet? Is the precedence
   (v4 over v3 over v2 over frozen) clear?
3. Anything that would make a case fail or block for a reason unrelated to the product.

Be concise. Say "agree" per item or give exact replacement text. If anything must change, end with the complete final
amendment v4 document in one fenced ```markdown block (it will be frozen as-is), and one line: `VERDICT: APPROVE`
(freeze the draft as is) or `VERDICT: CHANGES` (freeze only your final version).
