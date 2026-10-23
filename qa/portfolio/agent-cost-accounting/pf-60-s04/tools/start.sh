#!/bin/bash
# remote. usage: start.sh <session> <home> <cwd> [corbanu args...]
# Reads five lines on stdin (Z.AI key, Kimi Code key, Claude setup token, OpenAI key, DeepSeek key); each goes only into the
# environment of this session's private tmux server. Starts the recorded candidate in a 160x50 pane.
Q=$HOME/corbanu-rtx/pf60s04-qual; s=$1; h=$2; cwd=$3; shift 3
IFS= read -r ZAI_API_KEY; IFS= read -r KIMI_API_KEY; IFS= read -r CLAUDE_CODE_OAUTH_TOKEN; IFS= read -r OPENAI_API_KEY; IFS= read -r DEEPSEEK_API_KEY
export ZAI_API_KEY KIMI_API_KEY CLAUDE_CODE_OAUTH_TOKEN OPENAI_API_KEY DEEPSEEK_API_KEY
[ -n "$NO_OPENAI_ENV" ] && unset OPENAI_API_KEY
export CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h
export RUST_LOG="${RUST_LOG:-codex_api=trace,tungstenite::protocol=trace,codex_core::accounting=debug,codex_core=info,warn}"
mkdir -p "$h/logs" "$Q/cap"
unset TMUX
tmux -L "q-$s" kill-server 2>/dev/null
{ echo '#!/bin/bash'; printf '%q ' "$Q/bin/corbanu" -C "$cwd" -c "log_dir=\"$h/logs\"" "$@"; printf '%s\n' '2>>"$CODEX_HOME/logs/stderr.log"'; echo 'echo EXITED $?; sleep 600'; } > "$h/launch.sh"
cp "$h/launch.sh" "$Q/logs/$s.launch.sh"
tmux -L "q-$s" new-session -d -s "$s" -x 160 -y 50 "bash $h/launch.sh"
echo "started $s $(date -u +%FT%TZ)"
