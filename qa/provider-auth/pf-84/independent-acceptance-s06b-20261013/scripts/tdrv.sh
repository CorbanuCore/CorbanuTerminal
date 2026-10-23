# source tdrv.sh <socket-name>: driving context for the slice-B TUI runs on home t1
D=/Volumes/CorbanuDrive/Corbanu/worktrees/s06b-acceptance-20261013/qa/provider-auth/pf-84/independent-acceptance-s06b-20261013/scripts
source $D/env.sh; source $D/tmuxlib.sh; source $D/aws_setup.sh; source $D/tui_setup.sh; source $D/drv.sh $1 > /dev/null
export FH=$S/fh-t1 H=$S/homes/t1
MT=$S/logs/mock-t.jsonl
tz() { local s=$1; shift; export SOCK=$S/tmux-$s.sock; start_keyed_tui $SOCK $s provider/zai_api_key ZAI_API_KEY $H "$@"; }
vis() { screen $1 | sed "s#$S#\$S#g" > "$EV/captures/$2.txt"; echo "saved $2 (visible)"; }
# rows <session> [max]: walk a list popup with Down, print each selected row once (stops when it wraps or sticks)
rows() { local s=$1 max=${2:-40} i cur first="" prev=""
  for ((i = 0; i < max; i++)); do cur=$(screen $s | grep -E '^› +[0-9]+\. ' | tail -1)
    [ -z "$first" ] && first=$cur || { [ "$cur" = "$first" ] || [ "$cur" = "$prev" ]; } && [ $i -gt 0 ] && break
    echo "$cur"; prev=$cur; tx send-keys -t $s Down; sleep 0.3; done; }
# sel <session> <regex> [max]: Down until the selected popup row matches, then Enter
sel() { local s=$1 re=$2 max=${3:-60} i; for ((i = 0; i < max; i++)); do
    screen $s | grep -E '^› +[0-9]+\. ' | tail -1 | grep -Eq -- "$re" && { tx send-keys -t $s Enter; sleep 2; return 0; }
    tx send-keys -t $s Down; sleep 0.25; done; echo "sel: /$re/ not found" >&2; return 1; }
# /model persists the choice: put t1 back on Z.AI GLM before each run
reset_t1() { sed -i '' 's/^model = .*/model = "glm-5.3-flash"/; s/^model_provider = .*/model_provider = "zai"/; /^model_reasoning_effort = /d' $H/config.toml; }
# to_openai <session> <model-regex>: /model -> OpenAI tab -> model -> default effort
to_model() { local s=$1 tab=$2 re=$3; send_line $s "/model"; sleep 4
  for i in 1 2 3; do screen $s | grep -q "\[$tab\]" && break; tx send-keys -t $s Left; sleep 1.2; done
  sel $s "$re"; sleep 1.5; screen $s | grep -q 'Select Reasoning' && { tx send-keys -t $s Enter; sleep 3; }; }
