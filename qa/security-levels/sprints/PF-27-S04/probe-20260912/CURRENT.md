# Probe checkpoint — construction stage accepted

September12: source `deb7ed06e`, exact local/RTX Rust tree
`0934e62ae687f680746476c9374d35092a68ab83`, unchanged after final tests/reviews.
See README for counts, artifacts and the retained initial fixture failure.

- Default3, fixture23, affected356 with2 existing subprocess-helper skips;
  scoped strict Clippy/parity/build/private TMUX pass on the recorded RTX host.
- Astra12 exit0, no findings. Fable13 exit1, patch correct, sole P3 fixture
  portability item: `/usr/bin/python3` may be absent on other Linux/Bazel hosts.
  Manager verified/accepted deferral; do not claim those hosts are qualified.
- Manager's +2 review extension was consumed, not a reset. Both processes
  finished; do not duplicate reviews or rerun unchanged-source tests.
- Manager confirmed a serialized main window from `1b7e9f3df`; root must
  fetch/recheck before landing, then report the final remote SHA and release it.
  Check current Git refs rather than assuming an old checkpoint is still pending.
- Full transitive telemetry Clippy debt remains owned by manager. Scoped
  `--no-deps` proof does not imply a full transitive lint pass.

RTX `/home/travis/worktrees/security-broker-probe-20260912` retains the same
staged source. Candidate and raw proof remain in
`/home/travis/security-round5/evidence/pf27-probe-20260912/verified-rerun`.
Private review scripts/results remain in local
`.codex-work/pf27-probe-review-20260912`; structured results are published here.

The next bounded step is allocated in [launcher-next](../launcher-next-20260912.md):
launch recipe and child identity preparation, not the full root supervisor.
Manager granted up to3 necessary new review passes; none dispatched for it yet.
Probe landed at `3e8bf6c95` and its main window was released. A later frozen
installation manifest still requires separate approval.
Do not infer permission for privileged setup, fixed listener, enrollment,
identity changes, real Vault access, native CAS, protected activation or an
installed-app update. Default service/native modes still exit78; inspection
always reports native eligibility false. PF27 remains in_progress.
