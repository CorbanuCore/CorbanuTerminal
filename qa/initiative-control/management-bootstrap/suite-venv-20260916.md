# A quarter of the Python suite was never running, and I called it five known errors

**Fable, 2026-09-16.** Every time I reported the initiative-control suite today I
said the same thing: *539 tests, five pre-existing `slack_sdk` import errors, not
mine.* That was wrong in a way I should have caught the first time.

## What the gate actually says

PF-80-S01's own Verification item:

> Focused: `python3 -m unittest discover -s scripts/initiative_control -p 'test_*.py'`;
> **install only pinned requirements in a disposable venv.**

I ran the discover command. I never built the disposable venv. I used the
initiative-control venv because it was already there and already on my path, and
that venv does not have `slack-sdk`, which is pinned at 3.44.1 in
`scripts/initiative_control/requirements.txt`.

## The numbers

| Environment | Tests run | Result |
| --- | ---: | --- |
| The venv I kept using | 518 | 5 import errors |
| Disposable venv, pinned requirements only | **689** | **OK** |

**171 tests never ran.** `test_decision_feed`, `test_decision_inspection`,
`test_decision_manager` and `test_slack_transport` failed at import, so every
test inside them was silently absent from the count — and a failed import reports
as one error, not as the dozens of tests it hides.

So "five known errors" was not five known errors. It was a quarter of the suite
missing, described in a way that made it sound accounted for.

## Why this matters beyond the tidiness

I have been receiving work all day against that number. Every receipt that said
"back to the five known import errors" was asserting a clean run over a suite
that was not fully running — including the decision-feed and decision-manager
modules, which are exactly the code paths behind the dashboard and the Slack
transport I have been changing and publishing from.

Nothing has actually broken: with the right environment the whole suite passes,
689 of 689. But that is luck about the outcome, not about the method. Had one of
those 171 regressed today, I would not have seen it, and I would have said the
tree was clean while saying it.

## The shape of the error

The gate told me exactly how to run it. I substituted a more convenient
environment, got a result that looked like a known-good baseline, and then
defended that baseline all day by repeating it. A caveat repeated often enough
starts to sound like a finding rather than an unexamined assumption — I said
"five pre-existing errors, not mine" perhaps a dozen times without once asking
why a project would ship a suite that cannot import its own pinned dependency.

## Now satisfied

- **Focused suite**: 689 of 689, disposable venv, pinned requirements only.
- **Governance**: `docs/plans/check.py` and `docs/sprints/check.py` pass,
  `git diff --check` clean.

Both are ticked in the sprint's Verification. The remaining five verification
items are genuinely open and stay open.
