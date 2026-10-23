# PF-84 #572: Bedrock models offered while a named Bedrock account exists

Branch `fix/572-bedrock-named-model`. Linux (glitch) debug builds with developer-accounting, BASE = `origin/main`
060ce90947, CAND = this branch. Disposable home `t1` (`scripts/setup_t.sh`: Z.AI session, fake `~/.aws` files, named
Bedrock account `work` = AWS profile `work`, default Bedrock account not configured), stdlib mock (`scripts/mock.py`),
private tmux socket. Script `scripts/tui.sh B572`.

| Check | BASE | CAND |
| --- | --- | --- |
| `/providers` → *Amazon Bedrock · work* → *Use for this session*, then `/model` → *Other* | only `glm-5.2 via zai-anthropic`; Bedrock unreachable ([capture](captures/tui-B572-BASE.txt)) | five Bedrock models; *GPT-5.4 via amazon-bedrock* is signed by `AKIDWORKPROFILE`, `/status` names account `work`, no fallback-metadata warning ([capture](captures/tui-B572-CAND.txt)) |

Tests and clippy (acct and plain): [gate-final.txt](captures/gate-final.txt). The new unit tests use functions this change adds;
the behavioural before/after is the tmux table. `a_session_account_that_cannot_load_changes_nothing` fails on BASE too
(stale since S06; fixed by the switch-follow branch).
