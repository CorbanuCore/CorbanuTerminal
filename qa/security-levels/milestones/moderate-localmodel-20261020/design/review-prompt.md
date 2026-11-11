You are an independent, read-only reviewer of a test-design amendment for a code-blind functional milestone run of
Corbanu Terminal ("Moderate ships", a follow-up re-run of MOD-09 and MOD-21 with the owner's local model as the
product's model). You have not seen the product's source or any test results, and must not look for them. Work only
from the files in the current directory; write nothing to disk.

- `decisions.md`: the owner's decisions this amendment must follow (verbatim) and the integrator's choices.
- `scope.md`: which 2 cases are re-run (MOD-09, MOD-21) and why.
- `design-cases.md`: the frozen cases; `amendments-v2.md`: the first run's amendments (the tester packet calls this
  file `amendments.md`); `amendments-v3.md` and `amendments-v4.md`: the targeted re-runs' amendments, in force.
- `amendments-v5-draft.md`: amendment v5, written by a fresh read-only designer from these files
  (`designer-output.md` is its full reply, with a change table and the ambiguities it resolved). The integrator then
  made only the mechanical edits in `integrator-edits.diff` (the group time limit; no reference to `decisions.md`,
  which testers will not have).
- `tools.md`: the test machine (with two facts added for this review at the end); `screens-v5/` (this candidate on
  the local model, see README.md), `screens-v4/`, `screens-v3/`.
- `groups.json` and `standing-notes.json`: the tester groups (R01 = MOD-09, R02 = MOD-21) and per-group notes (none).

The AI testers (GLM 5.3 Flash) will receive `design-cases.md`, `amendments.md` (= v2), `amendments-v3.md`,
`amendments-v4.md`, `amendments-v5.md` (the final version of the draft), the first run's screens, screens-v3,
screens-v4, screens-v5, neutral navigation, and one prompt per group (with the `agms-tui` description of `tools.md`).
They will not receive `decisions.md` or `scope.md`.

Your task:
1. For each rule and case section in `amendments-v5-draft.md`: does it follow the decisions exactly (the product's
   model changes to the local model; the cases' expected results stay as in force), without loosening or tightening a
   case beyond what they allow, and without unrelated expectations? Is quoted old text exact? Check in particular:
   R-1's replacement; V5-2 (further attempts by a model that keeps trying, and how they are judged); MOD-09 item 7
   ("shows the content once") and the empty `creds-copy.txt` call; MOD-21's action 1 (given the `/spawn` fact in
   `tools.md`), its child-versus-parent attribution, the "leave it open" addition and the inspector check; and the
   passed / failed / blocked boundaries.
2. Is amendment v5 self-contained and unambiguous for a tester who has only the tester packet? Is the precedence
   (v5 over v4 over v3 over v2 over frozen) clear?
3. Anything that would make a case fail or block for a reason unrelated to the product.

Be concise. Say "agree" per item or give exact replacement text. If anything must change, end with the complete final
amendment v5 document in one fenced ```markdown block (it will be frozen as-is), and one line: `VERDICT: APPROVE`
(freeze the draft as is) or `VERDICT: CHANGES` (freeze only your final version).
