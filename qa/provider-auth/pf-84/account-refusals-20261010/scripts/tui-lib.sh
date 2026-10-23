# Private tmux server; the pane runs a wrapper that fetches the Z.AI key itself (never on a command line).
. "$(dirname "${BASH_SOURCE[0]}")/live-env.sh"
SOCK=/tmp/pf84f2.sock
T() { tmux -S $SOCK "$@"; }
# start <name> <home> args... : start the binary in a detached 160x45 window
start() { local n=$1 h=$2; shift 2
  cat > $S/run-$n.sh <<W
#!/bin/bash
export ZAI_API_KEY="\$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper provider/zai_api_key)"
for v in \$(compgen -e); do case \$v in ZAI_API_KEY) ;; *) unset "\$v" ;; esac; done
export HOME=$S/fakehome PATH=/usr/bin:/bin TERM=xterm-256color CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$h
cd $S/work
$B $(printf '%q ' "$@")
echo "[process exited: \$?]"
sleep 600
W
  chmod +x $S/run-$n.sh
  T new-session -d -s $n -x 160 -y 45 $S/run-$n.sh; }
# snap <name>: wait until two captures 1s apart are identical, then print
snap() { local a b i; for i in $(seq 1 ${2:-40}); do a=$(T capture-pane -p -t $1); sleep 1; b=$(T capture-pane -p -t $1); [ "$a" = "$b" ] && [ -n "$(printf %s "$a" | tr -d '[:space:]')" ] && break; done; printf '%s\n' "$b" | sed -e :a -e '/^\n*$/{$d;N;ba' -e '}' | sed 's/^/   | /'; }
send() { T send-keys -t $1 -l "$2"; sleep 0.5; T send-keys -t $1 Enter; }
