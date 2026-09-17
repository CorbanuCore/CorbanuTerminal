# RETURN

Allocation `acct-disclose-105`; claim `3d616882-c1a5-4b6e-96e4-dbe5a69751ab`.
Worker: `gpt-6-astra`, effort `high`. ACK preceded START and tools.
Base: `a72f03c3cc895a28df55b26bf7be756ecfbc3d5a`.
Brief SHA-256 verified:
`5f355d302a05ae2db898e38a0507be0bdf7f1579d73f2651c46e7f46f2b8f18a`.

The [acceptance disclosure](../acct-acceptance-75/acceptance.md) now reads:

> The manager reports five independent automated adversarial reviews before the
> round-103 revision, run at `claude-opus-5-plan` with `high` effort, with author and
> reviewer separated and each review blocking receipt until its findings were
> answered; that count is supplied review history, not an independently verified
> review roster or owner approval. No human has reviewed this record.

The five local review launch scripts name that exact model and effort;
[provenance](review-provenance.json) identifies them. The count, separation,
receipt-blocking history and absence of human review remain manager-supplied
history; this worker does not independently certify that roster.

The [corrected refusal table](../acct-guarantee-103/RETURN.md) is:

| Refusal | Message after STOP: | Exit |
| --- | --- | --- |
| Older Python | destination guard requires Python 3.9 or newer. | 1 |
| Existing destination (including a symlink to an existing target) | destination already exists; preserve it and choose a new path. | 1 |
| Dangling destination symlink | destination is a symlink; preserve it and choose a new path. | 1 |
| Failed inside-source ignore check | inside-source destination did not pass git check-ignore; choose an ignored path. | Git's nonzero status (1 observed for unignored paths) |

The guard itself is unchanged. This round directly exercised both symlink
branches; [checks](checks.json) retain their observed messages and exit 1.

The [witness](../acct-final-100/test_destination.py) now rebinds its summary names
from the unique recorded observation before any receipt is written:

```python
alias_records = [row for row in records if row["case"] == "alias-unignored-regression"]
if len(alias_records) != 1 or "old_guard_exit" not in alias_records[0]:
    raise RuntimeError("missing unique pinned alias-unignored-regression observation")
observed_old, observed_new = alias_records[0]["old_guard_exit"], alias_records[0]["exit"]
```

Normal and reversed-order runs each passed seven cases and reported observed
old=0/new=1, under ordinary Python and `-O`. Renaming the case failed with the
explicit missing-observation RuntimeError, exit 1, and no receipt, in both
modes. All eight focused checks passed: four successful witness runs, two
expected missing-observation failures, and two expected symlink refusals.
Raw stdout/stderr and successful witness receipts are retained here.

Prerequisites built first, exit 0. The requested gates ran sequentially from
`codex-rs`, through guarded `just test`, sharing
`acct-activation-33/feature/target` with `NEXTEST_TEST_THREADS=4`,
two build jobs and debug assertions enabled.

| Lane | Passed/run | Skipped | Failed | Timed out | Flaky | Leaky | Exit | Failure names |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Core accounting default | 124/124 | 3545 | 0 | 0 | 0 | 0 | 0 | [] |
| Core accounting developer-accounting | 127/127 | 3545 | 0 | 0 | 0 | 0 | 0 | [] |
| TUI usage | 91/91 | 4078 | 0 | 0 | 0 | 0 | 0 | [] |

[Gate receipts and lossless logs](gates-01/) retain exact commands, counts and
SHA-256 hashes. All three test lanes emitted the isolation banner. No native
credential prompt or live-profile read was observed; no failed Rust gate was
retried. The 342 passing executions overlap and are not distinct test cases.

Changed existing files: **+13/-6 across three files**:
acceptance.md +7/-4, test_destination.py +4/-0, prior RETURN.md +2/-2.
[Exact hunks](changed-lines.patch) are retained. New files under this round are
verification helpers, evidence and this RETURN; [scope inventory](scope.json)
accounts for their lines/bytes and hashes, excluding itself.

Reported and not done, as required by the brief's stop boundary:

- The acceptance paragraph edit invalidates its stored inventory entry and
  therefore the exact-output baseline. A read-only verifier run exits **3**:
  **18 agreement, 2 disagreement, 3 unavailable**. The disagreements are
  acceptance inventory bytes/lines/hash and numerical coverage, which flags
  `103` and the `5` in the exact model name. [Raw output](acceptance.stdout.txt).
  The inventory/reference and numerical-checker treatment need a scoped
  follow-up decision; none were refreshed, weakened or claimed passing.
- The prior review also noted that round 103's RETURN says the paragraph names
  “remaining defects” where it names defects found and corrected. That adjacent
  wording is outside the three requested corrections and remains unchanged.
- Existing S03 functional qualification and human acceptance remain open.
  This round does not replace historical evidence or claim acceptance readiness.

Brief precision: renaming/removing the special case can omit the binding;
reordering alone, with that case retained, does not. Reordered execution is
explicitly tested. No other factual error was found in the requested fixes.
The brief's clean prior-verifier assessment describes the prior tree, not this
changed paragraph.

Classification: routine internal evidence correction. Used
`corbanu-terminal-development`; product heading **Measurement targets**,
excerpt “No commercial performance numbers have been supplied.” No plan/sprint
lifecycle change. True-TUI, code-blind functional, live-repository and benchmark
qualification are N/A to this internal correction, which changes no product
workflow and makes no functional handoff; integrator acceptance of that N/A
remains with Fable and the later S03 functional gate remains required.
No workspace formatter, Rust/product edit, commit, push, approval or release.
