#!/usr/bin/env bash
set -euo pipefail
repo=/home/travis/worktrees/security-broker-adapter-20260912
out=${PF27_EVIDENCE_DIR:?fresh evidence directory required}
mkdir -p "$out"
export PATH=/home/travis/security-round5/tools:/home/travis/security-round5/evidence/broker/tools-venv/bin:/home/travis/.cargo/bin:$PATH
export CARGO_TARGET_DIR=/home/travis/repos/CorbanuTerminal-harness/codex-rs/target CARGO_BUILD_JOBS=4
export TMPDIR
TMPDIR=$(mktemp -d /home/travis/.pf27adapter.XXXXXX)
export PF27_SYNTHETIC_HOLD="$out/hold" PF27_SYNTHETIC_INSPECT="$out/inspect"
export PF27_SYNTHETIC_PROBE=/home/travis/security-round5/evidence/pf27-static-probe-20260912/linkage-retry/candidate/codex-protected-root-probe
cd "$repo"
exec 9>/home/travis/security-round5/locks/build.lock
flock 9
run() {
    local label=$1
    shift
    if "$@" > "$out/$label.log" 2>&1; then
        printf '0\n' > "$out/$label.exit"
    else
        local result=$?
        printf '%s\n' "$result" > "$out/$label.exit"
        tail -50 "$out/$label.log"
        return "$result"
    fi
}
git rev-parse HEAD HEAD:codex-rs > "$out/source.txt"
uname -a > "$out/platform.txt"
getconf GNU_LIBC_VERSION >> "$out/platform.txt"
dpkg-query -W libc6 >> "$out/platform.txt"
sha256sum /lib/x86_64-linux-gnu/libc.so.6 "$PF27_SYNTHETIC_PROBE" > "$out/artifacts.sha256"
test "$(id -u)" != 0
test "$(sha256sum "$PF27_SYNTHETIC_PROBE" | cut -d ' ' -f1)" = f6ca8e3368dcbc8e5ba92bbbc7860ee825628b9470ac53116c6c3f6996bef3f0
run fix just fix -p codex-linux-pidfd-spawn --features synthetic-fixture
run fmt just fmt
git diff --stat > "$out/post-format-stat.txt"
run lint just clippy -p codex-linux-pidfd-spawn --features synthetic-fixture --no-deps
for fixture in hold inspect; do
    run "build-$fixture" env RUSTUP_HOME=/home/travis/security-round5/evidence/pf27-static-probe-20260912/rustup rustc +1.95.0 --target x86_64-unknown-linux-musl -C target-feature=+crt-static -C relocation-model=pie -C linker-flavor=ld.lld -C linker=/home/travis/security-round5/evidence/pf27-static-probe-20260912/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld "qa/security-levels/sprints/PF-27-S04/descriptor-launch-20260912/fixtures/$fixture.rs" -o "$out/$fixture"
    readelf -hlWd "$out/$fixture" > "$out/$fixture-elf.txt"
    if grep -Eq 'INTERP|NEEDED' "$out/$fixture-elf.txt"; then exit 1; fi
    sha256sum "$out/$fixture" >> "$out/artifacts.sha256"
    run "profile-$fixture" env PF27_ELF_STATIC="$out/$fixture" PF27_ELF_GNU=/home/travis/security-round5/evidence/pf27-sealed-20260912/verified/candidate/codex-protected-root-probe PF27_ELF_INTERPRETER=/home/travis/security-round5/evidence/pf27-static-probe-20260912/uapi-retry/candidate/codex-protected-root-probe just test -p codex-secret-broker-service --features synthetic-fixture --run-ignored only -E 'test(pf_27_s01_elf_profile_frozen_artifacts_as_data)' --retries 0
done
run default just test -p codex-linux-pidfd-spawn --retries 0 --no-tests warn
run focused just test -p codex-linux-pidfd-spawn --features synthetic-fixture --retries 0 --test-threads 1
run os just test -p codex-linux-pidfd-spawn --features synthetic-fixture --run-ignored only -E 'not test(pf27_unqualified_libc_fails_closed)' --retries 0 --test-threads 1
run service just test -p codex-secret-broker-service --features synthetic-fixture --retries 0 --test-threads 4
run archive cargo nextest archive --manifest-path codex-rs/Cargo.toml -p codex-linux-pidfd-spawn --features synthetic-fixture --archive-file "$out/adapter-tests.tar.zst"
printf 'COMPLETE\n'
