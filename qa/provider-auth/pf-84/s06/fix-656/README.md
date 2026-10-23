# #656: flag-off resume of a named-account conversation

tmux, disposable home, `CORBANU_TEST_NO_NATIVE_KEYRING=1`, local mock Responses server, fake API keys only.
A conversation is recorded with `corbanu exec --account work` (named OpenAI API-key account), then
`corbanu resume <id> --disable named_accounts` runs in a tmux pane (script: `t656.sh`, run with `QA_SCRATCH=<scratch dir> t656.sh <label> <corbanu binary>`).

## BEFORE: installed dev build from main
```
== before: $ corbanu resume <id> --disable named_accounts
Error:
   0: Failed to resume session from $S/homes/t656-before/sessions/2026/10/13/rollout-202
6-10-13T05-21-51-<id>.jsonl: thread/resume failed during TUI bootstrap: thread/resume failed: failed to load
 configuration: the thread's recorded account openai:work: named accounts need the `named_accounts` feature (`--enable named_accounts`); res
ume with `--account openai:default` to use the default credentials instead (code -32600)
Location:
   tui/src/app.rs:1042
Backtrace omitted. Run with RUST_BACKTRACE=1 environment variable to display it.
Run with RUST_BACKTRACE=full to include source snippets.
EXITED-1
```

## AFTER: this PR
```
== after: $ corbanu resume <id> --disable named_accounts
ERROR: Could not resume session <id>: failed to load configuration: the thread's recorded account openai:wor
k: named accounts need the `named_accounts` feature (`--enable named_accounts`); resume with `--account openai:default` to use the default c
redentials instead
EXITED-1
```
