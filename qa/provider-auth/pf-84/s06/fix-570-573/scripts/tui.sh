#!/usr/bin/env bash
# TUI checks (private tmux socket, disposable home t1 from setup_t; mock on 34741 must run).
# usage: tui.sh <V1|V2|V3|B572|A573> <BASE|CAND>   output appended to $S/logs/tui-<variant>.txt
set -u
source ~/corbanu-rtx/f57x-qa/env.sh
V=$1; B=$2; bin=${!B}
export P=34741 U=http://127.0.0.1:34741 H=$S/homes/t1 FH=$S/fh-t1
XENV=(CODEX_REFRESH_TOKEN_URL_OVERRIDE=$U/oauth/token CORBANU_TEST_LOGIN_ISSUER=$U ZAI_API_KEY=fake-zai-S06CANARY OPENAI_API_KEY=sk-envdecoy-S06CANARY)
OUT=$S/logs/tui-$V-$B.txt; s=$V$B
reset_home() { sed -i 's/^model = .*/model = "glm-5.3-flash"/; s/^model_provider = .*/model_provider = "zai"/' $H/config.toml; }
to_openai() { send_line $s "/model"; sleep 3; tx send-keys -t $s Left; sleep 1.5; pick $s 'GPT-6 Luna'; sleep 1; tx send-keys -t $s Enter; sleep 4; }
turn() { local b=$(mocklines $S/logs/mock-t.jsonl); send_line $s "Reply with exactly: pong"; sleep 15
  echo "-- mock lines for this turn:"; tail -n +$((b+1)) $S/logs/mock-t.jsonl | grep -E '"responses' | sed 's/^/   /'; }
status() { send_line $s "/status"; sleep 3; cap $s -70 | grep -E 'Model provider|Account:' | sed 's/^ *//' | cut -c1-120; tx send-keys -t $s Escape; sleep 0.5; }
screen() { cap $s -60 | grep -v '^\s*$' | tail -${1:-14} | cut -c1-170; }
reset_home
{
echo; echo "######## $V on $B ($(basename $bin))"
case $V in
  V1) echo "TUI started on OpenAI with --account work (control)"
      start_tui $s $bin $H -c 'model_provider="openai"' -m gpt-5.4 --account work; sleep 14; turn; screen 6; status ;;
  V2) echo "TUI started on Z.AI with --account openai:work, then /model -> OpenAI GPT-6 Luna"
      start_tui $s $bin $H --account openai:work; sleep 14; to_openai; turn; screen 6; status ;;
  V2N) echo "TUI started on Z.AI with --account openai:work and NO default OpenAI login or OPENAI_API_KEY, then /model -> OpenAI GPT-6 Luna"
      XENV=("${XENV[@]/OPENAI_API_KEY=*/NO_OPENAI_KEY=1}")
      start_tui $s $bin $H --account openai:work; sleep 14; to_openai; turn; screen 6; status ;;
  V3) echo "TUI started on Z.AI; /providers -> OpenAI · work -> Use for this session; /model -> OpenAI GPT-6 Luna"
      start_tui $s $bin $H; sleep 14; send_line $s "/providers"; sleep 4; pick $s 'OpenAI · work'; sleep 2; tx send-keys -t $s Enter; sleep 6
      screen 3; to_openai; turn; screen 6; status ;;
  B572) echo "#572: TUI on Z.AI; /providers -> Amazon Bedrock · work -> Use for this session; /model -> Other tab"
      start_tui $s $bin $H; sleep 14; send_line $s "/providers"; sleep 4; pick $s 'Bedrock · work'; sleep 2; tx send-keys -t $s Enter; sleep 6
      screen 3; send_line $s "/model"; sleep 3; tx send-keys -t $s Right; sleep 1.5; echo "-- /model, tab right of Z.AI:"; screen 12
      if cap $s -30 | grep -Eq '^ *›? *[0-9]+\. .*GPT-5\.4'; then
        pick $s 'GPT-5.4' 20 && { sleep 1; tx send-keys -t $s Enter; sleep 4; }
        screen 4; turn; screen 6; status
      fi ;;
  A573) echo "#573: TUI on Z.AI; /providers -> OpenAI · default -> Add another account (methods offered)"
      start_tui $s $bin $H; sleep 14; send_line $s "/providers"; sleep 4; pick $s 'OpenAI · default'; sleep 2
      pick $s 'Add another account' 20; sleep 2; tx send-keys -t $s -l 'second'; sleep 0.5; tx send-keys -t $s Enter; sleep 2; screen 10 ;;
esac
tx send-keys -t $s Escape; sleep 0.5; tx send-keys -t $s Escape; sleep 0.5; send_line $s "/exit"; sleep 3; tx kill-session -t $s 2>/dev/null
reset_home
} >> "$OUT" 2>&1
cat "$OUT"
