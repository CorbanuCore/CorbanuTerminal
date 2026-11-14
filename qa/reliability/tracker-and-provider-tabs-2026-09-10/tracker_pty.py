import json, os, shlex, subprocess, tempfile, threading, time
from pathlib import Path
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit

ROOT=Path(__file__).parent
BIN=Path(os.environ['CORBANU_QA_BINARY']).resolve()
HOME_QA=Path(tempfile.mkdtemp(prefix='ct-',dir='/mnt/HC_Volume_101713660/pfrpc/scratch'))
WORKSPACE=ROOT/'workspace';WORKSPACE.mkdir(exist_ok=True)
STATE={'enabled':False,'workspace':'','campaigns':[],'campaignPosts':0,'activityReads':[],'failActivity':True,'keys':[],'modelCalls':0}
API='/api/terminal/tasknode/campaign-tracker'
class Handler(BaseHTTPRequestHandler):
 def log_message(self,*_):pass
 def reply(self,code,value):
  self.send_response(code);self.send_header('Content-Type','application/json');self.end_headers();self.wfile.write(json.dumps(value).encode())
 def do_POST(self):
  body=json.loads(self.rfile.read(int(self.headers.get('Content-Length',0))) or '{}')
  if self.path=='/api/auth/terminal/start/github':return self.reply(200,{'ok':True,'requestId':'qa-link','pollToken':'qa-poll','verificationUrl':ORIGIN+'/choose'})
  if self.path==API+'/enrollment':
   if body['enabled']:
    if not body.get('apiKey'):return self.reply(400,{'ok':False,'error':'tracker_credential_required','message':'A Corbanu API credential is required to enable or sync recording. Link Corbanu API in Providers, then retry. Existing local activity is retained.'})
    assert body['apiKey']=='fixture-api-key';STATE['keys'].append(True)
   STATE['enabled']=body['enabled'];STATE['workspace']=body['workspaceId'];return self.reply(200,{'ok':True})
  if self.path==API+'/campaigns':
   STATE['campaignPosts']+=1
   if len(body['title'].encode())>200:return self.reply(400,{'ok':False,'error':'tracker_text_invalid','message':'Campaign name exceeds 200 UTF-8 bytes. Shorten it and submit again.','validation':{'field':'title','reason':'max_bytes','maxBytes':200}})
   STATE['campaigns'].append({**body,'members':[]});return self.reply(200,{'ok':True,'campaignId':'campaign-fixture'})
  if self.path==API+'/events':return self.reply(200,{'ok':True,'summaryState':'ready'})
  STATE['modelCalls']+=1;return self.reply(500,{'error':'no-model-calls-in-this-fixture'})
 def do_GET(self):
  path=urlsplit(self.path).path
  if path=='/api/auth/terminal/session':return self.reply(200,{'ok':True,'accountId':'acct_fixture','githubUsername':'fixture','terminalToken':'fixture-session'})
  if path=='/api/terminal/tasknode/status':return self.reply(200,{'ok':True,'accountId':'acct_fixture','github':{'linked':True,'username':'fixture'},'wallet':{'linked':False,'address':None,'signingRequiredForActions':False},'counts':{'outstanding':0,'verification':0,'refused':0,'rewarded':0},'flags':{'terminalTaskActions':True}})
  if path==API+'/status':return self.reply(200,{'ok':True,'handle':'fixture','usage':{'events':0},'enrollments':[{'workspace_id':STATE['workspace'],'enabled':STATE['enabled']}]})
  if path==API+'/campaigns':return self.reply(200,{'ok':True,'items':STATE['campaigns']})
  if path==API+'/activity':
   STATE['activityReads'].append(self.path)
   if STATE['failActivity']:
    STATE['failActivity']=False;return self.reply(503,{'ok':False,'error':'tracker_gateway_unavailable','message':'Tracker service unavailable'})
   return self.reply(200,{'ok':True,'items':[]})
  return self.reply(200,{'ok':True,'items':[],'data':[]})
