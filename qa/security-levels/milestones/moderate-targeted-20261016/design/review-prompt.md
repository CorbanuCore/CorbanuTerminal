You are an independent, read-only reviewer of a test-design amendment for a code-blind functional milestone run of
Corbanu Terminal ("Moderate ships", targeted re-run). You have not seen the product's source or any test results, and
must not look for them. Work only from the files in the current directory; write nothing to disk.

- `decisions.md`: the owner's decisions this amendment must follow (verbatim) and the integrator's choices.
- `scope.md`: which 17 cases are re-run and why; the tester groups.
- `design-cases.md`: the frozen cases; `amendments-v2.md`: the amendments in force for the first run (the tester
  packet calls this file `amendments.md`).
- `amendments-v3-draft.md`: amendment v3, written by a fresh read-only designer from these files
  (`designer-output.md` is the designer's full reply, with a change table and the ambiguities it resolved). The
  integrator then made only mechanical edits for the tester packet, listed in `integrator-edits.diff`: screen numbers
  of this candidate are written "screens-v3 NN" (the packet keeps the first run's screens in `screens/` and this
  candidate's in `screens-v3/`, and numbers clash), the file name `amendments.md`, and no reference to files the
  testers will not have (`decisions.md`, `scope.md`).
- `tools.md`: the test machine; `screens/` (this candidate, see README.md; "screens-v3" in the amendment) and
  `screens-v1/` (the first run's design-time screens).
- `groups.json` and `standing-notes.json`: the tester groups and the integrator's navigation notes per group (the
  Q09 and Q11 notes clarify two first-run ambiguities: MOD-22's "repeat MOD-07 steps 2-3" and the picker's starting
  highlight and wrap-around).

The AI testers (GLM 5.3 Flash) will receive `design-cases.md`, `amendments.md` (= v2), `amendments-v3.md` (the
final version of the draft), both screen sets, neutral navigation, and one prompt per group with that group's note.
They will not receive `decisions.md` or `scope.md`.

Your task:
1. For each case section and rule in `amendments-v3-draft.md`: does it follow the decisions exactly, without loosening
   or tightening a case beyond what they allow, and without unrelated expectations? Is quoted old text exact? Check in
   particular MOD-01/22 against decision B; MOD-08's setup (protected launch off for one launch only) and whether it
   still tests what decision A(3) is about; MOD-06 against A(4) and D; MOD-26 against A(4) ("keep the .env prompt");
   MOD-29 against A(9); MOD-28 against A(11) (is an MCP "Allow for this session" a fair "session approval"; is the S2
   control sound; is the revoke-wording check right); MOD-10's narrowed warning wording against A(1); where decision C
   was and was not added; G-2's rewording; R-1 (the model for MOD-09/21).
2. Is amendment v3 self-contained and unambiguous for a tester who has only the tester packet (no decisions.md or
   scope.md)? Is the precedence (v3 over v2 over frozen) clear? Are the standing notes navigation only?
3. The designer flagged two items not covered by decisions (its ambiguity 10: MOD-09 item 7 and protected launch;
   MOD-07 not re-run although screens-v3 07 shows no "don't ask again"). Say whether anything must change in v3 for
   them (the scope and decisions bind; do not add re-runs).
4. Anything that would make a case fail or block for a reason unrelated to the product.

Be concise. Say "agree" per item or give exact replacement text. If anything must change, end with the complete final
amendment v3 document in one fenced ```markdown block (it will be frozen as-is), and one line: `VERDICT: APPROVE`
(freeze the draft as is) or `VERDICT: CHANGES` (freeze only your final version).
