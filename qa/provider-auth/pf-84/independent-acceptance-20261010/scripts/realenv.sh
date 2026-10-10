S=/Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch; B=${B:-$S/bin/corbanu-main}
HELPER=/Users/Neo/.local/bin/corbanu.bak-20261010
# add a named account whose secret comes straight from the vault helper (never printed)
addreal() { label=$1 home=$2; shift 2
  X="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME $HELPER vault auth-helper "$label")" B="$B" S="$S" H="$home" bash -c 'printf %s "$X" | env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$H $B account add "$@"' _ "$@"; }
addfake() { home=$1; shift; printf 'fake-%s-%s' "$*" "$(openssl rand -hex 12)" | tr ' ' '-' | env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$home $B account add "$@"; }
# run exec in a clean env; extra env assignments via E=(...) array
rx() { home=$1; shift; (cd $S/fakehome && env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$home "${E[@]}" $B exec --skip-git-repo-check "$@" "Reply with exactly the word pong and nothing else."); }
summ() { grep -v -E '^\s*$' | grep -E -i 'pong|error|401|unauthor|account|ignored|warning: `|tokens used' | grep -v -E 'Reply with exactly' | sort -u | head -6 | sed 's/^/   | /'; }