server=ThreadingHTTPServer(('127.0.0.1',0),Handler);ORIGIN='http://127.0.0.1:'+str(server.server_port)
threading.Thread(target=server.serve_forever,daemon=True).start()
(HOME_QA/'config.toml').write_text('''model = "gpt-6-astra"
model_provider = "fixture"
check_for_update_on_startup = false
[model_providers.fixture]
name = "Offline tracker QA"
base_url = "'''+ORIGIN+'''/v1"
wire_api = "responses"
requires_openai_auth = false
[projects.'''+json.dumps(str(WORKSPACE))+''']
trust_level = "trusted"
''')
env=os.environ.copy();env.update(CODEX_HOME=str(HOME_QA),CORBANU_HOME=str(HOME_QA),TASKNODE_ORIGIN=ORIGIN,RUST_LOG='trace')
aliases=['CORBANU_API_KEY','CORBANU_PLAN_API_KEY','PFTERMINAL_PLAN_API_KEY']
for key in aliases+['PFT_TASKNODE_ORIGIN','CODEX_THREAD_ID','CORBANU_TASKNODE_PROFILE']:env.pop(key,None)
for command in [('link',),('link','poll')]:
 r=subprocess.run([str(BIN),'tasknode',*command],env=env,capture_output=True,text=True,timeout=30);assert r.returncode==0,(r.stdout,r.stderr)
name='corbanu-tracker-qa-'+str(os.getpid())
def tmux(*args):return subprocess.check_output(['tmux',*args],text=True)
def screen():return tmux('capture-pane','-t',name,'-p')
def wait(text):
 deadline=time.monotonic()+30
 while time.monotonic()<deadline:
  s=screen()
  if text in ' '.join(s.split()):return s
  time.sleep(.1)
 raise AssertionError('Missing '+text+'\n'+screen())
def save(label,text): (ROOT/(label+'.txt')).write_text(wait(text).rstrip()+'\n')
def keys(*values):tmux('send-keys','-t',name,*values);time.sleep(.15)
def send(text):keys('-l',text);keys('Enter')
def choose(text):
 for _ in range(30):
  if any('›' in line and text in line for line in screen().splitlines()):keys('Enter');return
  keys('Down')
 raise AssertionError('Selection missing '+text+'\n'+screen())
def launch(alias=None):
 command=shlex.join([str(BIN),'--yolo','--no-alt-screen','-c','log_dir='+json.dumps(str(ROOT/'logs'))])
 args=['new-session','-d','-s',name,'-x','160','-y','45','-c',str(WORKSPACE)]
 launch_env=env.copy()
 if alias:launch_env[alias]='fixture-api-key'
 for key in ['CODEX_HOME','CORBANU_HOME','TASKNODE_ORIGIN','RUST_LOG',*aliases]:
  args+=['-e',key+'='+launch_env.get(key,'')]
 tmux(*args,command);wait('permissions:')
def stop():subprocess.run(['tmux','kill-session','-t',name],capture_output=True)
def tracker():send('/tasknode');wait('Campaign Tracker');choose('Campaign Tracker');wait('Enable recording for this workspace')
try:
 launch();tracker();choose('Enable recording');save('missing-credential','Link Corbanu API in Providers');assert STATE['keys']==[];stop()
 for i,alias in enumerate(aliases):
  launch(alias);tracker();choose('Enable recording');save('alias-'+alias,'Pause this workspace');assert len(STATE['keys'])==i+1
  choose('Pause this workspace');wait('Enable recording for this workspace')
  if i==0:
   choose('Campaigns');wait('Create campaign');choose('Create campaign');wait('First line: campaign name')
   original='X'*201+'\nRetain this objective'
   keys('-l',original);keys('C-d');save('campaign-rejected','Campaign name exceeds 200 UTF-8 bytes')
   assert STATE['campaignPosts']==1
   choose('Edit draft');save('draft-restored','Retain this objective');keys('Escape');assert STATE['campaignPosts']==1
   choose('Edit draft');wait('Retain this objective');keys('-N',str(len(original)),'BSpace');keys('-l','Fixed campaign\nRetain this objective');keys('C-d');save('campaign-corrected','Fixed campaign')
   assert STATE['campaignPosts']==2 and STATE['campaigns'][0]['objective']=='Retain this objective'
   choose('My activity');save('read-failed','Tracker service unavailable');choose('Retry');save('read-recovered','Search activity');assert len(STATE['activityReads'])==2 and len(set(STATE['activityReads']))==1
  stop()
 assert STATE['modelCalls']==0
 result={'ok':True,'binary':str(BIN),'home':str(HOME_QA),'checks':['missing credential guidance','all three credential aliases enable recording','failed campaign retains draft','cancel sends no second write','corrected draft submits once','read Retry repeats exact route'],'modelCalls':0,'productionWrites':0}
 (ROOT/'tracker-result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
finally:stop();server.shutdown()
