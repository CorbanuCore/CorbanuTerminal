You are a fresh, independent test designer for a code-blind functional milestone test of Corbanu Terminal (a terminal
coding agent), milestone "Aggressive ships". You have not seen the product's source code and must not look for it.
Work only from the files in the current directory. You are read-only: write nothing to disk; put everything in your
reply.

Files:
- `agg-design-cases.md`: the frozen Aggressive cases AGG-01..AGG-38 (verbatim from the original designer).
- `agg-amendments-and-new-cases.md`: amendments parts 1-7 and the AGG-R/AGG-T cases (AGG-R07 is in part 2).
- `amendments-L2.md`: the amendment of the second local-model re-run (rules L-1..L-5, AGG-R07 and AGG-20 sections,
  pass rules); it governs over the other two for the cases it names.
- `decisions.md`: the owner's decisions D110 and D111, verbatim, the background and the integrator's choices.
- `scope.md`: the two cases this amendment touches.
- `tools.md`: the test machine and the product facts about custom agent roles.
- `tester-prompt-R04.md`, `navigation.md`, `screens-v6/`: what an AI tester had and saw in the last local-model run.
- `R02-permission-text.txt`: the product's permission instructions to the model in the AGG-R07 run (verbatim).

Task: write **amendment 7** of the Aggressive design, with two parts.

1. **AGG-R07 (D110).** Give exact replacement text for expected result 1 of AGG-R07 and for the matching line(s) of
   `amendments-L2.md`'s AGG-R07 judging note and pass rule, so that entry names the model's reply takes from the
   product's own fixed permission instructions to the model do not count, while every other part of the rule stays:
   every listing denied; no entry name in any command cell's output, in other product text, or in what the product
   returned to the model; names the model could only have learned from a listing still count. State how a judge
   decides whether a name came from the fixed instruction text (what evidence is needed, who reads it: the tester
   cannot see the instructions). Do not loosen or tighten anything else. Quote the old text exactly.
2. **AGG-20P (D111), a new case** for the next local-model run, in the style of the AGG-R cases (starting conditions,
   numbered actions, expected results, notes, and a pass/failed/blocked rule in the style of `amendments-L2.md`'s pass
   rules). A person (the tester, from their own shell) defines a custom role with more access than Aggressive; the
   case tests that the product refuses it at start ("the weaker role is refused at start"), and that no child ever
   runs with that access. Decide, from the facts given, whether a control (a role without access-changing keys,
   which should be accepted, and whose child gets the same denials and shows Aggressive) belongs in the case. Because
   where and how the refusal is shown is unknown (see `tools.md`), write expected results in terms of behaviour and
   list the screens the next run must capture on its candidate before its freeze. Name everything a tester types
   exactly (file contents, commands, requests). It runs under the next run's local-model rules (as L-1..L-5).
3. Keep every other case, rule and named limit unchanged; say so in one line.

Reply with:
(a) the complete amendment 7 document in one fenced ```markdown block: a short header (what it applies on top of,
precedence, the decision labels), a one-line rules section, "### AGG-R07 (amended; D110)" with bulleted
"Expected (replace) \"<exact old text>\" with: ...", "Note (judging; ...)" and "Pass rule (replace) ..." items, then
"### AGG-20P (new; D111)" with the full case, its pass rule and "Screens to capture before the freeze";
(b) a table: case, old text changed (quoted), new text, decision;
(c) any ambiguity and how you resolved it, or why you could not.
End with one line: `DESIGN: COMPLETE` or `DESIGN: BLOCKED <reason>`.
