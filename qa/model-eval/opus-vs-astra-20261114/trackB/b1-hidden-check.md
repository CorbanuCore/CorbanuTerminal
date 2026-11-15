# Hidden check — task B1 (#2046)

Reference tests = the merged fix's test file, added unchanged as an extra test module (the engineers never saw it). Own tests = each solution's new regression test, run at its final commit (expected pass) and at its test-only commit (expected fail).

Clippy re-run by us: `cargo clippy -p codex-core --all-targets -- -D warnings` with the repository's pinned toolchain (codex-rs/rust-toolchain.toml = 1.99.1) PASSES (exit 0) on both solutions. With the machine's older default toolchain (1.95.0) both fail identically on 10 pre-existing nonminimal_bool errors in files neither solution touches.

## [opus]
```
## hidden: merged #2083 stale_poll_mail_tests.rs added unchanged as module stale_poll_mail_ref_tests, on [opus] HEAD a3d2532e77
test session::stale_poll_mail_ref_tests::guardian_stopped_turn_leaves_a_new_turns_input_and_mail_alone ... ok
test session::stale_poll_mail_ref_tests::stopped_turn_poll_leaves_mail_queued_on_every_stop_path ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 3850 filtered out; finished in 0.20s

## own tests (stale_poll) at HEAD a3d2532e77 [opus]
test session::stale_poll_mail_tests::guardian_stopping_turn_does_not_take_the_next_turns_input ... ok
test session::stale_poll_mail_tests::stopped_turn_poll_leaves_mail_queued_on_every_stop_path ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 3848 filtered out; finished in 0.35s
## own tests at test-only commit 4782ffd3f0 (expect FAIL)
test session::stale_poll_mail_tests::guardian_stopping_turn_does_not_take_the_next_turns_input ... FAILED
test session::stale_poll_mail_tests::stopped_turn_poll_leaves_mail_queued_on_every_stop_path ... ok
thread 'session::stale_poll_mail_tests::guardian_stopping_turn_does_not_take_the_next_turns_input' (65381958) panicked at core/src/session/stale_poll_mail_tests.rs:264:5:
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 3848 filtered out; finished in 0.21s
error: test failed, to rerun pass `-p codex-core --lib`
```

## [astra]
```
## hidden: merged #2083 stale_poll_mail_tests.rs added unchanged as module stale_poll_mail_ref_tests, on [astra] HEAD e073a3e107
test session::stale_poll_mail_ref_tests::guardian_stopped_turn_leaves_a_new_turns_input_and_mail_alone ... ok
test session::stale_poll_mail_ref_tests::stopped_turn_poll_leaves_mail_queued_on_every_stop_path ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 3850 filtered out; finished in 0.20s

## own tests (guardian_replacement_input) at HEAD e073a3e107 [astra]
test session::guardian_replacement_input_tests::guardian_stopping_turn_cannot_take_replacement_input ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3849 filtered out; finished in 0.10s
## own tests at test-only commit d503ffc7b7 (expect FAIL)
test session::guardian_replacement_input_tests::guardian_stopping_turn_cannot_take_replacement_input ... FAILED
thread 'session::guardian_replacement_input_tests::guardian_stopping_turn_cannot_take_replacement_input' (65415164) panicked at core/src/session/guardian_replacement_input_tests.rs:110:9:
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3849 filtered out; finished in 0.18s
error: test failed, to rerun pass `-p codex-core --lib`
```
