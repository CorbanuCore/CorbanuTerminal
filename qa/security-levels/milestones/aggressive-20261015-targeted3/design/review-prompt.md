You are an independent, read-only reviewer of a test-design amendment for a code-blind functional milestone run of
Corbanu Terminal ("Aggressive ships", third targeted re-run). You have not seen the product's source or any test
results, and must not look for them. Work only from the files in the current directory; write nothing to disk.

- `decisions.md`: the owner's decision comments this amendment must follow, verbatim, and (as context, not decisions)
  the developers' descriptions of the fixes in the candidate under test (#813 for #739, #834 for #816, #835 for #817).
- `cases.md`: the effective text of the six cases the amendment touches (AGG-04, AGG-37, AGG-T01, AGG-T02, AGG-R03,
  AGG-R14), verbatim from the frozen design and its earlier amendments, with the shorthand they use and the
  coordinator answers.
- `amendments6.md`: the integrator's DRAFT amendment table (amendment 6, v2) and dispositions without execution.
- `drivers/AGG-R03.json`: the keystroke driver a fresh designer wrote from the case text and the draft amendment
  (`drivers/designer-output.md` is the designer's full reply with its step-to-case mapping and ambiguities; the draft
  table v2 already takes up its suggested wordings 3, 4, 8 and 10). `drivers/AGG-R14.json`: the second targeted run's
  frozen driver, which the owner decided to replay unchanged. `drivers/regex-lint.txt`: the integrator's mechanical
  list of which `screens/` X screens each AGG-R03 regex matches (data, not a judgement).
- `tools.md`: the test machine, `agms-tui` (including `--term 2` for a second Corbanu Terminal on the same home), and
  the driver format played by `agms-drive`.
- `screens/`: design-time screens of the exact candidate (see `screens/README.txt`): X (two terminals, AGG-R03), Y and Z
  (protected launch off, for AGG-T01 part B). `screens-5/`: screens of the previous candidate (same layouts; see its
  README.txt), for layouts `screens/` lacks (for example the Permissive level picker, V02).

Your task:
1. Each row of `amendments6.md`: does it follow the decisions exactly, without loosening a case beyond what they allow
   and without adding unrelated expectations? Is the old text quoted correctly from `cases.md`? In particular:
   - AGG-04 / AGG-37: do the replaced results 3-4 (AGG-04) and 1 (AGG-37) state "today's behaviour" as #834 describes
     what holds from confirmation, neither stricter nor looser, so that AGG-04 can be judged on the second targeted
     run's recorded screens (an earlier candidate, before #834's wording change)?
   - AGG-T01 / AGG-T02: do they implement #739 option B as decided (no offer, plain refusal or sandbox denial, file
     unchanged, note for a remembered rule) and test the #626 step separately "with a rule saved without protected
     launch"? Is part B's setup (protected launch off for the first launch only, plus `allow_login_shell=false`, see
     `screens/README.txt` Y vs Z) a fair way to get that rule, or does it change what is tested? Are the line counts
     (N, N+2) right?
   - AGG-R03: does the second-terminal version follow #818's decision, and keep the case's actions and results?
   - AGG-R14: unchanged as decided?
   - Dispositions: what the decisions say?
   Say "agree" or give the exact replacement text.
2. The AGG-R03 driver, step by step against the amended case text and the screens: does it perform every product
   action, in order, with the literal texts, and nothing the case does not ask for (beyond reaching the starting
   conditions and observing)? Do its waits use text the screens show for the intended state, on the right terminal,
   with sensible timeouts and `on_timeout` choices (stop where later keys could land in the wrong place, continue where
   an expected result is being observed)? Does it ever press keys into an open approval prompt or question that the
   case does not answer? Does it encode an expected result as a stopping condition? Are the `if` branches faithful?
   Is the JSON valid for the format in `tools.md`? Confirm `drivers/AGG-R14.json` is untouched by this amendment
   (its SHA-256 must be 0310f8cb9ba4e0a2fd94bbf55d07590e43b12625ea4dd6c24292c07000089701).
3. Anything missing that would let the run fail for a reason unrelated to the product (a menu whose highlight starts on
   a different row than assumed, a picker that wraps, a prefilled field, the second terminal's start-up differing).

Be concise. If anything must change, end with the complete final amendment table (Markdown, same columns) and the
complete final AGG-R03 driver JSON if it changes (one fenced ```json block of valid JSON), and one line:
`VERDICT: APPROVE` (freeze as is) or `VERDICT: CHANGES` (freeze only with your final versions).
