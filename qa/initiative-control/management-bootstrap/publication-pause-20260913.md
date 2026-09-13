# Source publication must preserve the pause

PF-80-S01, in_progress; **Internal delivery control — TO BUILD**, “Use sequential
sprints per initiative”. Parent allocation is in the linked bootstrap research
record. This is an internal operations correction, not functional acceptance of
the combined dashboard/Slack workflow. Its independent isolated gate remains open.

## Observed failure and containment

Native Luna Extra High Maxwell `01a09bfd-f665-78a1-8f00-4a962d06bdef` ran the
existing sync wrapper once against clean pushed14019e412c91e9629f36a3cdcf21f59494512cc8.
Export `.codex-work/initiative-control.oGQGyA/export.90ntgY` produced
build-sagn5ou_, collected2026-09-13T18:19:20+00:00 and published18:20:15+00:00.
Tree digest7b86876119b35be10f01486e4dec196636202cac0960739156992f9372786774.
Both local8769 and remote8768 Facilities returned200 with X-Corbanu-Control:1.
However, activate.py unconditionally enabled/started the publication timer.
Worker correctly reported enabled/active instead of claiming a healthy pause.

Parent disabled only corbanu-control-publish.timer on the existing RPC publisher;
actual systemctl observation returned disabled/inactive, while the web service
remained active, PID1961479. Existing unit files and reports were retained.
This is reversible schedule containment, not deletion or a product-work restart.

## Correction and final-tree evidence

Remove implicit timer activation, retaining one-shot publication and web refresh.
No timer enable/disable is issued by activation. An enabled timer stays enabled;
a disabled/new installation stays unscheduled. Scheduling requires its separate
owner action. No new scheduler, CLI flag, credential or native product change.

The new exact-service-command assertion first failed against the old code:
37 tests, one failure proving the unintended timer command. After correction,
37 dashboard tests passed in0.239s. Broader decision-feed tests initially had
six failures among19 because their helper still expected five service calls.
That assertion now expects four and rejects any timer command; all19 passed
in32.157s. Total56 passing tests, including real read-only HTTP checks.
An initial dashboard-venv import lacked slack_sdk; the existing combined venv
was used without installation. Direct execution of test_decision_feed.py ran
no tests and is not counted; unittest discovery produced the actual19-test run.
Plan checker:3/3; sprint checker:115current/126archived; diff whitespace clean.

Two actual Fable5.1High reviews under the existing autoreview wrapper:
review01 session01a09c06-882e-7363-b31b-24f007ac5eed clean, then test-only
correction review02 session01a09c08-aac2-7ec3-a78a-ea14a7908ce7 clean, both exit0.
No production change followed review01. Private review artifacts are retained at
`.codex-work/publication-pause-review.0yRhul/`; allowance and earlier failures
are preserved, not reset. No extra unchanged-code opinion is required.

Corrected-source deployment and timer-state proof remain the next bounded
publication assignment. Normal product dispatch/recurrence remains paused;
private HTTPS, combined Slack/native ACK, confined browser execution and full
management rehearsal remain separate open gates. No completion Slack ping yet.
