# #658: whose sign-in the backend receives during a TUI session on a named ChatGPT account

tmux, disposable homes, `CORBANU_TEST_NO_NATIVE_KEYRING=1`, the local mock issuer/backend from the s06b acceptance
(`mock.py`), fake tokens only (Travis's login and `auth.json` were never used). Each home has a default ChatGPT login
(mock user `tuidefault`, in the home's own `auth.json`) and, where noted, named accounts. The TUI runs one turn, `/status`,
`/plugins` and `/apps`, then exits (`QA_SCRATCH=<dir> t658.sh <label> <corbanu binary>`; with `RESUME=1` it first records
a conversation on `work` with `corbanu exec --account work`, then runs `corbanu resume <id>` without `--account`).
Counts are every request the mock received from the TUI run (resume runs: after the exec), by path and by whose bearer
token it carried. `auth.json` was byte-identical before and after in every run.

## BEFORE: installed dev build (main), `--account work` (named ChatGPT account, mock user `tuiwork`)
```
   2 post-unhandled /backend-api/codex/codex/analytics-events/events           tuidefault
   2 get            /backend-api/codex/connectors/directory/list               tuidefault
   1 get            /backend-api/codex/plugins/featured                        tuidefault
   7 post-unhandled /backend-api/codex/ps/mcp                                  none
  43 get            /backend-api/codex/ps/plugins/installed                    tuidefault
   6 get            /backend-api/codex/ps/plugins/list                         tuidefault
   6 get            /backend-api/codex/ps/plugins/suggested                    tuidefault
   2 get            /backend-api/codex/ps/plugins/suggested                    tuiwork
   2 get            /backend-api/codex/wham/rate-limit-reset-credits           tuidefault
   1 get            /backend-api/codex/wham/settings/user                      tuidefault
   2 get            /backend-api/codex/wham/usage                              tuidefault
   1 get            /v1/models                                                 tuiwork
   1 responses      /v1/responses                                              tuiwork
```

## AFTER: this PR, same run
```
   2 post-unhandled /backend-api/codex/codex/analytics-events/events           tuiwork
   2 get            /backend-api/codex/connectors/directory/list               tuiwork
   1 get            /backend-api/codex/plugins/featured                        tuiwork
   7 post-unhandled /backend-api/codex/ps/mcp                                  none
  43 get            /backend-api/codex/ps/plugins/installed                    tuiwork
   6 get            /backend-api/codex/ps/plugins/list                         tuiwork
   8 get            /backend-api/codex/ps/plugins/suggested                    tuiwork
   2 get            /backend-api/codex/wham/rate-limit-reset-credits           tuiwork
   1 get            /backend-api/codex/wham/settings/user                      tuiwork
   2 get            /backend-api/codex/wham/usage                              tuiwork
   1 get            /v1/models                                                 tuiwork
   1 responses      /v1/responses                                              tuiwork
```

## BEFORE: installed dev build, `resume` of a conversation recorded on `work`
```
   3 post-unhandled /backend-api/codex/codex/analytics-events/events           tuidefault
   2 get            /backend-api/codex/connectors/directory/list               tuidefault
   1 get            /backend-api/codex/plugins/featured                        tuidefault
   7 post-unhandled /backend-api/codex/ps/mcp                                  none
  43 get            /backend-api/codex/ps/plugins/installed                    tuidefault
   6 get            /backend-api/codex/ps/plugins/list                         tuidefault
   6 get            /backend-api/codex/ps/plugins/suggested                    tuidefault
   1 get            /backend-api/codex/ps/plugins/suggested                    tuiwork
   2 get            /backend-api/codex/wham/rate-limit-reset-credits           tuidefault
   1 get            /backend-api/codex/wham/settings/user                      tuidefault
   2 get            /backend-api/codex/wham/usage                              tuidefault
   1 get            /v1/models                                                 tuidefault
   1 responses      /v1/responses                                              tuiwork
```

## AFTER: this PR, `resume` of a conversation recorded on `work` (nothing is sent before the conversation resumes; the model list is served from its cache until then)
```
   3 post-unhandled /backend-api/codex/codex/analytics-events/events           tuiwork
   2 get            /backend-api/codex/connectors/directory/list               tuiwork
   1 get            /backend-api/codex/plugins/featured                        none
   7 post-unhandled /backend-api/codex/ps/mcp                                  none
  34 get            /backend-api/codex/ps/plugins/installed                    tuiwork
   5 get            /backend-api/codex/ps/plugins/list                         tuiwork
   5 get            /backend-api/codex/ps/plugins/suggested                    tuiwork
   2 get            /backend-api/codex/wham/rate-limit-reset-credits           tuiwork
   1 get            /backend-api/codex/wham/settings/user                      tuiwork
   2 get            /backend-api/codex/wham/usage                              tuiwork
   1 responses      /v1/responses                                              tuiwork
```

## AFTER, named account `key` holding an OpenAI API key (`--account key`; `UNKNOWN` is that fake key): nothing that needs a ChatGPT sign-in is sent and nothing goes out on the default login (the featured-plugins list is fetched without credentials, as for a signed-out user)
```
   1 get            /backend-api/codex/plugins/featured                        none
   1 responses      /v1/responses                                              UNKNOWN
```

## AFTER, flag on, a session on the default login (no `--account`): unchanged
```
   2 post-unhandled /backend-api/codex/codex/analytics-events/events           tuidefault
   2 get            /backend-api/codex/connectors/directory/list               tuidefault
   1 get            /backend-api/codex/plugins/featured                        tuidefault
   7 post-unhandled /backend-api/codex/ps/mcp                                  none
  43 get            /backend-api/codex/ps/plugins/installed                    tuidefault
   6 get            /backend-api/codex/ps/plugins/list                         tuidefault
   8 get            /backend-api/codex/ps/plugins/suggested                    tuidefault
   2 get            /backend-api/codex/wham/rate-limit-reset-credits           tuidefault
   1 get            /backend-api/codex/wham/settings/user                      tuidefault
   2 get            /backend-api/codex/wham/usage                              tuidefault
   1 get            /v1/models                                                 tuidefault
   1 responses      /v1/responses                                              tuidefault
```

## AFTER, flag off (`named_accounts = false`): unchanged, everything on the default login
```
   2 post-unhandled /backend-api/codex/codex/analytics-events/events           tuidefault
   2 get            /backend-api/codex/connectors/directory/list               tuidefault
   1 get            /backend-api/codex/plugins/featured                        tuidefault
   7 post-unhandled /backend-api/codex/ps/mcp                                  none
  50 get            /backend-api/codex/ps/plugins/installed                    tuidefault
   7 get            /backend-api/codex/ps/plugins/list                         tuidefault
   9 get            /backend-api/codex/ps/plugins/suggested                    tuidefault
   2 get            /backend-api/codex/wham/rate-limit-reset-credits           tuidefault
   1 get            /backend-api/codex/wham/settings/user                      tuidefault
   2 get            /backend-api/codex/wham/usage                              tuidefault
   1 get            /v1/models                                                 tuidefault
   1 responses      /v1/responses                                              tuidefault
```
