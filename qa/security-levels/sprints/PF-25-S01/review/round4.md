**Verdict: APPROVE**

I only read the code; I did not build it or run any tests. All six round-3 findings are fixed or properly documented. The new changes introduce no regressions. What is left is low-severity and does not block handoff.

**Round-3 findings:**
1. **Folder shown unescaped: fixed.**
   - Every field in the review now goes through `escape_controls`: Agent, Action, Resource, Command, Folder and Digest (`tui/src/security/grant_view.rs:135-150`).
   - The history line and `/security` were already escaped, and the `/security` label is escaped in Core by `display_command`.
   - The new test checks that a `\n` and U+202E in the folder are escaped and that no fake Folder row appears.
   - Paths that aren't valid UTF-8 still display lossily (`orchestrator.rs:843`). That's acceptable because the digest is built from the real folder (`:846`), so the display can't widen the grant.
2. **Scan test: fixed.** It now checks that it still finds the call in `grant_view.rs` before filtering, and the `confirm as …` gap is closed.
3. **`open_test_offer` race: fixed.** The vacancy check now runs inside `register` under the same lock (`grant_offer.rs:264`).
4. **Actor-chain label collisions: fixed.**
   - `Pending` keeps the structured `ActorChain`, and `confirm` compares it with `!=` (`ActorChain` implements `PartialEq`).
   - The displayed labels are still bound through the `pending.offer != *shown` check.
5. **More invisible characters: fixed.** The added ranges match the round-3 list.
6. **Comment: fixed.** `orchestrator.rs:172-174` now matches what the code does.

**Findings:**
1. **Low: a test offer can still block a real offer.** `core/src/security/grant_offer.rs:509-540`, `:268`
   - **Problem:** this is the second half of round-3 finding 3. If a test offer is registered first, the real registration treats the id as shared, removes the test offer and returns `None`. The real approval then has no grant option.
   - **Impact:** reachable only from in-process code, and it can only deny service.
   - **Fix:** put `open_test_offer` behind `cfg(any(test, feature = "test-support"))`, or record this as a known limit in `disposition.md`. Right now the disposition says only "race fixed".
2. **Info: the escaping is still a list of specific ranges.** `core/src/security/grant_offer.rs:451-470`
   - **Missing:** some format characters, such as U+0600–0605, U+06DD, U+070F, U+110BD, U+13430–1343F, U+1BCA0–1BCA3 and U+1D173–1D17A.
   - **Ambiguity:** a literal backslash is not escaped, so a folder whose name contains the text `\n` looks the same as one with a real newline.
   - **Impact:** neither can add rows or fake shell syntax.
   - **Fix (optional):** escape `\` as well, and escape characters by general category (Cf, Zl and Zp).
3. **Info: the scan test has small gaps.** `tui/src/security/grant_view_tests.rs:80-116`
   - **Alias gap:** `security_grant::{self as x}` followed by `x::confirm(...)` is not caught.
   - **Windows:** with `cargo test` on Windows the paths contain `\`, so the new assertion that the review's own call is found would fail. It doesn't affect CI: the Windows Bazel lane skips the test because the sources aren't present there.
   - **Fix (optional):** normalize the path separators, and treat `self as` the same way as the module-alias case.