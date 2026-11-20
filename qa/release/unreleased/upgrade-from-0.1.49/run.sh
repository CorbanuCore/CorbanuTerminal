#!/usr/bin/env bash
# Run one phase of upgrade_from_release.py in an isolated container.
# Usage: run.sh <workdir> <phase>
# <workdir> holds old/ (released package: bin/, codex-resources/, codex-path/),
# new/ (candidate package, same layout) and upgrade_from_release.py.
# Image: ubuntu:24.04 with python3, tmux, ca-certificates, libssl3t64, libsqlite3-0.
set -euo pipefail
workdir="$(cd "$1" && pwd)"
phase="$2"
image="${UPGRADE_QA_IMAGE:-corbanu-astra-rel149-qa:latest}"
exec docker run --rm \
  --network none --cap-drop ALL --security-opt no-new-privileges \
  --memory 4g --pids-limit 256 \
  --user "$(id -u):$(id -g)" \
  -v "$workdir:/work" -w /work \
  "$image" python3 /work/upgrade_from_release.py "$phase"
