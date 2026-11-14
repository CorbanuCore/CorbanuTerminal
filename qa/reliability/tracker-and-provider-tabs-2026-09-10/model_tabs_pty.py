import json,os,shlex,subprocess,tempfile,time
from pathlib import Path
ROOT=Path(__file__).parent
BIN=Path(os.environ['CORBANU_QA_BINARY']).resolve()
HOME_QA=Path(tempfile.mkdtemp(prefix='mp-',dir='/mnt/HC_Volume_101713660/pfrpc/scratch'))
(HOME_QA/'config.toml').write_text('''model = "claude-fable-5-plan"
model_provider = "fixture"
check_for_update_on_startup = false
model_catalog_json = "/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-wallet-launch-hotfix/codex-rs/models-manager/models.json"
[model_providers.fixture]
name = "Offline model picker QA"
base_url = "http://127.0.0.1:1/v1"
wire_api = "responses"
requires_openai_auth = false
[projects.'''+json.dumps(str(ROOT))+''']
trust_level = "trusted"
''')
name='corbanu-model-tabs-qa-'+str(os.getpid())
fixture_env={'HOME':str(HOME_QA),'CODEX_HOME':str(HOME_QA),'CORBANU_HOME':str(HOME_QA),'PATH':os.environ['PATH'],'LANG':'C.UTF-8','TERM':'xterm-256color','CLAUDE_CODE_OAUTH_TOKEN':'fixture-claude-token'}
subprocess.run([str(BIN.parent/'examples/provider_qa'),str(HOME_QA)],env=fixture_env,check=True,capture_output=True)
subprocess.run([str(BIN),'login','--with-api-key'],input='fixture-openai-key',text=True,env=fixture_env,check=True,capture_output=True)
def tmux(*args):return subprocess.check_output(['tmux',*args],text=True)
def screen():return tmux('capture-pane','-t',name,'-p')
def keys(*args):tmux('send-keys','-t',name,*args);time.sleep(.2)
def send(text):keys('-l',text);keys('Enter')
def wait(text):
 deadline=time.monotonic()+30
 while time.monotonic()<deadline:
  s=screen()
  if text in s:return s
  time.sleep(.1)
 raise AssertionError('Missing '+text+'\n'+screen())
command=shlex.join(['env','-i','HOME='+str(HOME_QA),'CODEX_HOME='+str(HOME_QA),'CORBANU_HOME='+str(HOME_QA),'PATH='+os.environ['PATH'],'LANG=C.UTF-8','TERM=xterm-256color','RUST_LOG=trace','OPENAI_API_KEY=fixture-openai-key','CLAUDE_CODE_OAUTH_TOKEN=fixture-claude-token',str(BIN),'--yolo','--no-alt-screen','-c','log_dir='+json.dumps(str(ROOT/'model-logs'))])
try:
 tmux('new-session','-d','-s',name,'-x','180','-y','45','-c',str(ROOT),command);wait('permissions:')
 send('/providers');wait('Providers');time.sleep(1);keys('Escape');send('/model');wait('Select Model')
 # The catalog may first offer Auto / All models.
 if 'All models' in screen() and 'Select Model and Effort' not in screen():
  for _ in range(10):
   if any('›' in line and 'All models' in line for line in screen().splitlines()):keys('Enter');break
   keys('Down')
 wait('Select Model and Effort')
 found={}
 for _ in range(18):
  s=screen()
  if '[OpenAI]' in s:
   assert 'Claude Fable' not in s,s
   assert 'GPT-' in s,s
   (ROOT/'model-openai-tab.txt').write_text(s.rstrip()+'\n');found['openai']=True
  if '[Claude Plan]' in s:
   assert 'Claude Fable' in s,s
   (ROOT/'model-claude-tab.txt').write_text(s.rstrip()+'\n');found['claude']=True
  if len(found)==2:break
  keys('Right')
 assert len(found)==2,screen()
 keys('Escape')
 result={'ok':True,'binary':str(BIN),'checks':['Providers refresh with Claude current','OpenAI tab contains OpenAI models and no Claude Plan model','Claude Plan tab retains Fable','cancel closes picker'],'modelCalls':0,'credentials':'synthetic; fresh isolated home'}
 (ROOT/'model-tabs-result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
finally:subprocess.run(['tmux','kill-session','-t',name],capture_output=True)
