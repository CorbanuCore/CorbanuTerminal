import argparse
import json
import os
from pathlib import Path
import shlex
import signal
import subprocess
import time
import tempfile

parser = argparse.ArgumentParser()
parser.add_argument('--binary', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--baseline', action='store_true')
args = parser.parse_args()
root = args.output.resolve()
root.mkdir(parents=True, exist_ok=True)
home = Path(tempfile.mkdtemp(prefix='wq-', dir='/mnt/HC_Volume_101713660/pfrpc/scratch'))
(root/'test-home.txt').write_text(str(home)+'\n')
workspace = root / 'workspace'
workspace.mkdir(exist_ok=True)
(home / 'config.toml').write_text('''model = "gpt-6-astra"
model_provider = "fixture"
check_for_update_on_startup = false
[model_providers.fixture]
name = "Offline wallet QA"
base_url = "http://127.0.0.1:1/v1"
wire_api = "responses"
requires_openai_auth = false
''' + '\n[projects.' + json.dumps(str(workspace)) + ']\ntrust_level = "trusted"\n')
name = 'corbanu-wallet-p0-' + str(os.getpid())

def tmux(*values):
    return subprocess.check_output(['tmux', *values], text=True)

def screen():
    return tmux('capture-pane', '-t', name, '-p')

def wait(text):
    deadline = time.monotonic() + 35
    while time.monotonic() < deadline:
        value = screen()
        if text in ' '.join(value.split()):
            return value
        time.sleep(.1)
    raise AssertionError('Missing ' + text + '\n' + screen())

def save(label, text):
    (root / (label + '.txt')).write_text(wait(text))

def send(text):
    tmux('send-keys', '-t', name, '-l', text)
    time.sleep(.15)
    tmux('send-keys', '-t', name, 'Enter')

def launch():
    command = shlex.join([str(args.binary.resolve()), '--yolo', '--no-alt-screen', '-c', 'log_dir=' + json.dumps(str(root/'logs'))])
    tmux('new-session', '-d', '-s', name, '-x', '160', '-y', '45', '-c', str(workspace), '-e', 'CODEX_HOME='+str(home), '-e', 'CORBANU_HOME='+str(home), '-e', 'RUST_LOG=trace', command)
    wait('permissions:')

def stop():
    subprocess.run(['tmux', 'kill-session', '-t', name], capture_output=True)
    # Only terminate daemons owned by this disposable test home.
    for directory in Path('/proc').iterdir():
        if not directory.name.isdecimal():
            continue
        try:
            cmd = (directory/'cmdline').read_bytes().split(b'\0')
            if str(home).encode() in cmd and cmd and Path(os.fsdecode(cmd[0])).name in ('corbanu-walletd', 'pfterminal-walletd'):
                os.kill(int(directory.name), signal.SIGTERM)
        except (OSError, ProcessLookupError):
            pass
    time.sleep(.3)

try:
    launch()
    send('/wallet')
    if args.baseline:
        save('baseline-missing-daemon', 'pfterminal-walletd')
        assert 'required wallet daemon executable is missing' in ' '.join(screen().split())
        result = {'baselineReproduced': True, 'binary': str(args.binary)}
    else:
        save('cold-start-wallet', 'Create wallet')
        assert 'Unavailable:' not in screen()
        tmux('send-keys', '-t', name, 'Escape')
        send('/wallet')
        save('reopen-wallet', 'Create wallet')
        stop()
        daemon = args.binary.parent/'corbanu-walletd'
        held = daemon.with_name('held-wallet-daemon')
        daemon.rename(held)
        try:
            launch()
            send('/wallet')
            save('missing-daemon', 'required wallet daemon executable is missing')
        finally:
            held.rename(daemon)
        tmux('send-keys', '-t', name, 'Enter')
        save('retry-recovers', 'Create wallet')
        stop()
        launch()
        send('/wallet')
        save('restart-wallet', 'Create wallet')
        result = {'ok': True, 'binary': str(args.binary), 'checks': ['cold startup', 'cancel and reopen', 'missing daemon error', 'Retry recovers after file restored', 'cold restart'], 'modelCalls': 0, 'walletsCreated': 0, 'payments': 0}
    (root/'result.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result))
finally:
    stop()
