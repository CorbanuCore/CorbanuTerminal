You are an independent, read-only reviewer of a test-design amendment for a code-blind functional milestone test of
Corbanu Terminal (milestone "Moderate ships"). You have not seen the product's source or any test results, and must not
look for them. Work only from the files in the current directory; write nothing to disk.

- `decisions.md`: the owner's decision this amendment must follow (D140, verbatim), background, the D114 tester rule
  and integrator choices.
- `scope.md`: the one case amended (MOD-21) and what is in force for it.
- `design-cases.md`: the frozen cases; `amendments-v2.md` (the tester packet calls it `amendments.md`),
  `amendments-v3.md`, `amendments-v4.md`, `amendments-v6.md`, `amendments-v7.md`: amendments in force.
- `amendments-v9-draft.md`: amendment v9, written by a fresh read-only designer from these files
  (`designer-output.md` is its full reply, with a change table and the ambiguities it resolved). No integrator edits.
- `tools.md`: the test machine (with facts added for this review at the end); `screens-spawn/`: the `/spawn` and
  Spawn status screens of an earlier build that the designer worked from.

AI testers (GLM 5.3 Flash) will receive the frozen cases, amendments v2, v3, v4, v6, v7, the final amendment v9,
screens, neutral navigation and one prompt per group. They will not receive `decisions.md`, `scope.md`, `tools.md` or
`screens-spawn/` (say if the amendment depends on anything only those files contain).

Your task:
1. Does each change follow D140 exactly (the tester sends the task from the Spawn status screen), without loosening or
   tightening any expected result and without unrelated expectations? Is quoted old text exact (MOD-21's amendments-v2
   action 1, the frozen actions 2 and 3)? Check in particular: the role chosen and the child's model under R-1; the
   exact task text and CONFIG-PATH; what counts as giving the task another way; G-1's resubmission route; the "message
   instead of a turn" note and its blocked outcome; the 3-minute and 20-minute rules and the D114 line; the inspector
   step; whether the starting-conditions items fit the starting conditions in force.
2. Is amendment v9 self-contained and unambiguous for a tester who has only the tester packet? Is precedence clear?
3. Anything that would make the case fail or block for a reason unrelated to the product.

Be concise. Say "agree" per item or give exact replacement text. If anything must change, end with the complete final
amendment v9 document in one fenced ```markdown block (it will be frozen as-is), and one line: `VERDICT: APPROVE`
(freeze the draft as is) or `VERDICT: CHANGES` (freeze only your final version).
