All three fixes are merged, and the code-blind re-run is merged as docs PR #311, next to #292's evidence. #286 and #287 now pass. #287 could only be confirmed on Linux. #288 still fails; that one belongs to acct-s03a and I didn't touch it.

**#287: clock behind the ledger blocks every model request.** PR #299, merged.
- **Root cause:** a clock running behind the ledger's last checkpoint made every ledger write fail. Accounting was built to stop the request on any failure, so every turn died, with an error that wrongly named Anthropic.
- **Clock fix:** writes now wait at the checkpoint time instead of failing. Each request still records the time the clock actually read.
- **Fail open:** any accounting failure now sends the request unrecorded, and the turn shows one warning that /cost won't include it. This covers ordinary turns and `/compact`.
- **Redirects:** a provider redirect gets the request resent once, unrecorded, on the normal client. Bad usage data no longer ends the response stream.
- **Only remaining stop:** a stage-one memory denial mid-stream. That's a privacy guard, not accounting, and it now has its own message.
- **Re-run:** on Linux, turns 58 s and 6 days behind complete and are recorded, exact to the micro-dollar. A turn 95 days behind completes and is reported as a gap in the turn output. No message names Anthropic. macOS couldn't be checked: libfaketime hangs any process whose clock is behind real time, with or without accounting.
- **Issue closed.**

**#286: deleting a session makes /cost claim "no other requests".** PR #303, merged.
- **What changed:** the day page now says "Deleted conversations or subagents sent N request attempts on this day. Their recorded cost was deleted with them…". Days older than 90 days, or days where the count can't be read, say why instead of claiming zero.
- **Re-run:** passes on macOS and Linux. The counts shown (7, 5, 6) match the deleted sessions' real request counts, subagents included.
- **Count, not amount:** the page shows how many requests were deleted, not their dollar amount. Keeping the amount needs a new ledger table, and existing ledgers have no way to upgrade. If you want it, that's a migration follow-up.
- **Issue closed.**

**#289: /cost wording.** PR #306, merged.
- **Fixed, confirmed by the re-run:**
  - a fresh home no longer says "Collection remains off"
  - the cost-detail lines show the token counts each cost was computed from
  - week and month buckets that reach today show the interval they cover
  - a future date gets a reason instead of the usage syntax
  - requests are numbered in the order they were sent
- **Still open, with the leftover wording points posted on it:**
  - In a default build, the "accounting ledger not installed" page tells you to send a turn, which can't help there.
  - A bucket reaching today is still titled "Bucket unavailable".
  - The range header disagrees with the day pages about how far back daily totals go.

**Gate, for each PR:**
- **Tests:** focused accounting tests pass, with and without the developer-accounting feature. The core test suite has 3 failures that also fail identically on main: `config_schema_matches_fixture`, `remote_compact_trim_estimate_uses_session_base_instructions` and `token_budget_auto_compact_fallback_uses_buffer_until_new_context`.
- **Linux clippy:** clean on the RTX box with `-D warnings`, with and without the feature, each in its own target dir.
- **Review:** Opus 5.5 High approved every PR after 2–3 rounds, and the findings were fixed.
- **CI:** green before each merge.

**New issues from the re-run:**
- #308: `corbanu delete` with the clock behind the checkpoint exits 1, but only after it has already removed the session. This should be fixed before wider developer activation.
- #310: an API key passed as `ZAI_API_KEY` is readable by the model's shell commands.
- #314 (I filed): when requests go unrecorded, nothing in /cost says so, and two log messages are misleading or contain test wording.

**Decisions for Travis:**
- Should a default build open the `/usage requests` page at all? This is the developer-only activation criterion.
- Is a count instead of a dollar amount acceptable for deleted spend?

My worktrees and target dirs are removed on both hosts.