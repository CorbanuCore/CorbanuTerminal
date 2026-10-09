#!/bin/bash
# remote side. usage: execrun.sh <home> <tag> <KEYVAR> <prompt> [extra args]; key read from stdin (never echoed or written)
R=$HOME/pf60-s05-rerun
h=$1; tag=$2; kv=$3; p=$4; shift 4
IFS= read -r KEYVAL
cd $R/work/repo
export PATH="$R/pathbin:$PATH" RUST_LOG="${RUST_LOG:-codex_api=trace,tungstenite::protocol=trace,codex_core::accounting=debug,warn}" CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h
case "$kv" in
  CLAUDE_CODE_OAUTH_TOKEN) CLAUDE_CODE_OAUTH_TOKEN="$KEYVAL" $R/corbanu-acct exec --json --skip-git-repo-check -s read-only "$@" "$p" < /dev/null > $R/logs/$tag.jsonl 2> $R/logs/$tag.err ;;
  KIMI_API_KEY) KIMI_API_KEY="$KEYVAL" $R/corbanu-acct exec --json --skip-git-repo-check -s read-only "$@" "$p" < /dev/null > $R/logs/$tag.jsonl 2> $R/logs/$tag.err ;;
  CODEX_API_KEY) CODEX_API_KEY="$KEYVAL" $R/corbanu-acct exec --json --skip-git-repo-check -s read-only "$@" "$p" < /dev/null > $R/logs/$tag.jsonl 2> $R/logs/$tag.err ;;
esac
echo "$tag exit=$? end=$(date -u +%FT%TZ)"
