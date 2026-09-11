import json, os, shlex, shutil, subprocess, tempfile, threading, time
from pathlib import Path
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit
ROOT=Path(__file__).parent
BIN=Path('/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-wallet-launch-hotfix/codex-rs/target/debug/corbanu')
API='/api/terminal/tasknode/team/context'
state={'fail':False,'block':False,'pending':False,'reads':0,'posts':0}
started=threading.Event(); release=threading.Event()
def report():
 return {'ok':True,'status':'pending' if state['pending'] else 'current','generatedAt':'2026-09-11T00:00:00Z','showingPreviousReport':state['pending'],'includeInPersonalContext':False,'overview':'Shared contributor updates','members':[{'displayName':'Alice','hiveHandle':'alice','taskHistoryVisible':True,'tasksPastDay':2,'tasksPastWeek':5,'focus':'Reliable task delivery','completedChanges':['Added retry coverage.','Shipped the worker fix.'],'operationalEffect':'Fewer stalled requests.'},{'displayName':'Bob','taskHistoryVisible':False,'tasksPastDay':None,'tasksPastWeek':None,'recentWork':''}]}
class Handler(BaseHTTPRequestHandler):
 def log_message(self,*_):pass
 def reply(self,status,body):
  self.send_response(status);self.send_header('Content-Type','application/json');self.end_headers();self.wfile.write(json.dumps(body).encode())
 def do_POST(self):
  self.rfile.read(int(self.headers.get('Content-Length',0)))
  if self.path=='/api/auth/terminal/start/github':return self.reply(200,{'ok':True,'requestId':'fixture-link','pollToken':'fixture-poll','verificationUrl':ORIGIN+'/choose'})
  state['posts']+=1;return self.reply(405,{'error':'read_only_fixture'})
 def do_GET(self):
  path=urlsplit(self.path).path
  if path=='/api/auth/terminal/session':return self.reply(200,{'ok':True,'accountId':'acct_fixture','githubUsername':'fixture','terminalToken':'fixture-session'})
  if path=='/api/terminal/tasknode/status':return self.reply(200,{'ok':True,'accountId':'acct_fixture','github':{'linked':True,'username':'fixture'},'wallet':{'linked':False,'signingRequiredForActions':False},'counts':{'outstanding':0,'verification':0,'refused':0,'rewarded':0}})
  if path==API:
   state['reads']+=1
   assert self.headers.get('Authorization')=='Bearer fixture-session'
   if state['block']:started.set();release.wait(20)
   if state['fail']:return self.reply(503,{'ok':False,'error':'team_temporarily_unavailable'})
   return self.reply(200,report())
  return self.reply(200,{'ok':True,'items':[],'data':[]})
