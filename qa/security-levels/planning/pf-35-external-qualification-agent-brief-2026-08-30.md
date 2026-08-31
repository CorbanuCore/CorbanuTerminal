# PF-35 external qualification agent brief

Use this document as the copy-ready handoff for the agent who will own PF-35's
remaining classifier qualification. It supplements, but does not replace, the
active plan and sprint records.

## Mission

Complete PF-35's remaining externally measured qualification without weakening
its fail-closed evaluator contract or exposing private blind data. The immediate
unit is
[PF-35-S01](../../../docs/sprints/current/p0-security-levels/pf-35-s01-classifier-corpus-and-evaluation.md):
freeze the licensed corpus, grouped splits and independent blind custody. The
model artifact and N100 qualification belong to S02; calibrated blind evaluation
and ingress enforcement belong to S03. Advance only one allocated sprint at a
time and hand each result to the integration owner for final-tree verification
and archival.

Product authority: **Non-negotiable controls** — “Classify instruction intent
and provenance before external content can influence tools or financial
actions.” The feature contract is
[PF-35 — Local classifier and blind qualification](../../../docs/plans/active/p0-security-levels.md#pf-35).

## Authority gate before writes

The previous PF-35 implementation was merged to `main`; its branch, worktree,
owner and base in the sprint record describe that completed implementation
round. They are not a standing allocation for a new agent.

Before changing repository or external campaign state, the integration owner
must:

1. name the new execution owner;
2. record a current `origin/main` base, distinct branch and CorbanuDrive
   worktree in the active plan and PF-35-S01 front matter;
3. confirm a literal write scope and integration gate that do not overlap the
   other active lanes;
4. run `python3 docs/plans/check.py` and `python3 docs/sprints/check.py`; and
5. identify the human blind-data custodian and the people authorized to operate
   the RTX, signing and Intel N100 systems; and
6. amend the plan/sprint ledgers to remove the current circular dependency: S01
   presently asks for model-dependent blind and N100 evidence even though S02,
   which owns that artifact, depends on S01. Preserve the product targets while
   assigning corpus/custody freeze to S01, artifact/N100 proof to S02 and final
   blind/calibration proof to S03.

If any item is missing, perform read-only discovery and report the precise
missing allocation or access. Do not make an implementation commit from the
shared `main` checkout and do not infer authority from this brief.

## Decisions already made

- Generate an English, synthetic-first corpus using pinned Qwen3.5-27B through
  pinned vLLM on the owner-supplied RTX 6000 Pro host. Stop ComfyUI during the
  campaign.
- Supplement synthetic data only with commercial-safe sources whose exact
  origin, revision, SHA-256, license, permitted use, attribution and
  transformations are recorded. Allowed baseline licenses are CC0-1.0,
  CC-BY-4.0, Apache-2.0, MIT, BSD-2-Clause and BSD-3-Clause.
- Target approximately 250,000 accepted training records, 25,000 accepted
  development/calibration records and 150,000 separately custodied blind
  records. Work in versioned rounds and reassess scaling after the first 10,000
  accepted records.
- Keep blind records and labels encrypted under a human custodian. Training
  agents, model builders, reviewers and Git receive no record-level blind data.
- Use DeBERTa-v3-xsmall exported as optimized INT8 ONNX Runtime as the primary
  detector path. A custom lightweight classifier is allowed only after recorded
  evidence shows that the primary cannot meet the gates.
- Treat Intel N100, 16 GiB RAM, x86-64 Linux as the provisional weakest
  supported CPU. Measure complete 2,048-token tokenization plus inference.
- Map one calibrated score through two signed thresholds to
  `allow`/`suspicious`/`hostile`. Artifact, runtime or resource failure produces
  `unavailable`; it never silently produces `allow`.
- Support English initially. Unsupported or uncertain language fails closed;
  multilingual attacks remain adversarial fixtures.
- Use an offline human-custodied Ed25519 root to authorize a rotating release
  key. Publish immutable signed assets, verify locally, install atomically and
  retain rollback. Never place a private key in Git, chat, logs or reviewer
  context.

## Existing foundation — preserve it

Start by reading:

- [classifier preparation README](../classifier/README.md)
- [corpus manifest](../classifier/corpus-manifest.json)
- [split manifest](../classifier/split-manifest.json)
- [PF-35-S01 evidence](../sprints/PF-35-S01/evidence.md)
- `scripts/security_classifier_eval.py`
- `scripts/test_security_classifier_eval.py`
- `codex-rs/content-security/`

The checked-in evaluator is intentionally strict. It binds manifests, inputs,
model and threshold identities; rejects record-level blind material; computes
Wilson confidence bounds; requires group-tuple accounting; and leaves artifact,
signature, overlap-audit and hardware claims unqualified until independently
verified. Extend it only when required by real evidence. Do not loosen a schema
or gate to make an external result pass.

All corpus, model, build, cache, temporary and review data must live on
CorbanuDrive or the explicitly authorized external machines, never the Mac's
main system drive. The local gitignored `AgentCredentials.md` is an index to
approved test-machine credentials; do not copy credentials from it into Git,
chat, evidence, prompts or logs. Confirm separately whether the RTX and N100
hosts are represented there.

## Execution stages

### Stage 1 — access and reproducibility inventory

- Record secret-free machine facts for the RTX host: GPU, VRAM, OS, driver,
  CUDA, container/runtime and storage paths.
- Pin the exact Qwen3.5-27B revision, tokenizer, vLLM version, container digest,
  prompt templates, seeds and sampling settings.
- Establish an append-only campaign ledger containing campaign/round IDs,
  timestamps, operator, prompt/config hashes, source-manifest hash, accepted and
  rejected counts, output hashes and adjudication/audit counts.
- Freeze a source ledger with license evidence before ingesting any open source.
  Ambiguous or incompatible licensing excludes that source.
- Confirm private storage, backup, deletion and transfer procedures with the
  human blind custodian. Do not create the blind split in a training-visible
  location, even temporarily.

The agent must return a readiness report before bulk generation. Missing RTX,
custodian, signing or N100 access is a measured blocker, not permission to mock
the result.

### Stage 2 — 10,000-acceptance pilot

- Generate a balanced pilot across the split manifest's coverage dimensions,
  attack families, benign hard negatives, source positions and cross-segment
  cases.
- Keep every derivative of a source template in one canonical group so it
  cannot cross train/development/blind boundaries.
- Treat generated labels as provisional. Human-adjudicate every disagreement,
  low-confidence and suspicious record.
- Audit non-overlapping stratified samples of high-confidence agreements: 1%
  by a human and 1% by Claude Opus 5 Max. The reviewer receives only the
  authorized sample, never blind records or production secrets.
- Report acceptance rate, duplication, class/coverage balance, adjudication
  load, error taxonomy, throughput and projected storage/runtime. Ask product
  authority before changing targets or audit fractions.

Proceed to the full campaign only after the integration owner accepts the pilot
and confirms that the adjudication method is scalable enough. Falling back to a
smaller final corpus requires a product decision and may not weaken statistical
minimums.

### Stage 3 — freeze train and development corpora

- Run versioned generation/import rounds to the accepted targets.
- Perform exact, near-duplicate and semantic-group checks before freezing the
  splits. Reject contradictory labels and cross-split groups.
- Preserve raw and accepted dataset hashes, but commit only secret-free source,
  campaign, schema and aggregate evidence. Do not commit record data, model
  weights, credentials, customer information or protected financial data.
- Produce the development predictions/report through the checked-in evaluator.
  Development metrics guide model selection and calibration but are not blind
  qualification.

### Stage 4 — seal the independent blind corpus

The human custodian or a separately authorized evaluator performs this stage.
The training agent must not receive blind contents, labels or row-level errors.

- Freeze, encrypt and sign the blind corpus manifest under human custody without
  giving the training lane record-level access.
- Produce a signed train/development/blind canonical group-tuple overlap audit.
  Merely showing unequal fingerprints is not proof of disjointness.
- Bind the sealed blind manifest to the exact corpus/split manifests and
  evaluator version. Do not evaluate it until S02 has produced a frozen model,
  tokenizer, artifact digest and signed threshold identities.
- Any record-level blind leakage or training access invalidates the blind set.
  Rotate to a fresh blind set before continuing.

After S02 freezes the artifact, the custodian runs the actual blind evaluator as
part of S03 and returns only the allowlisted aggregate object and detached
signature envelope. Threshold retuning from blind results invalidates that run.
The aggregate must bind the development report, artifact and threshold
identities.

That later S03 aggregate must demonstrate, with the evaluator's confidence
bounds:

| Measure | Required result |
| --- | --- |
| Held-out benign | At least 100,000 records and false positives ≤0.1% |
| Known-family detection | At least 1,000 records and recall ≥80% |
| Unseen-source/evasion detection | At least 1,000 records and recall ≥65% |
| Hard-negative perturbation | At least 1,000 records and rejection increase ≤2 percentage points |
| Total blind size | At least 140,000 and within ±10,000 of the 150,000 target |
| Privacy | No real customer secrets, credentials or protected financial records |

### Stage 5 — S01 → S02 → S03 handoff boundary

PF-35-S02 owns production model training/export, artifact verification, signed
distribution and the runtime implementation. PF-35-S03 owns calibrated blind
qualification and deterministic ingress enforcement. PF-35-S01 must not
silently implement either successor.

The required sequence is:

1. after the authority-gate amendment, finish and archive PF-35-S01 with the
   licensed corpus campaign, frozen train/development splits, sealed blind
   manifest/custody and signed overlap-audit evidence;
2. allocate PF-35-S02 from the new `main` head in a fresh recorded worktree;
3. build/sign the deterministic production artifact and measure it on the N100;
4. require model ≤300 MiB, p95 ≤50 ms per complete 2,048-token input and peak
   RSS ≤512 MiB, including tokenization;
5. archive S02 only after its artifact, signature and resource evidence passes;
   and
6. keep PF-35-S03 blocked until S02 plus PF-34-S01, PF-30-S03 and PF-23-S01 are
   all completed and archived, then have the custodian blind-evaluate the frozen
   artifact before protected ingress is accepted.

If the primary detector misses a quality or resource gate, preserve the failed
evidence and request authorization before exercising the custom-classifier
fallback. Do not move the threshold, change the CPU floor or reduce the holdout
to manufacture a pass.

## Repository deliverables

Commit only secret-free, reviewable material within the newly allocated scope:

- versioned source and campaign ledgers with immutable hashes;
- revised preparation manifests that identify actual frozen inputs while
  retaining private-data exclusions;
- S01 aggregate development report, sealed blind-manifest identity and signed
  overlap-audit statement;
- later S02 artifact/N100 evidence and S03 aggregate blind report, each committed
  only under its separately allocated sprint scope;
- commands, tool versions, test counts and artifact digests under
  `qa/security-levels/sprints/PF-35-S01/`;
- an honest PF-35-S01 Done/Remaining ledger; and
- a handback listing the candidate commit, literal diff scope, external artifact
  locations, signatures/digests, known limitations and integration reruns.

Large corpora, blind records/labels, model weights and signing keys remain
outside Git. Evidence should use digests and safe aggregate facts rather than
paths or metadata that disclose protected identities.

## Verification and review

Run from CorbanuDrive-backed caches and output roots:

```console
python3 docs/plans/check.py
python3 docs/sprints/check.py
python3 -m unittest scripts.test_security_classifier_eval -v
python3 scripts/security_classifier_eval.py \
  --manifest qa/security-levels/classifier/corpus-manifest.json \
  --splits qa/security-levels/classifier/split-manifest.json \
  --output /Volumes/CorbanuDrive/Corbanu/.codex-work/pf35-qualification/preparation
cd codex-rs
just fix -p codex-content-security
just fmt
just test -p codex-content-security pf_35_s01
just test -p codex-content-security
```

Also run `git diff --check`. Exercise a supporting real Corbanu TUI smoke with
the Rust TMUX harness, explicit CorbanuDrive `log_dir`/temporary/cache paths and
literal keys sent separately. Use Corbanu Terminal in TMUX with visibly selected
Claude Opus 5 Max for a read-only autoreview. Preserve transcript and trace
outside Git and record SHA-256 digests. Verify each finding, fix only in scope,
rerun affected tests and repeat review until there are no actionable P0/P1/P2
findings.

PF-26 final-candidate adversarial, true-TUI, TensorCash, Isometric Game and human
acceptance remain later release gates; a supporting PF-35 smoke does not replace
them.

## Stop conditions

Stop and report rather than proceeding when:

- allocation, branch, worktree, base, owner or literal scope is absent/stale;
- the RTX, custodian, signing or N100 authority is not confirmed;
- a source lacks provable commercial-safe licensing or immutable revision/hash;
- blind data becomes visible to training, model-building or review agents;
- a split/group overlap, contradictory label or record-level blind leak occurs;
- a required statistical, privacy, model-size, latency or RSS gate fails;
- a proposed remedy changes a product target, trust boundary or supported CPU;
- credentials, signing keys, real secrets, protected financial data, blind rows
  or production weights appear in Git, chat, logs or reviewer context; or
- another active lane needs a shared file outside the allocated write scope.

No classifier result can grant authority. Even a qualified `allow` verdict
cannot erase provenance, untaint content, expose a secret or authorize a
financial action.
