# Combined human-memory fixture qualification — in progress

Exact merged source `67fec6a6a5d728f2fc9c17998301b41e9628ad92` in fresh RTX
mirror `/home/travis/worktrees/security-human-combined-67fec6a6`. Earlier
worktrees and evidence are preserved. This record is QA only, not a product
change or named-human acceptance.

Product source `b12e32db3`, immutable CLI
`/home/travis/security-round5/evidence/anchor/combined-b12e32d/candidate/codex`,
SHA256 `c567826ff5f15fccd71f8294c93210a158217a2ba31224c55d7d78269b1d2bea`.
No product rebuild requested: merged changes are tests/QA/registration only.

## Running gates

RTX shared build lock, fresh TMPDIR, pinned Rust tools, jobs8. Rust source
mtimes and cargo-bin repo marker refreshed to prevent cross-worktree artifacts.
Scoped `just fix -p codex-tui`, full `just fmt`, then `git diff --exit-code`
before tests. Any source delta stops qualification for coordinator inspection.

- [ ] Python exact pinned completion regressions2.
- [ ] Fixture plus TMUX support11, existing memory-policy/security/slash4.
- [ ] Manual nextest profile parse; freshly compiled runner pinned under lock.
- [ ] Strict ignored manual-entry startup and cancellation outside lock.
- [ ] Owned home/socket/process cleanup; exact runner/source/hash recorded.

Evidence root:
`/home/travis/security-round5/evidence/human-memory/combined-67fec6a6/`.
Initial run completed: Python2/2, nextest14/15, run
`644abb53-b14b-4bb3-a77c-d48f7bd17af0`,89.022seconds. No source delta.
Only five-case fixture failed at PendingExit: `exit missed pending window or
job already failed; not exercised`. Earlier Startup/ProviderSwitch completed;
later Cancel/Timeout in that loop were not reached. Existing memory-policy,
security2, slash1, support9 and pending-proof unit passed.

Saved status: one canary request, zero source output, provider_change_denied
false,29seconds remaining, actual `/exit` keys. This is consistent with the
deferred expected-owner-denial race, but the raw reason cannot be confirmed
retrospectively: prior helper retained viewport/status/keys, not its disposable
log. Root authorized exact failure-reason classification and sanitized diagnostic
retention, then new final proof and review5. Original source remains clean and
failure evidence is preserved. Pin/manual steps were skipped on test failure;
shared build lock released. This run is **not a qualification pass**.
The initial deferral became impractical once combined proof failed. Coordinator
authorized a narrow test-only correction and review5 after final tests; no
sixth review or human session is authorized.

## Corrected source proof

Patch checkpoints `d03a4ac1c`, `f0cc34fd2`, `ea158a210` preserve the exact
failure-reason distinction and formatter result. Remote source is67fec6a6 plus
those patches (remote cherry-pick HEAD3b69e9b10 plus formatter), equivalent to
the corresponding files at laneea158a210. SHA256 file checks matched both
locations; no product change. First remediation compile failure (unavailable
direct Core dependency) is preserved under `remediated/`; no manifests changed.

Final fix/fullfmt preceded Python2/2 and **Rust16/16**, run
`4456fa0a-28fc-43ef-b528-daeacf23ef4b`,91.980seconds. All five fixture cases
passed, including pending owner exit/restart at524ms/30second window,1request
and0outputs. Existing memory policy, normal/narrow security and invalid config,
slash/status-exit, and all nine TMUX support/cleanup regressions passed.
Exact/mixed owner failure regression and expiry/output rejection passed.

Evidence `remediated-2/qualification.log`, `rehearsal/`, `memory-policy/`,
`security-ui/`, `profile.json`. Newly compiled runner pinned under shared lock:
`remediated-2/candidate/all`, SHA256
`dea1c0286c4c1e3cf23f3954a28cb6c9274939dd7d5e4033ca296ceb5e748673`.
Product remains the immutableb12e32db3 candidate/hash above. Pinned ignored-entry
startup/cancel outside lock is running; its result is not yet claimed.
