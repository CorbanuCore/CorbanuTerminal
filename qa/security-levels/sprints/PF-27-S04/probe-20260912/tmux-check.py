#!/usr/bin/env python3
"""Actual-key PTY support proof; not native containment or user-facing TUI QA."""
import json
import os
from pathlib import Path
import shlex
import subprocess
import tempfile
import time

repo = Path('/home/travis/worktrees/security-broker-probe-20260912')
evidence = Path(os.environ.get('PF27_EVIDENCE_DIR', '/home/travis/security-round5/evidence/pf27-probe-20260912/verified'))
candidate = evidence / 'candidate/codex-secret-broker-service'
if not candidate.is_file():
    raise SystemExit('Complete qualify-rtx.sh before TMUX checks')

with tempfile.TemporaryDirectory(prefix='.pf27c-', dir='/home/travis') as temp:
    socket = str(Path(temp) / 't')
    keys = []

    def tmux(*args, check=True):
        return subprocess.run(['tmux', '-S', socket, *args], check=check,
                              capture_output=True, text=True)

    def command(value):
        keys.append({'text': value, 'submit': 'Enter separately'})
        tmux('send-keys', '-t', 'children:0.0', '-l', value)
        tmux('send-keys', '-t', 'children:0.0', 'Enter')

    def wait_for(marker):
        deadline = time.monotonic() + 180
        while time.monotonic() < deadline:
            pane = tmux('capture-pane', '-p', '-J', '-S', '-', '-t', 'children:0.0').stdout
            if any(line.strip() == marker for line in pane.splitlines()):
                return pane
            time.sleep(.2)
        raise RuntimeError(f'Missing completion marker: {marker}')

    tmux('new-session', '-d', '-s', 'children', '-x', '180', '-y', '55',
         '-c', str(repo), 'bash --noprofile --norc')
    try:
        command(f'{shlex.quote(str(candidate))}; printf "\\nproduction-exit=%s\\n" "$?"')
        denial = wait_for('production-exit=78')
        assert 'qualified OS bootstrap required' in denial
        (evidence / 'tmux-denial.txt').write_text(denial)
        probe = evidence / 'candidate/codex-protected-root-probe'
        command(f'{shlex.quote(str(probe))} --inspect-post-exec; printf "\\\\nprobe-exit=%s\\\\n" "$?"')
        inspection = wait_for('probe-exit=0')
        reports = [json.loads(line) for line in inspection.splitlines() if line.startswith('{"no_new_privileges"')]
        assert reports and reports[-1]['native_eligible'] is False
        assert all(reports[-1][field] for field in ('no_new_privileges', 'nondumpable', 'keepcaps_disabled', 'capabilities_empty', 'ambient_empty'))
        (evidence / 'tmux-probe.txt').write_text(inspection)
        command(f'{shlex.quote(str(probe))} --open-existing; printf "\\\\nprobe-native-exit=%s\\\\n" "$?"')
        native_denial = wait_for('probe-native-exit=78')
        (evidence / 'tmux-native-denial.txt').write_text(native_denial)
        command(
            'export PATH=/home/travis/security-round5/tools:/home/travis/.cargo/bin:$PATH; '
            'export CARGO_TARGET_DIR=/home/travis/repos/CorbanuTerminal-harness/codex-rs/target; '
            f'export TMPDIR={shlex.quote(temp)}; '
            'export CARGO_BUILD_JOBS=8 RUST_MIN_STACK=8388608 CORBANU_TEST_NO_NATIVE_KEYRING=1; '
            'flock /home/travis/security-round5/locks/build.lock '
            'just test -p codex-secret-broker-service --features synthetic-fixture '
            '--retries 0 --test-threads 4; printf "\\nchildren-exit=%s\\n" "$?"'
        )
        lifecycle = wait_for('children-exit=0')
        assert '23 tests run: 23 passed' in lifecycle
        (evidence / 'tmux-lifecycle.txt').write_text(lifecycle)
        (evidence / 'tmux-keys.json').write_text(json.dumps(keys, indent=2) + '\n')
        (evidence / 'tmux-result.json').write_text(json.dumps({
            'passed': True, 'tests': 23, 'production_exit': 78,
            'candidate': str(candidate), 'native_isolation_proven': False,
            'privileged_installation': False, 'user_facing_tui': False,
        }, indent=2) + '\n')
    finally:
        tmux('kill-server', check=False)
print('PF27 post-exec probe TMUX checks passed')
