#!/usr/bin/env bash
# Exact-source synthetic admission qualification; invoke by real TMUX keys.
set -euo pipefail
repo=${PF27_PAIR_REPO:?exact checkout required}
out=${PF27_EVIDENCE_DIR:?new evidence directory required}
test ! -e "$out"
test -z "$(git -C "$repo" status --porcelain)"
# Retain every Stage A regression and its per-command exit receipts.
source "$repo/qa/security-levels/sprints/PF-27-S04/descriptor-pair-20260912/qualify-rtx.sh"
fixture="$repo/qa/security-levels/sprints/PF-27-S04/descriptor-admission-20260912/connector.c"
export PF27_PAIR_CONNECT="$out/connector"
test "$(sha256sum "$fixture" | cut -d ' ' -f1)" = a510a0787db823bed77d55cd9860284fecf2aefe7e60efc329c50fdf426b48a4
cc --version > "$out/compiler.txt"
printf '%s\n' 'cc -nostdlib -static-pie -fPIE -fcf-protection=none -fno-stack-protector -fno-builtin -Os -Wl,--build-id=none,-z,now -o connector connector.c' > "$out/fixture-command.txt"
run fixture-build cc -nostdlib -static-pie -fPIE -fcf-protection=none -fno-stack-protector -fno-builtin -Os -Wl,--build-id=none,-z,now -o "$PF27_PAIR_CONNECT" "$fixture"
sha256sum "$fixture" "$PF27_PAIR_CONNECT" > "$out/fixture-hashes.txt"
test "$(sha256sum "$PF27_PAIR_CONNECT" | cut -d ' ' -f1)" = 4b1e56b4bab25158d089197e5fe02c1523e9e52ad77075acba9c7ee96bb3fa17
readelf -hlWd "$PF27_PAIR_CONNECT" > "$out/fixture-elf.txt"
# Profile as data must pass before any connector invocation.
run connector-profile env PF27_ELF_STATIC="$PF27_PAIR_CONNECT" just test --locked -p codex-secret-broker-service --features synthetic-fixture --run-ignored only -E 'test(pf_27_s01_elf_profile_frozen_artifacts_as_data)' --retries 0
run admission just test --locked -p codex-secret-broker-service --features synthetic-fixture --run-ignored all -E 'test(pf_27_s01_admission_)' --retries 0
run strict-clippy cargo clippy --manifest-path codex-rs/Cargo.toml --locked --no-deps -p codex-secret-broker-service -p codex-linux-pidfd-spawn --features synthetic-fixture --tests -- -D warnings
sha256sum codex-rs/Cargo.lock MODULE.bazel.lock > "$out/locks-before.txt"
run bazel-parity bazel --output_base="${PF27_BAZEL_TARGET:?exclusive target required}" mod deps --lockfile_mode=error
sha256sum codex-rs/Cargo.lock MODULE.bazel.lock > "$out/locks-after.txt"
run unchanged-locks diff "$out/locks-before.txt" "$out/locks-after.txt"
run unchanged-source git diff --exit-code
test -z "$(git status --porcelain)"
printf 'PF27_ADMISSION_B_TMUX_COMPLETE\n'
