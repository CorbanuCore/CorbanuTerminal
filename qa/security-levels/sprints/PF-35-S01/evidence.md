# PF-35-S01 evidence

Dates: 2026-08-30 through 2026-08-31. Sprint status: **in progress**. This
evidence proves the public deterministic contracts, evaluator implementation,
and the exact RTX generator qualification described below. It does not claim a
completed or adjudicated corpus, private blind access, blind qualification,
production detector weights, signing, or Intel N100 performance.

## Candidate and scope

- Recorded dispatch base: `9d08b15fa94676c1383ee1605b77e7cc7218dcc4`;
  allocation commit reviewed as the candidate baseline:
  `e0c23fe95165636d621dae8c16a5366c4f7250ac`.
- Branch/worktree: `feat/p0-security-classifier-corpus` at the worktree recorded
  in the sprint front matter.
- Current public corpus manifest SHA-256:
  `c0dd2e6c5028ddf9889af6d85bd60074dab94ca55b44f6694db4bcf932ed3384`.
- Current split contract SHA-256:
  `5bede5a495031dda8b523ef3ece01e6f7a69ba46550a998b1a3a8fb78e81f8c7`.
- The external continuation uses recorded base `2bcaf8d0b70f039f48165d0e4a4f291101574a41`,
  branch `feat/pf-35-s01-external-qualification-20260830`, and the distinct
  CorbanuDrive worktree in sprint front matter.
- No Cargo, Bazel, lock, workspace-registry or schema-registry edge changed.
- Integration registration completed: recurring CI now runs
  `python -m unittest scripts.test_security_classifier_eval` plus the shipped
  evaluator smoke across Linux, macOS and Windows. `.github/` was outside the
  implementation lane's literal write scope, so the integration owner made
  this final-tree change.
- Active-plan CLI handoff: update the future classifier release-gate row from
  `--manifest … --artifact … --output …` to the preparation/evaluation shape
  `--manifest … --splits … [--predictions … | --blind-aggregate …
  --development-report …] --output …`. Production `--artifact`
  hashing, signature verification and qualification belong to PF-35-S02, not
  this PF-35-S01 preparation tool.
- Plan-state handoff: the integration owner must reconcile the stale PF-35
  execution-map `draft` row and superseded readiness-blocked prose with this
  allocated `in_progress` sprint; the shared active plan is outside write scope.
- Sprint-process handoff: ratify the allocation-coordinate convention in the
  shared sprint policy. This record preserves the original dispatch
  `base_commit` and separately records the allocation commit from which this
  branch was forked; the shared policy is outside write scope.

## Verification

- `ruff format` and `ruff check`: clean.
- `python3 -m unittest discover -s scripts -p 'test_security_classifier_eval.py' -v`:
  21 passed, 0 failed.
- `cd codex-rs && just fix -p codex-content-security && just fmt`: clean.
- `cd codex-rs && just test -p codex-content-security pf_35_s01`: 1 passed,
  21 skipped; the named PF-35 test ran.
- `cd codex-rs && just test -p codex-content-security`: 22 passed, 0 skipped.
- Governance: active plan and sprint checks pass; final diff check is clean.

## RTX generator qualification

The owner-authorized Ubuntu host `rtx6000-blackwell-01` has an NVIDIA RTX PRO
6000 Blackwell Workstation Edition with 97,887 MiB VRAM, driver 595.84, CUDA
driver 13.2, an AMD Ryzen 9 9950X3D and 89 GiB RAM. ComfyUI was stopped before
model work and remained stopped. Model traffic binds only to
`127.0.0.1:8000`; no corpus request or response crosses the network API
boundary.

The selected generator is
`preetpatel/Qwen3.8-27B-Uncensored-NVFP4@37b5130a2d2a1f7d4456ab3f8d05d0b2a45ea350`.
The 18.36 GiB model, tokenizer and config SHA-256 identities, Apache-2.0 parent
chain, calibration-provenance limit, host facts, exact vLLM 0.27.1 environment,
launch configuration, benchmark method and safe result hashes are recorded in
[generator qualification](generator-qualification-2026-08-31.json). The direct
isolated venv is identified by the SHA-256 of its sorted package freeze; no
container was used or claimed.

