import json, os, shlex, subprocess, tempfile, threading, time, tomllib
from pathlib import Path
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

ROOT = Path(__file__).parent
SOURCE = Path('/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-wallet-launch-hotfix')
BIN = SOURCE / 'codex-rs/target/debug/corbanu'
MODEL = 'corbanu/deepseek-v4.1-flash'
results = []

for repo in ['tensorcash', 'isometricgame']:
    worktree = ROOT / repo
    evidence = ROOT / ('pty-' + repo)
    evidence.mkdir(exist_ok=True)
    home = Path(tempfile.mkdtemp(prefix='c42-'))
    requests, ids, statuses = [], [], []
    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *args): pass
        def do_GET(self):
            self.send_response(200); self.send_header('Content-Type','application/json'); self.end_headers()
            self.wfile.write(json.dumps({'data':[], 'corbanuApi':{'balanceUsd':'10','availableUsd':'10'}}).encode())
        def do_POST(self):
            body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
            requests.append(body); ids.append(self.headers.get('x-pfterminal-request-id'))
            if len(requests) == 1:
                statuses.append(503); self.send_response(503)
                self.send_header('Content-Type','application/json')
                self.send_header('X-Corbanu-Request-Id',ids[-1])
                self.send_header('X-Corbanu-Request-State','released'); self.end_headers()
                self.wfile.write(b'{"error":{"message":"Synthetic preparation failure"}}'); return
            statuses.append(200); self.send_response(200)
            self.send_header('Content-Type','text/event-stream'); self.end_headers()
            if len(requests) == 2:
                names = [tool['function']['name'] for tool in body['tools'] if tool.get('type') == 'function']
                tool = next(name for name in names if name.endswith('exec_command'))
                delta = {'role':'assistant','tool_calls':[{'index':0,'id':'call_release_pwd','type':'function','function':{'name':tool,'arguments':json.dumps({'cmd':'pwd','max_output_tokens':100})}}]}
                finish = 'tool_calls'
            else:
                delta = {'role':'assistant','content':'RELEASE_042_WORKFLOW_OK'}; finish = 'stop'
            for d, f in [(delta,None), ({},finish)]:
                event = {'id':'chatcmpl-release-qa','object':'chat.completion.chunk','created':1,'model':MODEL,'choices':[{'index':0,'delta':d,'finish_reason':f}]}
                self.wfile.write(('data: '+json.dumps(event)+'\n\n').encode())
            self.wfile.write(b'data: [DONE]\n\n'); self.wfile.flush()
    server = ThreadingHTTPServer(('127.0.0.1',0),Handler)
    threading.Thread(target=server.serve_forever,daemon=True).start()
    (home/'config.toml').write_text('model = "gpt-5.6-sol"\nmodel_provider = "fixture"\ncheck_for_update_on_startup = false\nmodel_catalog_json = '+json.dumps(str(SOURCE/'codex-rs/models-manager/models.json'))+'\n[model_providers.fixture]\nname = "Isolated release QA"\nbase_url = "http://127.0.0.1:1/v1"\nwire_api = "responses"\nrequires_openai_auth = false\n[projects.'+json.dumps(str(worktree))+']\ntrust_level = "trusted"\n')
    env = {'HOME':str(home),'CODEX_HOME':str(home),'CORBANU_HOME':str(home),'PATH':os.environ['PATH']}
    subprocess.run([str(BIN),'login','--with-api-key'],input='fixture-openai-key',text=True,env=env,capture_output=True,check=True)
    name = 'corbanu-release-'+repo+'-'+str(os.getpid())
    def tmux(*args): return subprocess.check_output(['tmux',*args],text=True)
    def screen(): return tmux('capture-pane','-t',name,'-p')
    def keys(*args): tmux('send-keys','-t',name,*args); time.sleep(.2)
    def send(value): keys('-l',value); keys('Enter')
    def wait(value):
        end = time.monotonic()+90
        while time.monotonic()<end:
            s=screen()
            if value in ' '.join(s.split()): return s
            time.sleep(.15)
        raise AssertionError('Missing '+value+'\n'+screen())
    def launch():
        cmd=shlex.join(['env','-i',*[k+'='+v for k,v in env.items()],'LANG=C.UTF-8','TERM=xterm-256color','RUST_LOG=trace','CORBANU_API_KEY=fixture-corbanu-key','PFTERMINAL_PLAN_API_KEY=fixture-corbanu-key','CORBANU_API_BASE_URL=http://127.0.0.1:'+str(server.server_port)+'/v1',str(BIN),'--yolo','--no-alt-screen','-c','log_dir='+json.dumps(str(evidence/'logs'))])
        tmux('new-session','-d','-s',name,'-x','180','-y','48','-c',str(worktree),cmd); wait('permissions:')
    def stop(): subprocess.run(['tmux','kill-session','-t',name],capture_output=True)
    def save(file): (evidence/file).write_text('\n'.join(line.rstrip() for line in screen().splitlines()).rstrip()+'\n')
    def picker():
        send('/providers'); wait('Providers'); time.sleep(.5); keys('Escape'); send('/model'); wait('Select Model and Effort')
        for _ in range(15):
            if '[Corbanu API]' in screen(): return
            keys('Right')
        raise AssertionError(screen())
    try:
        launch(); save('startup.txt'); assert '0.1.42' in screen(),screen()
        picker(); assert screen().count('DeepSeek V4.1 Flash')==1; save('picker.txt'); keys('Escape')
        assert tomllib.loads((home/'config.toml').read_text())['model']=='gpt-5.6-sol'
        picker()
        for _ in range(12):
            if any('›' in line and 'DeepSeek V4.1 Flash' in line for line in screen().splitlines()): keys('Enter'); break
            keys('Down')
        time.sleep(.5)
        if 'Select Reasoning' in screen() or 'High (default)' in screen(): keys('Enter')
        send('Use exec_command to run pwd in this repository. After reading the result, reply with RELEASE_042_WORKFLOW_OK. Do not modify any files.')
        end = time.monotonic()+90
        while time.monotonic()<end:
            if len(requests)>=3 and screen().count('RELEASE_042_WORKFLOW_OK')>=2: break
            time.sleep(.2)
        else: raise AssertionError(str(statuses)+'\n'+screen())
        save('tool-response.txt')
        assert statuses==[503,200,200], statuses
        assert ids[0] and ids[1] and ids[0]!=ids[1]
        assert all('parallel_tool_calls' not in request for request in requests)
        assert requests[0]['reasoning_effort']=='high' and requests[0]['tools']
        outputs = [m for m in requests[-1]['messages'] if m.get('role')=='tool']
        assert str(worktree) in json.dumps(outputs),outputs
        stop(); launch(); send('/model'); wait('DeepSeek V4.1 Flash (current)'); save('restart.txt'); keys('Escape')
        results.append({'repo':repo,'worktree':str(worktree),'base':subprocess.check_output(['git','-C',str(worktree),'rev-parse','HEAD'],text=True).strip(),'version':'0.1.42','mode':'real PTY and tool execution; synthetic inference and release headers','statuses':statuses,'rotatedReleasedId':True,'toolsPresent':True,'parallelControlAbsent':True,'selectionCancelAndRestart':True,'repositoryUnchanged':not subprocess.check_output(['git','-C',str(worktree),'status','--porcelain'],text=True).strip()})
        print(json.dumps(results[-1]),flush=True)
    finally:
        stop(); server.shutdown()
        # Only fixture credentials were used; delete them after the run.
        for file in ['auth.json','config.toml']:
            (home/file).unlink(missing_ok=True)
(ROOT/'pty-results.json').write_text(json.dumps(results,indent=2)+'\n')
