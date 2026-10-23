# Worker brief: Task Node task_f4bec7ef5c4454dacbe0f7494ed35f16 — find and fix one reproducible bug in postfiatl1v2

You are an Opus 5.5 High worker. Travis (owner, GitHub `IridiumMaster`) accepted this 100 PFT network task. Work autonomously; do not ask questions.

## Task (from Task Node, verbatim requirements)
- Find one real, reproducible bug in the RPC layer, consensus, storage, or Orchard/Halo2 proof code of `postfiatorg/postfiatl1v2`, and fix it.
- Prove it first: exact input/command, wrong behavior observed, correct behavior expected. Not a local misconfiguration.
- Smallest correct fix + one new regression test that fails on old code and passes with the fix. Existing tests in the touched crates must still pass.
- PR against `main` with: reproduction steps, one-paragraph root cause, the fix, the regression test, a note that the test suite passes, and the base commit SHA.
- Do NOT redo the Orchard fix already done in `postfiatorg/postfiatd`; pick something new.
- Payment requires a PR authored by `IridiumMaster`, merged after the task was created, not used for any other task.

## Setup
- Clone: `/Volumes/CorbanuDrive/Corbanu/worktrees/postfiatl1v2-bug-20261003` (main at `ce25aaa8`; the task text mentions `b9e7fb42`, which is older — base on current main and state the SHA).
- Read the repo's `AGENTS.md`, `CONTRIBUTING.md`, `STATUS.md` and follow them. Use the `postfiat-l1-development` skill (`/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home/skills/.system/postfiat-l1-development/SKILL.md`); its hard invariants outrank everything.
- Mac has 24 cores. If a build is too heavy locally, the RTX box (`ssh ambient@100.99.88.49`, 32 cores, 239 GB free) may be used; clean up anything you put there.
- Prefer bugs that are clearly wrong by the repo's own docs/invariants and testable with a narrow unit/integration test (e.g. RPC input validation, storage encode/decode round trips, boundary/overflow handling). Avoid anything that changes signed bytes or consensus semantics unless clearly a bug per versioned architecture docs; if source and docs disagree, pick a different bug.
- Check open issues and open PRs in the upstream repo first (`gh issue list -R postfiatorg/postfiatl1v2`, `gh pr list -R postfiatorg/postfiatl1v2`) so you don't duplicate in-flight work.

## Required proof
1. Write the regression test, commit-less run it on unmodified code, capture the failing output.
2. Apply the fix, capture the passing output.
3. Run the touched crate(s)' full test suites (`cargo test -p <crate>`), plus clippy/fmt per CONTRIBUTING. Record verbatim counts.

## Delivery
- Fork with `gh repo fork postfiatorg/postfiatl1v2 --clone=false` (account IridiumMaster), push a branch `fix/<short-name>`, and open the PR **as a draft** against `postfiatorg/postfiatl1v2:main` with the required body sections. Commit author must be the IridiumMaster identity already configured for git/gh on this machine; check `git config user.email` matches a GitHub-verified email of IridiumMaster (`gh api user/emails` if permitted) before committing.
- Do not mark it ready, do not merge, do not touch Task Node (the coordinator submits evidence).
- Never print secrets. `rm -f` is blocked by policy; use other means.

## Final report (your last message)
Bug, where (file:line), reproduction, root cause paragraph, fix diff summary, test before/after output, suite counts, PR URL, base SHA, and any risks or doubts.
