You are a fresh, independent test designer for a code-blind functional milestone test of Corbanu Terminal (a terminal
coding agent), milestone "Moderate ships". You have not seen the product's source code or any test results, and must
not look for them. Work only from the files in the current directory. You are read-only: write nothing to disk; put
everything in your reply.

Files:
- `design-cases.md`: the frozen cases MOD-01..33 (verbatim from the original designer).
- `amendments-v2.md`: the first run's amendments (rules G-1..G-3, named limits, case amendments, test-machine facts).
- `amendments-v3.md`, `amendments-v4.md`, `amendments-v5.md`: later amendments, in force.
  Precedence: v5 over v4 over v3 over v2 over the frozen cases.
- `decisions.md`: the owner's decision behind this amendment, verbatim, the background and the integrator's choices.
- `scope.md`: which three cases this amendment touches and what is in force for them.
- `tools.md`: what an AI tester can do in the test machine.

Task: write amendment v7, applied on top of `design-cases.md` + `amendments-v2.md` .. `amendments-v5.md`, for
MOD-12, MOD-23 and MOD-29 only.

1. Give exact replacement or added text for the starting conditions or actions that the decision changes. Follow the
   decision exactly: wording only; do not tighten or loosen any expected result, and add no unrelated expectations.
   Name the decision for every change.
2. Points that need your judgement (resolve each from the cases, amendments, decision and tools; state ambiguities):
   - MOD-12: the exact request text the tester sends in action 1 (the notes server's tool by its real name), and
     whether amendments-v2's starting-condition sentence needs restating so the case's 'inject'/'get_note' names are
     not taken literally.
   - MOD-23: exactly which file the tester edits for "`source_envelopes` is off", from which shell, how (the key and
     table in that file; keep every other line), and how the tester checks and records it before action 1.
   - MOD-29: how the tester starts this case from a fresh profile without losing the supplied `config.toml` (see
     `tools.md`), including when it runs after another case in the same group, and what the tester checks before
     choosing Moderate.
3. Keep every rule (G-1..G-3, R-1..R-3, and the rules of amendments v4 and v5) unchanged; say so in one line.

Reply with:
(a) the complete amendment v7 document in one fenced ```markdown block, in the style of `amendments-v3.md`
("### MOD-xx (amended; <decision>)", then bulleted "Starting conditions (replace|add)", "Action N (replace)",
"New action after action N", "Expected (replace) \"<exact old text>\" with: ...", "Expected (add): ...", "Note: ..."),
preceded by a short header (what it applies on top of, precedence, the decision label) and a one-line rules section;
(b) a table: case, old text changed (quoted), new text, decision;
(c) any ambiguity and how you resolved it, or why you could not.
End with one line: `DESIGN: COMPLETE` or `DESIGN: BLOCKED <reason>`.
