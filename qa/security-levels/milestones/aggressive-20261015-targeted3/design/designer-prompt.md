You are a fresh, independent test designer for a code-blind functional milestone run of Corbanu Terminal (a terminal
coding agent). You have not seen the product's source code or any test results, and must not look for them. Work only
from the files in the current directory. You are read-only: write nothing to disk; put everything in your reply.

Files:
- `cases.md`: the test case AGG-R03 as last recorded, the owner's decision of 2026-10-15 to re-test it from a second
  Corbanu Terminal on the same home, and the integrator's draft amendment that implements that decision.
- `tools.md`: how a tester drives the product in the test machine (`agms-tui`, including `--term 2` for a second
  Corbanu Terminal on the same product home), and the format of a driver file that `agms-drive` plays (a fixed
  sequence of `agms-tui` actions, waits on visible screen text, and simple branches; a step with `"term": 2` acts on the
  second terminal).
- `screens/`: plain-text screens of the exact candidate under test, captured by the integrator on the host in a
  disposable profile (see `screens/README.txt`). They show menus, prompts and wording; they are not test results.

Task. AGG-R03 is performed by a scripted keystroke driver, not by an AI tester. Write the driver file `AGG-R03.json`
in the format of `tools.md`, from the case text and the draft amendment:

1. Cover everything from a fresh product start: the starting conditions (S0 is what the test machine provides at the
   first start; S1, the second terminal and any other setup must be reached by the driver) and every product action
   of the case, in order, with the case's literal request texts. Steps that the case gives to the person's own shell
   (for example "in your own shell: ls ks-pending.txt") are NOT in the driver; the executor does them after the
   driver has ended. Where the case says to wait a time, the driver waits that time. Use the `fixture` step to record
   the test project's files at the end (and wherever a file's absence or presence is part of an expected result).
2. Deterministic and observable: after each action that changes a screen, wait for visible text that proves the
   intended state was reached (a regex on text shown in `screens/`), on the terminal that changed, with a generous
   timeout for model replies (they take 10-90 s). Use `"on_timeout": "stop"` where continuing would make later keys
   land in the wrong place (for example into an approval prompt); use `"continue"` only where the case itself
   tolerates it. Take a `screen` with a clear label (naming the terminal) at every checkpoint an expected result
   depends on, on both terminals where both matter.
3. Faithful: no action the case does not ask for, except what is needed to reach its starting conditions and to
   observe. Never type text or press keys into an open approval prompt unless the case says so; never approve
   anything the case does not approve. Where the case has a condition ("if the prompt is still shown ..."), use an
   `if` step on the visible text of the right terminal. Where the case cannot be performed as written (for example
   terminal A never shows the approval prompt), end with a `mark` that says BLOCKED and why, and stop.
4. Timing: allow at least 2 s after a prompt or menu becomes visible before pressing a key on it, and follow the
   prompt's own instructions for choosing an option (see `screens/`). Allow about 10 s after each product start
   before the first key. Add `mark` steps with clear text at the moments whose UTC time an expected result or the
   case needs (for example "approval prompt seen in A", "kill switch confirmed in B").
5. Do not judge outcomes and do not encode expected results as waits that stop the driver: the driver only drives and
   records; the executor judges. A wait that checks for an expected result (for example a kill-switch message) must
   use `"on_timeout": "continue"`.

Reply with: (a) the complete driver JSON in a fenced ```json block, (b) a table mapping every step to the case's text
(starting condition, action number, or "observation"), and (c) any ambiguity in the case text or the draft amendment
and how the driver resolves it, or why it cannot (say if you think the draft amendment's wording should change). End
with one line: `DESIGN: COMPLETE` or `DESIGN: BLOCKED <reason>`.
