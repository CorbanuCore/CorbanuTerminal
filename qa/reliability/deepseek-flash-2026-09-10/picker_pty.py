import json,os,shlex,subprocess,tempfile,threading,time,tomllib
from pathlib import Path
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
ROOT=Path(__file__).parent
BIN=Path('/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-wallet-launch-hotfix/codex-rs/target/debug/corbanu').resolve()
QA_HOME=Path(tempfile.mkdtemp(prefix='ds41-',dir='/mnt/HC_Volume_101713660/pfrpc/scratch'))
MODEL='corbanu/deepseek-v4.1-flash'
requests=[]
class Handler(BaseHTTPRequestHandler):
 def log_message(self,*args):pass
 def do_GET(self):
  self.send_response(200);self.send_header('Content-Type','application/json');self.end_headers();self.wfile.write(json.dumps({'data':[],'corbanuApi':{'balanceUsd':'10','availableUsd':'10'}}).encode())
 def do_POST(self):
  body=json.loads(self.rfile.read(int(self.headers.get('Content-Length',0))));requests.append(body)
  assert body['model']==MODEL,body.get('model')
  self.send_response(200);self.send_header('Content-Type','text/event-stream');self.end_headers()
  for payload in [
   {'id':'qa-flash','object':'chat.completion.chunk','model':MODEL,'choices':[{'index':0,'delta':{'role':'assistant','content':'FLASH_DROPDOWN_OK'},'finish_reason':None}]},
   {'id':'qa-flash','object':'chat.completion.chunk','model':MODEL,'choices':[{'index':0,'delta':{},'finish_reason':'stop'}],'usage':{'prompt_tokens':20,'completion_tokens':5,'total_tokens':25}}]:
   self.wfile.write(('data: '+json.dumps(payload)+'\n\n').encode())
  self.wfile.write(b'data: [DONE]\n\n');self.wfile.flush()
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
 end=time.monotonic()+35
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
 (ROOT/'picker.txt').write_text(s.rstrip()+'\n');keys('Escape');assert tomllib.loads((QA_HOME/'config.toml').read_text())['model']=='gpt-5.6-sol'
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
 send('Reply with the integration success marker.');s=wait('FLASH_DROPDOWN_OK')
 (ROOT/'response.txt').write_text(s.rstrip()+'\n')
 assert len(requests)==1,len(requests)
 assert requests[0].get('reasoning_effort')=='high',requests[0].get('reasoning_effort')
 assert requests[0].get('stream') is True
 stop();launch();send('/model');s=wait('DeepSeek V4.1 Flash (current)');(ROOT/'restart.txt').write_text(s.rstrip()+'\n');keys('Escape')
 result={'ok':True,'model':MODEL,'binary':str(BIN),'checks':['visible in Corbanu API','cancel preserves current model','selection persists Corbanu provider and model','High reasoning and streaming request use exact public model','response renders','restart retains selected model'],'fixtureRequests':len(requests),'productionWrites':0}
 (ROOT/'picker-result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
finally:stop();server.shutdown()
