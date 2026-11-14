import json
import os
from pathlib import Path
import shlex
import subprocess
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

SOURCE = Path(__file__).resolve().parents[3]
BIN = SOURCE / 'codex-rs/target/debug/corbanu'
ROOT = Path('/mnt/HC_Volume_101713660/pfrpc/scratch/provider-replacement-qa')
ROOT.mkdir(exist_ok=True)
requests = []
BAD = 'fixture-invalid-subscription-token'
GOOD = 'fixture-replacement-subscription-token'

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        token = self.headers.get('Authorization', '')
        accepted = token == 'Bearer ' + GOOD
        requests.append({'accepted': accepted, 'model': body.get('model')})
        if not accepted:
            self.send_response(401)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            self.wfile.write(b'{"type":"error","error":{"type":"authentication_error","message":"Invalid bearer token"}}')
            return
        events = [
            ('message_start', {'message': {'id': 'msg_fixture', 'model': 'claude-fable-5-1', 'usage': {'input_tokens': 10, 'output_tokens': 0}}}),
            ('content_block_start', {'index': 0, 'content_block': {'type': 'text', 'text': ''}}),
            ('content_block_delta', {'index': 0, 'delta': {'type': 'text_delta', 'text': 'REPLACEMENT_WORKS'}}),
            ('content_block_stop', {'index': 0}),
            ('message_delta', {'delta': {'stop_reason': 'end_turn'}, 'usage': {'output_tokens': 4}}),
            ('message_stop', {}),
        ]
        self.send_response(200)
        self.send_header('Content-Type', 'text/event-stream')
        self.end_headers()
        for kind, event in events:
            event['type'] = kind
            self.wfile.write(('event: ' + kind + '\ndata: ' + json.dumps(event) + '\n\n').encode())
        self.wfile.flush()

server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
threading.Thread(target=server.serve_forever, daemon=True).start()

def tmux(*args):
    return subprocess.check_output(['tmux', *args], text=True)

results = []
for repo in ['tensorcash', 'isometricgame']:
    home = ROOT / (repo + '-' + str(os.getpid()))
    home.mkdir()
    cwd = Path('/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-0.1.42-qa') / repo
    session = 'provider-replacement-' + repo
    env = {'HOME': str(home), 'CODEX_HOME': str(home), 'PATH': os.environ['PATH'], 'TERM': 'xterm-256color', 'LANG': 'C.UTF-8', 'RUST_LOG': 'trace'}
    config = f'''model = "claude-fable-5-1-plan"
model_provider = "fixture"
check_for_update_on_startup = false
model_catalog_json = {json.dumps(str(SOURCE / 'codex-rs/models-manager/models.json'))}
[model_providers.fixture]
name = "Fixture Claude"
base_url = "http://127.0.0.1:{server.server_port}/v1"
wire_api = "anthropic"
requires_openai_auth = false
[model_providers.fixture.auth]
command = {json.dumps(str(BIN))}
args = ["internal-claude-oauth-token"]
[projects.{json.dumps(str(cwd))}]
trust_level = "trusted"
'''
    (home / 'config.toml').write_text(config)
    subprocess.run([str(BIN), 'login', '--with-api-key'], input='fixture-openai', text=True, env=env, capture_output=True, check=True)

    def screen():
        return tmux('capture-pane', '-p', '-t', session)

    def keys(*args):
        tmux('send-keys', '-t', session, *args)
        time.sleep(.2)

    def send(s):
        keys('-l', s)
        keys('Enter')

    def wait(text):
        end = time.monotonic() + 40
        while time.monotonic() < end:
            s = screen()
            if text in ' '.join(s.split()):
                return s
            time.sleep(.15)
        raise AssertionError(text + '\n' + screen())

    def save(name):
        (home / name).write_text(screen())

    def launch():
        cmd = shlex.join(['env', '-i', *[k+'='+v for k,v in env.items()], str(BIN), '--yolo', '--no-alt-screen', '-c', 'log_dir='+json.dumps(str(home/'logs'))])
        tmux('new-session', '-d', '-s', session, '-x', '150', '-y', '45', '-c', str(cwd), cmd)
        wait('permissions:')

    def stop():
        subprocess.run(['tmux', 'kill-session', '-t', session], capture_output=True)

    def account():
        send('/providers')
        wait('Claude Account')
        time.sleep(.5)
        # A refreshed manager remembers the last provider.
        if not any('›' in line and 'Claude Account' in line for line in screen().splitlines()):
            keys('Down')
        keys('Enter')

    def token_form(replace=False):
        account()
        wait('Replace with Claude account' if replace else 'Set up with Claude account')
        if replace:
            keys('Down')
        keys('Enter')
        wait('Long-lived subscription token')
        keys('Enter')
        wait('Long-lived token')

    try:
        launch()
        token_form()
        send(BAD)
        time.sleep(1)
        keys('Escape')
        send('Reply briefly without tools.')
        wait('Invalid bearer token')
        save('01-rejected-token.txt')
        before = (home/'.claude-auth-selection-revision').read_text()
        token_form(True)
        keys('Escape')
        assert (home/'.claude-auth-selection-revision').read_text() == before
        keys('Escape')
        token_form(True)
        send('invalid token with spaces')
        wait('not accepted')
        save('02-invalid-replacement.txt')
        assert (home/'.claude-auth-selection-revision').read_text() == before
        keys('Escape')
        keys('Escape')
        token_form(True)
        send(GOOD)
        time.sleep(1)
        assert (home/'.claude-auth-selection-revision').read_text() != before
        keys('Escape')
        send('Try again, reply briefly without tools.')
        wait('REPLACEMENT_WORKS')
        save('03-recovered.txt')
        stop()
        launch()
        send('Confirm after restart, without tools.')
        wait('REPLACEMENT_WORKS')
        save('04-restart.txt')
        results.append({'repo': str(cwd), 'base': subprocess.check_output(['git', '-C', str(cwd), 'rev-parse', 'HEAD'], text=True).strip(), 'home': str(home), 'passed': True})
    finally:
        save('final.txt')
        stop()
        (ROOT/'results.json').write_text(json.dumps({'results': results, 'requests': requests}, indent=2))
server.shutdown()
print(json.dumps(results, indent=2))
