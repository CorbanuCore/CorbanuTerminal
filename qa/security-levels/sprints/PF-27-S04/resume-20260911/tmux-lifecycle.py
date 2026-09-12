#!/usr/bin/env python3
"""Supporting PTY proof for the non-interactive, synthetic broker service stage."""

import json
from pathlib import Path
import shlex
import subprocess
import tempfile
import time


repo = Path('/home/travis/worktrees/security-broker-resume-20260911')
evidence = Path('/home/travis/security-round5/evidence/pf27-resume-20260911')
candidate = evidence / 'candidate/codex-secret-broker-service'
if not candidate.is_file():
    raise SystemExit('Run qualify-rtx.sh to completion before this TMUX check')
with tempfile.TemporaryDirectory(prefix='.pf27t-', dir='/home/travis') as temp:
    socket = str(Path(temp) / 't')
    keys = []

    def tmux(*args, check=True):
        return subprocess.run(['tmux', '-S', socket, *args], check=check,
                              capture_output=True, text=True)

    def command(text):
        keys.append({'text': text, 'submit': 'Enter separately'})
        tmux('send-keys', '-t', 'service:0.0', '-l', text)
        tmux('send-keys', '-t', 'service:0.0', 'Enter')

    def wait_for(text):
        deadline = time.monotonic() + 180
        while time.monotonic() < deadline:
            pane = tmux('capture-pane', '-p', '-S', '-', '-t', 'service:0.0').stdout
            if any(line.strip() == text for line in pane.splitlines()):
                return pane
            time.sleep(.2)
        raise RuntimeError(f'Expected completion marker was not emitted: {text}')

    tmux('new-session', '-d', '-s', 'service', '-x', '150', '-y', '50',
         '-c', str(repo), 'bash --noprofile --norc')
    try:
        command(f'{shlex.quote(str(candidate))} --synthetic-inherited-socket; '
                'printf "\\nproduction-exit=%s\\n" "$?"')
        denial = wait_for('production-exit=78')
        assert 'qualified OS bootstrap required' in denial
        (evidence / 'tmux-default-denial.txt').write_text(denial)
        env = (
            'export PATH=/home/travis/security-round5/tools:/home/travis/.cargo/bin:$PATH; '
            'export CARGO_TARGET_DIR=/home/travis/repos/CorbanuTerminal-harness/codex-rs/target; '
            'export CARGO_BUILD_JOBS=8 RUST_MIN_STACK=8388608 CORBANU_TEST_NO_NATIVE_KEYRING=1; '
            f'export TMPDIR={shlex.quote(temp)}; '
        )
        command(env + 'flock /home/travis/security-round5/locks/build.lock '
                'just test -p codex-secret-broker-service --features synthetic-fixture '
                '--retries 0 --test-threads 4; printf "\\nsynthetic-exit=%s\\n" "$?"')
        lifecycle = wait_for('synthetic-exit=0')
        assert '6 tests run: 6 passed' in lifecycle
        (evidence / 'tmux-synthetic-lifecycle.txt').write_text(lifecycle)
        (evidence / 'tmux-keys.json').write_text(json.dumps(keys, indent=2) + '\n')
        (evidence / 'tmux-result.json').write_text(json.dumps({
            'passed': True, 'production_exit': 78, 'synthetic_subprocess_tests': 6,
            'candidate': str(candidate), 'user_facing_tui': False,
            'native_isolation_proven': False, 'privileged_installation': False,
        }, indent=2) + '\n')
    finally:
        tmux('kill-server', check=False)
print('PF27 TMUX lifecycle passed')
