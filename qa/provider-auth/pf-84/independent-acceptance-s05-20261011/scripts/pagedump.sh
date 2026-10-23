# Source after tmuxlib.sh. pagedump <session> <out> [max_downs]: press Down repeatedly,
# recording each new selected line ("›") and the viewport, until the selection stops moving.
pagedump() {
  local s=$1 out=$2 max=${3:-80} prev="" cur i
  : > "$out"
  for ((i = 0; i < max; i++)); do
    cur=$(cap "$s" -100 | grep -E '^› ' | head -1)
    [ "$cur" = "$prev" ] && [ $i -gt 0 ] && break
    echo "[$i] $cur" >> "$out"
    prev=$cur; tx send-keys -t "$s" Down; sleep 0.35
  done
  echo "--- final viewport ---" >> "$out"; cap "$s" -100 | grep -v '^\s*$' >> "$out"
}
# select_match <session> <regex> [max]: press Down until the selected line matches, then Enter.
select_match() {
  local s=$1 re=$2 max=${3:-80} i
  for ((i = 0; i < max; i++)); do
    cap "$s" -100 | grep -E '^› ' | head -1 | grep -Eq -- "$re" && { tx send-keys -t "$s" Enter; sleep 3; return 0; }
    tx send-keys -t "$s" Down; sleep 0.2
  done
  echo "select_match: /$re/ not found" >&2; return 1
}
