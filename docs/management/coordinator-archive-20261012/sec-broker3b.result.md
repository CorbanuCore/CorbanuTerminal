PF-28-S01 (the central secret and protected-output gate) is merged to main in PR #208, behind the `secret_output_gate` flag, which is off by default. The same PR carries the PF-27-S02 close-out commit. All CI checks were green at merge (merge commit `24a57e38c4`).

**Where the previous worker stopped:** the gate commit existed, and three demos plus an attempt at a fourth were recorded but not published. Its review had died before writing a result, so I re-ran it.

**Gate evidence** (full write-up in `qa/security-levels/sprints/PF-28-S01/README.md`):
- **Focused tests:** all 38 `pf_28_s01` tests pass (secret-broker 20, core 16, vault 1, login 1).
- **Wider suites:** 798 of 798 pass in the affected crates.
  - Core subset: 1,439 of 1,442. The 3 failures (two skills developer-message tests and one remote-compact trim test) fail the same way on a clean `origin/main` worktree.
  - TUI subset: 104 of 105. A Claude-login tmux test timed out because the long worktree path cuts off the status line it waits for. That path runs with the gate off, but I didn't confirm the failure on main.
- **GLM 5.2 tmux runs:** four demo videos recorded and published at commit `9261bd416f`, listed in `qa/demos/index/PF-28-S01.md`:
  - **tool output:** the value was redacted in raw, base64, URL and JSON form, so the model never saw it;
  - **split output:** a value printed in two halves showed only one redaction marker;
  - **files on disk:** 0 files held the canary with the gate on;
  - **gate off (control):** 2 files held it.

  Two later commits landed after the videos. They change how values in JSON keys, numbers and very large documents are handled, which these runs don't exercise. A lint fix and merges followed.
- **Review:** six rounds of the independent Opus 5.5 High review. Rounds 1 to 5 asked for changes. The ones that mattered:
  - a secret split across output chunks leaked in review mode;
  - values in JSON keys reached clients unredacted;
  - large clean events (over 16 MiB) were wrongly withheld;
  - ordinary settings like git author names and `postgres:postgres` were being redacted.

  All of these are fixed or recorded as known limits. Round 6 approved with minor nits, which are also recorded. The reports are `review-opus-1.md` to `review-opus-6.md` in the evidence folder.
- **CI:** besides re-running flaky Windows/macOS canary and Windows contract jobs, I fixed a real Windows lint failure (literal arguments without parameter-name comments).

**Still open:**
- **Sprint status stays `draft`.** The sprint checker won't accept `in_progress` until PF-27-S02 is archived, and that waits on Travis's two open decisions (recommendations are in the PF-27-S02 sprint file). To pass the same checker, I also added one unchecked verification item to PF-27-S02 (Windows verification with PF-27-S06).
- **Known gaps left for later**, listed in the sprint's Remaining section and the README:
  - Encodings wrapped across lines, and secrets echoed back in responses, belong to PF-28-S02.
  - MCP OAuth tokens refreshed after startup aren't registered.
  - Seed phrases written comma-separated or numbered aren't caught.
  - Some tool-output fields can be dropped when an item is rebuilt.
  - Stream state uses one global lock, which may need performance work under load.
- **Unrelated test failures** are the three core tests and one TUI test above.

I left the probe build output at `.codex-work/targets/pf28-mainprobe-20261006`, since the rules forbid `rm -f`. A short result file is at `.codex-work/workers-20261002/sec-broker3.result.md`.