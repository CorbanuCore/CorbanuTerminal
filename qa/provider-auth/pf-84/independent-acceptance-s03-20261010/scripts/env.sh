# Shared helpers. Every candidate/baseline run: env -i, fake HOME, disposable CORBANU_HOME,
# CORBANU_TEST_NO_NATIVE_KEYRING=1. Real keys only via the installed signed helper, inside the consuming command.
S=${S:-/Volumes/CorbanuDrive/Corbanu/tmp/pf84s03acc}
B=${B:-$S/bin/corbanu-main}
PRE=$S/bin/corbanu-pre
HELPER=/Users/Neo/.local/bin/corbanu.bak-20261010
PROMPT="Reply with exactly the word pong and nothing else."
helper() { env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME $HELPER vault auth-helper "$1"; }
# cx <home> args... : candidate in a clean env
cx() { local h=$1; shift; mkdir -p "$h"; (cd $S/work && env -i HOME=$S/fakehome PATH=/usr/bin:/bin TERM=xterm-256color CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME="$h" "${E[@]}" "$B" "$@"); }
# addreal <label> <home> <provider> <name>: named account from the operator vault (never printed)
addreal() { local label=$1 home=$2; shift 2
  X="$(helper "$label")" B="$B" S="$S" H="$home" bash -c 'printf %s "$X" | env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$H $B account add "$@"' _ "$@"; }
# addval <home> <value> <provider> <name> [args]: named account with a synthetic value
addval() { local home=$1 v=$2; shift 2; printf %s "$v" | env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$home $B account add "$@"; }
# summ: keep the interesting lines of an exec run
summ() { grep -v -E '^\s*$' | grep -E -i 'pong|error|401|unauthor|account|ignored|warning|refus|not configured|session id' | grep -v -E 'Reply with exactly|corbanu_call_metrics' | sort -u | head -8 | sed 's/^/   | /'; }
