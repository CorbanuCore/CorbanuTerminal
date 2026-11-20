#!/usr/bin/env bash
# Run one phase of upgrade_from_release.py in an isolated container.
# Usage: run.sh <workdir> <phase>
# <workdir> holds old/ (released package: bin/, codex-resources/, codex-path/),
# new/ (candidate package, same layout) and upgrade_from_release.py.
# Phases, in order: baseline upgrade downgrade rollback summarize.
# Image: the Dockerfile beside this script (built on first use), or UPGRADE_QA_IMAGE.
set -euo pipefail
workdir="$(cd "$1" && pwd)"
phase="$2"
image="${UPGRADE_QA_IMAGE:-corbanu-upgrade-qa:ubuntu-24.04}"
if ! docker image inspect "$image" > /dev/null 2>&1; then
  docker build -q -t "$image" "$(dirname "$0")" > /dev/null
fi
exec docker run --rm \
  --network none --cap-drop ALL --security-opt no-new-privileges \
  --memory 4g --pids-limit 256 \
  --user "$(id -u):$(id -g)" \
  -v "$workdir:/work" -w /work \
  "$image" python3 /work/upgrade_from_release.py "$phase"
