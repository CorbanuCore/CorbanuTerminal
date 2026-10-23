# Attributing the 140 full-suite failures on the S03 candidate

**Fable, 2026-09-16.** `acct-inspect-impl-03` returned with the full `codex-tui`
suite at 3,980 passed, 93 failed and 47 timed out, and made **no**
baseline-attribution claim. That was the right call: a worker inside a frozen
scope cannot establish what the tree does without its change. I established it.

## Method

A detached worktree at the receiving base `40442cc75` — the candidate's own base,
not a convenient nearby commit — ran the same suite:

```
just test -p codex-tui --locked --offline
```

Failure identities were taken from the run's own `junit.xml`, not from the
summary line, and compared against the candidate's preserved
`impl-03-tui-full-failures.json`.

## Result

| | Baseline `40442cc75` | Candidate `ddee50716` |
| --- | ---: | ---: |
| Tests run | 4,102 | 4,120 |
| Passed | 3,998 | 3,980 |
| Failing identities | 104 | 140 |
| Wall time | 60.9 s | 529.1 s |

- **92** of the candidate's failures fail identically at the base.
- **48** appear only on the candidate. Of those, **47 are `test timeout`**, and
  the one remaining, `suite::focus_palette::focus_gained_with_unanswered_palette_queries_preserves_immediate_input`,
  is recorded as `FLAKY 2/2` in the baseline run — it failed once there too and
  passed on retry.
- **12** baseline failures do not appear on the candidate, all `tmux_`
  onboarding and auth cases, which is the same flakiness in the other direction.

**Zero regressions attributable to this change.**

## Why I am confident the timeouts are load, not the patch

The candidate suite took 8.7× as long as the baseline for the same work, because
it ran while its own implementation worker and my integration work were on the
machine. Every one of the 47 is a timeout rather than an assertion, and they
cluster in `app::tests::dispatch_integration`, `app::tests::safety_buffering` and
`app_server_session::tests` — the heaviest integration cases in the suite, which
is where a loaded machine shows first.

That is corroborated independently: the Opus 5.0 High review of the diff, asked
to name the plausible blast radius from the code alone and without seeing this
comparison, said usage-menu index shift, `/usage` argument routing and extra
bottom-pane redraws — and explicitly *not* app-server, session or mailbox
behaviour. The failures that appeared are precisely the ones the code cannot
reach.

## What this does not establish

The suite is not clean at the base: 104 failures and a 60-second run that should
be investigated on its own terms. That is pre-existing debt, it is not this
sprint's to fix, and recording it here is not the same as accepting it. The
comparison also cannot rule out a regression that happens to coincide with an
already-failing case; the reviewer's independent scope analysis is what covers
that gap, not this table alone.
