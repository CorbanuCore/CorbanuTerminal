You are an independent, read-only reviewer of a test-design amendment for a code-blind functional milestone test of
Corbanu Terminal (milestone "Moderate ships"). You have not seen the product's source or any test results, and must not
look for them. Work only from the files in the current directory; write nothing to disk.

- `decisions.md`: the owner's decision this amendment must follow (D96, verbatim), background and integrator choices.
- `scope.md`: the one case amended (MOD-22) and what is in force for it.
- `design-cases.md`: the frozen cases; `amendments-v2.md` (the tester packet calls it `amendments.md`),
  `amendments-v3.md`, `amendments-v4.md`, `amendments-v5.md`: amendments in force.
- `amendments-v6-draft.md`: amendment v6, written by a fresh read-only designer from these files
  (`designer-output.md` is its full reply, with a change table and the ambiguities it resolved). No integrator edits.
- `tools.md`: the test machine (with facts added for this review at the end).

AI testers (GLM 5.3 Flash) will receive the frozen cases, amendments v2-v5, the final amendment v6, screens, neutral
navigation and one prompt per group. They will not receive `decisions.md` or `scope.md`.

Your task:
1. Does each change follow D96 exactly (add `/security`, `i` after the relaunch step), without loosening or tightening
   any expected result and without unrelated expectations? Is quoted old text exact (MOD-22's action 4 in the frozen
   case)? Check in particular: placement relative to amendments-v3's action 5 and its "send only this request"; the
   row-by-row comparison with the 'baseline' screen and what "match" means; the network-row note; the Esc, Esc exit.
2. Is amendment v6 self-contained and unambiguous for a tester who has only the tester packet? Is precedence clear?
3. Anything that would make the case fail or block for a reason unrelated to the product.

Be concise. Say "agree" per item or give exact replacement text. If anything must change, end with the complete final
amendment v6 document in one fenced ```markdown block (it will be frozen as-is), and one line: `VERDICT: APPROVE`
(freeze the draft as is) or `VERDICT: CHANGES` (freeze only your final version).
