# A safety test started failing and no code had changed

**Fable, 2026-09-16.** A full-suite run at the integration tip reported two
failures beyond the five known `slack_sdk` import errors, both in
`test_owner_tmux.TmuxTests.test_bridge_incomplete_and_malformed_rollouts_never_issue_evidence`
— the test that proves the TMUX bridge never issues evidence from an incomplete
or malformed rollout. That is a safety property, so it was worth stopping for.

## It was not a regression

The same test fails identically at `40442cc75`, this morning's tip, before most
of today's merges. Nothing merged today touches `owner_tmux.py`, and earlier
full-suite runs today reported only the five import errors, so it passed then and
fails now on the same code.

The failure is timing: `deliver()` asserts the elapsed handoff stays inside a
budget, and the `partial` mode expects more than one read. Under load the first
read consumes the budget and only one read happens. The file's own history —
"retry pending TMUX bridge baseline within handoff deadline", "Fix TMUX bridge
append polling and capture freshness budgets", "Reserve a collection window
before the durable bridge attempt" — says this has been fragile before.

## The cause was me

```
load averages: 15.43 6.74 5.56
108 tmux processes
```

Eighty-nine of those were worker TUIs still running long after their actions had
been received or failed. I had launched every one of them and reaped none. At
load 15 the test failed; after reaping, at load 8, it passed twice in a row, and
the full suite returned to exactly the five known import errors.

| | Load | Result |
| --- | ---: | --- |
| Before reaping | 15.43 | FAIL (`partial`) + ERROR (`malformed`) |
| After reaping | 7.68 | OK, twice |

## What changed

`reap.sh` kills the tmux server for any worker whose action has reached a
terminal state, and it runs inside the poll cycle beside the feed refresh and the
allocation compaction. It never touches a session whose action is still live, and
never the default tmux server, which carries work I did not launch. Workers from
before this run have no claim record, so they cannot be attributed and are left
alone — 110 were kept on that rule, which is the conservative direction.

## The part worth remembering

I nearly filed this as a regression in a safety test. It was my own uncollected
garbage, and the only reason I caught it is that the same test had passed in
earlier runs of the same day on the same code, which made "what changed?" a
question about the machine rather than about the diff.

Two hygiene tasks have now each produced a real defect: uncompacted allocations
broke the manager briefing, and unreaped sessions broke a test. Both were
invisible until they were not, and both had been accumulating for hours while I
watched more interesting things.
