You are a fresh, independent test designer for a code-blind functional milestone test of Corbanu Terminal (a terminal
coding agent), milestone "Moderate ships". You have not seen the product's source code or any test results, and must
not look for them. Work only from the files in the current directory. You are read-only: write nothing to disk; put
everything in your reply.

Files:
- `design-cases.md`: the frozen cases MOD-01..33 (verbatim from the original designer).
- `amendments-v2.md`: the first run's amendments (rules G-1..G-3, named limits, case amendments, test-machine facts);
  the tester packet calls it `amendments.md`.
- `amendments-v3.md`, `amendments-v4.md`, `amendments-v6.md`, `amendments-v7.md`: later amendments, in force.
  Precedence: v7 and v6 over v4 over v3 over v2 over the frozen cases (`amendments-v5.md`, which v6 and v7 mention,
  belongs to a separate local-model run and is not in force here).
- `decisions.md`: the owner's decision behind this amendment, verbatim, the background, the D114 tester rule and the
  integrator's choices.
- `scope.md`: which one case this amendment touches and what is in force for it.
- `tools.md`: what an AI tester can do in the test machine, including how `/spawn` and Spawn status look.
- `screens-spawn/`: text copies of the `/spawn` and Spawn status screens of an earlier build (`README.md` there).

Task: write amendment v9, applied on top of `design-cases.md` + `amendments-v2.md`, `-v3`, `-v4`, `-v6`, `-v7`, for
MOD-21 only.

1. Give exact replacement or added text for the starting conditions or actions that the decision changes. Follow the
   decision exactly: the tester sends the task from the Spawn status screen. Wording only; do not tighten or loosen
   any expected result, and add no unrelated expectations. Name the decision for every change.
2. Points that need your judgement (resolve each from the cases, amendments, decision, tools and screens; state
   ambiguities):
   - Which pane the task goes to (role, and how it is created with `/spawn`, including the child's model under R-1),
     and the exact task text the tester types into the "Send Task to <pane>" box (keep the case's task; tester-only
     set-up such as creating the synthetic `~/.aws/credentials` is done from the tester's own shell before, not typed
     into the task).
   - What the tester records after sending (screen labels), how it follows the child and answers each of the child's
     prompts (action 2: approve each prompt), how it returns to the main pane, and where it checks the inspector
     (action 3).
   - What counts as giving the task another way (not allowed), how G-1's single unchanged resubmission is sent, and
     what the tester records if the product shows a message instead of starting the child's turn.
   - Waiting: add the D114 line where the amendment sets any waiting rule.
3. Keep every rule (G-1..G-3, R-1..R-3, and the rules of amendments v4, v6 and v7) unchanged; say so in one line.

Reply with:
(a) the complete amendment v9 document in one fenced ```markdown block, in the style of `amendments-v6.md`
("### MOD-xx (amended; <decision>)", then bulleted "Starting conditions (replace|add)", "Action N (replace)",
"New action after action N", "Expected (replace) \"<exact old text>\" with: ...", "Expected (add): ...", "Note: ..."),
preceded by a short header (what it applies on top of, precedence, the decision label) and a one-line rules section;
(b) a table: case, old text changed (quoted), new text, decision;
(c) any ambiguity and how you resolved it, or why you could not.
End with one line: `DESIGN: COMPLETE` or `DESIGN: BLOCKED <reason>`.
