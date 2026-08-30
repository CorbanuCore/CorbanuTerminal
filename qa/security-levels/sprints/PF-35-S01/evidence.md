# PF-35-S01 evidence

Date: 2026-08-30. Sprint status: **in progress**. This evidence proves only the
public deterministic contracts and evaluator implementation. It does not claim
an RTX campaign, private corpus access, blind qualification, production model
weights, signing, or Intel N100 performance.

## Candidate and scope

- Recorded dispatch base: `9d08b15fa94676c1383ee1605b77e7cc7218dcc4`;
  allocation commit reviewed as the candidate baseline:
  `e0c23fe95165636d621dae8c16a5366c4f7250ac`.
- Branch/worktree: `feat/p0-security-classifier-corpus` at the worktree recorded
  in the sprint front matter.
- Public corpus manifest SHA-256:
  `ea5c27983dd2ff4ffef18a7f423dea92439a71f7bdf021ba3fbcf97785dbc339`.
- Split contract SHA-256:
  `58e6eaf7a2add997d5194fb9a619c9612e8b9ac3580744dec1e2bc1a2f1a0dcf`.
- No Cargo, Bazel, lock, workspace-registry or schema-registry edge changed.
- Integration registration handoff: wire
  `python3 -m unittest discover -s scripts -p 'test_security_classifier_eval.py'`
  into recurring CI; `.github/` is outside this sprint's write scope.
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

## Independent review

Claude Opus 5 Plan at Max reviewed the uncommitted candidate through the exact
Corbanu binary in a private TMUX PTY, read-only/never, with no delegation. Cycle
1 found 4 P1 and 8 P2 issues. Verified in-scope findings were remediated at their
causes: truth-based metrics, Wilson-bound gates and floors, permanently honest
external hardware/artifact gates, strict allowlisted report schemas,
manifest-bound development rows, aggregate-only blind input, gate tests, and a
meaningful Rust identity-mismatch regression. CI wiring is recorded above as an
integration-owner handoff. The cycle-1 transcript SHA-256 is
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
one opened regular-file snapshot. Recurring CI remains the integration handoff
above. Cycle-6 transcript SHA-256:
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
policy remain shared-owner handoffs. Cycle-7 transcript SHA-256:
`5a308886aaa75d1e1666fc619646c602368cbe354bd2d392d5adbbafcfd9a7c2`.
Cycle-7 trace SHA-256:
`c80f91b2ca856a9cdc18f4440ca8195d0150fbdc6a7c4ff389c5e14233a19f7c`.
An exact committed-HEAD confirmation review is performed again before handback
and sealed below without changing implementation bytes.

## External blockers kept open

- Qwen3.5-27B/vLLM generation and corpus hashes on the owner-supplied RTX host.
- Human-custodied private blind corpus, aggregate report and signed group-tuple overlap audit; unequal split fingerprints alone do not prove disjointness.
- Production DeBERTa/ONNX artifact, weights, rotating release signature and offline-root authorization.
- Intel N100, 16 GiB, x86-64 Linux measurements at 2,048 tokens. The available
  Xeon Windows host and EPYC Linux VM are non-qualifying and were not used to
  claim the weakest-CPU target.
