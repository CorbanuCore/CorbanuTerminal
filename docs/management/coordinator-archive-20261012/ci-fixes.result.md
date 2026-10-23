None of the PRs are green yet, and they can't be until a maintainer acts. Five of them (#142–#145 and #133) never actually failed: their CI has never run. They come from the outside-contributor fork `citadelculture` (read-only access), and every workflow run is waiting for maintainer approval. Approving runs fork code that edits workflow files, so I left that decision to you. In-repo, I removed every failure #124, #126 and #138 caused themselves, and opened **#146** for main's breakage that no PR owned.

`main` has no branch protection and no rulesets, so no check is required. Every PR below is already mergeable with red checks; the red CI isn't what blocks merging.

## Per-PR status

| PR | Failing at start | Root cause | Pushed | Checks now | Still red, and why |
|---|---|---|---|---|---|
| #142 | none; only a skipped `cla` check | Fork runs need approval | — | Never run | Needs approval. It owns the Windows `contract` failure (`wrong_probe_identity`) seen on #138 and #146. Its test passes locally (8/8). |
| #143 | same | same | — | Never run | Needs approval. It owns #138's `real-tui` failure: the harness waits for `qa-model default` but the footer now says `qa-model via qa default`. Its test passes locally. |
| #144 | same | same | — | Never run | Needs approval. Test passes locally (3/3). It conflicts with #143 (`qa/orchestrate_tui_matrix.sh`) and #145 (the matrix workflow file), so whichever merges second needs a rebase. |
| #145 | same | same | — | Never run | Needs approval. Test passes locally (6/6). |
| #133 | same | same | — | Never run | Needs approval. It owns the rustls advisory failure in cargo-deny (RUSTSEC-2026-0285). I checked rustls 0.23.45's checksum against crates.io and it isn't yanked. |
| #124 | Bazel ×4, Codespell, cargo-deny, repo-checks, cargo shear, sdk, tmux smoke, arg-lint ×3 | Two unnamed `None` arguments it added; everything else is main's | `abcc02608b` (two comments; its 2 tests pass) | Same set, all main's | Main's failures only. Arg-lint is still running (about 2–3h); the PR's own two violations are fixed, and the job stays red on main's backlog. |
| #126 | same as #124 | Nine unnamed arguments it added (same as fork PR #130); rest is main's | `bf7e27e29b` (13 targeted tests pass) | Same set, all main's | Main's failures only. #130 is now redundant; I didn't close it. |
| #138 | Above, plus Format, Blob size, CodeQL, Windows `contract`, and later `real-tui` | 23 unformatted Rust files and 6 spelling findings from the branch; plus items needing owner decisions | `5ddbc622a6` (rustfmt), `4f6234a8f0` (spelling). Both fast-forward. | **Format now passes**; Codespell will pass once #146 merges | Owner decisions: **Blob size** (`tui/src/app/tests.rs` is 572 KB; it's already over the limit on main, and the branch adds two QA logs over 1 MB), **CodeQL** (1 high and 3 medium alerts in QA broker scripts; they look like false positives, but dismissing security alerts isn't my call), **repo-checks** (adds crate features to core, tui, protected-state and linux-pidfd-spawn). `contract` waits on #142, `real-tui` on #143. |
| **#146 (new)** | — | Main's failures that no PR owned | `19524bde43` codespell, `5a6fd43762` cargo shear, `aa374a2291` duplicate tui Bazel label, `4186349250` reqwest exceptions in cargo-deny | **Codespell and cargo shear pass**; cargo-deny bans pass | cargo-deny advisories wait on #133; Bazel on #131; tmux smoke on #132; `contract` on #142; repo-checks needs a decision; arg-lint is a known backlog. |

In #146, the reqwest change only adds two crates to the migration-debt list; a reviewer may prefer migrating them instead. Seven doc-only security placeholder modules that cargo-shear flags are kept and excluded, not deleted.

Someone else is actively pushing to #138 (IridiumMaster, about 14:37 today, from a separate worktree). My two commits went in cleanly as fast-forwards.

I didn't retry anything; none of the failures were flakes or runner problems.

## Failures already on `main`
`main` (`c9f358c`) has no CI on push, so all of these built up unnoticed. Owners:
- **Bazel zlib download 404** (Bazel test/clippy/release, `sdk`, and probably the 3-hour Linux arg-lint timeout): fork PR #131. I confirmed the new URL's sha256.
- **tmux smoke runs before the CLI is built**: fork PR #132.
- **rustls advisory**: #133. **Windows contract**: #142. **real-tui matrix**: #143.
- **Codespell, cargo shear, duplicate tui Bazel label, reqwest ban**: #146.
- **repo-checks**: `secret-broker-service` has a `synthetic-fixture` feature that the manifest policy forbids. Since it's the secret broker, the owner needs to decide between an exception and restructuring.
- **Argument-comment lint**: about 1,100 existing unnamed arguments, mostly in security-audit, core/security, vault, provider-auth and secret-broker tests. It needs its own large mechanical PR, which I didn't start.

## What would turn things green
1. Approve workflow runs for #131, #132, #133 and #142–#145. I reviewed their diffs (small, only local operations) and ran their tests locally.
2. Merge #146 plus #131/#132/#133/#142/#143, then merge `main` into #124, #126 and #138.
3. Decide on the crate-features policy, #138's blob size and CodeQL alerts, and whether to do the arg-lint cleanup.

The worktrees `ci-124`, `ci-126`, `ci-138` (local branch `ci-138-fmt`) and `ci-main-baseline` are still in place. The cargo target dir is shared across worktrees, which gave me a stale #126 test run until I rebuilt it separately; other agents running tests there could hit the same thing.