#!/bin/bash
# usage: execrun.sh <home> <tag> <KEYVAR> <vault-label|placeholder|-> <prompt> [extra corbanu exec args]
# Runs the built binary (CORBANU_BIN, default corbanu-acct) in a disposable home from <scratch>/work/repo.
# The vault substitution sits on the consuming command (env ... $B exec); the value is never printed or written.
S=<scratch>
B=$S/bin/${CORBANU_BIN:-corbanu-acct}
h=$1; tag=$2; kv=$3; lab=$4; p=$5; shift 5
cd "${RUN_CWD:-$S/work/repo}"
export RUST_LOG="${RUST_LOG:-codex_api=trace,codex_core::accounting=debug,warn}"
common=(CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h)
if [ "$lab" = "-" ]; then
  env "${common[@]}" $B exec --json --skip-git-repo-check ${SANDBOX_ARGS:--s read-only} "$@" "$p" < /dev/null > $S/logs/$tag.jsonl 2> $S/logs/$tag.err
elif [ "$lab" = "placeholder" ]; then
  env "$kv=placeholder-not-a-real-key" "${common[@]}" $B exec --json --skip-git-repo-check ${SANDBOX_ARGS:--s read-only} "$@" "$p" < /dev/null > $S/logs/$tag.jsonl 2> $S/logs/$tag.err
else
  env "$kv=$(~/.local/bin/corbanu vault auth-helper "$lab")" "${common[@]}" $B exec --json --skip-git-repo-check ${SANDBOX_ARGS:--s read-only} "$@" "$p" < /dev/null > $S/logs/$tag.jsonl 2> $S/logs/$tag.err
fi
echo "$tag exit=$? end=$(date -u +%FT%TZ)"
