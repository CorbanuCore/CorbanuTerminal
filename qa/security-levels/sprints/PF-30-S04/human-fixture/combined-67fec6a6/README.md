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
`qualification.log` is live. No result inferred before completion.
Review count remains4/5; coordinator accepted the documented nonblocking
false-negative deferral. No additional review or human session is authorized.
