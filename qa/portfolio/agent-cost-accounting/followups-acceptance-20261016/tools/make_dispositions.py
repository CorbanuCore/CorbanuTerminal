"""Write dispositions.md from tools/final.json, design.json and the runs' raw verdicts. usage: make_dispositions.py RECORD_DIR"""
import json, re, pathlib, sys
from collections import Counter
root = pathlib.Path(sys.argv[1]); final = json.load(open(root / 'tools/final.json')); design = json.load(open(root / 'design.json'))
groups = {g: sorted(p.name[10:-5] for p in (root / 'runs' / g).glob('isolation-*.json')) for g in ['B', 'A', 'C', 'D', 'E', 'F', 'C2', 'E2']}
def raw(g, c):
    m = re.findall(r'VERDICT %s: (passed|failed|blocked)' % c, (root / 'runs' / g / 'collected/evidence/result.md').read_text()); return m[-1] if m else 'missing'
L = ['# Dispositions (integrator, pf60-s10)', '', 'Independent disposition per case, judged against the frozen case as amended (amendments govern). Raw executor verdicts are',
     'kept as written; where they differ, the reason is in the summary. Final run = the run the disposition rests on. Priority as amended.', '',
     '| Case | Priority | Disposition | Final run | Raw verdicts (all runs) | Summary |', '| --- | --- | --- | --- | --- | --- |']
for c in design['cases']:
    i = c['id']; f = final[i]
    raws = '; '.join(f'{g}: {raw(g, i)}' for g in groups if i in groups[g])
    L.append(f"| {i} | {f.get('priority', c['priority'])} | **{f['d']}** | {f['run']} | {raws} | {f['s'].replace('|', '/')} |")
cnt = Counter(final[c['id']]['d'] for c in design['cases'])
L += ['', f"Totals: {cnt['passed']} passed, {cnt['failed']} failed, {cnt['blocked']} blocked, of {len(design['cases'])}.", '',
      'Differences from raw verdicts: ACC-04 (passed -> blocked: not exercised), ACC-11 and ACC-18 (passed -> blocked: Request IDs not re-read; ACC-18 sums checked for ALPHA only), ACC-14 (failed -> blocked: Request IDs not observed; the executor gave its skipped CHARLIE step as the reason), ACC-15 (failed -> blocked: one expected result not judged by the executor), ACC-22 (failed -> blocked: starting condition not met), ACC-34 (C2/E2 failed -> passed: case-insensitive matches of non-credential text). C and E first attempts are superseded by the replays C2 and E2 (navigation note; harness fix 1); their raw verdicts stay above.']
(root / 'dispositions.md').write_text('\n'.join(L) + '\n'); print(cnt)