server=ThreadingHTTPServer(('127.0.0.1',0),Handler);ORIGIN='http://127.0.0.1:'+str(server.server_port)
threading.Thread(target=server.serve_forever,daemon=True).start()
results=[]
try:
 for repo in ['tensorcash','isometricgame']:
  workspace=Path('/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-0.1.42-qa')/repo
  out=ROOT/('pty-'+repo);out.mkdir(exist_ok=True)
  home=Path(tempfile.mkdtemp(prefix='tc-'))
  (home/'config.toml').write_text('model = "gpt-6-astra"\nmodel_provider = "fixture"\ncheck_for_update_on_startup = false\n[model_providers.fixture]\nname = "Offline Team Context QA"\nbase_url = '+json.dumps(ORIGIN+'/v1')+'\nwire_api = "responses"\nrequires_openai_auth = false\n[projects.'+json.dumps(str(workspace))+']\ntrust_level = "trusted"\n')
  env={'HOME':str(home),'CODEX_HOME':str(home),'CORBANU_HOME':str(home),'CORBANU_TASKNODE_PROFILE':'null','TASKNODE_ORIGIN':ORIGIN,'RUST_LOG':'trace','PATH':os.environ['PATH'],'TERM':'xterm-256color','LANG':'C.UTF-8'}
  for command in [('link',),('link','poll')]:
   r=subprocess.run([str(BIN),'tasknode',*command],env=env,text=True,capture_output=True,timeout=30);assert r.returncode==0,(r.stdout,r.stderr)
  r=subprocess.run([str(BIN),'tasknode','team','context','--json'],env=env,text=True,capture_output=True,timeout=30);assert r.returncode==0,(r.stdout,r.stderr)
  assert json.loads(r.stdout)==report(),r.stdout
  (out/'cli.json').write_text(r.stdout)
  name='corbanu-team-'+repo+'-'+str(os.getpid())
  def tmux(*args):return subprocess.check_output(['tmux',*args],text=True)
  def screen():return tmux('capture-pane','-t',name,'-p')
  def keys(*args):tmux('send-keys','-t',name,*args);time.sleep(.2)
  def send(text):keys('-l',text);keys('Enter')
  def wait(text):
   end=time.monotonic()+40
   while time.monotonic()<end:
    s=screen()
    if text in ' '.join(s.split()):return s
    time.sleep(.1)
   raise AssertionError('Missing '+text+'\n'+screen())
  def save(label): (out/(label+'.txt')).write_text('\n'.join(line.rstrip() for line in screen().splitlines()).rstrip()+'\n')
  def choose(text):
   for _ in range(30):
    if any('›' in line and text in line for line in screen().splitlines()):keys('Enter');return
    keys('Down')
   raise AssertionError('Selection missing '+text+'\n'+screen())
  def launch():
   cmd=shlex.join(['env','-i',*[k+'='+v for k,v in env.items()],str(BIN),'--yolo','--no-alt-screen','-c','log_dir='+json.dumps(str(out/'logs'))])
   tmux('new-session','-d','-s',name,'-x','160','-y','45','-c',str(workspace),cmd);wait('permissions:')
   tooling=(home/'skills/.system/tasknode-usage/references/tooling.md').read_text()
   snippet=tooling.split('## Team Context',1)[1].split('```bash',1)[1].split('```',1)[0]
   helper_env={**env,'CORBANU_BIN':shutil.which('corbanu')}
   check=subprocess.run(['bash','-c',snippet],env=helper_env,text=True,capture_output=True,timeout=30)
   assert check.returncode==0,(check.stdout,check.stderr)
   assert json.loads(check.stdout)==report(),check.stdout
  def stop():subprocess.run(['tmux','kill-session','-t',name],capture_output=True)
  try:
   launch();send('/tasknode');wait('Team Context');save('main-menu');choose('Team Context');wait('Status: current');save('team-menu')
   choose('Read full report');wait('Fewer stalled requests.');wait('2 in 24 hours');save('report');keys('Down');keys('PageDown');keys('Home');keys('Escape');wait('Refresh Team Context')
   state['pending']=True;choose('Refresh Team Context');wait('Showing the previous report');save('pending');choose('Read full report');wait('Team Context is being prepared');keys('Escape');state['pending']=False
   state['fail']=True;choose('Refresh Team Context');wait('could not be loaded');save('failure');assert 'Read full report' not in screen();state['fail']=False
   choose('Refresh Team Context');wait('Status: current');save('recovered')
   state['block']=True;started.clear();release.clear();choose('Refresh Team Context');assert started.wait(10)
   keys('Escape');state['block']=False;release.set();time.sleep(1);assert 'Read full report' not in screen();save('cancelled')
   if 'Search Task Node actions' in screen():keys('Escape')
   send('/tasknode team');wait('Status: current');save('slash');stop();launch();send('/tasknode team');wait('Status: current');save('restart')
   assert state['posts']==0,state
   assert not subprocess.check_output(['git','-C',str(workspace),'status','--porcelain'],text=True).strip()
   result={'repo':repo,'version':subprocess.check_output([str(BIN),'--version'],text=True).strip(),'ok':True,'checks':['CLI JSON exact report and authenticated route','Task Node menu entry','read-only pager with summaries and counts','previous-report status','failure and refresh recovery','cancel rejects late response','direct slash command','restart keeps linked account','embedded guidance resolves old-release/new-debug helper without changing profile','no workspace changes or model calls'],'mode':'real terminal and keys, synthetic HTTP identity/report'}
   results.append(result);print(json.dumps(result),flush=True)
  finally:release.set();stop()
 (ROOT/'pty-results.json').write_text(json.dumps(results,indent=2)+'\n')
finally:server.shutdown()
