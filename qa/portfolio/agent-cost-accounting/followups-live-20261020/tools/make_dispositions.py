"""Write dispositions.md from tools/final.json, design.json and the runs' raw verdicts. usage: make_dispositions.py RECORD_DIR"""
import json, re, pathlib, sys
from collections import Counter
root = pathlib.Path(sys.argv[1]); final = json.load(open(root / 'tools/final.json')); design = json.load(open(root / 'design.json'))
groups = {g: sorted(p.name[10:-5] for p in (root / 'runs' / g).glob('isolation-*.json')) for g in ['Z', 'O', 'R', 'M']}
def raw(g, c):
    m = re.findall(r'VERDICT %s: (passed|failed|blocked)' % c, (root / 'runs' / g / 'collected/evidence/result.md').read_text()); return m[-1] if m else 'missing'
L = ['# Dispositions (integrator, acc-tart1)', '', 'Independent disposition per case, judged against the frozen case as amended (amendments govern). Raw executor verdicts are',
     'kept as written; where they differ, the reason is in the summary. Final run = the run the disposition rests on. Priority as amended.', '',
     '| Case | Priority | Disposition | Final run | Raw verdicts (all runs) | Summary |', '| --- | --- | --- | --- | --- | --- |']
for c in design['cases']:
    i = c['id']; f = final[i]
    raws = '; '.join(f'{g}: {raw(g, i)}' for g in groups if i in groups[g])
    L.append(f"| {i} | {f.get('priority', c['priority'])} | **{f['d']}** | {f['run']} | {raws} | {f['s'].replace('|', '/')} |")
cnt = Counter(final[c['id']]['d'] for c in design['cases'])
L += ['', f"Totals: {cnt['passed']} passed, {cnt['failed']} failed, {cnt['blocked']} blocked, of {len(design['cases'])}.", '',
      open(root / 'tools/differences.md').read().strip()]
(root / 'dispositions.md').write_text('\n'.join(L) + '\n'); print(cnt)
