# Live-check helpers. Debug binaries run with env -i, a fake HOME, a disposable CORBANU_HOME and
# CORBANU_TEST_NO_NATIVE_KEYRING=1. Real keys come only from the installed helper, inside the consumer.
S=/Volumes/CorbanuDrive/Corbanu/tmp/pf84fix2
B=${B:-$S/bin/corbanu-after}
PROMPT="Reply with exactly the word pong and nothing else."
helper() { env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper "$1"; }
# cx <home> args...: the debug binary with an emptied environment; only the variables named in KEEP
# (provider keys exported by the caller) pass through, so no key value is ever on a command line.
cx() { local h=$1 bin=$B dir=$S; shift; ( cd $S/work || exit 1
  for v in $(compgen -e); do case " ${KEEP[*]:-} " in *" $v "*) ;; *) unset "$v" ;; esac; done
  export HOME=$dir/fakehome PATH=/usr/bin:/bin TERM=xterm-256color CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME="$h"
  exec "$bin" "$@" ); }
addreal() { local label=$1 home=$2; shift 2
  X="$(helper "$label")" B="$B" S="$S" H="$home" bash -c 'printf %s "$X" | env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$H $B account add "$@"' _ "$@"; }
addval() { local home=$1 v=$2; shift 2; printf %s "$v" | env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$home $B account add "$@"; }
summ() { grep -v -E '^\s*$' | grep -E -i 'pong|child|error|401|unauthor|account|ignored|refus|not configured|missing environment|session id|spawn' | grep -v -E 'Reply with exactly|corbanu_call_metrics' | sort -u | head -10 | sed 's/^/   | /'; }
