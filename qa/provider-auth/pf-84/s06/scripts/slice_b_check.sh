#!/usr/bin/env bash
# PF-84-S06 slice B functional check: named ChatGPT/OpenAI sign-in accounts.
# Disposable HOME and Corbanu home, CORBANU_TEST_NO_NATIVE_KEYRING=1, a mock OpenAI issuer
# and Responses endpoint (fake tokens only). The default login is an API-key login in
# auth.json; its bytes must never change. A decoy OPENAI_API_KEY is set throughout.
# usage: slice_b_check.sh <debug corbanu> <out dir>
set -u
BIN=$1; OUT=$2
HERE=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$OUT"; OUT=$(cd "$OUT" && pwd)
RUN=$(mktemp -d); export HOME=$RUN/home CODEX_HOME=$RUN/corbanu
mkdir -p "$HOME" "$CODEX_HOME" "$RUN/ws"
export CORBANU_TEST_NO_NATIVE_KEYRING=1
unset CORBANU_HOME PFTERMINAL_HOME ZAI_API_KEY CODEX_API_KEY
PORT=$((20000 + RANDOM % 20000)); LOG=$OUT/mock-events.jsonl; : > "$LOG"
URL=http://127.0.0.1:$PORT
python3 "$HERE/mock_openai.py" --port "$PORT" --log "$LOG" & MOCK=$!
trap 'kill $MOCK 2>/dev/null; tmux -L pf84s06b kill-server 2>/dev/null' EXIT
sleep 1
export CODEX_REFRESH_TOKEN_URL_OVERRIDE=$URL/oauth/token
export OPENAI_API_KEY=sk-fake-envdecoy
cat > "$CODEX_HOME/config.toml" <<EOF
model = "gpt-5.4"
model_provider = "openai"
openai_base_url = "$URL/v1"
chatgpt_base_url = "$URL/backend-api/"
cli_auth_credentials_store = "file"
[features]
named_accounts = true
[projects."$RUN/ws"]
trust_level = "trusted"
EOF

next_user() { curl -s -X POST -H 'content-type: application/json' -d "{\"user\":\"$1\"}" "$URL/_next_user" > /dev/null; }
step() { printf '\n== %s\n' "$*"; }
run() { printf '$ %s\n' "$*"; "$@" 2>&1; printf '[exit %s]\n' "$?"; }
login() { next_user "$2"; run "$BIN" login --account "$1" --device-auth --experimental_issuer "$URL"; }
ex() { "$BIN" exec --skip-git-repo-check -C "$RUN/ws" "$@" 2>&1 | grep -E "openai-mock|ERROR|Error|rror:" | sort -u; }
sum() { sha256sum "$CODEX_HOME/auth.json" | cut -c1-16; }
{
step "default login: an API-key login in auth.json"
printf 'sk-fake-default' | run "$BIN" login --with-api-key
echo "auth.json sha256: $(sum)"
step "B3: device-code sign-in into named accounts (default untouched)"
login work work
login play play
step "B5: same ChatGPT user under a second account: warned, not refused"
login twin work-2
login bad bad-user
run "$BIN" account list openai
echo "auth.json sha256: $(sum)"
step "B2: concurrent refresh in two accounts (both access tokens expire in 60 s)"
ex --account work "Reply with exactly: pong" > "$OUT/b2-work.txt" & W1=$!
ex --account play "Reply with exactly: pong" > "$OUT/b2-play.txt" & W2=$!
wait $W1 $W2
cat "$OUT/b2-work.txt" "$OUT/b2-play.txt"
step "the rotated tokens were saved to each account: no second refresh, no reuse"
ex --account work "Reply with exactly: pong"
ex --account play "Reply with exactly: pong"
step "default account: its own API key; OPENAI_API_KEY decoy unused"
ex "Reply with exactly: pong"
echo "auth.json sha256: $(sum)"
step "an expired refresh token: sign in again, default intact"
ex --account bad "Reply with exactly: pong"
ex "Reply with exactly: pong"
step "unknown account refused before start"
"$BIN" exec --skip-git-repo-check -C "$RUN/ws" --account nope "Reply with exactly: pong" 2>&1 | grep -iE "error" | head -2
step "B3 logout: only play signed out and revoked"
run "$BIN" logout --account play
run "$BIN" account list openai
echo "auth.json sha256: $(sum)"
step "a home with no default login: the named account alone works"
H2=$RUN/corbanu2; mkdir -p "$H2"; cp "$CODEX_HOME/config.toml" "$H2/"
next_user solo; printf '$ CODEX_HOME=<second home> corbanu login --account solo --device-auth\n'
CODEX_HOME=$H2 "$BIN" login --account solo --device-auth --experimental_issuer "$URL" 2>&1 | grep -E "Signed in|rror"
CODEX_HOME=$H2 ex --account solo "Reply with exactly: pong"
CODEX_HOME=$H2 ex "Reply with exactly: pong"
echo "default auth.json in the second home: $(ls "$H2/auth.json" 2>/dev/null || echo none)"
step "mock events (users and generations only)"
cat "$LOG"
} > "$OUT/cli.txt" 2>&1

