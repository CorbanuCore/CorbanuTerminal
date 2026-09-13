# PF-80-S01 read-only Slack reply collector — internal engineering handoff

Change class: product initiative, internal delivery control. Product citation:
`docs/corbanu-product-spec.md`, **Internal delivery control — TO BUILD**,
“Use sequential sprints per initiative”. Active plan:
`docs/plans/active/initiative-delivery-control.md`; sprint PF-80-S01 remains
`in_progress`. Skill applied: `corbanu-terminal-development`; root policy read.
The parent's September 13 **Read-only Slack reply collection allocation** and
active-plan coordinates authorize these three files; policy/plan ledgers are
parent-owned and unchanged here.

Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/slack-reply-poll-20260913`.
Branch: `bootstrap/slack-reply-poll-20260913`.
Base: `327eade129e186a5c66a3bacbf60c2342771de86`. Uncommitted tested diff.

## Owner invocation contract

Import `collect` from `scripts/initiative_control/slack_reply_poll.py` using the
existing scripts import convention. Call `collect(web, coordinator, team=...,
bot=..., human=..., channel=..., threads={parent_ts: decision_id}, max_pages=4,
page_size=15)`. Supply an authenticated synchronous WebClient and initialized
owner-only Coordinator. The injected client must have `retry_handlers=[]`, a
positive timeout <=30 seconds, and a logger with `disabled=True`. No client is
constructed, credentials loaded, or qualification journal consulted or modified.

Each invocation verifies `auth_test` team/bot identity, then reads only exact
owner-mapped threads from the pinned channel. The first page must contain the
pinned bot's parent. Replies require exact thread and human author, matching
team/channel when present, supported subtype and valid timestamps. Edited human
replies also require the pinned editor. Original UTF-8 text up to 16,000 bytes
is stored unchanged inside private Coordinator events; oversized/malformed text
produces an explicit issue, never a truncated answer. Reply text never supplies
decision IDs, approvals, resolutions, or dispatch instructions.

Parent-identified live-shape correction: the pinned bot's threaded follow-ups
are counted as `ignored`, provided bot ID, authenticated bot user, thread, scope,
timestamp and subtype match. They are neither human evidence nor coverage errors.
Other bots/authors remain explicit denials. The representative parent + human +
own-bot follow-up fixture passes; the initial draft behavior is retained here as
a corrected parent finding, not a previously passing live acceptance result.

Event IDs hash immutable pins, decision mapping, message timestamp, edited
timestamp and original text. Duplicate reads/restarts do not advance coordinator
revision; distinct edits and reversions remain separate. `Coordinator.event`
commits before observation counters/receipt return, even with dispatch paused.
An exception after commit reports `persistence_uncertain`; a full retry safely
deduplicates. No separate persistence framework, cursor store or Slack journal.

Bounds: 1..16 threads, 1..64 replies pages total, 1..100 items per page, plus one
authentication call. No automatic retries or sleeps. Every invocation starts at
the first page, including older replies/edits. Short/empty pages with a cursor
continue; absent termination evidence, missing cursors and cursor cycles fail.
Receipts expose per-thread pages, issues, ignored/committed/duplicate counts,
end-of-scan coverage and next cursor. A spent budget leaves partial/unscanned
threads. Cursors are diagnostic continuation evidence, not resumable input;
owner reruns from the start with sufficient budget or a narrower pinned map.
A thread beyond the hard scan bound remains partial and requires parent action.

`scan_complete` means only that these API reads reached their indicated ends
without rejected evidence. Reads are non-atomic; unseen deletions, intermediate
edits and new arrivals remain unknown. No deletion is inferred from absence.
Failures/429 expose safe codes and numeric Retry-After without exception bodies
or reply text. See Slack's [replies contract](https://docs.slack.dev/reference/methods/conversations.replies/).

## Verification and remaining gates

- Final collector: **15 tests pass**, including real SDK 3.44.1 WebClient and
  SlackResponse/SlackApiError paths with HTTP fully stubbed (0.079s).
- Existing coordinator: **37 pass** (0.810s); existing decision replies:
  **30 pass** (1.424s). Both governance checkers passed before implementation:
  3/3 active plans; 115 current / 126 archived sprints.
- Supplemental suite started before the final corrections: **359 pass**
  (257.800s), with existing HTTP-fixture ResourceWarnings. It loaded the earlier
  collector tests; the final 15-test rerun above qualifies the corrected slice.
- Whitespace checks are clean for all three new files. Final size: **606 total /
  308 non-test lines**, under the allocation targets of 650/350.
- Preserved failed attempt: system Python has no `slack_sdk`; initial broad
  `test_slack*.py` discovery ran 13 passing collector fixtures plus one transport
  import error (`ModuleNotFoundError`). No existing test was edited or removed.
- SDK checks use the existing `.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python`
  with `-B`; focused command: `-m unittest discover -s scripts/initiative_control
  -p test_slack_reply_poll.py -v`. No dependency installation was performed.
- Scope: only collector, its test file and this handoff; no commits, pushes,
  reviewers, other agents, scheduling, sends, live calls or credential reads.
- Internal engineering only: true-TUI, live-repository benchmarks and isolated
  user-facing acceptance are not claimed by these fixtures. Parent owns the one
  Fable High review, receiving integration/tests, approved live replay, later
  Socket Mode/phone/native ACK qualification and any recurring enablement.
  Parent reported a successful approved live read separately; this worker did
  not execute it. No sprint completion, human sign-off or release claim.

Parent material review01 completed clean with exit0, Fable 5.1 High through the
recorded Corbanu wrapper and autoreview helper. Exact receipt:
`.codex-work/bootstrap-poll-review.JBT5DY/review01.json`. No findings or code edits.
Initial review preflight refused the untracked synthetic SDK fixture token;
parent inspected all three files, confirmed no real credential and staged them
before review. Parent's initial system-Python run repeated the missing-SDK import
error; exact SDK-venv rerun passed all15 in0.082s. Both failed attempts are retained
in private `attempts.md`; they are not acceptance. Parent accepts this internal-only
increment's N/A; later actual Slack/isolated functional gates remain required.
