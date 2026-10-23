#!/usr/bin/env bash
# Characterise which credential an OpenAI turn carries after the various TUI routes onto OpenAI, home t1
# (mock at 34631 must be running). One TUI per variant; each logs the mock lines its turn produced.
# usage: tui_variants.sh V1|V2|V3|V4|V5|V6 (BIN=$BASE for the baseline)   Output appended to captures/tui-13-openai-variants.txt
set -u
D=$(cd "$(dirname "$0")" && pwd); source "$D/env.sh"; source "$D/tmuxlib.sh"; source "$D/aws_setup.sh"; source "$D/tui_setup.sh"
OUT=$EV/captures/tui-13-openai-variants.txt; V=$1; export SOCK=$S/tmux-$V.sock
to_openai_model() { # /model -> OpenAI tab -> GPT-6 Luna -> default effort
  send_line $V "/model"; sleep 3; tx send-keys -t $V Left; sleep 1.5; pick $V 'GPT-6 Luna'; sleep 1; tx send-keys -t $V Enter; sleep 3; }
turn() { local b=$(mocklines $S/logs/mock-t.jsonl); send_line $V "Reply with exactly: pong"; sleep 12
  echo "-- mock lines for this turn (credential-bearing only):"; tail -n +$((b+1)) $S/logs/mock-t.jsonl | grep -E '"responses' ; }
{
echo; echo "######## $V ${BIN:+(BASE)}"
case $V in
  V1) echo "TUI started directly on OpenAI: -c model_provider=openai -m gpt-5.4 --account work"
      start_tui $V $H -c 'model_provider="openai"' -m gpt-5.4 --account work; sleep 14 ;;
  V2) echo "TUI started on Z.AI (real key) with --account openai:work, then /model -> OpenAI GPT-6 Luna"
      start_keyed_tui $SOCK $V provider/zai_api_key ZAI_API_KEY $H --account openai:work; sleep 14; to_openai_model ;;
  V4) echo "TUI started on Z.AI (real key) with --account openai:fake (named API-key account), then /model -> OpenAI GPT-6 Luna"
      start_keyed_tui $SOCK $V provider/zai_api_key ZAI_API_KEY $H --account openai:fake; sleep 14; to_openai_model ;;
  V3) echo "TUI started on Z.AI (real key); /providers -> OpenAI · work -> Use for this session; then /model -> OpenAI GPT-6 Luna"
      start_keyed_tui $SOCK $V provider/zai_api_key ZAI_API_KEY $H; sleep 14
      send_line $V "/providers"; sleep 3; pick $V 'OpenAI · work'; tx send-keys -t $V Enter; sleep 5
      cap $V -10 | grep -E 'New session|■' ; to_openai_model ;;
  V6) echo "Default login = ChatGPT (mock user tuidefault, auth.json), NO OPENAI_API_KEY in env; TUI on Z.AI (real key), no account selection, /model -> OpenAI GPT-6 Luna (${BIN:-CAND})"
      XENV=("${XENV[@]/OPENAI_API_KEY=*/NO_OPENAI_KEY=1}")
      start_keyed_tui $SOCK $V provider/zai_api_key ZAI_API_KEY $H; sleep 14; to_openai_model ;;
  V5) echo "TUI started on Z.AI (real key), no account selection, then /model -> OpenAI GPT-6 Luna (default account; OPENAI_API_KEY decoy in env)"
      start_keyed_tui $SOCK $V provider/zai_api_key ZAI_API_KEY $H; sleep 14; to_openai_model ;;
esac
turn
send_line $V "/status"; sleep 3
cap $V -60 | grep -v '^\s*$' | sed -n '/Reply with exactly/,$p' | grep -E 'Reply with|mock:|Model provider|Account:|■|⚠|ERROR' | cut -c1-170
tx send-keys -t $V Escape; sleep 0.5; send_line $V "/exit"; sleep 3; tx kill-server
# /model persists the choice; put the home back on Z.AI for the next variant.
sed -i '' 's/^model = "gpt-6-luna"/model = "glm-5.3-flash"/; s/^model_provider = "openai"/model_provider = "zai"/' $H/config.toml
} >> "$OUT" 2>&1
