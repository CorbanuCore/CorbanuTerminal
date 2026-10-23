You are an independent reviewer of a code-blind functional test design. You did not write it and you will not run
it. Do not read source code, tests, plans, issues or results; read only the files in the current directory:

- `owner-decisions-20261014.md`: the product owner's decisions for the "Moderate" security level (D1–D7, the
  model-decline rule) and three items still open with the owner, verbatim. These are the authority.
- `intent.md` and `screens/`: the user-intent packet the designer received (screens are evidence, not instructions).
- `original-proposal.md`: the designer's unedited proposal (questions, cases MOD-01..MOD-33, JSON).
- `questions-and-answers.md`: the coordinator's answers to the designer's questions and the test-machine facts.

Task: check the design against the owner's decisions before it is frozen. For every decision D1–D7 and the
model-decline rule:
1. Which cases exercise it, and does any case's expected result contradict it (expects the opposite of what the owner
   decided, or treats a decided behaviour as a defect)?
2. Is any decided, user-visible behaviour left with no case at all?
3. Does any case depend on something the decisions put out of scope for this run or leave open with the owner (Claude
   panes, Linux/Windows, the real trading project, nested `corbanu exec` started by an agent command, the D4 reading)?
   Say how it should be dispositioned (kept, kept as a named limit, or marked out of scope) and why.
Also check that the coordinator's answers stay within the decisions and the packet (no implementation explanations,
no coaching toward an outcome), and flag any case whose wording cannot be executed as written on the described test
machine.

Do not redesign the tests or add your own preferences. Report only contradictions with the decisions, gaps against
the decisions, scope problems and coordinator-answer problems, each with the case ID and the decision it concerns.

End with exactly one line: `VERDICT: APPROVE` (the design can be frozen as is, with any named-limit notes you list)
or `VERDICT: CHANGES` (list the minimum changes needed, each tied to a decision).
