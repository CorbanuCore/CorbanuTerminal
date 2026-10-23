You are a fresh, independent test designer for a code-blind functional milestone run of Corbanu Terminal (a terminal
coding agent), milestone "Moderate ships", targeted re-run. You have not seen the product's source code or any test
results, and must not look for them. Work only from the files in the current directory. You are read-only: write
nothing to disk; put everything in your reply.

Files:
- `design-cases.md`: the frozen cases MOD-01..33 (verbatim from the original designer).
- `amendments-v2.md`: the amendments in force for the first run (rules G-1..G-3, named limits, case amendments, new
  case MOD-34, the coordinator's answers and test-machine facts). For an amended case the amendment governs where it
  differs; the rest stays as frozen.
- `decisions.md`: the owner's decisions since the first run, verbatim, that this re-run must follow, and the
  integrator's choices.
- `scope.md`: which 17 cases are re-run and why, which are not, and the tester groups.
- `tools.md`: what an AI tester can do in the test machine.
- `screens/`: screens of the exact candidate under test (see `screens/README.md`); `screens-v1/`: the first run's
  design-time screens, which the frozen cases cite by number.

Task: write amendment v3, applied on top of `design-cases.md` + `amendments-v2.md`, for the 17 re-run cases only.

1. For each re-run case decide: (a) "re-run as in force, no change" (the decision fixed the product so it should now
   meet the existing expected results), or (b) amend: give exact replacement or added text for the starting
   conditions, actions or expected results that a decision changes, or that cannot be performed as written because of
   a decision. Follow the decisions exactly: do not tighten or loosen anything a decision does not touch, and add no
   unrelated expectations. Name the decision for every change.
2. Points that need your judgement (resolve each from the decisions and screens; state ambiguities):
   - MOD-01 and MOD-22: decision B (#739 option B) versus their Permissive outside-sandbox steps.
   - MOD-08: its starting conditions save "don't ask again" rules under Permissive by asking to run outside the
     sandbox; with protected launch on, decision B means no such approval is offered. How does the setup still get
     the rules decision A(3) is about, and what is then expected?
   - MOD-06: its git-commit part is not re-run (decision D); what remains, and how is that recorded.
   - MOD-09 and MOD-21: the different test model (decision F); anything else that changes.
   - MOD-26 and MOD-29: decisions A(4) and A(9).
   - MOD-28: "re-run with a session approval" (A(11)); the starting condition, actions and expected results, including
     what the revoke screen now says about saved rules. See screens 05, 06 and 09.
   - MOD-32: decision A(8).
   - Decision C (refusal wording): where a re-run case's expected results describe a refusal of a command that nobody
     was asked to approve, add the expectation that the product does not label it "Approval declined". Only there.
   - G-2: the D4 reading is now decided (decisions E); update G-2's wording if needed, without changing what it judges.
3. Also give the dispositions without execution: MOD-11 and MOD-06's git-commit part ("pending owner decision (D5,
   #837)"), and the 16 unchanged passed cases (their first-run results stand).

Reply with:
(a) the complete amendment v3 document in one fenced ```markdown block, in the style of `amendments-v2.md`'s
"Case amendments" section ("### MOD-xx (amended; <decision>)" or "### MOD-xx (re-run as in force; <decision>)", then
bulleted "Starting conditions (replace|add)", "Action N (replace)", "New action after action N", "Expected (replace)
\"<exact old text>\" with: ...", "Expected (add): ...", "Note: ..."), preceded by a short "Rules for this re-run"
section (any change to G-1..G-3, and the model rule for MOD-09/21) and followed by "Dispositions without execution";
(b) a table: case, old text changed (quoted), new text, decision;
(c) any ambiguity and how you resolved it, or why you could not.
End with one line: `DESIGN: COMPLETE` or `DESIGN: BLOCKED <reason>`.
