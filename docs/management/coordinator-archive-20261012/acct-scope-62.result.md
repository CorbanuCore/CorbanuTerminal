All four points are handled and the branch is pushed. Points 1 and 2 still needed fixing; points 3 and 4 were partly fixed already, and I finished them. The tmux scenario passes against a candidate built from the final tree.

## Per point

**1. Unqualified zero for the day — newly fixed.**
- **Before:** an empty conversation showed "No recorded attempts in this day" and `Known subtotal exact USD: 0`. The state read skipped other conversations, so nothing said they existed.
- **Now:** the day read also gets the other conversations' attempts for that day. They are never added to this conversation's total, and they may use at most a quarter of the read's work budget. Conversations it can't read are counted as unread, never treated as zero.
- **What the page says:**
  - "This conversation on <day>: no recorded requests."
  - "Other conversations on this day, not included above: N requests in M conversations."
  - One line per provider and model, then the other conversations' own estimate, and "open that conversation with /resume and run /cost there".
- **Other wording:** the empty subtotal now reads "none — no recorded attempts in this conversation". Range views say other conversations aren't included.

**2. Mixed-provider run omits custom-provider usage — partly fixed already, finished here.**
- Collection of custom providers was already fixed (`4b31ff612e`).
- Custom-provider usage in other conversations was still dropped silently. The same new read now names those providers and models, with a cost or "no price available".

**3. Missing price shows a zero subtotal with no next step — zero already fixed, next step added.**
- The zero subtotal was already gone (`bc01aef837`).
- Every page that shows an unpriced request now says "Next step for requests with no price: check the bill from <providers>…". It names only providers whose recorded tokens no price covers.

**4. Reader audit's money checks unbound — QA script already fixed, Rust test added.**
- The QA Python audit was already bound in the earlier acct-scope-62/64 rounds.
- There was no equivalent Rust test. I added one that fixes the exact list of pages and the money lines each must carry. Removing any one of those lines, or dropping a page, fails it, even when the same figure still appears on another page.

## Files and tests
- **State:** new `state/src/runtime/accounting_scope.rs`; edits to `accounting_store.rs` and `accounting_lifecycle.rs`.
  - New tests in `accounting_store_tests.rs`: `accounting_inspect_other_conversations_are_read_beside_the_tree` and `accounting_inspect_range_days_do_not_read_other_conversations`.
  - Four existing tests now state the other-conversation result explicitly: the busy-host and range-budget tests in `state/tests/accounting_store.rs`, and the concurrent-writer test in `accounting_store_tests.rs`.
- **TUI:** new `tui/src/chatwidget/tokens/scope.rs`, with five regression tests in `scope_tests.rs` (one per point plus a "were other conversations read" test).
  - Four snapshots: wide empty conversation, narrow 40-column mixed-provider day, the same narrow screen scrolled down, and the unpriced request page.
  - `tokens.rs` refactor so the new module can reuse its helpers; it grew by about 45 lines to 2,132. The new behaviour lives in `scope.rs`.

## Commits (on `fix/acct-scope-62-20261002`, not pushed to the integration branch)
- `cf73b8c1f7`: state read path
- `4946e1e49d`: TUI inspector, tests, snapshots, tmux scenario
- `a370fc67cb`: sprint ledger update and the evidence record at `qa/.../acct-scope-62/scope-fix-20261002.md`

## Test results
- **`just test -p codex-state`:** 364/364 passed.
- **`just test -p codex-tui`:** 4,177/4,199 passed. The same 22 tests also fail on the base commit with my changes stashed (command-popup branding snapshots, wallet menu, IDE IPC sockets, wallet-daemon onboarding). None are in accounting.
- **With `--features developer-accounting`, focused on tokens/usage/cost:** 177/177 passed.
- **Formatting:** `just fmt` reformatted 82 unrelated files across the repo. I restored them and checked my files with `rustfmt --check`, which is clean.

## tmux scenario
- **Name:** `tmux_cost_states_scope_other_conversations_and_missing_price_step`
- **Result:** passed, 0 retries, about 3.5 s. It left no artifacts, sockets or processes behind.
- **Candidate:** `corbanu 0.1.48` built from `4946e1e49d`, SHA-256 `a780f367…`.
- **What it does:** seeds two other conversations (two priced OpenAI requests at $0.00071 each, one `local-mock` request with no price), runs one mocked turn, opens `/cost`, and checks the scope wording, "at least $0.001420", the `/resume` step and the missing-price step. It then presses Esc and exits with `/exit`.
- **Running it:** `-p codex-tui` can't enable `codex-core/developer-accounting`, because codex-tui doesn't depend on codex-core directly. So the scenario reads a full developer-accounting CLI from an environment variable, `CORBANU_ACCOUNTING_CANDIDATE`. Without it, the scenario falls back to the test-built `codex-tui`, which shows `/cost` but doesn't record the turn.
- **Two earlier failed attempts are kept in the record:** one had no candidate, and one picked up another worktree's `corbanu` from the shared `CARGO_TARGET_DIR`, which has no `/cost`.

## Left open
- A route line says "no price available" when the real cause is a missing token count rather than a missing price; the scenario's own mocked turn shows this. The next-step line correctly leaves that provider out. This wording predates my change.
- The inspector can't open another conversation directly; the page tells the user to use `/resume`.
- On a very busy store, the other conversations' figures can be partial. The page then states how many conversations it couldn't read.
- Still owed for the sprint: code-blind functional design and execution, independent evidence review, named-human acceptance, and live-provider and live-repository proof. The sprint's no-usage, stale-estimate and current-command tests also remain.