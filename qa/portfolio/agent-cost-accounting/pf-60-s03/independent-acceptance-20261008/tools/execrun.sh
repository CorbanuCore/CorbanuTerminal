#!/bin/bash
# usage: execrun.sh <home> <cwd> <tag> <prompt> [extra args]
h=$1; cd "$2"; tag=$3; p=$4; shift 4
K="$(~/.local/bin/corbanu vault auth-helper provider/zai_api_key)"
ZAI_API_KEY="$K" RUST_LOG=codex_api=trace,warn CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h \
  <corbanu-root>/.codex-work/targets/pf60-s03-indep-acceptance-20261008/debug/corbanu exec --json --skip-git-repo-check -s read-only "$@" "$p" \
  > <scratch>/logs/$tag.jsonl 2> <scratch>/logs/$tag.err
echo "$tag exit=$? start=$(date -u +%H:%M:%S)"
