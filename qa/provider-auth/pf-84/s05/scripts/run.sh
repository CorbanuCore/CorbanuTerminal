#!/bin/bash
# PF-84-S05 live run: coordinator on the default Z.AI account, spawned agents on `work` (the same
# real key under a second name) and `fake` (a fake key), `corbanu exec` workers in the same home,
# and Kimi as a second provider. Captures /cost, /status and the per-account throttle rows.
set -u
. "$(dirname "${BASH_SOURCE[0]}")/env.sh"
C=$S/captures; mkdir -p "$S/bin" "$S/work" "$S/fakehome" "$H" "$C"
cp "${CANDIDATE:?set CANDIDATE to the debug developer-accounting build}" "$B"
cat > "$H/config.toml" <<CFG
model = "glm-5.3-flash"
model_provider = "zai"
approval_policy = "never"
sandbox_mode = "danger-full-access"
suppress_unstable_features_warning = true
[features]
named_accounts = true
[projects."$S/work"]
trust_level = "trusted"
CFG
git -C "$S/work" init -q 2>/dev/null
PONG="Reply with exactly the word pong and nothing else."
addreal provider/zai_api_key zai work
addval pf84-s05-fake-zai-key-0005 zai fake
addreal provider/kimi_api_key kimi-code alt
KEEP=(); cx account list > "$C/account-list.txt" 2>&1

# Workers in other conversations of the same home.
ex() { local tag=$1; shift; cx exec --skip-git-repo-check "$@" "$PONG" > "$C/exec-$tag.txt" 2>&1; echo "exit=$?" >> "$C/exec-$tag.txt"; }
KEEP=(); ex zai-work -m glm-5.3-flash -c model_provider=zai --account work
KEEP=(); ex zai-fake -m glm-5.3-flash -c model_provider=zai --account fake
( export KIMI_API_KEY="$(helper provider/kimi_api_key)"; KEEP=(KIMI_API_KEY); ex kimi-default -m k3 -c model_provider=kimi-code )
KEEP=(); ex kimi-alt -m k3 -c model_provider=kimi-code --account alt

# The coordinator in tmux.
start coord
waitfor coord 'Corbanu Terminal \(v' 60
send coord "$PONG"; waitfor coord '^\W*pong\W*$' 180
send coord "Use the spawn_agent tool exactly once with account set to work and the message '$PONG'. Wait for it, then use spawn_agent exactly once with account set to fake and the same message, and wait for it. Do not retry either on another account. Finally reply with the word done."
waitfor coord '^\W*done\W*$' 420
snap coord > "$C/tui-after-spawns.txt"
send coord "/status"; sleep 3; snap coord > "$C/tui-status.txt"; T send-keys -t coord Escape
send coord "/cost"; waitfor coord 'account fake' 90; sleep 2; snap coord > "$C/tui-cost.txt"
T capture-pane -p -J -S -400 -t coord > "$C/tui-scrollback.txt"
T send-keys -t coord Escape; sleep 1; send coord "/exit"; sleep 3
T kill-server 2>/dev/null
echo finished
