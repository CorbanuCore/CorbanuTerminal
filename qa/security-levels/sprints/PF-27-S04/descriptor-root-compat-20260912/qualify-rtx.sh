#!/usr/bin/env bash
# Final clean-tree synthetic compatibility proof, invoked via real TMUX keys.
set -euo pipefail
test -d "${TMPDIR:?explicit ext-family/XFS temporary directory required}"
# Retain the complete descriptor-admission and predecessor proof unchanged.
source "${PF27_PAIR_REPO:?}/qa/security-levels/sprints/PF-27-S04/descriptor-admission-20260912/qualify-rtx.sh"
fixture="$repo/qa/security-levels/sprints/PF-27-S04/descriptor-root-compat-20260912/relay.c"
export PF27_ROOT_RELAY="$out/root-relay"
test "$(sha256sum "$fixture" | cut -d ' ' -f1)" = 49d3888ebe9780e3321e996d73a7dd6ec01082f576ebc07d613cdcd8c321c28b
run relay-build cc -nostdlib -static-pie -fPIE -fcf-protection=none -fno-stack-protector -fno-builtin -Os -Wl,--build-id=none,-z,now -o "$PF27_ROOT_RELAY" "$fixture"
sha256sum "$fixture" "$PF27_ROOT_RELAY" > "$out/relay-hashes.txt"
test "$(sha256sum "$PF27_ROOT_RELAY" | cut -d ' ' -f1)" = ac3f6dd19d64c504e5be9bd0cb0cd190ef62325c34a252ab49f597b23eaa8e5c
readelf -hlWd "$PF27_ROOT_RELAY" > "$out/relay-elf.txt"
run relay-profile env PF27_ELF_STATIC="$PF27_ROOT_RELAY" just test --locked -p codex-secret-broker-service --features synthetic-fixture --run-ignored only -E 'test(pf_27_s01_elf_profile_frozen_artifacts_as_data)' --retries 0
run protected-default just test --locked -p codex-protected-state --retries 0
run protected-feature just test --locked -p codex-protected-state --features synthetic-fixture --retries 0
run root-compat just test --locked -p codex-protected-state --features synthetic-fixture --run-ignored all -E 'test(pf27_root_compat_)' --retries 0
run compatibility-clippy cargo clippy --manifest-path codex-rs/Cargo.toml --locked --no-deps -p codex-protected-state -p codex-linux-pidfd-spawn --features synthetic-fixture --tests -- -D warnings
sha256sum codex-rs/Cargo.lock MODULE.bazel.lock > "$out/compatibility-locks-after.txt"
run compatibility-locks diff "$out/locks-before.txt" "$out/compatibility-locks-after.txt"
run compatibility-source git diff --exit-code
test -z "$(git status --porcelain)"
printf 'PF27_ROOT_COMPAT_TMUX_COMPLETE\n'
