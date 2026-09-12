import json,os,shlex,subprocess,tempfile,threading,time,tomllib,http.client
from pathlib import Path
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
ROOT=Path(__file__).parent
BIN=Path('/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-wallet-launch-hotfix/codex-rs/target/debug/corbanu').resolve()
QA_HOME=Path(tempfile.mkdtemp(prefix='ds41-',dir='/mnt/HC_Volume_101713660/pfrpc/scratch'))
MODEL='corbanu/deepseek-v4.1-flash'
requests=[]
request_ids=[]
statuses=[]
class Handler(BaseHTTPRequestHandler):
 def log_message(self,*args):pass
 def do_GET(self):
  self.send_response(200);self.send_header('Content-Type','application/json');self.end_headers();self.wfile.write(json.dumps({'data':[],'corbanuApi':{'balanceUsd':'10','availableUsd':'10'}}).encode())
 def do_POST(self):
  body=json.loads(self.rfile.read(int(self.headers.get('Content-Length',0))));requests.append(body)
  request_id=self.headers.get('x-pfterminal-request-id');request_ids.append(request_id)
  connection=http.client.HTTPConnection('127.0.0.1',18084,timeout=90)
  connection.request('POST','/v1/chat/completions',json.dumps(body),{'Content-Type':'application/json','Authorization':'Bearer '+(ROOT/'live-qa-credential').read_text().strip(),'X-PfTerminal-Request-Id':request_id})
  response=connection.getresponse();statuses.append(response.status)
  self.send_response(response.status)
  for name in ['Content-Type','X-Corbanu-Request-Id','X-Corbanu-Request-State']:
   value=response.getheader(name)
   if value:self.send_header(name,value)
  self.end_headers()
  while True:
   chunk=response.read1(8192)
   if not chunk:break
   self.wfile.write(chunk);self.wfile.flush()
  connection.close()
server=ThreadingHTTPServer(('127.0.0.1',0),Handler);threading.Thread(target=server.serve_forever,daemon=True).start()
(QA_HOME/'config.toml').write_text('''model = "gpt-5.6-sol"
model_provider = "fixture"
check_for_update_on_startup = false
model_catalog_json = "/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-wallet-launch-hotfix/codex-rs/models-manager/models.json"
[model_providers.fixture]
name = "Isolated dropdown QA"
base_url = "http://127.0.0.1:1/v1"
wire_api = "responses"
requires_openai_auth = false
[projects.'''+json.dumps(str(ROOT))+''']
trust_level = "trusted"
''')
fixture_env={'HOME':str(QA_HOME),'CODEX_HOME':str(QA_HOME),'CORBANU_HOME':str(QA_HOME),'PATH':os.environ['PATH']}
subprocess.run([str(BIN),'login','--with-api-key'],input='fixture-openai-key',text=True,env=fixture_env,capture_output=True,check=True)
name='corbanu-flash-qa-'+str(os.getpid())
def tmux(*args):return subprocess.check_output(['tmux',*args],text=True)
def screen():return tmux('capture-pane','-t',name,'-p')
def keys(*args):tmux('send-keys','-t',name,*args);time.sleep(.2)
def send(value):keys('-l',value);keys('Enter')
def wait(value):
 end=time.monotonic()+90
 while time.monotonic()<end:
  s=screen()
  if value in ' '.join(s.split()):return s
  time.sleep(.1)
 raise AssertionError('Missing '+value+'\n'+screen())
def launch():
 cmd=shlex.join(['env','-i','HOME='+str(QA_HOME),'CODEX_HOME='+str(QA_HOME),'CORBANU_HOME='+str(QA_HOME),'PATH='+os.environ['PATH'],'LANG=C.UTF-8','TERM=xterm-256color','RUST_LOG=trace','CORBANU_API_KEY=fixture-corbanu-key','PFTERMINAL_PLAN_API_KEY=fixture-corbanu-key','CORBANU_API_BASE_URL=http://127.0.0.1:'+str(server.server_port)+'/v1',str(BIN),'--yolo','--no-alt-screen','-c','log_dir='+json.dumps(str(ROOT/'pty-logs'))])
 tmux('new-session','-d','-s',name,'-x','180','-y','48','-c',str(ROOT),cmd);wait('permissions:')
def picker():
 send('/providers');wait('Providers');time.sleep(1);keys('Escape');send('/model');wait('Select Model and Effort')
 for _ in range(15):
  if '[Corbanu API]' in screen():return screen()
  keys('Right')
 raise AssertionError(screen())
def stop():subprocess.run(['tmux','kill-session','-t',name],capture_output=True)
try:
 launch();s=picker();assert s.count('DeepSeek V4.1 Flash')==1,s
 (ROOT/'live-picker.txt').write_text(s.rstrip()+'\n');keys('Escape');assert tomllib.loads((QA_HOME/'config.toml').read_text())['model']=='gpt-5.6-sol'
 picker();wait('DeepSeek V4.1 Flash')
 for _ in range(12):
  if any('›' in line and 'DeepSeek V4.1 Flash' in line for line in screen().splitlines()):keys('Enter');break
  keys('Down')
 time.sleep(.5)
 if 'Select Reasoning' in screen() or 'High (default)' in screen():keys('Enter')
 end=time.monotonic()+20
 while time.monotonic()<end:
  config=tomllib.loads((QA_HOME/'config.toml').read_text())
  if config.get('model')==MODEL:break
  time.sleep(.2)
 assert config.get('model')==MODEL,(config,screen())
 assert config.get('model_provider')=='pfterminal-plan',config
 send('Use exec_command to run printf CORBANU_TOOL_OK. After reading its output, reply with FLASH_DROPDOWN_OK. Do not do anything else.')
 end=time.monotonic()+90
 while time.monotonic()<end:
  s=screen()
  if len(requests)>=3 and s.count('FLASH_DROPDOWN_OK')>=2:break
  time.sleep(.2)
 else:raise AssertionError('No completed real tool round trip: '+str(statuses)+'\n'+screen())
 (ROOT/'live-response.txt').write_text(s.rstrip()+'\n')
 assert len(requests)>=3,len(requests)
 assert statuses[0]==503,statuses
 assert statuses[-1]==200 and statuses.count(200)>=2 and all(status in [503,200] for status in statuses),statuses
 assert all(request_ids[i]!=request_ids[i+1] for i,status in enumerate(statuses[:-1]) if status==503),request_ids
 assert request_ids[0]!=request_ids[1],request_ids
 assert any(m.get('role')=='tool' for r in requests[2:] for m in r['messages']), 'No real tool result round trip'
 assert requests[0].get('reasoning_effort')=='high',requests[0].get('reasoning_effort')
 assert requests[0].get('stream') is True
 stop();launch();send('/model');s=wait('DeepSeek V4.1 Flash (current)');(ROOT/'live-restart.txt').write_text(s.rstrip()+'\n');keys('Escape')
 result={'ok':True,'model':MODEL,'binary':str(BIN),'checks':['visible in Corbanu API','cancel preserves current model','selection persists Corbanu provider and model','High reasoning and streaming request use exact public model','released attempts retry with a fresh ID','real tool execution and result round trip','response renders','restart retains selected model'],'gatewayRequests':len(requests),'statuses':statuses,'releasedAttemptRotated':request_ids[0]!=request_ids[1],'realToolResultRoundTrip':True,'productionWrites':0}
 (ROOT/'live-picker-result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
finally:stop();server.shutdown()
