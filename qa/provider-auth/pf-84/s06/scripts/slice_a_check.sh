#!/usr/bin/env bash
# PF-84-S06 slice A functional check: named Amazon Bedrock accounts use one AWS profile.
# Disposable HOME and Corbanu home, fake AWS files (fake key ids only), a mock Bedrock
# Mantle endpoint, CORBANU_TEST_NO_NATIVE_KEYRING=1. Decoy AWS env credentials are set
# throughout: a named account must ignore them; the default account keeps today's chain.
# usage: slice_a_check.sh <debug corbanu> <out dir>
set -u
BIN=$1; OUT=$2
HERE=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$OUT"; OUT=$(cd "$OUT" && pwd)
RUN=$(mktemp -d); export HOME=$RUN/home CODEX_HOME=$RUN/corbanu
mkdir -p "$HOME/.aws" "$CODEX_HOME" "$RUN/ws"
export CORBANU_TEST_NO_NATIVE_KEYRING=1
unset CORBANU_HOME PFTERMINAL_HOME OPENAI_API_KEY ZAI_API_KEY
PORT=$((20000 + RANDOM % 20000)); LOG=$OUT/mock-requests.jsonl; : > "$LOG"
python3 "$HERE/mock_bedrock.py" --port "$PORT" --log "$LOG" & MOCK=$!
trap 'kill $MOCK 2>/dev/null; tmux -L pf84s06a kill-server 2>/dev/null' EXIT

cat > "$HOME/.aws/config" <<EOF
[default]
region = eu-west-1
[profile work]
region = us-west-2
[profile other]
region = eu-central-1
EOF
cat > "$HOME/.aws/credentials" <<EOF
[default]
aws_access_key_id = AKIDDEFAULTPROFILE
aws_secret_access_key = fake-secret-default
[work]
aws_access_key_id = AKIDWORKPROFILE
aws_secret_access_key = fake-secret-work
[other]
aws_access_key_id = AKIDOTHERPROFILE
aws_secret_access_key = fake-secret-other
EOF
cat > "$CODEX_HOME/config.toml" <<EOF
model = "openai.gpt-5.4"
model_provider = "amazon-bedrock"
[features]
named_accounts = true
[model_providers.amazon-bedrock]
base_url = "http://127.0.0.1:$PORT/openai/v1"
[model_providers.amazon-bedrock.aws]
region = "us-west-2"
[projects."$RUN/ws"]
trust_level = "trusted"
EOF
# Decoys a named account must ignore.
export AWS_ACCESS_KEY_ID=AKIDENVIRONMENT AWS_SECRET_ACCESS_KEY=fake-secret-env
export AWS_PROFILE=default AWS_BEARER_TOKEN_BEDROCK=fake-bedrock-bearer-canary

step() { printf '\n== %s\n' "$*"; }
run() { printf '$ %s\n' "$*"; "$@" 2>&1; printf '[exit %s]\n' "$?"; }
ex() { run "$BIN" exec --skip-git-repo-check -C "$RUN/ws" "$@"; }
{
step "A1 add, list, rename (profile name is metadata, validated, shown in clear)"
run "$BIN" account add amazon-bedrock work --kind aws-profile --value work
run "$BIN" account add amazon-bedrock bad --kind aws-profile --value "not a profile"
run "$BIN" account add amazon-bedrock spare --kind aws-profile --value other
run "$BIN" account rename amazon-bedrock spare alt
run "$BIN" account add amazon-bedrock gone --kind aws-profile --value nosuchprofile
run "$BIN" account list amazon-bedrock
step "A2 named account signs with its profile only (env keys, AWS_PROFILE, bearer token set)"
ex --account work "Reply with exactly: pong"
ex --account alt "Reply with exactly: pong"
step "default account keeps today's chain (AWS_BEARER_TOKEN_BEDROCK wins)"
ex "Reply with exactly: pong"
step "A3 missing profile refused before any request; unconfigured account refused at start"
before=$(wc -l < "$LOG")
ex --account gone "Reply with exactly: pong"
ex --account nope "Reply with exactly: pong"
echo "mock requests during A3: $(( $(wc -l < "$LOG") - before ))"
step "flag off: named account refused"
ex --disable named_accounts --account work "Reply with exactly: pong"
step "account remove"
run "$BIN" account remove amazon-bedrock alt
run "$BIN" account list amazon-bedrock
} > "$OUT/cli.txt" 2>&1

# TUI: /status names the account; a turn is signed by the account's profile; restart is durable.
T="tmux -L pf84s06a"
cap() { $T capture-pane -p -t a > "$OUT/$1.txt"; }
waitfor() { for _ in $(seq 1 60); do $T capture-pane -p -t a | grep -qE "$1" && return 0; sleep 1; done; return 1; }
tui() {
  $T kill-server 2>/dev/null
  $T new-session -d -s a -x 120 -y 40 -e HOME="$HOME" -e CODEX_HOME="$CODEX_HOME" \
    -e CORBANU_TEST_NO_NATIVE_KEYRING=1 -e AWS_ACCESS_KEY_ID=AKIDENVIRONMENT \
    -e AWS_SECRET_ACCESS_KEY=fake-secret-env -e AWS_PROFILE=default \
    -e AWS_BEARER_TOKEN_BEDROCK=fake-bedrock-bearer-canary "$BIN" -C "$RUN/ws" "$@"
  waitfor "(›|>) " || true; sleep 2
}
send() { $T send-keys -t a -l "$1"; sleep 0.5; $T send-keys -t a Enter; }
# Move the list selection (›) to the row matching $1.
select_row() { for _ in $(seq 1 30); do $T capture-pane -p -t a | grep -qE "› +[0-9]+\. +$1" && return 0; $T send-keys -t a Down; sleep 0.3; done; return 1; }
tui --account work
send "/status"; waitfor "account work"; sleep 1; cap tui-1-status
$T send-keys -t a Escape; sleep 1
send "Reply with exactly: pong"; waitfor "signed by"; sleep 1; cap tui-2-turn
# /providers: the Bedrock rows; add an account on "Amazon Bedrock · default".
send "/providers"; waitfor "Providers"; sleep 1
select_row "Amazon Bedrock · default"; sleep 1; cap tui-3-providers
$T send-keys -t a Enter; waitfor "Add another account"; sleep 1; cap tui-4-manage
$T send-keys -t a Enter; waitfor "Account name"; sleep 0.5
send "prod"; waitfor "AWS profile name"; sleep 1; cap tui-5-profile-entry
send "other"; waitfor "Saved amazon-bedrock account"; sleep 1; cap tui-6-saved
$T kill-server 2>/dev/null; sleep 1
# Restart with `prod` as the default for new sessions: durable selection, other profile.
printf '[provider_accounts]\n"amazon-bedrock" = "prod"\n' >> "$CODEX_HOME/config.toml"
tui
send "Reply with exactly: pong"; waitfor "signed by"; sleep 1
send "/status"; waitfor "account prod"; sleep 1; cap tui-7-restart
$T kill-server 2>/dev/null

# Leak scan: no fake secret or bearer canary in any capture or the profile's logs.
grep -rlE "fake-secret-|fake-bedrock-bearer-canary" "$OUT" "$CODEX_HOME/log" "$CODEX_HOME/sessions" 2>/dev/null \
  > "$OUT/leak-scan.txt"; echo "files with a secret: $(wc -l < "$OUT/leak-scan.txt")" >> "$OUT/leak-scan.txt"
cp "$CODEX_HOME/config.toml" "$OUT/config.toml"
echo done
