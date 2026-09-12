"""Collect executor artifacts without silently changing their verdicts."""
from collections import Counter
import hashlib
import html
import json
from pathlib import Path
import re

ROOT = Path('/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911')
OUT = Path('/Volumes/CorbanuDrive/Corbanu/worktrees/provider-reauth-health/qa/provider-auth/pf-58/luna-yellow-20260911')
PACKET = Path(__file__).parent / 'packet'
NUMBERS = [1, 2, 3, 5, 6, 7, 11, 12, 15, 20, 24, 26]

def run_inventory(ledger):
    """Keep original attempts and replays separate, including unfinished runs."""
    runs = {f'case-{n:02}': n for n in NUMBERS}
    for item in ledger:
        runs[item.get('run_name', f"case-{item['case']:02}")] = item['case']
    return list(runs.items())

def execution_disposition(result, ledger_entry):
    if ledger_entry.get('exit_code') == 'timeout':
        return 'timeout'
    if ledger_entry and ledger_entry.get('exit_code') != 0:
        return 'blocked'
    return result.get('status', 'blocked')

def redact(text):
    text = re.sub(r'\bsk-[A-Za-z0-9_-]{12,}', '[REDACTED_CREDENTIAL]', text)
    text = re.sub(r'\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+', '[REDACTED_JWT]', text)
    text = re.sub(r'\b[A-Z0-9]{4}-[A-Z0-9]{5}\b', '[REDACTED_LOGIN_CODE]', text)
    text = re.sub(r'\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b', '[REDACTED_EMAIL]', text, flags=re.IGNORECASE)
    return text

