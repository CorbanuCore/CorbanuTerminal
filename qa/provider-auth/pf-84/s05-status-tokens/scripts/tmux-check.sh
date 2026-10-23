#!/usr/bin/env bash
# PF-84 /status per-account tokens: tmux functional check in a disposable home.
# usage: tmux-check.sh <corbanu-binary> <run-name> [cost]
set -u
BIN=$1; NAME=$2; COST=${3:-}
ROOT=/Volumes/CorbanuDrive/Corbanu/.codex-work/pf84-status-tokens/$NAME
SOCK=pf84st-$NAME
mkdir -p "$ROOT/home" "$ROOT/ws" "$ROOT/userhome" "$ROOT/logs" "$ROOT/captures"
cd "$ROOT/ws" && git init -q 2>/dev/null
cat > "$ROOT/home/config.toml" <<EOF
suppress_unstable_features_warning = true
approval_policy = "never"
sandbox_mode = "danger-full-access"
model = "glm-5.3-flash"
model_provider = "zai"
log_dir = "$ROOT/logs"
[features]
named_accounts = true
[projects."$ROOT/ws"]
trust_level = "trusted"
EOF
export CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME="$ROOT/home" CORBANU_HOME="$ROOT/home" PFTERMINAL_HOME="$ROOT/home"
# Enroll two named accounts with the same real key (read only inside the consuming command).
for account in work personal; do
  ZAI_API_KEY="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper provider/zai_api_key)" \
    bash -c 'printf %s "$ZAI_API_KEY" | "$0" account add zai "$1" >/dev/null' "$BIN" "$account" || { echo "enroll $account failed"; exit 1; }
done
"$BIN" account list 2>&1 | sed -n 1,10p > "$ROOT/captures/accounts.txt"

tmux -L "$SOCK" kill-server 2>/dev/null
tmux -L "$SOCK" new-session -d -s main -x 150 -y 50 \
  "env HOME=$ROOT/userhome CODEX_HOME=$ROOT/home CORBANU_HOME=$ROOT/home PFTERMINAL_HOME=$ROOT/home CORBANU_TEST_NO_NATIVE_KEYRING=1 RUST_LOG=info $BIN -C $ROOT/ws --account work; sleep 600"

cap() { tmux -L "$SOCK" capture-pane -p -t main -S -200; }
wait_for() { # regex timeout
  local deadline=$((SECONDS + $2)) prev="" cur
  while [ $SECONDS -lt $deadline ]; do
    cur=$(cap)
    if echo "$cur" | grep -Eq -- "$1" && [ "$cur" = "$prev" ]; then return 0; fi
    prev=$cur; sleep 1
  done
  echo "TIMEOUT waiting for: $1"; cap | tail -40; return 1
}
send() { tmux -L "$SOCK" send-keys -t main -l -- "$1"; sleep 0.5; tmux -L "$SOCK" send-keys -t main Enter; }
snap() { cap > "$ROOT/captures/$1.txt"; }

wait_for "Corbanu Terminal" 60 || exit 1
sleep 2
send "/status"; wait_for "Account usage:" 30 || exit 1; snap 1-status-before
send "Reply with the single word pong"; wait_for "^[^a-zA-Z]*pong[^a-zA-Z]*$" 180 || exit 1
send "/status"; sleep 3; wait_for "Account usage: +work · [0-9,]+ input" 30 || exit 1; snap 2-status-after-work
send "Use the spawn_agent tool exactly once with account set to personal and the message 'Reply with the single word pong'. Wait for it, then reply with the word done. Do not retry on another account."
wait_for "^[^a-zA-Z]*done[^a-zA-Z]*$" 400 || exit 1
send "/status"; sleep 3; wait_for "personal +[0-9,]+ +[0-9,]+" 30 || exit 1; snap 3-status-after-personal
if [ -n "$COST" ]; then
  send "/cost"; wait_for "account personal" 60 || exit 1; sleep 2; snap 4-cost
  tmux -L "$SOCK" send-keys -t main Escape; sleep 1
fi
send "/exit"; sleep 3
tmux -L "$SOCK" kill-server 2>/dev/null
# Resume the conversation: the coordinator's own counts come back.
tmux -L "$SOCK" new-session -d -s main -x 150 -y 50 \
  "env HOME=$ROOT/userhome CODEX_HOME=$ROOT/home CORBANU_HOME=$ROOT/home PFTERMINAL_HOME=$ROOT/home CORBANU_TEST_NO_NATIVE_KEYRING=1 RUST_LOG=info $BIN -C $ROOT/ws resume --last --account work; sleep 600"
wait_for "Corbanu Terminal" 60 || exit 1
sleep 3
send "/status"; sleep 3; wait_for "work \\(active\\) +[0-9,]+" 30 || exit 1; snap 5-resumed
send "/exit"; sleep 3
tmux -L "$SOCK" kill-server 2>/dev/null
echo "DONE $ROOT"
