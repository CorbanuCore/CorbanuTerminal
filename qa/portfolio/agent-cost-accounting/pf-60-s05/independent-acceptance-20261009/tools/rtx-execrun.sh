#!/bin/bash
# remote side. usage: rtx-execrun.sh <bin> <home> <tag> <KEYVAR> <prompt> [extra args]; key read from stdin (never echoed, never written)
R=<rtx-scratch>
b=$1; h=$2; tag=$3; kv=$4; p=$5; shift 5
IFS= read -r KEYVAL
cd $R/work/repo
env "$kv=$KEYVAL" RUST_LOG="${RUST_LOG:-codex_api=trace,codex_core::accounting=debug,warn}" CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h \
  $R/$b exec --json --skip-git-repo-check ${SANDBOX_ARGS:--s read-only} "$@" "$p" < /dev/null > $R/logs/$tag.jsonl 2> $R/logs/$tag.err
echo "$tag exit=$? end=$(date -u +%FT%TZ)"
