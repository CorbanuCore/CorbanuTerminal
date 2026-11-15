# Hidden check — task B4 (#2018, fresh issue, no reference fix)

There is no merged reference. Objective checks: the whole codex-core security:: lib test suite (the classifier's never-quieter and corpus tests) must still pass; each solution's own regression tests at its final commit (expected pass) and at its test-only commit (expected fail); clippy -D warnings with the pinned toolchain 1.99.1; the repository's release-note checker (scripts/release/release_notes.py check).

## [opus]
```
## [opus] HEAD 3c7187492c
codex-core lib security:: test suite (classifier never-quieter guard; all must pass):
test result: ok. 905 passed; 0 failed; 7 ignored; 0 measured; 2980 filtered out; finished in 62.36s
cargo clippy -p codex-core --all-targets -- -D warnings (1.99.1): exit=0
## own tests (brace) at HEAD 3c7187492c [opus]
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 3875 filtered out; finished in 9.17s
## own tests at test-only commit e27f5d407b (expect FAIL)
test security::tainted_action::shell::brace_group_tests::issue_2018_braces_after_reserved_words_delimit_groups ... FAILED
test security::tainted_action::tainted_action_brace_reading_tests::issue_2018_group_braces_need_no_second_judgment ... FAILED
test security::tainted_action::tainted_action_brace_reading_tests::issue_2018_group_verdicts_never_quieter_than_main ... FAILED
test result: FAILED. 14 passed; 3 failed; 0 ignored; 0 measured; 3875 filtered out; finished in 12.73s
release_notes.py check: 76 release-note fragment(s) OK 
```

## [astra]
```
## [astra] HEAD 9492f05d78
codex-core lib security:: test suite (classifier never-quieter guard; all must pass):
test result: ok. 907 passed; 0 failed; 7 ignored; 0 measured; 2980 filtered out; finished in 67.93s
cargo clippy -p codex-core --all-targets -- -D warnings (1.99.1): exit=0
## own tests (brace) at HEAD 9492f05d78 [astra]
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 3875 filtered out; finished in 11.66s
## own tests at test-only commit f8441066c1 (expect FAIL)
test security::tainted_action::tainted_action_brace_reading_tests::issue_2018_bash_reserved_group_boundaries ... FAILED
test security::tainted_action::tainted_action_brace_reading_tests::issue_2018_zsh_reserved_group_boundaries ... FAILED
test result: FAILED. 13 passed; 2 failed; 0 ignored; 0 measured; 3875 filtered out; finished in 11.52s
release_notes.py check: 77 release-note fragment(s) OK 
```
