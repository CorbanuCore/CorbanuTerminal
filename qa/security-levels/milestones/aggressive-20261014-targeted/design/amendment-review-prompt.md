You are an independent, read-only reviewer of a test-design amendment for a code-blind functional milestone run of
Corbanu Terminal ("Aggressive ships"). You have not seen the product's source or any test results, and must not look
for them. Work only from the files in the current directory:

- `design-cases.md`: the frozen cases (2026-10-10), verbatim.
- `amendments-and-new-cases.md`: the amendments of 2026-10-11 and 2026-10-13 and the new cases (AGG-R.., AGG-T..) as
  they stood for the third run.
- `screens/`: design-time screens of the first candidate; `new-screens-4/`: screens of the new candidate, captured by
  the integrator on 2026-10-14 for these amendments (see its README.txt).
- `decisions.md`: the owner's decision comments that this amendment must follow, verbatim.
- `scope.md`: which cases run in this targeted re-run and why.
- `amendments4.md`: the integrator's DRAFT amendment table for 2026-10-14 (the subject of this review).

Your task:
1. For each row of `amendments4.md`, check it against the decision comments only: does the new expectation follow
   the decision exactly, without loosening the case beyond what the decision allows and without adding unrelated
   expectations? Is the old expectation quoted correctly from the effective design? Say "agree" or give the exact
   replacement text.
2. Missing amendments: among the cases in `scope.md`, does any decision in `decisions.md` change an expected result
   that has no row? (In particular: any case that asserts what applies before the restart, or the restart-only list.)
   Give the row you would add.
3. Two judgement calls to answer explicitly: (a) AGG-25 part B keeps the frozen blocked/fix/recovery path with a
   different unmanaged secret, while part A records today's behaviour for the E1 flag; is that a faithful "amend to
   today's behaviour", or should part B be dropped? (b) AGG-04's added action 3 (an ordinary write requested before the
   restart must show an approval prompt): keep, change or drop?
4. AGG-R10: is `rg --hidden --no-ignore LATE_ .` the right literal for "use rg instead of grep -r"?

Be concise. End with a final table (case, final new expectation text, decisions) and one line:
`VERDICT: APPROVE` (the draft can be frozen as is) or `VERDICT: CHANGES` (freeze only with your final table).
