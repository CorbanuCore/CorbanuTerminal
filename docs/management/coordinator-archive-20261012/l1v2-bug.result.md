I found and fixed a bug in the `account_tx` RPC, and the fix is up as draft PR #55. The regression test fails on unmodified `main` and passes with the fix. The full `postfiat-node` suite passes on Linux. I haven't touched Task Node.

**Bug:** `account_tx` returns `truncated: true` when the matching rows exactly fill the limit, even though nothing was left out. It happens on both read paths:
- **Index / disk-shard path:** `crates/node/src/block_finality.rs:658` (`account_tx_from_rows`) used `rows_for_account.len() >= scan_limit`.
- **Scan path (no index):** `crates/node/src/block_finality.rs:598-606` (`account_tx_scan`) set the flag as soon as the row count reached the limit.

The rest of the repo's paginated queries go through `truncate_reports` (`lifecycle_queries.rs:3565`), which flags truncation only when `len > limit`. The visible harm is in the Python client: `account_tx_history` treats any truncated window as incomplete. By default it raises `account_tx_history did not complete without truncation` when a window holds exactly `limit_per_window` rows. The CLI's `account_tx_not_truncated` check fails the same way.

**Reproduction (unmodified `main`, `postfiat-node` CLI):** I ran `init --validators 1`, submitted two `mempool-submit-transfer` calls to `pfrecipient…0001` and `…0002` with `--amount 10`, then `mempool-batch` and `apply-batch`. Then `account-tx --address pfrecipient…0002 --from-height 1 --to-height 1 --limit 1`, once before `account-tx-index-build` and once after.

| Path | Before | After |
|---|---|---|
| scan, limit 1 | row_count=1 truncated=True | row_count=1 truncated=False |
| index, limit 1 | row_count=1 truncated=True | row_count=1 truncated=False |

That recipient has exactly one row, so `truncated` should be false.

**Root cause:** Both paths treated "the result reached the limit" as "rows were dropped." The index path compared with `>=` where it needed `>`. The scan path stopped at the limit without checking whether another matching row existed. An existing assertion in `snapshot_deployment.rs` asserted `truncated` for an address with exactly one row at `limit: 1`, which locked in the wrong behavior.

**Fix (+8/−9 lines across three files, plus the new test file):**
- **Index path:** `>=` became `>`.
- **Scan path:** a row is added only while there is room. If another match arrives after the limit is reached, it sets `truncated` and stops. Block bounds and limits are unchanged.
- **`snapshot_deployment.rs`:** the one-row recipient case now asserts `!truncated`, with a comment explaining why.

This only changes a read-only report. Signed bytes, state, receipts and storage formats are untouched.

**Regression test:** `tests/account_tx_truncation.rs::account_tx_truncated_only_when_rows_are_omitted` covers both paths. An exact fit (1 row at limit 1, or 2 at limit 2) must not be truncated; 2 rows at limit 1 must be.
- **Before the fix:** `FAILED … panicked at account_tx_truncation.rs:18:5: exact-limit window reported truncated`.
- **With only the scan path fixed:** it still fails, now in the index phase, so the test catches each path separately.
- **After the full fix:** `ok`.

**Suite results:** I ran `cargo test -p postfiat-node --locked --no-fail-fast` on the RTX box (Linux, toolchain 1.95.0). Every target passed with 0 failures:

| Target | Passed | Ignored |
|---|---|---|
| lib | 397 | 4 |
| `main.rs` | 173 | 4 |
| `asset_orchard_local_service` | 32 | 0 |
| `fastswap_bootstrap_payload` | 3 | 0 |
| `fastswap_wallet_payload` | 1 | 0 |
| `pftl_swapd` | 10 | 0 |
| `postfiat_cobalt_shadow` | 2 | 0 |
| integration: `atomic_swap_local_six` | 1 | 8 |
| integration: `fastpay_default_rpc` | 1 | 0 |
| integration: `fastswap_local_six` | 0 | 2 |
| integration: `wallet_test_vector_redaction` | 1 | 0 |
| doc-tests | 0 | 0 |

`cargo clippy -p postfiat-node --all-targets -- -D warnings` and `cargo fmt --all -- --check` are both clean. I didn't run workspace-wide clippy or tests, since only `postfiat-node` changed. The temporary clone on the RTX box has been deleted.

**PR:** https://github.com/postfiatorg/postfiatl1v2/pull/55 is a draft against `main` from branch `fix/account-tx-exact-limit-truncation` on the `IridiumMaster` fork. The commit is `b7a54b95`, authored by IridiumMaster with an email that GitHub maps to that account. **Base SHA:** `ce25aaa86f55bf7932f9efddac1c0a1ab89bd7ee`.

**Risks and doubts:**
- **Changed existing assertion:** a reviewer could argue that flagging an exactly full result as truncated was a deliberate cautious choice. The repo's own `truncate_reports` convention and the Python client's behavior argue against that.
- **Extra scan work:** when the result fills the limit, the scan path now keeps reading the remaining blocks in its already-bounded window to look for one more match.
- **macOS-only failures:** 6 socket-timing tests in `main.rs` fail on macOS. They fail the same way on unmodified `ce25aaa8` and pass on Linux, which the PR notes.
- **Second issue, not fixed:** when no `from_height` is given and rows overflow the limit, the scan path keeps the earliest rows but the index path keeps the latest. The same query can return different rows depending on whether an index exists. It's out of scope here and would make a separate fix.
- **No issue or PR overlap:** none of the open issues or PRs (#26–29, #43–47, #52, #54) cover this.