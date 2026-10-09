#!/bin/bash
# usage: execrun.sh <home> <tag> <KEYVAR> <vault-label|-> <prompt> [extra corbanu exec args]
# Runs the built binary in a disposable home from work/repo. The key is resolved by the INSTALLED wrapper without the
# test env vars (V) and substituted only on the consuming command; never printed or written.
S=<scratch>
B=$S/bin/${CORBANU_BIN:-corbanu-acct}
V(){ env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper "$1" < /dev/null; }
h=$1; tag=$2; kv=$3; lab=$4; p=$5; shift 5
mkdir -p "$h"; cd "${RUN_CWD:-$S/work/repo}"
export RUST_LOG="${RUST_LOG:-codex_api=trace,codex_core::accounting=debug,warn}"
export CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h
run(){ "$B" exec --json --skip-git-repo-check ${SANDBOX_ARGS:--s read-only} "$@" "$p" < /dev/null > $S/logs/$tag.jsonl 2> $S/logs/$tag.err; }
case "$kv" in
  KIMI_API_KEY) KIMI_API_KEY="$(V "$lab")" run "$@" ;;
  CODEX_API_KEY) CODEX_API_KEY="$(V "$lab")" run "$@" ;;
  OPENAI_API_KEY) OPENAI_API_KEY="$(V "$lab")" run "$@" ;;
  CLAUDE_CODE_OAUTH_TOKEN) CLAUDE_CODE_OAUTH_TOKEN="$(V "$lab")" run "$@" ;;
  QA_KEY) QA_KEY="$(V "$lab")" run "$@" ;;
  -) run "$@" ;;
  *) echo "unknown keyvar"; exit 2 ;;
esac
echo "$tag exit=$? end=$(date -u +%FT%TZ)"
