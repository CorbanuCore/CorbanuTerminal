#!/bin/bash
# remote. usage: ephem.sh <home> <tag> [exec args...]: corbanu exec on Z.AI with the key from stdin (environment only)
R=$HOME/corbanu-rtx/pf60s04-ia; h=$1; tag=$2; shift 2
IFS= read -r KEYVAL
cd $R/work/ws
export CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h RUST_LOG="codex_api=trace,codex_core::accounting=debug,warn"
ZAI_API_KEY="$KEYVAL" $R/bin/corbanu exec --json --skip-git-repo-check -s read-only "$@" < /dev/null > $R/logs/$tag.jsonl 2> $R/logs/$tag.err
echo "$tag exit=$? end=$(date -u +%FT%TZ)"
