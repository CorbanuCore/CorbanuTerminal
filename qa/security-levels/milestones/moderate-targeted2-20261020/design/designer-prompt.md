You are a fresh, independent test designer for a code-blind functional milestone run of Corbanu Terminal (a terminal
coding agent), milestone "Moderate ships", second targeted re-run. You have not seen the product's source code or any
test results, and must not look for them. Work only from the files in the current directory. You are read-only: write
nothing to disk; put everything in your reply.

Files:
- `design-cases.md`: the frozen cases MOD-01..33 (verbatim from the original designer).
- `amendments-v2.md`: the amendments in force for the first run (rules G-1..G-3, named limits, case amendments, the
  coordinator's answers and test-machine facts).
- `amendments-v3.md`: the amendment in force for the first targeted re-run (rules R-1..R-3, case amendments incl.
  MOD-15 and MOD-28). Precedence: v3 over v2 over the frozen cases.
- `decisions.md`: the owner's decisions since that re-run, verbatim, and the integrator's choices.
- `scope.md`: which 2 cases are re-run and why.
- `tools.md`: what an AI tester can do in the test machine.
- `screens-v4/`: screens of the exact candidate under test (see its README); `screens-v3/`: the previous re-run's
  design-time screens, which amendment v3 cites as "screens-v3 NN".

Task: write amendment v4, applied on top of `design-cases.md` + `amendments-v2.md` + `amendments-v3.md`, for MOD-15
and MOD-28 only.

1. For each case decide: (a) "re-run as in force, no change", or (b) amend: give exact replacement or added text for
   the starting conditions, actions or expected results that a decision changes, or that cannot be performed as
   written because of a decision. Follow the decisions exactly: do not tighten or loosen anything a decision does not
   touch, and add no unrelated expectations. Name the decision for every change.
2. Points that need your judgement (resolve each from the decisions and screens; state ambiguities):
   - MOD-15 and amendment v3 R-2: the candidate now includes #900 (D38). Does R-2's "not judged" carve-out still apply
     to MOD-15, or is the frozen expectation ("The message names the security level and what the person can do")
     judged in full on what the product shows? Say exactly what the tester checks and where (the refused commands'
     cells, the model's reply, Recent denials).
   - MOD-28 under D40: how the tester looks for a session ("for this session") approval the UI offers under Moderate
     now (screens-v4 05, 08, 09), what to do if one exists (judge the frozen expected results), and what to record if
     none is offered ("no session approval exists to revoke"), including which steps still run (for example the
     revoke review and `g` after it) and what is judged from the host capture screens-v4 06 instead (the revoke
     wording of A(11) in amendment v3). Keep the tester's search bounded and concrete.
3. Rules: say whether G-1..G-3 and R-1..R-3 change for this re-run (R-1 concerns MOD-09/21, not re-run).

Reply with:
(a) the complete amendment v4 document in one fenced ```markdown block, in the style of `amendments-v3.md`
("### MOD-xx (amended; <decision>)" or "### MOD-xx (re-run as in force; <decision>)", then bulleted
"Starting conditions (replace|add)", "Action N (replace)", "New action after action N", "Expected (replace)
\"<exact old text>\" with: ...", "Expected (add): ...", "Note: ..."), preceded by a short "Rules for this re-run"
section, and followed by "How each outcome is recorded" (for MOD-28: passed / accepted under D40 / failed / blocked);
(b) a table: case, old text changed (quoted), new text, decision;
(c) any ambiguity and how you resolved it, or why you could not.
End with one line: `DESIGN: COMPLETE` or `DESIGN: BLOCKED <reason>`.
