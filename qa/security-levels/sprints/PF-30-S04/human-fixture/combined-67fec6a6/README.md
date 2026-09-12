# Combined human-memory fixture qualification — passed

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

- [x] Python exact pinned completion regressions2.
- [x] Fixture plus TMUX support12, existing memory-policy/security/slash4.
- [x] Manual nextest profile parse; freshly compiled runner pinned under lock.
- [x] Strict ignored manual-entry startup and cancellation outside lock.
- [x] Owned home/socket/process cleanup; exact runner/source/hash recorded.
- [x] Authorized Astra review5 returned clean after the narrow correction.
- [ ] Named human acceptance21–23 (not performed by machine rehearsal).

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
startup/cancel passed outside lock, exact `Ok(Complete)` / `Ok(Cancelled)`
outcomes and owned home/socket removal. Evidence `remediated-2/pinned-entry.log`
and `remediated-2/pinned-entry/{startup,cancel}`. No operator session remains.

Review5 Astra High is now invoked on the narrow673c378b6-to-final lane delta,
using the structured helper and bundled CLI, with review-5-scope.md. Five of
five original memory invocations consumed; no sixth authorized. Review5 returned
helper exit0, no findings, `patch is correct` with0.91 confidence. Original
structured/text outputs are preserved here in `review-5-astra.*`. The review
was read-only, did not run tests, and covered the narrow673c378b6..bc0b9c553
delta. No additional review is needed. Review4's original finding remains
preserved; the earlier deferral is superseded by this tested correction.

The remote mirror is now clean at
`c137c953d9ebe2223ab6b2ff0453c1cc8bca6aca`, exactly67fec6a6 plus cherry-picks of
d03a4ac1c, f0cc34fd2 and ea158a210. This records the already-tested formatter
delta without changing its bytes. Per-case final status copies are versioned
alongside this report. Full actual keys/viewports remain under remote evidence:
`rehearsal/<Case>/input-events.txt`, `worker.txt`, `restart.txt` where applicable.

## Reproduction and operator handoff

Automated command after scoped fix/fullfmt, on RTX with immutable candidate
and synthetic-only environment described above:

```sh
python3 -B qa/security-levels/sprints/PF-30-S04/human-fixture/test_pinned_rehearsal.py
cd codex-rs
just test -p codex-tui --test all -E 'test(suite::memory_human_fixture) | test(support::tmux::tests) | test(suite::memory_stage_one_policy) | test(suite::security_profiles) | test(suite::slash_dispatch)' --retries 0 --test-threads 1
```

Actual keys: startup synthetic foreground prompt then separate Enter and
`/exit`; provider case `/providers` → deactivate A → replace with B → Esc →
`/model` → distinct Memory Fixture b model → effort → synthetic foreground
prompt → `/exit`; pending case prompt → pending canary → `/exit` → explicit
restart → `/status` → `/exit`. Cancel/timeout use only owned fixture signals.
Existing policy test covers Permissive/protected sources and restarts; profile
tests cover normal/narrow/inert Enter/cancel and invalid config; slash test
covers single-Enter status dispatch and clean exit.

When the coordinator arranges human testing (none left running now), use the
**new combined runner and product**, superseding earlier standalone paths:

```sh
cd /home/travis/worktrees/security-human-combined-67fec6a6
export TMPDIR=$(mktemp -d /home/travis/security-round5/memory-tmp/human.XXXXXX)
export CARGO_BIN_EXE_codex=/home/travis/security-round5/evidence/anchor/combined-b12e32d/candidate/codex
export CORBANU_MEMORY_CANDIDATE_SHA256=c567826ff5f15fccd71f8294c93210a158217a2ba31224c55d7d78269b1d2bea
export CORBANU_MEMORY_HUMAN_OPT_IN=1 CORBANU_TMUX_REQUIRED=1 RUST_MIN_STACK=8388608
export CORBANU_MEMORY_HUMAN_CASE=startup
# New path, not an existing directory. Do not create it first.
export CORBANU_MEMORY_HUMAN_EVIDENCE=/home/travis/security-round5/evidence/human-memory/operator-combined-startup-1
/home/travis/security-round5/evidence/human-memory/combined-67fec6a6/remediated-2/candidate/all --ignored --exact suite::memory_human_fixture::human_memory_fixture --nocapture
```

No build lock or Cargo during human waiting. Use another SSH terminal for
`bash <evidence>/attach.sh`; change case to `provider-switch` or `pending-exit`
and choose a new evidence path for each subsequent case. The full operator
journey is in the parent fixture README; the combined binary paths here take
precedence. There is no named-human acceptance or Mac launcher update in this
qualification. Direct `/model` custom-provider UX findings remain unpatched.
