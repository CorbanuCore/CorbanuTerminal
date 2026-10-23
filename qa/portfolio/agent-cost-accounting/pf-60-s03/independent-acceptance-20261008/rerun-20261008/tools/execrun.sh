#!/bin/bash
# usage: execrun.sh <home> <cwd> <tag> <faketime|-> <prompt> [extra corbanu exec args]
# Prefix assignments go straight onto the (unsigned, non-SIP) binary: /usr/bin/env would strip DYLD_* on macOS.
# The key substitution is first, so the helper runs with the caller's real environment, not the disposable homes.
S=<scratch>
B=<corbanu-root>/.codex-work/targets/pf60-s03-indep-rerun-20261008/debug/corbanu
h=$1; cd "$2"; tag=$3; ft=$4; p=$5; shift 5
export RUST_LOG=codex_api=trace,warn
if [ "$ft" = "-" ]; then
  ZAI_API_KEY="$(~/.local/bin/corbanu vault auth-helper provider/zai_api_key)" CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h \
    $B exec --json --skip-git-repo-check -s read-only "$@" "$p" < /dev/null > $S/logs/$tag.jsonl 2> $S/logs/$tag.err
else
  ZAI_API_KEY="$(~/.local/bin/corbanu vault auth-helper provider/zai_api_key)" CORBANU_TEST_NO_NATIVE_KEYRING=1 CODEX_HOME=$h CORBANU_HOME=$h PFTERMINAL_HOME=$h \
    TZ=UTC FAKETIME_DONT_FAKE_MONOTONIC=1 FAKETIME="@$ft" DYLD_FORCE_FLAT_NAMESPACE=1 DYLD_INSERT_LIBRARIES=$S/libfaketime/src/libfaketime.1.dylib \
    $B exec --json --skip-git-repo-check -s read-only "$@" "$p" < /dev/null > $S/logs/$tag.jsonl 2> $S/logs/$tag.err
fi
echo "$tag exit=$? real_end=$(date -u +%T)"