def main():
    OUT.mkdir(parents=True, exist_ok=True)
    rows = []
    ledger = json.loads((ROOT / 'ledger.json').read_text()) if (ROOT / 'ledger.json').exists() else []
    indexed = {item.get('run_name', f"case-{item['case']:02}"): item for item in ledger}
    for run_name, number in run_inventory(ledger):
        root = ROOT / run_name
        evidence = root / 'evidence'
        packet = next(PACKET.glob(f'{number:02}-*.md'))
        title = packet.read_text().splitlines()[0].removeprefix('# ')
        dest = OUT / run_name
        dest.mkdir(exist_ok=True)
        actual_prompt = root / 'instructions.md'
        actual_instruction = packet.read_text()
        if actual_prompt.exists():
            actual_instruction = actual_prompt.read_text().split('HUMAN INSTRUCTIONS BEGIN\n', 1)[1].split('HUMAN INSTRUCTIONS END', 1)[0]
        (dest / 'human-instructions.md').write_text(actual_instruction)
        result = {'case_number': number, 'status': 'pending', 'summary': 'Not completed.'}
        if (evidence / 'results.json').exists():
            try:
                result = json.loads((evidence / 'results.json').read_text())
            except json.JSONDecodeError as exc:
                result = {'case_number': number, 'status': 'blocked', 'summary': 'Executor output was not valid JSON: ' + str(exc)}
        artifacts = []
        if evidence.exists():
            for path in sorted(evidence.rglob('*')):
                if path.is_file() and path.suffix in ('.md', '.json', '.txt', '.log'):
                    relative = path.relative_to(evidence)
                    text = path.read_text(errors='replace')
                    target = dest / relative
                    target.parent.mkdir(parents=True, exist_ok=True)
                    target.write_text(redact(text))
                    artifacts.append({'path': str(target.relative_to(OUT)), 'sha256': hashlib.sha256(target.read_bytes()).hexdigest(), 'redacted': redact(text) != text})
        transcript = ROOT / (run_name + '-events.jsonl')
        thread, actions, usage = None, [], None
        if transcript.exists():
            for line in transcript.read_text().splitlines():
                try:
                    event = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if event['type'] == 'thread.started':
                    thread = event.get('thread_id')
                if event['type'] == 'item.completed' and event.get('item', {}).get('type') == 'command_execution':
                    item = event['item']
                    actions.append({k: item.get(k) for k in ('command', 'exit_code', 'status', 'aggregated_output')})
                if event['type'] == 'turn.completed':
                    usage = event.get('usage')
        (dest / 'actions.json').write_text(redact(json.dumps(actions, indent=2)))
        isolation = ROOT / (run_name + '-isolation.json')
        if isolation.exists():
            (dest / 'isolation.json').write_bytes(isolation.read_bytes())
        execution = indexed.get(run_name, {})
        if execution.get('exit_code') == 'timeout' and not (evidence / 'results.json').exists():
            result = {'case_number': number, 'status': 'incomplete',
                      'summary': 'Executor timed out without a final report. Captures and executed actions are retained; no pass is claimed.'}
        rows.append({'number': number, 'run_name': run_name, 'execution': execution,
                     'title': title + (' (replay)' if run_name != f'case-{number:02}' else ''), 'agent_status': result.get('status', 'blocked'),
                     'summary': result.get('summary', ''), 'thread_id': thread, 'usage': usage,
                     'result': result, 'artifacts': artifacts})
    (OUT / 'results.json').write_text(redact(json.dumps(rows, indent=2)))
    annotations_path = OUT / 'coordinator-notes.json'
    annotations = json.loads(annotations_path.read_text()) if annotations_path.exists() else {}
    for row in rows:
        note = annotations.get(str(row['number']), {}) if row['run_name'] == f"case-{row['number']:02}" else annotations.get(row['run_name'], {})
        row['coordinator_note'] = note
        row['display_status'] = note.get('status', execution_disposition(row['result'], row['execution']))
    (OUT / 'results.json').write_text(redact(json.dumps(rows, indent=2)))
    counts = Counter(r['display_status'] for r in rows)
    cards = []
    for row in rows:
        note = row['coordinator_note'].get('note', '')
        run = html.escape(row['run_name'])
        report_link = f'<a href="{run}/report.md">Original agent report</a> · ' if (OUT / row['run_name'] / 'report.md').exists() else 'No final report · '
        cards.append(f'''<article class="{html.escape(row['display_status'])}"><div class="status">{html.escape(row['display_status'].upper())} · agent verdict: {html.escape(row['agent_status'])}</div><h2>{html.escape(row['title'])}</h2><p>{html.escape(row['summary'])}</p><p><strong>Coordinator note:</strong> {html.escape(note or 'No separate disposition.')}</p><p><a href="{run}/human-instructions.md">Instructions supplied</a> · {report_link}<a href="{run}/actions.json">Executed actions</a></p></article>''')
    page = '''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Luna Max · Yellow-check pilot</title><style>
body{font:17px/1.6 system-ui,sans-serif;background:#f4f6fa;color:#192436;margin:0}main{max-width:1000px;margin:48px auto;padding:0 24px 60px}h1{font-size:40px;line-height:1.15}h2{font-size:21px;margin:6px 0}a{color:#2058ae}header,.notice,article{background:white;border:1px solid #dbe2ed;border-radius:12px;padding:24px;margin:18px 0}.notice{background:#fff4ce}article{border-left:6px solid #98a3b2}.passed{border-left-color:#19825c}.failed{border-left-color:#cc4141}.blocked{border-left-color:#d79c13}.status{font-weight:750;font-size:13px;letter-spacing:.1em}.counts{font-size:21px;font-weight:650}code{font-size:14px;overflow-wrap:anywhere}small{color:#586779}</style><main><header><small>11 September 2026 · Black-box execution pilot</small><h1>The yellow checks, tested by Luna Max</h1><p>Fresh standalone Corbanu CLI agents, Max effort, executed sequentially. Filesystem-enforced source isolation; no implementation code or previous reports supplied.</p><p class="counts">COUNTS</p></header><aside class="notice"><strong>Executor results, not human acceptance or release approval.</strong> Native Applications/Desktop placement and native Keychain behavior cannot be certified by TMUX. Source reads are OS-blocked, including symlink attempts; system libraries, the current package, and private run state remain available. Network access for live inference is available; this is not a network-isolation or adversarial sandbox certification.</aside><section>CARDS</section><footer><p><a href="README.md">Method, provenance and limitations</a> · <a href="results.json">Combined structured results</a></p><p>No product fixes, rebase, merge, or new code review were performed by this pilot. Each test retains its original human expectations.</p></footer></main></html>'''
    count_text = 'Execution attempts (replays retained): ' + ' · '.join(f'{counts[s]} {s}' for s in ('passed', 'failed', 'blocked', 'invalid_run', 'timeout', 'pending'))
    page = page.replace('<section>CARDS</section>', '<aside class="notice"><strong>Known human-reported failure:</strong> the second Applications launch opened on the wrong desktop. That native launcher defect remains open; an agent’s blocked result does not supersede it. Counts below describe this pilot’s test executions, not all known product issues.</aside><section>CARDS</section>')
    native = Path('/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-native-20260911/report.md')
    if native.exists():
        (OUT / 'native-execution.md').write_text(redact(native.read_text()))
        page = page.replace('<section>CARDS</section>', '<aside class="notice"><strong>Separate native Computer Use attempt:</strong> <a href="native-execution.md">Luna’s native action record</a>. Instruction-only code blindness, not the CLI filesystem sandbox. Native observations are not included in CLI attempt counts. <a href="harness-repairs.md">Harness fixes and remaining fixtures</a>.</aside><section>CARDS</section>')
    (OUT / 'report.html').write_text(page.replace('COUNTS', count_text).replace('CARDS', '\n'.join(cards)))
    print(count_text)

if __name__ == '__main__':
    main()
