#!/bin/bash
# #656 tmux check: flag-off `corbanu resume <id>` of a thread recorded on a named account. Usage: t656.sh <label> <bin>
set -u
L=$1 BIN=$2
S=${QA_SCRATCH:?set QA_SCRATCH to a scratch directory}
SCR=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)/qa/provider-auth/pf-84/independent-acceptance-s06b-20261013/scripts
P=34782 U=http://127.0.0.1:34782
H=$S/homes/t656-$L FH=$S/fh-$L LOG=$S/logs/mock656-$L.jsonl SOCK=$S/tmux656-$L.sock
mkdir -p $S/logs $S/ws $FH; [ -d $H ] && rm -r $H < /dev/null; mkdir -p $H; : > $LOG
python3 $SCR/mock.py --port $P --log $LOG > /dev/null 2>&1 &
MOCK=$!; trap 'kill $MOCK 2>/dev/null' EXIT; sleep 1
cat > $H/config.toml <<EOT
model = "gpt-5.4"
model_provider = "openai"
approval_policy = "never"
sandbox_mode = "read-only"
openai_base_url = "$U/v1"
chatgpt_base_url = "$U/backend-api/codex"
suppress_unstable_features_warning = true
[features]
named_accounts = true
[projects."$S/ws"]
trust_level = "trusted"
EOT
ENVV=(env -i HOME=$FH USER=$USER TERM=xterm-256color LANG=en_US.UTF-8 PATH=/opt/homebrew/bin:/usr/bin:/bin CORBANU_HOME=$H CORBANU_TEST_NO_NATIVE_KEYRING=1)
printf 'sk-fake-656-default' | "${ENVV[@]}" "$BIN" login --with-api-key 2>&1 | tail -1
printf 'sk-fake-656-apikey' | "${ENVV[@]}" "$BIN" login --account work --with-api-key 2>&1 | tail -1
"${ENVV[@]}" "$BIN" exec --skip-git-repo-check -C $S/ws --account work "Reply with exactly: pong" < /dev/null > $S/logs/t656-exec-$L.txt 2>&1
ID=$(sed -n 's/^session id: //p' $S/logs/t656-exec-$L.txt | head -1); echo "recorded thread $ID on openai:work"
T="tmux -S $SOCK"
CMD=$(printf '%q ' "${ENVV[@]}" "$BIN" -C $S/ws resume "$ID" --disable named_accounts)
$T new-session -d -s t -x 140 -y 30 "$CMD; echo EXITED-\$?; sleep 600"
for i in $(seq 1 60); do $T capture-pane -p -t t | grep -q EXITED- && break; sleep 0.5; done
echo "== $L: \$ corbanu resume <id> --disable named_accounts"
$T capture-pane -p -t t -S -60 | grep -v '^\s*$' | sed "s#$S#\$S#g; s#$ID#<id>#g"
$T kill-server
