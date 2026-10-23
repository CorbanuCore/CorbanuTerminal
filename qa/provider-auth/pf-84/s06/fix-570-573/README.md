# PF-84 #570, #571, #573: named OpenAI accounts and login restrictions

Branch `fix/pf84-named-login-restrictions`. Linux (glitch) debug builds with developer-accounting:
BASE = `origin/main` 060ce90947, CAND = this branch. Disposable homes, `CORBANU_TEST_NO_NATIVE_KEYRING=1`,
the acceptance's stdlib mock (`scripts/mock.py`: issuer, refresh with rotation, Responses) and fake tokens only.
The TUI runs on a private tmux socket (`scripts/env.sh`).

| Check | BASE | CAND | Capture |
| --- | --- | --- | --- |
| #570 cases 1/5/7 (`exec --account`, restriction rules out the default) | `auth.json` deleted; case 7 refused for the default's workspace | `auth.json` hash unchanged; 7 answered as `key` | [forced.txt](captures/forced.txt) |
| #570 case 3 (default session) | logged out, continues | same (unchanged) | [forced.txt](captures/forced.txt) |
| #570 TUI on `chat`, *Use default account for this session* | login screen at start, `auth.json` deleted | refused with the reason, `auth.json` unchanged | [switchdef](captures/tui-switchdef-CAND.txt) |
| #570 review: `resume --last` / `fork --last --account chat` of a named thread | sign-in screen, `auth.json` deleted | resumes/forks on `chat`, `auth.json` unchanged; `fork` without `--account` shows the sign-in (documented) | [resumefork](captures/tui-resumefork-CAND.txt) |
| #571 V2 (`--account openai:work` on Z.AI, `/model` → OpenAI) | `scheme: none` | ChatGPT token of `work` | [V2](captures/tui-V2-CAND.txt) |
| #571 V3 (*Use for this session*, `/model` → OpenAI) | `scheme: none` | ChatGPT token of `work` | [V3](captures/tui-V3-CAND.txt) |
| #571 V2 with no default OpenAI credential at all | no OpenAI tab in `/model` | reaches OpenAI as `work` | [V2N](captures/tui-V2N-CAND.txt) |
| V1 control (started on OpenAI with `--account work`) | `work` | `work` | [V1](captures/tui-V1-CAND.txt) |
| #573 `/providers` → OpenAI → *Add another account* on Z.AI | API key only | device code or API key | [A573](captures/tui-A573-CAND.txt) |

Regression tests failing on BASE source and passing on CAND: [tests-before-after.txt](captures/tests-before-after.txt),
[test-resume-before.txt](captures/test-resume-before.txt); final Linux gate (commit named inside):
[gate-final.txt](captures/gate-final.txt); macOS `just test`: [mac-just-test.txt](captures/mac-just-test.txt) (failure causes listed there: the stale test, four tests
hitting the macOS socket-path limit that pass on Linux, and six tmux onboarding tests that fail on `origin/main` too:
[onboarding-linux-main.txt](captures/onboarding-linux-main.txt); that main run's untracked leftovers are not referenced by main's
module lists, so they were not compiled). The TUI test
`a_session_account_that_cannot_load_changes_nothing` fails on BASE too (stale since S06; fixed by the switch-follow branch).
