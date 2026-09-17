# RETURN

Action `acct-criterion-97`; allocation digest
`6bebbcd5c8efc9a8c4865a7764cc7940346f21a40a648efca473de254e5e4af5`;
claim `2bcbd4ed-5482-44d6-9e09-290f3a8913d9`. Worker: `gpt-6-astra`, high.
The brief SHA-256 matched
`801cf69be07f97b1f2e3b539846627730a13249716615f2383fd62be1adcf94e`.
Base and replayed candidate: `2903d37c60dff3b8bec469fcf82347729b53f7fd`.

The [corrected operator note](../acct-derive-95/OPERATOR.md) and its
[frozen executed copy](OPERATOR.executed.md) contain these decisions:

| Source-status case | Required action |
| --- | --- |
| Empty status | Clone and replay the printed committed candidate. |
| Ignored scratch/cache outputs or untracked artifacts outside inventories and replay/build inputs, whose omission cannot change the checked claim | Preserve status; record paths and irrelevance in `REPLAY_OMISSIONS_NOTE`; rerun block 1 unchanged. Report candidate-only replay with those omissions. |
| Any staged/unstaged tracked change, inventoried or required input, intended candidate change, or uncertain relevance | Stop. Preserve changes; obtain the integrator's intended committed candidate or use a separate clean checkout. Resolve relevant/uncertain omissions before rerunning. Never silently discard changes or claim HEAD includes them. |

Ignore/untracked status alone does not establish irrelevance. Classification of
omissions is an explicit operator assessment, not an automated proof.
The block checks staged and unstaged differences separately, refuses tracked
changes even with an omission explanation, and requires an explanation for
untracked/ignored entries. Local ignored historical binaries remain unavailable
to the clean-checkout claim.

The contradictory working-directory claim was dropped. Each remaining block
requires entering the printed audit root and guards that requirement in-block
using the Git top-level path plus local `corbanu.replayRoot` and
`corbanu.replayCandidate` markers. All five wrong-root executions exited 1
before replay/build/tests. These markers prevent accidental misrouting; they
do not provide independent authenticity.

Destination:
`audit=${CORBANU_AUDIT_DIR:-"$PWD/$q/acct-criterion-97/target/audit-copy"}`.
Set `CORBANU_AUDIT_DIR` to a new absolute path to relocate or repeat a run.
Preserve earlier destinations/transcripts. Existing destinations are refused.
Inside-source destinations must be ignored; outside-source destinations work.
The default path, relocated path, existing-destination refusal, and second run
at a new path were all executed successfully with their expected exits.

The [complete verbatim transcript](transcript.txt) has 6,881 lines, preserving
exact shell inputs, explicit environment inputs, working directories, merged
raw stdout/stderr, and exits. [Results and receipts](results.json) bind each
script/output by SHA-256; individual JSON receipts and lossless `.log.gz`
files are retained. All 32 recorded attempts matched their expected exits.

The final six blocks ran unchanged, each in a fresh
`bash --noprofile --norc` process. A fresh full-history source clone supplied
the candidate; block 1 created another fresh detached audit clone at
`acct-criterion-97/target/final-audit-copy`. The revised note was an external
frozen instruction, not copied over candidate files. The source deliberately
contained an untracked synthetic text artifact and ignored scratch output.
The transcript records refusal without assessment, then successful candidate
replay with both omissions named. Separate tracked-edit and staged-cancellation
fixtures both stopped. The final audit clone's tracked/untracked status was empty.

Final note SHA-256:
`d46df323a5d5538c6b4e8bddec84aa08266346d5cc827bc7ac5581d204457864`.
Transcript SHA-256:
`b44f78b644072cbbf49c6cf9e2d954baa097297e119b6906ae0ab5b0f271164f`.

| Lane | Passed / run | Skipped | Failed | Timed out | Failure names | Exit |
| --- | ---: | ---: | ---: | ---: | --- | ---: |
| Core default accounting | 124 / 124 | 3545 | 0 | 0 | `[]` | 0 |
| Core developer-accounting | 127 / 127 | 3545 | 0 | 0 | `[]` | 0 |
| TUI usage | 91 / 91 | 4078 | 0 | 0 | `[]` | 0 |

Prerequisites passed first, exit 0. The three lanes ran from the final clone's
`codex-rs` via the exact requested `just test` commands, sharing one initially
empty dedicated `CARGO_TARGET_DIR`, with `NEXTEST_TEST_THREADS=4`.
All emitted the required disposable-profile/native-keyring-disabled banner.
No native credential prompt or live-profile read was observed. There were
342 overlapping executions, not 342 distinct tests; no failed test was retried.
The historical auxiliary scope/event parity test passed in 31.405s default
and 31.245s feature. These passes do not resolve its retained historical timeout.
Compiler warnings remain in the transcript.

Acceptance reported `BASELINE MATCH` and
`RESULT agreement=20 disagreement=0 unavailable=3 exit=2`; all 13 controls
passed. Replay matched expected stdout, with nested exits 2/0, empty stderr
and clean status. [Raw replay streams and receipt](replay-streams/receipt.json)
are retained. The self-reference disclosure and three-binary exclusion remain:
this demonstrates internal candidate consistency, not independent acceptance.

Earlier notes and attempts are preserved as `OPERATOR.initial.md`,
`OPERATOR.intermediate.md`, and their logs/receipts. During preparation, the
destination existence tests were separated to avoid Bash `set -e`/AND-list
behavior, and tracked checks were split to catch staged changes canceled in
the working tree. Final refusal cases and the full six-block run were then
executed from the final frozen note. One preliminary build's original runner
sampled the note hash at completion after the note changed; `results.json`
explicitly records its intermediate launch note and identical block bytes.
The runner now freezes that hash before execution. The final note did not
change during the final six-block run.

Changed operator-note lines: **83 added, 14 removed**; exact hunks are in
[changed-lines.patch](changed-lines.patch). Other additions are solely this
round's runner/collector and preserved evidence under `acct-criterion-97/`.
`git diff --check` passed; runner/collector syntax parsed successfully.
All workspace changes remain within the assigned QA scope. No Rust/product
edit, workspace formatter, commit, push, or approval fabrication occurred.

Nothing in the supplied brief was found to be wrong. Its three findings were
reproduced or confirmed and addressed.

Classification: routine audit-note/evidence correction. Product context:
**Measurement targets**, “No commercial performance numbers have been supplied.”
No plan/sprint lifecycle change is claimed. True-TUI/code-blind/live-repository
qualification is N/A for this internal revision, which changes no product
workflow and makes no functional handoff; the separate S03 functional gate
remains open. Scope-zero, settlement, historical-package, platform/profile,
live-repository and independent-acceptance gaps remain unchanged. No human
acceptance, benchmark qualification, S03 acceptance or release authority is claimed.
