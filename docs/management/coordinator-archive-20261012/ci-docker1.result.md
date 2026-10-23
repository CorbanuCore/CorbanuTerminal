PR checks no longer pull anonymously from Docker Hub. **PR #374 is merged** (merge commit `1ada2133`), merged only after all 24 non-skipped checks passed, using `--match-head-commit 04f962db`.

**What changed:**
- **Codespell** (`codespell.yml`): the old action built its image from Docker Hub's `python:3.13-alpine`. The job now installs codespell 2.4.3 on the runner with pip, with a required hash, using the repo's existing `actions/setup-python` pin (v6.2.0) on Python 3.13. 2.4.3 is the version the action actually installed in the last green main run (the action itself only asks for `>=2.2.4`); the pin is in the new file `.github/codespell-requirements.txt`. The command, ignore file, skip and `.codespellrc` handling are the same as before. The problem matcher is the same (identical JSON) and is removed after the run, as the action did.
- **cargo-deny** (`cargo-deny.yml`): it did pull from Docker Hub (`rust:1.85.0-alpine3.20`). The job now downloads the same cargo-deny 0.19.0 release the action used and runs it with the action's default arguments on Rust 1.95.0. The download is now checked against a sha256 hash (the action never checked it); the hash matches both GitHub's published digest and the release's own `.sha256` file.
- **python-sdk** (`sdk.yml`): it still runs in the same glibc container, now pulled from Amazon's public mirror of Docker's official images (`public.ecr.aws/docker/library/python:3.12-slim`) and pinned by digest (`sha256:a6e34c59…`). That is the exact image Docker Hub served in the last green run.

**Proof:**
- On #374, all three replaced checks passed. The logs show codespell 2.4.3 with the same command, the cargo-deny download passing its hash check with "advisories ok, bans ok, licenses ok, sources ok", and the image pulled from the mirror with the matching digest. Run: https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/37993729377
  - Codespell: https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/37993729377/job/114034261997
  - cargo-deny: https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/37993729377/job/114034261741
  - python-sdk: https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/37993729377/job/114034261787
- **Seeded typo:** I put the typo on a separate scratch branch with a draft PR (#375), which I closed and deleted afterwards; nothing was force-pushed. The new step failed with exit code 65 and added the annotation `codespell-seed.md:1 teh ==> the`. Running the old action's own image locally on the same seed gave the same output and exit code. Run: https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/37993763614/job/114034380068

**#369 and #373:** I didn't rerun anything. By the time #374 merged, #373 had been merged, and #369's failed jobs had been rerun and were green (Codespell, cargo-deny, python-sdk, CI required): https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/37991001508. Any other PR branch made before the fix still has the old Docker steps, even on rerun, until it's updated from main.

**Still pulling from Docker Hub (tag-triggered release jobs or disabled paths, not changed):**
- `python-sdk-release.yml`, which runs `python:3.12-slim`.
- `rust-release-zsh.yml`, which uses `ubuntu:24.04` and `arm64v8/ubuntu:24.04` containers.
- The `facebook/dotslash-publish-release` action in the rust-release workflows, which is Docker-based.
- `scripts/test-remote-env.sh`, which only runs with `remote_env: true` and nothing currently sets that.

My first `gh pr create` command mangled the PR body and ran `scripts/test-remote-env.sh`, which only printed "source this script instead". I rewrote the body from a file. The worktree is removed; the local branch is still there because `git branch -d` refused it as unmerged, and the remote branch wasn't deleted.