You are an independent, read-only reviewer of a test-design amendment for a code-blind functional milestone test of
Corbanu Terminal (milestone "Aggressive ships"). You have not seen the product's source or any test results beyond the
files here, and must not look for them. Work only from the files in the current directory; write nothing to disk.

- `decisions.md`: the owner's decisions this amendment must follow (D110, D111, verbatim), background and integrator
  choices. `scope.md`: the two cases.
- `agg-design-cases.md`, `agg-amendments-and-new-cases.md`, `amendments-L2.md`: the cases and amendments in force.
- `amendment7-draft.md`: amendment 7, written by a fresh read-only designer from these files (`designer-output.md` is
  its full reply, with a change table and the ambiguities it resolved). No integrator edits.
- `tools.md`, `tester-prompt-R04.md`, `navigation.md`, `screens-v6/`: the test machine; `R02-permission-text.txt`: the
  product's permission instructions to the model in the AGG-R07 run.

AI testers (GLM 5.3 Flash) will receive the cases, the amendments in force, the final amendment 7, screens, neutral
navigation and one prompt per group. They will not receive `decisions.md` or `scope.md`.

Your task:
1. AGG-R07 (D110): does the change follow D110 exactly (names the model repeats from the product's own fixed
   permission instructions don't count; every listing denied; nothing else loosened or tightened)? Is quoted old text
   exact? Is the evidence rule (literal path in "Denied filesystem reads"; the list being the fixed one; who reads the
   session file; blocked when unreadable) sound and not stricter or looser than D110? Applied to R02's recorded facts
   in `decisions.md` and `R02-permission-text.txt`, what verdict does the amended rule give?
2. AGG-20P (D111): does it test a person-defined role with more access than Aggressive being refused at start, and that
   no agent ever runs with that access? Are the shell blocks correct and safe on macOS (heredocs inside a tester's
   shell command, BSD `sed -i ''`, the REMOVE-WIDE range delete and its check)? Is the control role sound? Are the
   expected results and pass rule unambiguous, not tied to unknown wording, and free of failures unrelated to the
   product? Is it too long for one ~110-minute tester session on the local model (replies take 10-80 s; one request at
   a time)? If so, say how to split it.
3. Is amendment 7 self-contained for a tester who has only the tester packet? Is precedence clear? Is "every other case
   unchanged" true?

Be concise. Say "agree" per item or give exact replacement text. If anything must change, end with the complete final
amendment 7 document in one fenced ```markdown block (it will be frozen as-is), and one line: `VERDICT: APPROVE`
(freeze the draft as is) or `VERDICT: CHANGES` (freeze only your final version).
