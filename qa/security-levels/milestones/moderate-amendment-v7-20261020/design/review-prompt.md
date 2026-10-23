You are an independent, read-only reviewer of a test-design amendment for a code-blind functional milestone test of
Corbanu Terminal (milestone "Moderate ships"). You have not seen the product's source or any test results, and must not
look for them. Work only from the files in the current directory; write nothing to disk.

- `decisions.md`: the owner's decision this amendment must follow (D97, verbatim), background and integrator choices.
- `scope.md`: the three cases amended (MOD-12, MOD-23, MOD-29) and what is in force for it.
- `design-cases.md`: the frozen cases; `amendments-v2.md` (the tester packet calls it `amendments.md`),
  `amendments-v3.md`, `amendments-v4.md`, `amendments-v5.md`: amendments in force.
- `amendments-v7-draft.md`: amendment v7, written by a fresh read-only designer from these files
  (`designer-output.md` is its full reply, with a change table and the ambiguities it resolved). No integrator edits.
- `tools.md`: the test machine (with facts added for this review at the end).

AI testers (GLM 5.3 Flash) will receive the frozen cases, amendments v2-v5, the final amendment v7, screens, neutral
navigation and one prompt per group. They will not receive `decisions.md` or `scope.md`.

Your task:
1. Does each change follow D97 exactly (wording only: MCP tool naming in MOD-12, which config file to edit in MOD-23,
   how to reset the profile in MOD-29), without loosening or tightening any expected result and without unrelated
   expectations? Is quoted old text exact? Check in particular: the shared profile reset PR-1..PR-4 (safe, complete,
   and does it leave the supplied `config.toml` as `tools.md` describes); MOD-12's restated starting conditions and
   action 1; MOD-23's `sed`, its checks and its fallback (macOS `sed`); MOD-29's picker check before Moderate; and
   whether adding the reset to MOD-23 (not named in D97) stays within "wording only".
2. Is amendment v7 self-contained and unambiguous for a tester who has only the tester packet? Is precedence clear?
3. Anything that would make a case fail or block for a reason unrelated to the product.

Be concise. Say "agree" per item or give exact replacement text. If anything must change, end with the complete final
amendment v7 document in one fenced ```markdown block (it will be frozen as-is), and one line: `VERDICT: APPROVE`
(freeze the draft as is) or `VERDICT: CHANGES` (freeze only your final version).