# TUI: /status, a turn on `work`, the /providers rows and a device-code sign-in from /providers.
T="tmux -L pf84s06b"
cap() { $T capture-pane -p -t a > "$OUT/$1.txt"; }
waitfor() { for _ in $(seq 1 60); do $T capture-pane -p -t a | grep -qE "$1" && return 0; sleep 1; done; return 1; }
send() { $T send-keys -t a -l "$1"; sleep 0.5; $T send-keys -t a Enter; }
select_row() { for _ in $(seq 1 30); do $T capture-pane -p -t a | grep -qE "› +[0-9]+\. +$1" && return 0; $T send-keys -t a Down; sleep 0.3; done; return 1; }
$T kill-server 2>/dev/null
$T new-session -d -s a -x 120 -y 40 -e HOME="$HOME" -e CODEX_HOME="$CODEX_HOME" \
  -e CORBANU_TEST_NO_NATIVE_KEYRING=1 -e OPENAI_API_KEY=sk-fake-envdecoy \
  -e CODEX_REFRESH_TOKEN_URL_OVERRIDE="$CODEX_REFRESH_TOKEN_URL_OVERRIDE" \
  -e CORBANU_TEST_LOGIN_ISSUER="$URL" \
  "$BIN" -C "$RUN/ws" --account work
waitfor "(›|>) "; sleep 2
send "/status"; waitfor "account work"; sleep 1; cap tui-1-status
$T send-keys -t a Escape; sleep 1
send "Reply with exactly: pong"; waitfor "authorized as"; sleep 1; cap tui-2-turn
send "/providers"; waitfor "Providers"; sleep 1; select_row "OpenAI · default"; sleep 1; cap tui-3-providers
$T send-keys -t a Enter; waitfor "Add another account"; sleep 1; cap tui-4-manage
select_row "Add another account"; $T send-keys -t a Enter; waitfor "Account name"; sleep 0.5
next_user tui
send "tui"; waitfor "device code"; sleep 1; cap tui-5-methods
$T send-keys -t a Enter; waitfor "Signed in openai account"; sleep 1; cap tui-6-signed-in
$T kill-server 2>/dev/null

# Leak scan: no fake token, refresh token or API key in captures, logs or rollouts.
JWT_HEADER=$(printf '%s' '{"alg": "none", "typ": "JWT"}' | base64 | tr '+/' '-_' | tr -d '=\n' | cut -c1-20)
grep -rlE "rt-[a-z0-9-]+-[0-9]|sk-fake-|$JWT_HEADER" "$OUT"/*.txt "$CODEX_HOME/log" "$CODEX_HOME/sessions" 2>/dev/null \
  > "$OUT/leak-scan.txt"; echo "files with a token: $(wc -l < "$OUT/leak-scan.txt")" >> "$OUT/leak-scan.txt"
echo done
