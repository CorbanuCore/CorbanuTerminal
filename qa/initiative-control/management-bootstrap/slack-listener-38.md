# slack-listener-38 revision evidence

Allocation: slack-listener-38; base: c28238d87d99d975891596eba222451afa608e7d.
Brief SHA-256 verified: 8812d50587d484692e1fc8f02ab60d56a45843e54ed023c6e1ae4ade0c18ad34.
Routine test/harness isolation correction within PF-80-S01; production code unchanged.
Product basis: **Internal delivery control — TO BUILD**, “durable event dispatch, acknowledgments and watchdog”.
No user-facing change: TUI/code-blind N/A proposed for this correction; broader PF-80 acceptance stays with the integrator.

The retained CLI test runs parent and child through a test-only bootstrap with -I -S -B.
It replaces the environment with fixture HOME/profile aliases and synthetic Slack tokens.
An audit hook denies every socket event and raises ModuleNotFoundError on slack_sdk import.
A marker proves the denied import occurred in _listen-child; the exception has private fixture detail.
Assertions require exact fixed CLI stderr, empty stdout, exit 1, one durable held child-exit record,
and unchanged ingress, fence, watermark, binding, gap reviews and lifecycle in the fixture.
Every existing test is retained. No production redaction, listener, transport or retry code changed.

Harness invocation is now: slack-listener-34-reproduce.py DESTINATION SOURCE_STORE.
No default operator path remains; source inspection uses hashes/copying only, with no Store or status/lock call.
Status runs only on the disposable destination; destinations within the source store are rejected.
Synthetic source used: /private/tmp/slack-listener-38.l5JysH/harness-source.
Replay asserted status never accessed that source; all source hashes and protected copy fields were unchanged.
No live store was read or modified; live ingress, fence, watermark, binding and gap reviews were untouched.

Read docs/development/test-isolation.md first. Venv: /private/tmp/slack-listener-38.l5JysH/venv.
Built with python3 -m venv, then venv/bin/python -m pip install -r scripts/initiative_control/requirements.txt.
Python 3.14.4; system site-packages disabled; markdown-it-py 3.0.0, mdurl 0.1.2, slack_sdk 3.44.1 only (plus bootstrap pip).
Both suite runs use env -i, fixed system PATH, HOME/TMPDIR=/private/tmp/slack-listener-38.l5JysH,
PYTHONDONTWRITEBYTECODE=1, PYTHONPATH=scripts/initiative_control, and venv/bin/python -B -m unittest discover -v -s scripts/initiative_control -p '*test*.py'.
Second run adds CORBANU_SLACK_BOT_TOKEN=synthetic-slack-listener-38-bot and CORBANU_SLACK_APP_TOKEN=synthetic-slack-listener-38-app.
Token-unset: 737/737 passed in 440.939s, exit 0. Synthetic replay: 729/737 passed, 8 failures in 451.222s, exit 1; no errors/skips. Identical-full-suite gate UNMET.
Original synthetic run: 737 tests in 525.419s, 706 passed, 31 TMUX failures, no errors/skips; suite-synthetic.log retained.
Focused synthetic replay: 2/2 passed in 1.050s (failure-replay.log). Startup class: unset 7/7 in 0.857s, synthetic 7/7 in 0.875s (startup-*.log); revised CLI passes every run.
Remaining failures: eight FAIL entries in suite-synthetic-replay.log (TMUX fixture readiness/launcher failures); all raw attempts retained under /private/tmp/slack-listener-38.l5JysH/.
Initial single-test setup hit the macOS /tmp symlink invariant; explicit TMPDIR replay passed (1/1, 0.326s).
First harness replay caught a source-variable collision; fixed replay passed; both logs retained (harness.log, harness-final.log).
The brief's two findings are correct. Literal “qualify was never called” conflicts with the required suite:
existing tests call qualify on synthetic fixtures; no operational/live qualification was invoked. No Slack post or push.
Changed lines (added + removed): tests 37 + 3 = 40; harness 12 + 10 = 22; this report 38 + 0 = 38; outside tests total 60.
