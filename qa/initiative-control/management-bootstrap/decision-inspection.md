# DEC021 inspection correction — worker engineering receipt

Product initiative PF-80, active `initiative-delivery-control`, sprint PF-80-S01
`in_progress`. Product citation: **Internal delivery control — TO BUILD**,
“Use sequential sprints per initiative”. Root policy and the Corbanu development
skill route this work through the parent's September13 DEC021 allocation.
Original DEC021 expectations and prior partial results remain unchanged.

Binding: action `decision-inspection-01`, allocation digest
`031b09bda1e0a280aaa29fb5854afa8cefcd513a3eaa1d07f5bd74e27b9b82a4`,
claim `c581d873-0ed5-44c9-92c6-59050e8a85b8`, worker
`01a09cf4-ed86-78c1-a46a-b6920d4c5720`.
Worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/decision-inspection-20260913`,
branch `bootstrap/decision-inspection-20260913`; clean launch
`9bb2cbad55ae924544a54d7b829afbe9989359be`; declared creation base
`239f7f82d5d410adb936963ef6be891a9184bbdd` is an ancestor. No commit or push.

Literal changes: inspection +40/-0, feed +21/-8, attention +24/-7,
new tests +294/-0, this receipt +79/-0: 458 additions, 15 deletions,
473 changed lines (target500/hard700); implementation 100 changed (max350).

Schema 3 adds exactly `inspection` and `withheld_count` to the snapshot envelope.
It requires `status=invalid`, `feed=null`, nonempty strictly validated inspection
records, and a positive bounded integer count. The immutable artifact digest binds
both fields. Canonical payload/identity metadata stays null and health/open count
stays unknown. Schema 3 carries no Slack projection. Schema 1/2 behavior remains;
the prior reader's explicit version allowlist rejects schema 3.

Extraction first rejects invalid envelopes, ambiguous JSON keys, nonfinite JSON,
oversized raw/canonical bytes and untrusted private files. Each complete decision
history passes the unchanged strict validator independently. Every duplicate ID
copy is withheld, including a structurally broken duplicate; rejected contents,
IDs and paths never enter the decision artifact. No survivors retains the prior
invalid representation. Safe records and timestamps are preserved exactly.
Rendering prominently labels incomplete inspection, retains summaries, evidence,
history and exact sprint links, and makes no aggregate or operational claim.

Nine focused tests retain validated pre-poison baselines in each disposable run;
the full export/activate/publish test also retains its valid baseline generation.
They cover current/history/resolution/link canaries, duplicate IDs and invalid
histories, malformed envelopes/JSON, byte bounds, mode/UID/hardlink/symlink/FIFO/
directory rejection, partial transfer tampering, export input drift, exact links,
unknown health, repeated publication, canonical byte preservation and strict
Slack prepare/interpret/persist/dispatch rejection. Existing tests are read-only.
UID rejection uses a synthetic UID observation; no file ownership is changed.
Activation calls the actual installer with only its service calls stubbed inside
tests; its printed service-success message is not evidence of any live service.

Commands run from the worktree above (no packages installed):

```sh
export PYTHONPATH=/Volumes/CorbanuDrive/Corbanu/.codex-work/bootstrap-core-review.ZmRFnr/venv/lib/python3.14/site-packages
PF80_PY=/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/bin/python
"$PF80_PY" -B -m unittest discover -s scripts/initiative_control -p test_decision_inspection.py -v
"$PF80_PY" -B -m unittest discover -s scripts/initiative_control -p 'test_decision*.py'
"$PF80_PY" -B -m unittest discover -s scripts/initiative_control -p test_attention.py -v
"$PF80_PY" -B docs/plans/check.py
"$PF80_PY" -B docs/sprints/check.py
git diff --check
```

Focused: 9 tests pass, exit 0 (6.412s). Attention: 16 pass, exit 0 (0.047s).
Combined decisions: 124 pass, exit 0 (132.838s): 117 existing tests plus the
seven focused cases loaded before the final two test additions. All nine final
focused cases separately pass above; production code was unchanged. Governance:
3/3 active plans; 115 current and 126 archived sprints. Diff whitespace passes.
Retained failed attempts: requested interpreter alone failed import because it
lacks Slack SDK; existing local pinned SDK 3.44.1 supplied through PYTHONPATH.
Next attempt had six setup errors (initial revision 2 save) and one invalid test
expectation (queued resume returns retained status); corrected test fixtures,
then seven tests passed before adding two focused open/persistence regressions.

Parent owns Fable review, receiving/combined-tree verification, permission-isolated
code-blind functional execution and evidence review. This worker does not pass
DEC021/SLK cases, declare human readiness, current decision state, all-clear,
Slack authority, live delivery, running workers, product resumption or recurrence.
No live state, credentials, network/services, installs, Rust, other agents or
reviews were used. Native TUI and live TensorCash/Isometric journeys are outside
this internal document projection increment; no release/benchmark claim is made.
