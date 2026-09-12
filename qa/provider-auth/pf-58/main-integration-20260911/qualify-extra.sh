#!/usr/bin/env bash
set -euo pipefail
repo=/home/travis/worktrees/pf13-main-release-20260911
evidence=/home/travis/security-round5/evidence/pf13-main-release-20260911
export PATH=/home/travis/security-round5/tools:/home/travis/security-round5/evidence/broker/tools-venv/bin:/home/travis/.cargo/bin:$PATH
export CARGO_TARGET_DIR=/home/travis/repos/CorbanuTerminal-harness/codex-rs/target
export CARGO_BUILD_JOBS=8 RUST_MIN_STACK=8388608 CORBANU_TEST_NO_NATIVE_KEYRING=1
export TMPDIR
TMPDIR=$(mktemp -d /home/travis/.pf13.XXXXXX)
cd "$repo"
exec 9>/home/travis/security-round5/locks/build.lock
flock 9
just test -p codex-cli --bin corbanu -E 'test(team_context) | test(helper)' --retries 0 --test-threads 4 > "$evidence/cli.log" 2>&1
python3 -m unittest discover -s scripts/codex_package -p 'test_*.py' > "$evidence/package-python.log" 2>&1
python3 -m unittest discover -s qa/provider-auth/pf-58 -p test_handoff_gate.py > "$evidence/gate-python.log" 2>&1
printf '%s EXTRA COMPLETE\n' "$(date -Is)"
