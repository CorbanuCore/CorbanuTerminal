# PF-84-S04 acceptance helpers. Source with `. lib.sh`.
# Every candidate/baseline run: env -i, fake HOME, disposable CORBANU_HOME, CORBANU_TEST_NO_NATIVE_KEYRING=1.
# Real keys only via the installed signed helper, inside the consuming command; never printed.
# TUI driving follows docs/tmuxHarness.md: private socket, text and Enter sent separately, a wait needs
# the pattern plus two identical captures, bounded captures, sessions end with /exit.
S=${S:-/Volumes/CorbanuDrive/Corbanu/tmp/pf84s04acc}
B=${B:-$S/bin/corbanu-main}
HELPER=/Users/Neo/.local/bin/corbanu.bak-20261010
T="tmux -L pf84s04acc"
PROMPT="Reply with exactly the word pong and nothing else."
helper() { env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME $HELPER vault auth-helper "$1"; }
# cx <home> args... : candidate CLI in a clean env
cx() { local h=$1; shift; mkdir -p "$h"; (cd $S/work && env -i HOME=$S/fakehome PATH=$S/pathbin:/usr/bin:/bin TERM=xterm-256color CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME="$h" ${CX_ENV:-} "$B" "$@"); }
# canary <id>: synthetic per-account value (stored in $S/proxy/canaries.json)
canary() { python3 -c "import json;d=json.load(open('$S/proxy/canaries.json'));print(next(k for k,v in d.items() if v=='$1'))"; }
cap() { $T capture-pane -p -t "$1" -S -${2:-400}; }
capv() { $T capture-pane -p -t "$1"; }
wait_for() { # session pattern [timeout_s]
  local s=$1 pat=$2 to=${3:-60} prev="" cur i=0
  while [ $i -lt $((to*2)) ]; do cur=$(capv "$s"); if printf '%s' "$cur" | grep -q -E -- "$pat" && [ "$cur" = "$prev" ]; then return 0; fi; prev=$cur; sleep 0.5; i=$((i+1)); done
  echo "TIMEOUT waiting for /$pat/ in $s"; return 1; }
settle() { local s=$1 prev="" cur i=0; while [ $i -lt 8 ]; do cur=$(capv "$s"); [ "$cur" = "$prev" ] && return 0; prev=$cur; sleep 0.5; i=$((i+1)); done; }
send_text() { $T send-keys -t "$1" -l -- "$2"; }
key() { local s=$1; shift; for k in "$@"; do $T send-keys -t "$s" "$k"; sleep 0.4; done; settle "$s"; }
type_and_enter() { send_text "$1" "$2"; wait_for "$1" "$(printf '%s' "$2" | cut -c1-20 | sed 's/[][\.*^$(){}|+?]/./g')" 15; $T send-keys -t "$1" Enter; settle "$1"; }
# paste_secret <session> <vault-label>: real key into the pane via a tmux buffer deleted on paste (never printed)
paste_secret() { X="$(helper "$2")" bash -c 'printf %s "$X" | '"$T"' load-buffer -b sec -'; $T paste-buffer -p -d -b sec -t "$1"; settle "$1"; }
# paste_value <session> <value>: synthetic value through a tmux buffer deleted on paste
paste_value() { printf %s "$2" | $T load-buffer -b sec -; $T paste-buffer -p -d -b sec -t "$1"; settle "$1"; }
# tui_start <session> <home> [args...]: start a TUI (binary $B) in a clean env; extra env in TUI_ENV
tui_start() { local s=$1 home=$2; shift 2; local args="" a; for a in "$@"; do args="$args '$a'"; done; mkdir -p $S/logs/$s "$home"
  $T new-session -d -s "$s" -x 160 -y 50 "cd $S/work && env -i HOME=$S/fakehome PATH=$S/pathbin:/usr/bin:/bin TERM=xterm-256color CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$home ${TUI_ENV:-} $B -c log_dir='\"$S/logs/$s\"' --no-alt-screen $args; echo TUI_EXITED rc=\$?; sleep 9999"
  sleep 3; settle "$s"; }
snap() { capv "$1" > $S/caps/$2.txt; echo "--- $2"; grep -v '^\s*$' $S/caps/$2.txt; }
tui_stop() { local s=$1; type_and_enter "$s" "/exit"; wait_for "$s" 'TUI_EXITED' 30; cap "$s" 3000 > $S/caps/scroll-$s.txt; $T kill-session -t "$s"; }
# select_row <session> <regex> [key]: move the highlight (Down by default) until the '>'/'›' line matches
select_row() { local s=$1 re=$2 k=${3:-Down} i=0; while [ $i -lt 60 ]; do capv "$s" | grep -E '^\s*[>›]' | grep -q -E -- "$re" && { settle "$s"; return 0; }; $T send-keys -t "$s" "$k"; sleep 0.25; i=$((i+1)); done; echo "NO ROW /$re/"; return 1; }
# list_rows <session> [n]: walk Down n times and print every distinct numbered row of a list popup
list_rows() { local s=$1 n=${2:-30} i; for i in $(seq 1 $n); do capv "$s" | grep -E '^\s*[>›]?\s*[0-9]+\. '; $T send-keys -t "$s" Down; sleep 0.2; done | sed -E 's/^\s*[>›]?\s*//' | sort -u -t. -k1,1n; }
add_acct () 
{ 
    local s=$1 row=$2 name=$3 val=$4;
    type_and_enter $s "/providers";
    select_row $s "$row" || return 1;
    key $s Enter;
    select_row $s 'Add another account' || return 1;
    key $s Enter;
    send_text $s "$name";
    settle $s;
    key $s Enter;
    capv $s | grep -q 'masked' || { 
        echo "no masked prompt";
        capv $s | tail -5;
        return 1
    };
    paste_value $s "$val";
    key $s Enter;
    sleep 2;
    cap $s 40 | grep -E '^[•■⚠]' | tail -1;
    key $s Escape
}
# acct_action <session> <row regex> <action regex>: open /providers, pick the row and the action
acct_action() { local s=$1; type_and_enter $s "/providers"; select_row $s "$2" || return 1; key $s Enter; select_row $s "$3" || return 1; key $s Enter; sleep 2; }
# ask <session>: send the pong prompt and wait for the reply or an error
ask() { type_and_enter $1 "$PROMPT"; sleep 3; wait_for $1 '• pong|rror|401|rejected' 90; }
lastreq() { tail -${1:-1} $S/proxy/requests.jsonl | python3 -c 'import sys,json;[print(r["bearer"],r["status"],r["canary_values_in_body"],r["real_key_in_body"]) for r in map(json.loads,sys.stdin)]'; }
ob_add () 
{ 
    local s=$1 row=$2 name=$3;
    select_row $s "$row";
    key $s Enter;
    select_row $s 'Add another account';
    key $s Enter;
    send_text $s $name;
    settle $s;
    key $s Enter
}
hashm () 
{ 
    ( cd $S/homes/hM && shasum config.toml secrets/local.age secrets/keyring-fallback/* provider-eligibility.json 2> /dev/null )
}