The initial deliberately conservative eager/O0 server delivered 22.63 output
tokens/s at concurrency 1 and 87.72 at concurrency 4. Enabling vLLM O2,
FlashInfer autotuning and bounded CUDA graphs raised the same fixed 1,024-input,
512-output workload to 75.54 at concurrency 1, 280.33 at 4, 516.26 at 8,
904.96 at 16, 1,392.41 at 32 and 1,860.61 at 64; every measured request
succeeded. The single-stream result is consistent with the cited published
non-MTP comparison, while the larger concurrency points establish this host's
campaign capacity rather than claiming cross-host equivalence.

PF-35 requests use strict JSON-schema constrained decoding. The final 128-request
bakeoff at concurrency 64 produced 128/128 exact-format responses, zero
refusals, zero request errors and 1,290.724 completion tokens/s. A separate
1,600-candidate campaign smoke retained 1,587 provisional records, rejected
secret-like material and duplicates, populated distinct human and Opus queues,
and sustained 100% GPU utilization without swap, OOM, thermal failure or power
limiting. The earlier host reboots were isolated to unconstrained parallel
kernel compilation; serial compilation peaked with ample RAM and is now part of
the frozen launch configuration. No hardware repair or capacity upgrade is
indicated for this campaign.

## Pilot generation — review still required

The versioned `pilot-r1` round issued 3,000 requests at concurrency 64 and
requested 12,000 candidates. It completed in 1,000.1 seconds with 1,466,372
completion tokens and retained 11,767 provisional records (98.0583%). The
validator rejected 46 exact duplicates, 33 cross-group near duplicates, 18
secret-pattern matches and 34 request-level invalid/truncated JSON responses;
the 97 record-level and 34 four-record request-level rejections account exactly
for the 233-candidate shortfall.

The provisional set contains 7,667 `allow`, 578 `suspicious` and 3,522
`hostile` records. Every required coverage dimension has 1,160–1,195 records;
all configured positions and attack families are present. The deterministic
review selection contains 579 mandatory human reviews, 130 separate
high-confidence human audits and 130 disjoint Opus audits. The human queue has
709 records and the Opus queue has 130. Exact label/scope/family/coverage
counts, file sizes, output hashes and ledger identities are recorded in the
[pilot generation aggregate](pilot-generation-2026-08-31.json).

The external campaign ledger verifies to
`ae033f22c5f8b0d06d906fccb0aac7413287bd14b2a76d2ae711ba51175e49e8`.
Independent SHA-256 and line-count checks reproduced all five recorded output
identities. Raw responses, provisional records and review queues remain solely
under `/home/travis/pf35-qualification/campaign/pilot-v1/pilot-r1`; Git contains
only the safe aggregate. `final_accepted_records` remains zero until Travis and
the independent Opus reviewer return complete decisions and the adjudication
subcommand succeeds. Therefore this run clears generation capacity but does
not yet claim the brief's 10,000 accepted-record gate.

## Exact-candidate TMUX smoke

The runtime source was unchanged by the evaluator-only remediation and Rust
test replacement after this build, so this remains the exact product binary for
the candidate runtime tree.

- Binary SHA-256: `0beb2ca29e03109effb0b2c5f8afb2d5571c3ce229678a3bdfac151e4be10e16`.
- Private TMUX server, 160×48 PTY, `RUST_LOG=trace`, explicit CorbanuDrive
  `log_dir`, `TMPDIR` and cache paths.
- Sent `/status` as literal text and Enter separately; observed v0.1.35,
  `gpt-5.6-terra`, exact worktree directory and workspace permissions.
