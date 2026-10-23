# PF-84-S05 live-run helpers. The debug candidate runs with an emptied environment, a fake HOME,
# a disposable CORBANU_HOME and CORBANU_TEST_NO_NATIVE_KEYRING=1. Real keys come only from the
# installed helper, inside the consuming command; no key value is on a command line or screen.
S=${S:-/Volumes/CorbanuDrive/Corbanu/tmp/pf84s05}
B=${B:-$S/bin/corbanu}
H=$S/home
SOCK=$S/tmux.sock
T() { tmux -S "$SOCK" "$@"; }
helper() { env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper "$1"; }
# cx args...: the candidate with only the variables named in KEEP passed through.
cx() { ( cd "$S/work" || exit 1
  for v in $(compgen -e); do case " ${KEEP[*]:-} " in *" $v "*) ;; *) unset "$v" ;; esac; done
  export HOME=$S/fakehome PATH=/usr/bin:/bin TERM=xterm-256color CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME="$H"
  exec "$B" "$@" ); }
# addreal <label> <provider> <name>: enrol a real vault key as a named account (stdin only).
addreal() { X="$(helper "$1")" B="$B" S="$S" H="$H" bash -c 'printf %s "$X" | env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$H $B account add "$@"' _ "$2" "$3"; }
addval() { printf %s "$1" | env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$H $B account add "$2" "$3"; }
# start <session> [args...]: the TUI on the default Z.AI account; the wrapper fetches the key itself.
start() { local n=$1; shift; cat > "$S/run-$n.sh" <<W
#!/bin/bash
export ZAI_API_KEY="\$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper provider/zai_api_key)"
for v in \$(compgen -e); do case \$v in ZAI_API_KEY) ;; *) unset "\$v" ;; esac; done
export HOME=$S/fakehome PATH=/usr/bin:/bin TERM=xterm-256color CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$H
cd $S/work
$B $(printf '%q ' "$@") -m glm-5.3-flash -c model_provider=zai
echo "[process exited: \$?]"
sleep 600
W
  chmod +x "$S/run-$n.sh"; T new-session -d -s "$n" -x "${COLS:-160}" -y "${ROWS:-50}" "$S/run-$n.sh"; }
snap() { local a b i; for i in $(seq 1 "${2:-60}"); do a=$(T capture-pane -p -t "$1"); sleep 1; b=$(T capture-pane -p -t "$1"); [ "$a" = "$b" ] && [ -n "$(printf %s "$a" | tr -d '[:space:]')" ] && break; done; printf '%s\n' "$b"; }
send() { T send-keys -t "$1" -l "$2"; sleep 0.5; T send-keys -t "$1" Enter; }
# waitfor <session> <regex> [seconds]
waitfor() { local i; for i in $(seq 1 "${3:-180}"); do T capture-pane -p -J -S -200 -t "$1" | grep -Eq "$2" && return 0; sleep 1; done; echo "TIMEOUT waiting for $2"; return 1; }
