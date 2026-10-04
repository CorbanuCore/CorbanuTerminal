#!/usr/bin/env bash
# Exact-source, synthetic-only proof; send this command and Enter separately.
set -euo pipefail
source "${PF27_PAIR_REPO:?}/qa/security-levels/sprints/PF-27-S04/descriptor-root-compat-20260912/qualify-rtx.sh"
fixture="$repo/qa/security-levels/sprints/PF-27-S04/descriptor-root-dispatch-20260912/relay.c"
export PF27_DISPATCH_RELAY="$out/dispatch-relay"
test "$(sha256sum "$fixture" | cut -d ' ' -f1)" = 5cbe518e919f36766347df29cd958c9fef71846b614fc781d48557b5745cd08c
run dispatch-relay-build cc -nostdlib -static-pie -fPIE -fcf-protection=none -fno-stack-protector -fno-builtin -Os -Wl,--build-id=none,-z,now -o "$PF27_DISPATCH_RELAY" "$fixture"
sha256sum "$fixture" "$PF27_DISPATCH_RELAY" > "$out/dispatch-relay-hashes.txt"
test "$(sha256sum "$PF27_DISPATCH_RELAY" | cut -d ' ' -f1)" = 36f1fa188f3e95faa5643ec20e6640504ce44a2cfb3093e7aa0f205ec3a32867
readelf -hlWd "$PF27_DISPATCH_RELAY" > "$out/dispatch-relay-elf.txt"
run dispatch-relay-profile env PF27_ELF_STATIC="$PF27_DISPATCH_RELAY" just test --locked -p codex-secret-broker-service --features synthetic-fixture --run-ignored only -E 'test(pf_27_s01_elf_profile_frozen_artifacts_as_data)' --retries 0
run root-dispatch just test --locked -p codex-secret-broker-service --features synthetic-fixture --run-ignored only -E 'test(pf27_root_dispatch_)' --retries 0
sha256sum codex-rs/Cargo.lock MODULE.bazel.lock > "$out/dispatch-locks-after.txt"
diff "$out/locks-before.txt" "$out/dispatch-locks-after.txt"
git diff --exit-code
test -z "$(git status --porcelain)"
printf 'PF27_ROOT_DISPATCH_TMUX_COMPLETE\n'