- Sent `/exit` as literal text and Enter separately; session exited cleanly.
- Transcript SHA-256: `c34a5942c8e8704d91fad27dfc0f8df01a5583a9eabe78d22ad41c183e9de9a8`.
- Trace SHA-256: `1f0c920f75859b38e1ac7b354c7a9dd97ee1b5ca91a4e30f3bd49041427fdd3b`.
- Raw transcript and trace remain outside Git under
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/classifier-corpus/tmux/`.

## Integration completion

Commit `9ac9fb682f0f1fee967036d4a4888c7c230e06c3` added the PF-35 source and
evidence paths plus recurring evaluator tests and a shipped-CLI smoke to
`.github/workflows/security-ingress-contract.yml`. Commit
`8dbbf6a16565f153cc7143d356d0087fc95444a7` made the smoke invocation explicitly
portable by calling `python scripts/security_classifier_eval.py` after
`actions/setup-python`, avoiding reliance on a Unix shebang or executable bit on
the Windows runner. The workflow matrix covers `ubuntu-24.04`, `macos-15` and
`windows-2022`. Integration reran all 21 evaluator unit tests, the manifest-only
smoke, YAML parsing and the plan/sprint governance checks successfully. This
closes the recurring-CI handoff only; the external qualification blockers below
remain open.

## Independent review

Claude Opus 5 Plan at Max reviewed the uncommitted candidate through the exact
Corbanu binary in a private TMUX PTY, read-only/never, with no delegation. Cycle
1 found 4 P1 and 8 P2 issues. Verified in-scope findings were remediated at their
causes: truth-based metrics, Wilson-bound gates and floors, permanently honest
external hardware/artifact gates, strict allowlisted report schemas,
manifest-bound development rows, aggregate-only blind input, gate tests, and a
meaningful Rust identity-mismatch regression. CI wiring was an integration-owner
handoff at that review point and is now completed as recorded above. The cycle-1
transcript SHA-256 is
`f85a654cf20f7a1bc6ef706b125967b09f0d4a18f65a1f9eb6a6b0ce0e156555`.

Cycle 2 reduced the result to 0 P0/P1 and 5 P2. Those in-scope findings were
also remediated: the dead same-split check was replaced by an explicit external
fingerprint-audit contract, measured failures now report `fail`, development
statistics are explicitly not blind qualification, whole manifests are scanned
for record material, and the aggregate schema now carries a strictly bounded
but explicitly unverified detached-signature envelope. New tests cover each
case. Cycle-2 transcript SHA-256:
`3595d1c3ea9d0868e288074f282be8423e1d5609ef045fc5c088e3aedccdde38`.

Cycle 3 reduced the result to 0 P0/P1 and 4 P2. The candidate now enforces exact
manifest structure allowlists with bounded strings and arrays; gates the
hard-negative perturbation by a conservative difference of Wilson bounds;
accepts only blind totals within a 140,000 floor and ±10,000 tolerance
around the approximately 150,000 target while reporting the observed count; and
pins, reports and tests a canonical development group-tuple fingerprint with
same-label sibling accounting and contradictory-label rejection. Cycle-3
transcript SHA-256:
`bffa5eeed1dc6b4d44afd6168d1e5e1226cdba6b45798e6fb253629e2289e32d`.
Cycle 4 found 0 P0/P1 and 2 P2: the report lacked an
exact evaluated-input digest and the active-plan release CLI divergence lacked
an explicit handoff. Both are now remediated; the report records evaluator ID,
report-schema version, input path/kind/size and SHA-256, and the
integration/PF-35-S02 handoff is explicit
above. Cycle-4 transcript SHA-256:
`503eff201c820457e4deb16b985b9cb99e741e83b7aca82edbcc6d0b5ccc1e6c`.

Cycle 5 found 0 P0/P1 and 2 P2: legitimate same-group siblings were rejected,
and evaluated inputs were hashed, sized and parsed through separate opens. The
candidate now collapses same-label siblings into the canonical group set while
reporting their count, and snapshots every regular-file input once so parsing,
size and digest use identical bytes. Feasible P3 observations were also closed:
strict blind schema typing, portable paths, prior-development-report binding,
precise identical-fingerprint wording, source hash/license-subject clarity,
the plan-state handoff above and atomic report writes. Cycle-5 transcript
SHA-256: `0cc04d9fdf3513a6251f859e9a504b7bab99f47797373ad0f99232974854c744`.

Cycle 6 returned `clean: 0 actionable P0/P1/P2 findings` and ten non-blocking
P3 observations. Every safe in-scope observation was also closed: terminal
control characters are escaped in path errors; duplicate JSON keys fail;
report paths use POSIX separators; prior development reports reject record
material and bind the exact model/threshold identity; fingerprints are labeled
as declarations rather than leakage proof; per-cohort deduplicated group counts
are reported; input bytes, JSONL line length and record count have explicit
bounds; the output must remain outside the repository and its parent directory
is synced after atomic replacement on supported hosts; and source hashes use
one opened regular-file snapshot. Recurring CI was still the integration handoff
at that review point and was completed during final-tree integration. Cycle-6
transcript SHA-256:
`074270f8b74174304d4e68de89d07a276f670ff4006b3a027de0a890dd6ae362`.
Cycle-6 trace SHA-256:
`1c8aa51cfcdfd72578ed8f65c9629b956290596b1fefa297de8f7b135e8aa781`.
Cycle 7 reviewed exact clean committed HEAD
`745df3594b34021947ffc27131c35828eaa4c524` and found 0 P0/P1, one P2 and seven
P3 observations. The P2 showed that escaped JSON keys could still place a raw
terminal control byte in a record-material error. The evaluator now terminal-
escapes every dynamic error fragment and final stderr message. It also closes
the feasible P3s by measuring the 64 KiB JSONL limit in bytes, splitting only
on the JSONL LF delimiter, normalizing oversized-integer `ValueError`s, forcing
LF report output, and reconciling prior-report group plus duplicate counts with
`record_count`. Test-only file helpers continue to delegate to the same
single-open regular-file snapshot path; recurring CI and sprint-coordinate
policy were shared-owner handoffs at that review point. Recurring CI is now
completed; the policy history remains recorded. Cycle-7 transcript SHA-256:
`5a308886aaa75d1e1666fc619646c602368cbe354bd2d392d5adbbafcfd9a7c2`.
Cycle-7 trace SHA-256:
`c80f91b2ca856a9cdc18f4440ca8195d0150fbdc6a7c4ff389c5e14233a19f7c`.
An exact committed-HEAD confirmation review is performed again before handback
and sealed below without changing implementation bytes.

Cycle 8 reviewed exact clean implementation HEAD
`7fa31d8a0043e88a9d725dff9b9f8bf7e85ebe06` against allocation baseline
`e0c23fe95165636d621dae8c16a5366c4f7250ac` and returned
`clean: 0 actionable P0/P1/P2 findings`. The reviewer independently reproduced
the review-7 remediations, statistical bounds, fail-closed qualification,
strict schema/provenance chain, explicit resource caps, literal write scope and
honest external-gate ledger. Eight P3 observations were explicitly
non-blocking/no-change-requested; the shared active-plan and recurring-CI items
were integration handoffs at that review point. Recurring CI is now completed as
recorded above, and the remaining notes do not weaken the preparation contract.
This evidence-only sealing edit does not change any
reviewed implementation byte. Cycle-8 transcript SHA-256:
`7f965811cb88273ca4a15236dc8bae45b543bc0c9a17b5a240014ac6d94866da`.
Cycle-8 trace SHA-256:
`1c081e658fc41667449f5a623b4f40658abc804a5dd3d3048e06ea31e2adc052`.

## External blockers kept open

- Human and Opus decisions for the generated pilot, successful hash-bound
  adjudication, and integration-owner acceptance before scaling beyond the
  pilot.
- Frozen approximately 250k training and 25k development corpora with grouped
  split and leakage-audit evidence.
- Human-custodied private blind corpus, aggregate report and signed group-tuple overlap audit; unequal split fingerprints alone do not prove disjointness.
- Production DeBERTa/ONNX artifact, weights, rotating release signature and offline-root authorization.
- Intel N100, 16 GiB, x86-64 Linux measurements at 2,048 tokens. The available
  Xeon Windows host and EPYC Linux VM are non-qualifying and were not used to
  claim the weakest-CPU target.
