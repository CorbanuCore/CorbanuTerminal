You are the CI-health worker. Read sec-common.md in /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/ (binding).
The credential boundary canary's `proxy-injection-boundary` probe fails intermittently on Linux and macOS
(`security-credential-canary: probe proxy-injection-boundary failed`), e.g. today on PR #297 (Linux, run 37779216421) and
PR #299 (macOS), unrelated to those PRs; see issue #222. Find the root cause from the workflow, the probe source
(scripts/ and codex-rs security canary code) and failed-job logs across recent runs (gh run list -w "credential boundary
canary"; compute the failure rate). Fix the flake properly (timing/port/readiness race, not by weakening the security
assertion or adding blind retries); if a retry is truly appropriate, make it a single bounded retry that still fails on a
real leak and logs both attempts. Add a regression test that reproduces the race if feasible. Opus 5.5 High review,
PR, merge when green (`gh pr merge N --merge --match-head-commit <sha>`), close #222 with the explanation. Also make the
probe print WHY it failed (which assertion, expected vs observed, redacted) so future failures are diagnosable. Report.
