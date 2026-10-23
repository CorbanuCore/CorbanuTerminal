#!/usr/bin/env bash
# Functional check: N interactive Corbanu sessions (TUI, in tmux) on one
# disposable home holding a copy of a large state DB, on GLM 5.3 Flash with
# developer accounting on. Each session runs a few shell tool calls. Afterwards
# the panes and the home's logs are searched for busy/locked errors, and the
# ledger is checked for every request.
#
# usage: glm_tmux_check.sh <corbanu binary> <state DB copy> <out dir> [sessions]
# The Z.AI key comes from the vault and is never printed.
set -euo pipefail
BIN=$1
DB=$2
OUT=$3
N=${4:-8}
TMUX_BIN=${TMUX_BIN:-tmux}
mkdir -p "$OUT/home" "$OUT/work" "$OUT/panes"
cp "$DB" "$OUT/home/pfterminal_state_5.sqlite"
cat > "$OUT/home/config.toml" <<EOF
[projects."$OUT/work"]
trust_level = "trusted"
EOF
SOCK="$OUT/tmux.sock"
export CORBANU_TEST_NO_NATIVE_KEYRING=1
export CORBANU_HOME="$OUT/home" PFTERMINAL_HOME="$OUT/home" CODEX_HOME="$OUT/home"
# The key is fetched by each session's own command, inline, and never
# exported or printed.
KEY_CMD="env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME $HOME/.local/bin/corbanu vault auth-helper provider/zai_api_key"
if [ "${WARMUP:-0}" = 1 ]; then
  # One session alone first, so the home's runtime DBs exist before eight
  # first launches race to create them (a separate, pre-existing issue).
  (cd "$OUT/work" && ZAI_API_KEY="$($KEY_CMD)" "$BIN" exec --skip-git-repo-check \
    -m glm-5.3-flash -c model_provider='"zai"' \
    "Reply with exactly the word READY." > "$OUT/warmup.txt" 2>&1) || true
fi
PROMPT='Use the shell tool three times, one command per call: pwd, then ls, then date. Then reply with exactly the word FINISHED-OK.'
START_MS=$(($(date +%s) * 1000))
for i in $(seq 1 "$N"); do
  cmd="cd '$OUT/work' && ZAI_API_KEY=\"\$($KEY_CMD)\" '$BIN' --dangerously-bypass-approvals-and-sandbox -m glm-5.3-flash -c model_provider=\"zai\" \"session $i: $PROMPT\""
  if [ "$i" = 1 ]; then
    "$TMUX_BIN" -S "$SOCK" new-session -d -s glm -n "s$i" -x 160 -y 48 "$cmd"
    # Keep a session's pane if it exits, to see why.
    "$TMUX_BIN" -S "$SOCK" set-option -g remain-on-exit on
  else
    "$TMUX_BIN" -S "$SOCK" new-window -t glm -n "s$i" "$cmd"
  fi
done
echo "started $N sessions at $(date -u +%H:%M:%S)"
deadline=$((SECONDS + ${WAIT_SECONDS:-420}))
while [ $SECONDS -lt $deadline ]; do
  done_count=0
  for i in $(seq 1 "$N"); do
    # The answer, not the prompt that asks for it.
    if "$TMUX_BIN" -S "$SOCK" capture-pane -p -t "glm:s$i" -S -200 2>/dev/null \
      | grep "FINISHED-OK" | grep -vq "reply with"; then
      done_count=$((done_count + 1))
    fi
  done
  echo "$(date -u +%H:%M:%S) sessions done: $done_count/$N"
  [ "$done_count" -ge "$N" ] && break
  sleep 15
done
# The sessions' ledger writers finish what they hold (the first writer of an
# hour validates the whole ledger first) before the sessions are closed.
for _ in $(seq 1 36); do
  left=$(find "$OUT/home/accounting-pending" -name '*.json' 2>/dev/null | wc -l | tr -d ' ')
  [ "$left" = 0 ] && break
  sleep 5
done
echo "records still saved for a later writer: ${left:-0}"
for i in $(seq 1 "$N"); do
  "$TMUX_BIN" -S "$SOCK" capture-pane -p -t "glm:s$i" -S -400 > "$OUT/panes/s$i.txt" 2>/dev/null || true
done
echo "--- busy/locked/failed lines in panes:"
grep -HiE "database is locked|state database is busy|developer accounting failed|could not record|Fatal error|throttle state" "$OUT"/panes/*.txt || echo "(none)"
echo "--- busy/locked/accounting warnings in the home's logs DB:"
sqlite3 "$OUT/home/pfterminal_logs_2.sqlite" \
  "SELECT count(*) FROM logs WHERE level IN ('WARN','ERROR') AND (feedback_log_body LIKE '%database is locked%' OR feedback_log_body LIKE '%state database is busy%' OR feedback_log_body LIKE '%developer accounting failed%')" \
  2>/dev/null || echo "(no logs DB)"
echo "--- ledger: attempts, conversations and observations recorded by this run:"
sqlite3 "$OUT/home/pfterminal_state_5.sqlite" \
  "SELECT count(*), count(DISTINCT json_extract(a.payload, '\$.thread_id')), (SELECT count(*) FROM draft_accounting_observations o WHERE o.attempt_id IN (SELECT attempt_id FROM draft_accounting_attempts WHERE json_extract(payload, '\$.provider') = 'zai' AND json_extract(payload, '\$.dispatched_at_ms') >= $START_MS)) FROM draft_accounting_attempts a WHERE json_extract(a.payload, '\$.provider') = 'zai' AND json_extract(a.payload, '\$.dispatched_at_ms') >= $START_MS"
"$TMUX_BIN" -S "$SOCK" kill-server 2>/dev/null || true
