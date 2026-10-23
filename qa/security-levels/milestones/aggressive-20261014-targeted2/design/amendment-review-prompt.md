You are an independent, read-only reviewer of a test-design amendment for a code-blind functional milestone run of
Corbanu Terminal ("Aggressive ships", second targeted re-run). You have not seen the product's source or any test
results, and must not look for them. Work only from the files in the current directory; write nothing to disk.

- `decisions.md`: the owner's decision comments this amendment must follow, verbatim.
- `cases.md`: the effective text of the four cases this run executes (AGG-04, AGG-37, AGG-R03, AGG-R14), verbatim
  from the frozen design and its earlier amendments, with the shorthand they use.
- `amendments5.md`: the integrator's DRAFT amendment table (amendment 5) and dispositions without execution.
- `drivers/AGG-R03.json`, `drivers/AGG-R14.json`: keystroke drivers written by a fresh designer from the case text
  (`drivers/designer-output.md` is the designer's full reply with its step-to-case mapping and ambiguities;
  `drivers/regex-lint.txt` is the integrator's mechanical list of which screens each regex matches).
- `tools.md`: the test machine, `agms-tui`, and the driver format played by `agms-drive`.
- `screens/`: design-time screens of the exact candidate (see `screens/README.txt`).

Your task:
1. Each row of `amendments5.md`: does it follow the decision exactly, without loosening a case beyond what the
   decision allows and without adding unrelated expectations? Is the old text quoted correctly from `cases.md`? Is the
   AGG-37 added action a fair way to have denial messages to review after a restart, or does it change the case? Are
   the dispositions (AGG-18, AGG-R07, AGG-R10, AGG-20, AGG-T01/T02, AGG-R06) what the decisions say? Say "agree" or
   give the exact replacement text.
2. Each driver, step by step against the case text and the screens: does it perform every product action of the case,
   in order, with the literal texts, and nothing the case does not ask for (beyond reaching the starting conditions
   and observing)? Do its waits use text that the screens show for the intended state, with sensible timeouts and
   `on_timeout` choices (stop where later keys could land in the wrong place, continue where an expected result is
   being observed)? Does it ever press keys into an open approval prompt or question that the case does not answer?
   Does it encode an expected result as a stopping condition? Are the `if` branches faithful to the case's
   conditions? Check the JSON is valid for the format in `tools.md`.
3. Anything missing that would let the run fail for a reason unrelated to the product (for example a menu whose
   highlight starts on a different row than assumed, a picker that wraps, a field that is prefilled).

Be concise. If anything must change, end with the complete final amendment table and the complete final driver JSON
for each driver that changes (one fenced ```json block of valid JSON per
driver), and one line: `VERDICT: APPROVE` (freeze as is) or `VERDICT: CHANGES` (freeze only with your final versions).
