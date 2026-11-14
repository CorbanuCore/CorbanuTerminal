# rollout_view.py <home> <thread_id> [depth]: account-relevant view of a thread and its spawned children
# (turn-context provider/account, spawn/wait calls and results, errors). Never prints instructions.
import glob, json, sys
home, tid = sys.argv[1], sys.argv[2]
def show(tid, ind=''):
    fs = glob.glob(f'{home}/sessions/**/*{tid}*.jsonl', recursive=True)
    if not fs:
        print(f'{ind}(no rollout for {tid})'); return
    kids, seen = [], set()
    for l in open(fs[0]):
        d = json.loads(l); t = d.get('type'); p = d.get('payload', {})
        if t == 'session_meta':
            src = p.get('source'); role = None
            if isinstance(src, dict):
                ts = (src.get('subagent') or {}).get('thread_spawn') or {}
                role = ts.get('agent_role')
            print(f'{ind}thread {tid[:13]} provider={p.get("model_provider")} role={role}')
        elif t == 'turn_context':
            k = (p.get('model_provider'), p.get('provider_account', '<none>'))
            if k not in seen:
                seen.add(k); print(f'{ind}  turn_context provider={k[0]} provider_account={k[1]}')
        elif t == 'response_item' and p.get('type') == 'function_call' and ('spawn' in p.get('name', '') or 'wait' in p.get('name', '')):
            print(f'{ind}  call {p["name"]} {p.get("arguments")[:200]}')
        elif t == 'response_item' and p.get('type') == 'function_call_output':
            o = p.get('output'); o = o if isinstance(o, str) else json.dumps(o)
            if 'agent_id' in o or 'account' in o or 'status' in o:
                print(f'{ind}  result {o[:300]}')
                try:
                    a = json.loads(o).get('agent_id')
                    if a: kids.append(a)
                except Exception:
                    pass
        elif t == 'event_msg' and p.get('type') in ('error', 'stream_error'):
            print(f'{ind}  error {str(p.get("message"))[:220]}')
    for k in kids:
        show(k, ind + '    ')
show(tid)
