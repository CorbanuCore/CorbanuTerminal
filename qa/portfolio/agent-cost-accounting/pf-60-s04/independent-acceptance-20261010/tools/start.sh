#!/bin/bash
# remote (Linux). usage: start.sh <session> <home> <cwd> [corbanu args...]
# Reads four lines on stdin (Z.AI key, Kimi Code key, Claude setup token, OpenAI key); each goes only into the environment
# of this session's private tmux server (tmux -L ia-<session>). Starts the candidate in a 160x50 pane.
R=$HOME/corbanu-rtx/pf60s04-ia; s=$1; h=$2; cwd=$3; shift 3
IFS= read -r ZAI_API_KEY; IFS= read -r KIMI_API_KEY; IFS= read -r CLAUDE_CODE_OAUTH_TOKEN; IFS= read -r OPENAI_API_KEY
export ZAI_API_KEY KIMI_API_KEY CLAUDE_CODE_OAUTH_TOKEN OPENAI_API_KEY
[ -n "$NO_OPENAI_ENV" ] && unset OPENAI_API_KEY
export CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h
# Frames only: never tungstenite handshake/http traces (S05 incident: a broad trace logged a bearer).
export RUST_LOG="${RUST_LOG:-codex_api=trace,tungstenite::protocol=trace,codex_core::accounting=debug,codex_core=info,warn}"
export PATH="$R/bin:$PATH"
mkdir -p "$h/logs" "$R/cap" "$R/logs"
unset TMUX
tmux -L "ia-$s" kill-server 2>/dev/null
{ echo '#!/bin/bash'; printf '%q ' "$R/bin/corbanu" -C "$cwd" -c "log_dir=\"$h/logs\"" "$@"; printf '%s\n' '2>>"$CODEX_HOME/logs/stderr.log"'; echo 'echo EXITED $?; sleep 600'; } > "$h/launch.sh"
cp "$h/launch.sh" "$R/logs/$s.launch.sh"
tmux -L "ia-$s" new-session -d -s "$s" -x 160 -y 50 "bash $h/launch.sh"
echo "started $s $(date -u +%FT%TZ)"
