#!/bin/bash
# remote side. usage: rtx-execrun.sh <home> <cwd> <tag> <faketime|-> <prompt> [extra args]; ZAI key read from stdin (never echoed, never written)
R=<rtx-dir>
h=$1; cd "$2"; tag=$3; ft=$4; p=$5; shift 5
IFS= read -r ZAI_API_KEY; export ZAI_API_KEY
pre=(); [ "$ft" != "-" ] && pre=(env TZ=UTC FAKETIME_DONT_FAKE_MONOTONIC=1 FAKETIME="@$ft" LD_PRELOAD=$R/libfaketime/src/libfaketime-time64.so.1)
"${pre[@]}" env RUST_LOG=codex_api=trace,warn CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h \
  ${BIN:-$R/corbanu-acct} exec --json --skip-git-repo-check -s read-only "$@" "$p" < /dev/null > $R/logs/$tag.jsonl 2> $R/logs/$tag.err
echo "$tag exit=$? real_end=$(date -u +%T)"
