# Hidden check — task B1, second independent run (variance check)

Same task and checks as B1: the merged fix's tests added unchanged as an extra module; each solution's own regression test at its final commit (expected pass) and its test-only commit (expected fail). Clippy was not re-run by us for this run; see result.md claims (the pinned toolchain is 1.99.1).

## [opus]
```
## hidden: merged #2083 stale_poll_mail_tests.rs added unchanged as module stale_poll_mail_ref_tests, on [opus] HEAD b838ef0a4c
test session::stale_poll_mail_ref_tests::guardian_stopped_turn_leaves_a_new_turns_input_and_mail_alone ... ok
test session::stale_poll_mail_ref_tests::stopped_turn_poll_leaves_mail_queued_on_every_stop_path ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 3850 filtered out; finished in 0.18s

## own tests (stale_poll) at HEAD b838ef0a4c [opus]
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 3848 filtered out; finished in 0.25s
## own tests at test-only commit ee542f47cf (expect FAIL)
test session::stale_poll_mail_tests::guardian_stopping_turn_does_not_take_the_new_turns_input ... FAILED
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 3848 filtered out; finished in 0.19s
```

## [astra]
```
## hidden: merged #2083 stale_poll_mail_tests.rs added unchanged as module stale_poll_mail_ref_tests, on [astra] HEAD fb81142de8
test session::stale_poll_mail_ref_tests::guardian_stopped_turn_leaves_a_new_turns_input_and_mail_alone ... ok
test session::stale_poll_mail_ref_tests::stopped_turn_poll_leaves_mail_queued_on_every_stop_path ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 3850 filtered out; finished in 0.20s

## own tests (stale_poll) at HEAD fb81142de8 [astra]
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 3848 filtered out; finished in 0.26s
## own tests at test-only commit 6366d8d52a (expect FAIL)
test session::stale_poll_mail_tests::guardian_stopped_turn_cannot_take_replacement_turn_input ... FAILED
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 3848 filtered out; finished in 0.18s
```
