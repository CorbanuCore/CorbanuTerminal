# PF-60-S03: #289 residuals fix (PR #318), 2026-10-08

These are the fixes for the residuals that the second re-run found ([#315](../independent-acceptance-20261008/rerun2-20261008/README.md)).

| Item | Now | Capture |
| --- | --- | --- |
| R1, R1b, 13b: default build `/usage requests` (today, past day, future day, range) | One line: "Per-request cost history is not part of this build." No cost page and no next step. `/usage` (signed out) and its slash-popup description no longer mention it | `def-01`…`def-06` |
| R2: a week or month bucket reaching today | Titled "Cost so far — this conversation". Shows "In progress — totals so far, recorded through 2026-10-08T19:25:49.323Z", is counted in the range total, and has no next step. Weeks after today read "Not started yet" | `acct-02`…`acct-07` |
| R3: retention wording | The range header and the day pages both say "daily totals kept since 2025-10-09" | `acct-01` vs `acct-02`, `acct-04`, `acct-06` |
| Developer build, future day | "Run /cost for today." This opens a working page | `acct-08` |

## Method

- **Builds:** debug, from `248aaaa468`.
  - `corbanu-default` is the default features, sha256 `155003230a…9df062`.
  - `corbanu-acct` uses `--features codex-core/developer-accounting,codex-tui/developer-accounting`, sha256 `718074ba6a…55a0c6`.
- **Real turns:** one GLM 5.2 turn on `zai` per binary, each in its own disposable home, with `CORBANU_TEST_NO_NATIVE_KEYRING=1`. The prompt was "Read README.md and summarize it in one sentence."
  - The key was passed only as `ZAI_API_KEY="$(corbanu vault auth-helper provider/zai_api_key)"` on the `exec` command.
  - Recorded totals: 2 requests; 25,555 input, 12,672 cache read and 73 output tokens. At $1.40, $0.26 and $4.40 per 1M tokens, that is 0.02165212 USD. The product shows the same figure.
- **Screens:** view-only TUI sessions in tmux, resumed with a placeholder key, so no turns were sent from them.
  - Popup captures are the union of the lines seen while scrolling, so lines can be out of screen order.
  - Paths are redacted.
- **Key scan:** a `grep -rlF` for the real key found 0 matches in these captures, the logs and both homes.

## Review

Installed `corbanu exec`, `claude-opus-5-5-plan` on `claude-plan`, `high` effort, `-s read-only`.

- **Round 1** ([review-result.md](review-result.md)): REQUEST CHANGES, with 1 Major. Buckets that had not started still reached the dead end.
- **Round 2** ([review2-result.md](review2-result.md)): APPROVE WITH NITS. The nits were fixed in `248aaaa468`.
- **Recorded, not fixed:** a bucket that contains, or ends after, the last recorded request but is not in progress is still "Partial — excluded". Examples are yesterday, or the current hour, before any turn since. This is the review's Minor 2/5 and is tracked in the sprint file.
