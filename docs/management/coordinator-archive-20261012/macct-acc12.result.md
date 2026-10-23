S01 passes and can be accepted. S02 has three failures and one criterion I couldn't verify, so I'd only accept it behind its off-by-default flag. The evidence is merged as PR **#420** (all 24 checks passed, 9 skipped; merge commit `bd3ab5a53d`).

I tested `origin/main` at `0fa45b54f0`, which was the latest when I started. A later merge (#410, sandbox log redaction) isn't PF-84 code and isn't included. For comparison I also built `44b527f11f`, from just before PF-84. Every run used throwaway homes with the no-keychain setting, and real keys only went through the vault helper. A key scan of the evidence folder is clean. Scratch homes, the build folder and the worktree are removed.

This was source-blind but not the full isolated "code-blind" check (no separate designer, no handoff checker). So the sprints' "code-blind handoff checker" box stays unticked unless Travis agrees to limited testing.

**S01: home override**
- **S01-1 PASS:** clashing home settings print one warning per run, naming the home used, paths only. Same path written differently (trailing slash, symlink) prints nothing.
- **S01-2 PASS:** which home wins is unchanged. All 9 cases match the pre-PF-84 build, which stayed silent.
- **S01-3 PASS:** the checked-in launcher keeps the caller's home after sourcing `activate.sh`; the repo's launcher test passes 5/5.
- **S01-4a PASS:** the installed `corbanu` wrapper never replaces a caller's home; the repo's installer tests pass 23/23.
- **S01-4b FAIL (low):** `corbanu-debug` silently ignores `CORBANU_HOME`, with no warning. Filed as #418.
- **S01-5 PASS:** a worker started with `CORBANU_HOME=homeB` got the 401 from homeB's fake OpenAI key. The coordinator's home, with the real key, answered `pong`.
- **S01-6 PASS:** in the TUI on GLM 5.3 Flash, an agent-started worker landed on homeB with one warning.
- **S01-7 PASS:** `docs/authentication.md` matches what I saw.

**S02: named accounts**
- **S02-1 PASS:** the account name rules hold and `default` is reserved.
- **S02-2 PASS:** labels have the expected `provider/<id>/accounts/<name>/<kind>` shape and the existing label is untouched. Whether this adds keychain items can't be checked under the no-keychain rule.
- **S02-3a PASS:** with a mock provider, each account's fake key reached the server unchanged and never crossed to another account. With real Z.AI, a named account ignored a valid `ZAI_API_KEY` in the environment, and a missing account failed closed.
- **S02-3b FAIL (medium):** Claude Plan gets its token by running whatever `corbanu` is first on PATH, and only passes the account in an environment variable. With an older `corbanu` on PATH, a session set to the fake account answered `pong` from the default account — a silent fallback. Filed as #414.
- **S02-4 PASS:** `auth.command` receives the account name, and an account that isn't added never runs the command.
- **S02-5 PASS:** `internal-claude-oauth-token --account` works for a token or a Claude Code directory, and an unknown account fails closed.
- **S02-6 PASS:** the flag is off by default, the config selector works, and `corbanu account` only takes secrets from piped stdin.
- **S02-7 PASS (agent commands only):** an agent's `corbanu account add` is refused in Aggressive. You running it yourself still works, which the `/security` screen says isn't covered yet.
- **S02-8 PASS:** OpenAI sign-in and AWS named accounts fail closed. The default OpenAI account (`gpt-5.4`) still works.
- **S02-9a PASS:** with the flag off, `exec` and the TUI behave as before, apart from a new "ignored" warning printed twice.
- **S02-9b FAIL (low):** with the flag off, `internal-claude-oauth-token` still hands out named-account tokens. Filed as #415.
- **S02-10 PASS:** a home created by the pre-PF-84 build is byte-identical until the first named account is added, and the old build can still read it afterwards.
- **S02-11 PASS:** a fake-key 401 makes exactly one request and never retries on another account.
- **S02-12 PASS:** none of the 4 real keys or the test values turned up in logs, sessions, captures or the TUI. The only hits were where keys are meant to be stored.
- **S02-13 PASS:** Z.AI in the TUI: `main` answers pong, `fake` gets a 401, an unknown account fails closed. Kimi, Claude Plan and OpenAI match from the command line.
- **S02-14 FAIL (UX):** the 401 hint tells you to fix the default key, not the failing named account. Filed as #416.
- **S02-15 NOT VERIFIABLE:** I couldn't tell from outside which model routes count as "siblings" for model correction.
- **S02-16 PASS:** `vault auth-helper` refuses named Claude tokens.

Other issues: #417 (the TUI demands a default key when you only have named accounts, probably S03/S04) and #419 (small fixes).

The Opus 5.5 High review said "accept with changes". I've made those changes, and its review plus my responses are in `REVIEW.md`.

**Recommendation for Travis:** Accept S01; #418 is a small follow-up. Accept S02 only behind its off-by-default flag. The API-key and `auth.command` paths are solid and existing homes carry over exactly. Track #414, #415 and #416 as follow-ups, and leave S02-15 open. Fix #414 before removing the flag or relying on named Claude Plan accounts, because it is a real silent switch to the wrong account.