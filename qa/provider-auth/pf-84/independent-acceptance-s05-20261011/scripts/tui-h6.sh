#!/bin/bash
# tui-h6.sh : built-in zai through the forwarding TLS intercept (wire log $S/logs/wire-h6.jsonl).
# Session 1 (default): pong; spawn work + fake; a long prompt interrupted with Esc; /cost.
# Session 2 (--account work): pong; /compact; pong; /status; /cost.
# Then `exec resume` of session 2 without --account (must stay on work).
cd "$(dirname "$0")" && . ./env.sh && . ./tmuxlib.sh && . ./pagedump.sh
H=$S/homes/h6 C=$EV/captures
export HTTPS_PROXY=http://127.0.0.1:18470 SSL_CERT_FILE=$S/certs/ca.pem
SOCK=$S/tui-h6a.sock; start_keyed_tui "$SOCK" a provider/zai_api_key ZAI_API_KEY "$H"; sleep 8
send_line a "Reply with exactly the word pong and nothing else." || exit 1; sleep 12
send_line a 'Use your spawn agent tool (not the shell) to start two sub-agents, one after the other. The first uses account "work"; the second uses account "fake". Give each the task: Reply with exactly the word pong and nothing else. Wait for both, then report each result in one line.' || exit 1
sleep 70
send_line a "Write a 600-word story about a lighthouse keeper." || exit 1; sleep 4; tx send-keys -t a Escape; sleep 3
cap a -200 | grep -v '^\s*$' > "$C/tui-h6-session-default.txt"
tx send-keys -t a C-l; send_line a /cost; sleep 5; pagedump a "$C/tui-h6-cost-default.txt" 70
tx send-keys -t a Escape; sleep 1; tx send-keys -t a Escape; sleep 1; send_line a /exit; sleep 3
cap a -10 | grep -v '^\s*$' | tail -3 >> "$C/tui-h6-session-default.txt"; /opt/homebrew/bin/tmux -S "$SOCK" kill-server

SOCK=$S/tui-h6b.sock; start_keyed_tui "$SOCK" b provider/zai_api_key ZAI_API_KEY "$H" --account work; sleep 8
send_line b "Reply with exactly the word pong and nothing else." || exit 1; sleep 12
send_line b /compact || exit 1; sleep 25
send_line b "Reply with exactly the word pong and nothing else." || exit 1; sleep 12
cap b -200 | grep -v '^\s*$' > "$C/tui-h6-session-work-compact.txt"
tx send-keys -t b C-l; send_line b /status; sleep 4; cap b -60 | grep -E 'Model provider|Session:' >> "$C/tui-h6-session-work-compact.txt"
tx send-keys -t b Escape; tx send-keys -t b C-l; send_line b /cost; sleep 5; pagedump b "$C/tui-h6-cost-work.txt" 70
tx send-keys -t b Escape; sleep 1; tx send-keys -t b Escape; sleep 1; send_line b /exit; sleep 3
cap b -10 | grep -v '^\s*$' | tail -3 >> "$C/tui-h6-session-work-compact.txt"; /opt/homebrew/bin/tmux -S "$SOCK" kill-server

TH=$(grep -o 'corbanu resume [0-9a-f-]*' "$C/tui-h6-session-work-compact.txt" | tail -1 | awk '{print $3}')
key provider/zai_api_key env CORBANU_HOME="$H" S="$S" python3 ./cleanexec.py ZAI_API_KEY -- "$CAND" exec --skip-git-repo-check -C "$S/ws" \
  resume "$TH" "Reply with exactly the word pong and nothing else." > "$S/logs/h6-resume.out" 2> "$S/logs/h6-resume.err"
echo "exec resume $TH (no --account) -> exit=$? reply=$(tr -d '\n' < "$S/logs/h6-resume.out" | cut -c1-20)" > "$C/h6-resume.txt"
sqlite3 "$H"/pfterminal_state_*.sqlite "select json_extract(payload,'$.turn'), json_extract(payload,'$.account') from draft_accounting_attempts where json_extract(payload,'$.thread_id')='$TH' order by json_extract(payload,'$.dispatched_at_ms')" >> "$C/h6-resume.txt"
LC_ALL=C sed -i '' "s#$S#\$S#g" "$C"/tui-h6-*.txt "$C/h6-resume.txt"
cat "$C/h6-resume.txt"
