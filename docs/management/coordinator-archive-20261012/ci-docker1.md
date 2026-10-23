You are a small CI-reliability worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
(binding) and the repo skill /Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home/skills/pushing-ci-changes/SKILL.md.
Problem (2026-10-09 ~20:45–21:25Z, repeated on reruns): PR checks fail at random because GitHub runners hit Docker Hub's
anonymous pull rate limit ("toomanyrequests: You have reached your unauthenticated pull rate limit", and
"failed to resolve source metadata for docker.io/library/python:3.13-alpine"). Affected on PRs #369 and #373:
- `Codespell` (.github/workflows/codespell.yml uses the Docker-based codespell-project/actions-codespell);
- `sdk / python-sdk` (.github/workflows/sdk.yml runs `docker run ... python:3.12-slim`);
- `cargo-deny` (check which action/image it uses).
Make PR CI independent of anonymous Docker Hub pulls without weakening any check:
- Codespell: run the same codespell version the action pins, installed with pip (pinned version, ideally hash-pinned) via
  a pinned actions/setup-python, with the same ignore_words_file/skip/config and the problem matcher; prove it reports a
  seeded typo (in a scratch commit you then drop) the same way.
- python-sdk: keep the glibc-container intent but pull the image from a mirror without the anonymous limit (e.g.
  public.ecr.aws/docker/library/python:3.12-slim or ghcr), pinned by digest; or run on the runner's Python if that
  exercises the same manylinux wheel path — justify.
- cargo-deny: same treatment if it pulls from Docker Hub.
- Grep all workflows for other Docker Hub pulls on PR paths (release-only workflows can be listed, not necessarily changed).
Keep action pins as commit SHAs as the repo does. Show green runs on your PR for every touched check. PR to main; merge only
when ALL checks are green (`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f). Then rerun the
failed jobs on #369 and #373 if they are still red for this reason. Report: what changed and the run links.
