# RETURN

Action `acct-divergence-102`; allocation digest
`22270be5ce0bc85f8ceed050ee8ab7467fbdc8c61648826b89f75f4f0b0b10ad`;
claim `faa70ecc-9c82-4885-aba9-36f01c41fb1b`.
Worker: `gpt-6-astra`, `high`. ACK preceded START and every tool call.
Brief SHA-256 verified:
`d674f9bfdd1969f0e909b51419f3c4ad0fde44411fda508cdbb0e723880e4fc9`.
Assigned/observed base: `a8dfff98892e60aaa0d7f05321fd7e57b79cdc2a`;
initial working tree clean.

**Divergence disclosed, not equivalence restored.** Both the
[live operator note](../acct-derive-95/OPERATOR.md) and
[round-97 RETURN](../acct-criterion-97/RETURN.md) now identify the changes since
the frozen executed note: only **block 1** changed. Round 100 replaced lexical
inside-source matching with resolved-path ancestry and ignore checks; round 102
added an early Python-version refusal, with the check before pathlib/subprocess
imports. Provenance and Python prerequisite prose also changed. Blocks **2–6**
remain byte-identical. Round 97's complete verbatim execution claim applies only
to its frozen bytes, not today's note. No fresh complete six-block execution is
claimed for this revision.

[Final note checks](note-check-final.json) bind each frozen block to its
round-97 command/script hash, note hash and decompressed log hash, all six with
exit 0; they also check the retained transcript hash. Frozen note SHA-256:
`d46df323a5d5538c6b4e8bddec84aa08266346d5cc827bc7ac5581d204457864`.
Live final note SHA-256:
`338baf8fc210e86b49f0debc60a8b2ffe25aac158cc611aadbfd0eee4f1aa819`.
No frozen note, transcript, raw log or historical execution receipt was edited.
`note-check.json` preserves the preliminary check before the import-order
refinement; `note-check-final.json` is the check of the final note.

**Pinned regression witness.** The amended
[historical helper](../acct-final-100/test_destination.py) reads the old note from
`967425cc72ea28b5f59ecc38a56419b7db6885e8`, the commit where round 100 observed
the old behavior, and the revised note from
`a8dfff98892e60aaa0d7f05321fd7e57b79cdc2a`, which contains the round-100 bytes.
[Pinned reproduction](pinned-destination-results.json) passed all **7** cases,
including the outside symlink into an unignored source path: **old=0, new=1**.
Both note and extracted-guard hashes are recorded; the revised note hash matches
the original round-100 receipt. This is a historical destination-segment witness,
not a current-note or full-clone replay. Python 3.9+ is required to rerun that
historical guard. From the repository root:

```sh
python3 -B qa/portfolio/agent-cost-accounting/pf-60-s03/acct-final-100/test_destination.py
```

The default writes a fresh receipt under ignored scratch and was also exercised
successfully. An optional `--output` selects a new receipt path; exclusive
creation preserves existing receipts. Round 100's RETURN discloses the pinning
and that its earlier scope/final-check hashes predate these amendments.

**Version refusal.** The current guard checks `sys.version_info < (3, 9)`
before importing pathlib or resolving paths. It prints exactly
`STOP: destination guard requires Python 3.9 or newer.` and exits 1 before
cloning. The current destination segment passed **7/7** cases. Executing its
exact Python heredoc on the installed interpreter with simulated version values
3.8 and 3.9 produced **2/2** expected outcomes: refusal exit 1 with that stdout
and empty stderr; supported-version external destination exit 0 with empty
stdout/stderr. These are branch simulations, not actual old-interpreter
qualification. The final receipt preserves the launcher, heredoc and outputs.

**Adversarial reread.** [The single narrow manual reread](prose-review.md)
examines the touched equivalence, execution and reproduction sentences.
Verdict: no unsupported claim in those revised sentences was found. The current
note does not inherit round 97's verbatim-run claim; the historical witness is
fixed to the bytes its receipt names; version-branch evidence is explicitly
limited. This is the revise worker's own read, not independent acceptance.
It does not clear the unrelated historical prose findings in round 100.

Prerequisites built first, exit **0**. Tests then ran sequentially from this
checkout's `codex-rs` through guarded `just test`, with
`NEXTEST_TEST_THREADS=4`, two build jobs, debug assertions enabled and shared
dedicated `acct-activation-33/feature/target`. The required isolation document
was read before testing. [Lane receipts and lossless logs](gates-01/) record:

| Exact command | Passed/run | Skipped | Failed | Timed out | Flaky | Leaky | Exit | Failure names |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `just test -p codex-core accounting` | 124/124 | 3545 | 0 | 0 | 0 | 0 | 0 | [] |
| `just test -p codex-core accounting --features codex-core/developer-accounting` | 127/127 | 3545 | 0 | 0 | 0 | 0 | 0 | [] |
| `just test -p codex-tui usage` | 91/91 | 4078 | 0 | 0 | 0 | 0 | 0 | [] |

These are **342 overlapping passing executions**, not distinct tests.
All three lanes emitted the required disposable-profile/native-keyring-disabled
banner. No native credential prompt or live-profile read was observed.
The historically intermittent auxiliary-scope test passed in 31.735s default
and 31.468s feature; that does not resolve its historical timeout. No failed
lane was retried. Compiler warnings remain in the retained logs.

Existing-file changes: **+66/-10** across four files:
round-97 RETURN +12/-3; live OPERATOR +19/-2; round-100 RETURN +16/-0;
round-100 witness +19/-5. [Exact hunks](changed-lines.patch) are preserved.
All new helpers, receipts, logs and this report are under `acct-divergence-102/`.
[File accounting](scope.json) records per-file line counts and hashes, excluding
its own self-entry; [final checks](final-check.json) validate scope, whitespace,
Python syntax, final note hash, pinned historical hashes and all gate logs.

**Brief precision.** The three requested defects were confirmed; no factual
error was found in those findings or the gate instructions. “Round 100 is NOT
received” and “last accounting round before the owner's decision” are manager
context, not status/authority independently established here. No approval or
owner decision is inferred.

Classification: routine internal audit-note/evidence correction. Product context:
**Measurement targets**, “No commercial performance numbers have been supplied.”
Used the repository `corbanu-terminal-development` skill. No product code or
plan/sprint lifecycle changes. Code-blind functional design/execution, true-TUI,
live-repository and benchmark qualification are N/A to this internal evidence
revision; the later S03 functional gate remains open and integrator acceptance
of that N/A remains with Fable. No human-test readiness, S03 acceptance or
release qualification is claimed. No workspace formatter, commit or push.
