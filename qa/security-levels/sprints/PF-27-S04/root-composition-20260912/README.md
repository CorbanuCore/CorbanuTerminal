# PF27 existing-root composition — construction evidence

Product initiative PF-27-S04, `/root`, existing allocated branch and worktree.
Product heading **Non-negotiable controls**: “Permit agents to reference
credentials only by label; resolve them solely inside the trusted execution boundary.”
Baseline `34bba8814` already contains the approved scope/test contract in
[the next-stage record](../root-composition-next-20260912.md).

## Review scope

Review only the new service root adapter, separate synthetic tests, library
export and Linux dependency/lock edge. Existing `TrustedChildRun` and PF20 code
are dependencies to inspect, not new implementation to re-review broadly.
The adapter opens real fixed-path existing roots, routes only admitted child
roles and retains typed terminal PF20 errors. No public fake root constructor,
worker-selected namespace, socket listener, executable launcher, enrollment,
Core/Vault/provider behavior change or privileged action. Normal binary exit78
is unchanged. Private fake handlers test composition; they do not prove native CAS.

Check ownership on partial construction, role selection, handler/root lifetime,
error retention (especially ambiguous publication), terminal fence/shutdown,
non-root denial and the adequacy/truthfulness of these construction tests.
No nested reviewers or new design panel. Do not expand into the later launcher,
installed containment or full broker data plane. No human-ready package claimed.

## Execution and known limitation

Fresh RTX worktree `/home/travis/worktrees/security-broker-root-composition-20260912`
at baseline `34bba8814`; earlier staged worktrees are untouched. Build lock,
eight-job limit, shared target and short on-disk TMPDIR are in `qualify-rtx.sh`.
Runtime output: `/home/travis/security-round5/evidence/pf27-root-composition-20260912`.

Initial `just fix -p codex-secret-broker-service --features synthetic-fixture`
and `just fmt` passed. Full transitive `just clippy -p codex-secret-broker-service`
failed on unchanged `codex-api/src/telemetry.rs`: redundant method closures at
94 and 127, and `expect_used` at 128–129. The new protected-state dependency
traverses config and exposes this pre-existing dependency lint debt. Original
`clippy-default.log` is retained. No dependency source was changed or lint rule
weakened. Manager acknowledged ownership of shared-code triage; do not silently
claim transitive strict Clippy passes. Changed-crate `--no-deps` checks are
separate evidence, with their scope in the log filenames.

New test fixture uses immediate assertions for setup/IPC failure, matching
existing service fixtures; its local unwrap allowance is test-only. Production
retains `forbid(unsafe_code)`. Final counts below are from the new stage,
not inferred from compilation or old proof.

## Final-tree construction proof

Local and RTX Rust tree: **`5775cd2053ca9eac47c04560ed503e0ae6eaa0ea`**.
New runtime module 131 lines, separate tests 258 lines; five Rust/build files,
396 changed lines total. No source changed after formatting or these runs.

- Default service: **3/3**; synthetic fixture: **17/17**, including four new cases.
- PF20/broker/Vault/network-proxy affected suite: **356 passed, 2 skipped**.
  Skips are PF20 subprocess entrypoints `native_child` and `lock_child`, explicitly
  marked ignored for invocation by their owning subprocess tests, not native
  qualification cases newly disabled by this change.
- Changed-crate strict Clippy default and fixture **pass** with `--no-deps`.
  Full transitive strict Clippy **fails** as recorded above; original failure is
  [preserved](initial/clippy-default.log), not represented as a pass.
- Cargo/Bazel parity **pass**, `MODULE.bazel.lock` unchanged after regeneration.
- Build **pass**; private actual-key TMUX **17/17 plus normal exit78**.
  Commands/captures/result are in `rtx/`. This is supporting construction proof,
  not a user-facing TUI or native root-controller deployment.

Candidate directory on RTX is the runtime evidence path above plus
`/verified/candidate`. SHA256: service
`800b1cde6ff82473dc9478b70cf9ed0d08a8fa7e873ea489a0f3f0b66f11e810`;
fixture `70c80699c3e64d4772ced2f6e53b2bfad920a8e71bc388635a3522dd7c1815db`.
Neither executable invokes the new adapter in its normal path; test binaries
exercise the new library. A changed digest is not proof of a native launcher.

Astra review 10: global autoreview helper, engine codex, installed binary
`/opt/homebrew/bin/codex`, model `gpt-6-astra`, thinking `high`, mode `local`,
this README scope packet. **Exit0, findings[]**, no repairs requested. It reviewed
formatted code while tests completed separately; no tests were run by the reviewer.
Original text/JSON retained here.

## External review and publication disposition

Fable review 11 ran through the same helper with engine codex, the existing
Corbanu Fable-plan wrapper, model `claude-fable-5-1-plan`, thinking `high`, mode
`local`, in private TMUX `fable-eleven`. Thread
`01a0945e-6a73-70b2-a4f8-bb4240eb009c`; helper **exit1**, overall patch correct,
no runtime finding. [Original result](fable-eleven.json) is retained unchanged.

Accepted P3: the initial review bundle omitted globally ignored `.log` evidence
and the later checkpoint file. Root had explicitly staged those during review;
the reviewer correctly identified their absence in the earlier frozen bundle.
Verified `git ls-files` contains `initial/clippy-default.log`, every `rtx/*.log`
and `CURRENT.md` before commit. Published TMUX/Bazel captures remove only trailing
blank rows for whitespace checking; original full output stays on RTX.
No runtime/test source changed, so no new test or review pass is required for
this publication correction. Do not relabel this nonzero helper result as clean.
Five slots used in this window, history retained in the budget ledger.

Source commit **`794080a4f`** has the exact tested/reviewed Rust tree above.
No privileged native action or installed-app update was performed.

## Remaining gates

- Stage code review is complete, and its publication finding is resolved;
  full transitive lint debt remains explicitly open under manager triage.
  Do not replay reviews 6–11 merely to produce an exit0 line.
- Native journal/policy positive CAS, installed launcher/UID/FD/ptrace isolation,
  enrollment, real Vault migration, all-platform and provider streaming remain
  unverified. Nothing in this stage activates protected mode.
- Product-TUI/blind UX and live TensorCash/Isometric evidence are N/A for this
  uncalled internal adapter, not waived for future interactive integration.
  Installed Mac package, human acceptance and benchmark records are unchanged.
