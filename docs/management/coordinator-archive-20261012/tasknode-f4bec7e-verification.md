(1) Confirmed: IridiumMaster is the GitHub account linked to this Task Node profile (Task Node status reports github.linked=true, username IridiumMaster). PR #55 and commit b7a54b95 are authored by IridiumMaster.

(2) Exact reproduction, run on unmodified main (ce25aaa8) with the postfiat-node CLI built from that tree (`cargo build -p postfiat-node --locked`):

    BIN=target/debug/postfiat-node; D=$(mktemp -d)/node
    $BIN init --data-dir $D --validators 1
    $BIN mempool-submit-transfer --data-dir $D --to pfrecipient000000000000000000000000000001 --amount 10
    $BIN mempool-submit-transfer --data-dir $D --to pfrecipient000000000000000000000000000002 --amount 10
    $BIN mempool-batch --data-dir $D --batch-file $D/batch.json --max-transactions 10
    $BIN apply-batch --data-dir $D --batch-file $D/batch.json
    $BIN account-tx --data-dir $D --address pfrecipient000000000000000000000000000002 --from-height 1 --to-height 1 --limit 1   # scan path
    $BIN account-tx-index-build --data-dir $D
    $BIN account-tx --data-dir $D --address pfrecipient000000000000000000000000000002 --from-height 1 --to-height 1 --limit 1   # index path

Observed before the fix (the recipient has exactly one row at height 1):

    scan path:  index_used=False scan_limit=1 row_count=1 truncated=True
    index path: index_used=True  scan_limit=1 row_count=1 truncated=True
    (with --limit 2 both paths report truncated=False)

Expected/after the fix: row_count=1 truncated=False on both paths.

Regression test command (fails on ce25aaa8 with "exact-limit window reported truncated", passes with the fix):

    cargo test -p postfiat-node --lib --locked account_tx_truncation

Full suite with the fix: `cargo test -p postfiat-node --locked --no-fail-fast` on Linux, 0 failures.
