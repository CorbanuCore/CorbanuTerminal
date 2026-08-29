#!/bin/sh
set -eu

if [ "$#" -lt 2 ]; then
  echo "usage: run_with_isolated_home.sh CODEX_HOME COMMAND [ARG ...]" >&2
  exit 2
fi

isolated_home=$1
shift
mkdir -p "$isolated_home"
export CODEX_HOME="$isolated_home"
exec "$@"
