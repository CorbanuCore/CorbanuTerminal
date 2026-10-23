**Verdict: APPROVE WITH NITS.** All eight round-1 responses check out in the code. The follow-up adds no blocking regressions or dead ends. I didn't build anything, run tests or run clippy, because the sandbox is read-only. Everything below comes from reading the code.

**Checking the round-1 responses**
- **1 (Major) is fixed.**
  - `progress()` returns `NotStarted` when `bucket.start_ms > read_at` (`tokens.rs:1808-1810`).
  - Days from buckets that haven't started are left out of the freshness and ancestry checks (`tokens.rs:1866-1882`). They count as complete for the range total (`tokens.rs:1939-1942`).
  - Their page is just a header plus `NOT_STARTED`, with no diagnostics (`tokens.rs:2021`, `2052-2059`).
  - A new test feeds real store output into the range view (`tokens_tests.rs:670-740`), and another covers the rendered result (`tokens_tests.rs:593-667`).
- **2 is recorded** as an open follow-up in the sprint file (sprint file line 76). See finding 5.
- **3 is fixed.**
  - Usage error: `slash_dispatch.rs:821-826`.
  - `/usage` description: `slash_command.rs:121-124`.
  - Signed-out hint: `slash_dispatch.rs:43-45`, with a default-build assertion at `tests/slash_commands.rs:1504-1509`.
  - No snapshot still contains the old description.
- **4 is fixed:** the comments at `tokens.rs:785-788`, `1619` and `slash_command.rs:313-318` are updated.
- **5 is fixed, except one path** (finding 1).
- **6 is reworded but still reads badly** (finding 2).
- **7 is fixed.**
  - `REPLAY_MS` is defined once, in `accounting.rs:211`. The lifecycle and scope modules alias it.
  - Both of those modules sit below `accounting` in the module tree, so the private constant is visible to them.
  - The retention, late-import and incremental modules pick it up through their single `use super::*`. No module imports two different `REPLAY_MS` names through globs, so there is no ambiguity.
- **8 is fixed:** the tests are at `tokens_tests.rs:503-560`, `562-590`, `593-667` and `670-740`. Their fixtures match what `progress()` checks.

**Clippy, by reading:** I expect `-D warnings` to be clean in both feature sets. This is unverified because nothing ran.
- The `match` inside `filter` has a non-literal arm, so `match_like_matches_macro` doesn't fire.
- `zip(&progress)` is fine because `progress` is reused afterwards.
- No new item is dead in the default build: `cfg!` keeps both branches compiled.
- `Count::try_from(i64)` is a real conversion, so `useless_conversion` doesn't apply.
- All test crates are already dependencies of the TUI crate.

**Findings**

1. **Minor: "Snapshot is not current" still appears while a bucket is in progress, including on the in-progress bucket's own page.** (`tokens.rs:1897-1902`, `2093` vs `2118`)
   - The `!running` check was only added to the `Ready` arm. The `DetailUnavailable` arm still fires, and while a bucket is running the read time is always later than the checkpoint.
   - So any range that also covers days with expired detail puts the line into `context`. Example: `/cost 2026-07-01 2026-10-08 week`, assuming those early-July days have stored detail that is now expired.
   - The line then reaches the in-progress page too. The `retain` at 2093 runs before the header is spliced in at 2118.
   - **Fix:** add `&& !running` to the `DetailUnavailable` arm, or remove `NOT_CURRENT` from `header` when the bucket is in progress. Add a test with one older bucket of `DetailUnavailable` days and one in-progress bucket.

2. **Nit: the Staging case now renders "daily totals kept not known yet".** (`tokens.rs:1855-1862`)
   - **Fix:** render "daily totals: retention not known yet" instead, and add a test for the `None` floor.

3. **Nit: buckets that haven't started still say "effective coverage …: unavailable".** This appears on the range page and in each bucket header (`tokens.rs:1983-1986`, `1992`, `2001`). For today's hours, that is up to about 23 "unavailable" lines next to a correct total.
   - **Fix:** show "not started" in place of "unavailable" when the bucket is `NotStarted`.

4. **Nit: "nothing is recorded for it" can be false when the system clock is behind the ledger.** (`tokens.rs:1832`)
   - In that case requests are recorded at the ledger's time (`accounting_store.rs:213-274`). A bucket starting after the read time could then already contain requests.
   - **Fix:** reword it to something like "Starts after this read — nothing to show yet."

5. **Minor (record): the deferred follow-up is described too narrowly.** (sprint file line 76)
   - The same rule also hits the *current* bucket when it started after the last recorded request.
   - Example: `/cost <today> <tomorrow> hour` (today's date and the next day) with no turn yet this hour. The current hour shows "Partial — excluded", with no next step and the range total withheld. The app test's comment at `app/tests.rs:160-162` already allows for this.
   - **Fix:** widen the follow-up to cover "a bucket containing or ending after the last recorded request, past or current".

6. **Nit: one app-test assertion can never fail.** (`app/tests.rs:168`)
   - `accounting_scroll` only renders page 0, and next steps only appear on bucket pages, so `!text.contains("Next step: send a turn")` always passes.
   - **Fix:** assert `text.contains("(not started)")` when the hours match, or remove the assertion.

7. **Nit: the developer-build usage error is no longer pinned.** The snapshot test strips `" or /usage requests [YYYY-MM-DD]"` before comparing (`tests/slash_commands.rs:1557-1560`), so a broken developer-build string would still pass.
   - **Fix:** when the feature is on, assert that the full developer string is present.