#!/usr/bin/env python3
"""Launch build/inspection tooling with real TMUX keys, not the probe itself."""
import json
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile
import time

root = Path('/home/travis/security-round5/evidence/pf27-static-probe-20260912')
choices = {(): ('build-only.sh', ''), ('--openssl-retry',): ('openssl-retry.sh', 'retry-control'),
           ('--uapi-retry',): ('uapi-retry.sh', 'uapi-control'),
           ('--linkage-retry',): ('linkage-retry.sh', 'linkage-control')}
if tuple(sys.argv[1:]) not in choices:
    raise SystemExit('Unsupported build attempt selector')
script, record_name = choices[tuple(sys.argv[1:])]
record = root / record_name
if record_name:
    record.mkdir()
with tempfile.TemporaryDirectory(prefix='.pf27static-', dir='/home/travis') as temp:
    socket = str(Path(temp) / 't')
    def tmux(*args, check=True):
        return subprocess.run(['tmux', '-S', socket, *args], check=check,
                              capture_output=True, text=True)
    tmux('new-session', '-d', '-s', 'build', '-x', '180', '-y', '55',
         'bash --noprofile --norc')
    command = f'bash {shlex.quote(str(root / "scripts" / script))}; printf "\\nPF27_BUILD_TOOL_EXIT=%s\\n" "$?"'
    (record / 'tmux-keys.json').write_text(json.dumps([
        {'text': command, 'submit': 'Enter separately', 'executes_probe': False}
    ], indent=2) + '\n')
    try:
        tmux('send-keys', '-t', 'build:0.0', '-l', command)
        tmux('send-keys', '-t', 'build:0.0', 'Enter')
        deadline = time.monotonic() + 7200
        while time.monotonic() < deadline:
            pane = tmux('capture-pane', '-p', '-J', '-S', '-', '-t', 'build:0.0').stdout
            markers = [line for line in pane.splitlines() if line.startswith('PF27_BUILD_TOOL_EXIT=')]
            if markers:
                result = int(markers[-1].split('=')[1])
                (record / 'tmux-capture.txt').write_text(pane)
                (record / 'tmux-result.json').write_text(json.dumps({
                    'build_tool_exit': result, 'probe_invoked': False,
                    'user_facing_tui': False,
                }, indent=2) + '\n')
                raise SystemExit(result)
            time.sleep(2)
        raise SystemExit('Build tool timeout; inspect session, do not retry blindly')
    finally:
        # On timeout leave an ongoing build intact for owner inspection.
        if (record / 'tmux-result.json').exists():
            tmux('kill-server', check=False)
