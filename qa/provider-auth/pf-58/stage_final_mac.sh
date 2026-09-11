#!/usr/bin/env bash
set -euo pipefail
task_runtime=/Volumes/CorbanuDrive/Corbanu/.codex-work/provider-reauth-health
task_package=$task_runtime/macos-candidate-final9-20260910
task_build=$task_runtime/macos-target/release
test ! -e "$task_package"
cp -a "$task_runtime/macos-candidate-blind-repairs-20260910" "$task_package"
for task_binary in corbanu codex-code-mode-host corbanu-walletd corbanu-acp; do
  install -m755 "$task_build/$task_binary" "$task_package/bin/$task_binary"
  strip -S "$task_package/bin/$task_binary"
  codesign --force --sign DB43D92853B5EF2BCE87DA9B2F28360DC44BBEBD \
    --identifier "com.corbanu.$task_binary" --timestamp "$task_package/bin/$task_binary"
  codesign --verify --strict --verbose=2 "$task_package/bin/$task_binary"
done
shasum -a256 "$task_package"/bin/*
"$task_package/bin/corbanu" --version
