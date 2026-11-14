"""Run independent Luna Max test executors serially; coordinator-only."""
import hashlib
import argparse
import json
import os
import signal
from pathlib import Path
import shlex
import subprocess
import time
from pilot import ROOT, PACKET, prepare

SCRIPT = Path(__file__).parent / 'pilot.py'
NUMBERS = [1, 2, 3, 5, 6, 7, 11, 12, 15, 20, 24, 26]

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--from-case', type=int, default=1)
    parser.add_argument('--only', type=int)
    parser.add_argument('--cases', help='Comma-separated frozen case numbers, executed in listed order')
    parser.add_argument('--suffix', default='')
    args = parser.parse_args()
    ledger = json.loads((ROOT / 'ledger.json').read_text()) if (ROOT / 'ledger.json').exists() else []
    selected = [int(n) for n in args.cases.split(',')] if args.cases else NUMBERS
    if len(set(selected)) != len(selected) or any(n not in NUMBERS for n in selected):
        raise ValueError('Cases must be unique frozen case numbers')
    for number in [n for n in selected if n >= args.from_case and (args.only is None or args.only == n)]:
        name = f'case-{number:02}' + args.suffix
        root, env = prepare(name)
        subprocess.run(['python3', str(SCRIPT), name], check=True, stdout=subprocess.DEVNULL)
        probe = json.loads((ROOT / (name + '-isolation.json')).read_text())
        if not probe['passed']:
            raise RuntimeError('Isolation preflight failed: ' + name)
        instruction = next(PACKET.glob(f'{number:02}-*.md')).read_text()
        target = root / 'package/bin/corbanu'
        sock = root / 'tmp/t'
        launch = shlex.join(['env', 'CORBANU_HOME=' + str(root / 'target-home'),
                            'CODEX_HOME=' + str(root / 'target-home'),
                            'PFTERMINAL_HOME=' + str(root / 'target-home'),
                            'CORBANU_TEST_NO_NATIVE_KEYRING=1', 'RUST_LOG=warn',
                            str(target), '--no-alt-screen', '-C', str(root / 'work'),
                            '-c', 'log_dir="' + str(root / 'target-home/log') + '"'])
        (root / 'start-target.sh').write_text('#!/bin/sh\nexec ' + launch + '\n')
        start_command = shlex.join(['/opt/homebrew/bin/tmux', '-S', str(sock), 'new-session', '-d', '-s', 'target', '-x', '150', '-y', '48', '/bin/sh ' + str(root / 'start-target.sh')])
        guide = f'''You are an independent black-box functional TEST EXECUTOR, not a code reviewer or test designer. Execute the supplied human acceptance instructions verbatim where possible, on the supplied compiled application. Do not inspect any source, binary internals, old reports, test implementations, credential files, project policies, or filesystem outside this scratch run. Do not fetch code or guidance online. Do not spawn more agents. You are Luna at Max effort, with no prior task history.

This task uses the Corbanu development/TUI-testing workflow, overridden explicitly by the user to forbid reading repository instructions or code. Use raw TMUX actual-key interactions, not `corbanu exec` as a substitute for the user interface. A human checklist is not evidence that behavior passes. Do not fix anything. Record blocked rather than inventing prerequisites. Keep conclusions bounded to what you actually observe. You have up to 8 minutes, not an obligation to spend that long.

Your working directory: {root / 'work'}
Evidence directory: {root / 'evidence'}
Compiled candidate: {target}
Candidate SHA-256: {hashlib.sha256(target.read_bytes()).hexdigest()}
Platform: macOS arm64. No Linux inference is permitted.
TMUX: /opt/homebrew/bin/tmux
Dedicated socket: {sock}
Target session name: target
Start the target with: {start_command}

All child processes inherit an enforced outer filesystem sandbox. You cannot read the source repository, prior reports, other run directories, or the user's home files. Do not attempt to escape this boundary, use AppleScript/GUI automation, connect to outside TMUX servers, use remote-shell services, or inspect other processes. The binary package includes its required helpers. A private clone of an existing test profile was provisioned; no normal user profile is being modified. OpenAI account and Claude managed-token configurations were copied, but their current live validity must be observed, not assumed. Other credentials/services may not be configured. Do not read, print, export, replace, or delete credentials; inspect only UI metadata. No purchases, payments, wallet key creation/revocation, real permission changes, or browser login completion are authorized for this executor. Browser inspection/cancellation is allowed through TUI controls; if browser/native access is required, record that portion as blocked. Do not request new user actions yourself.

The native Applications shortcut and macOS Desktop placement are NOT accessible through this interface. If required by a case, keep that original criterion blocked; do not substitute TMUX windows as proof. The cloned test profile uses file credential storage and a native-keyring test fallback. No Keychain-prompt pass may be inferred from it. Its stored setting disables startup update checks, so the absence of an update notice is not evidence of correct version comparison. These are harness limits, not product results.

Harness commands: capture with `tmux -S SOCKET capture-pane -p -t target` (add `-S -` to inspect your own complete scrollback); literal text with `tmux -S SOCKET send-keys -t target -l -- TEXT`; press Enter in a SEPARATE command; use Down/Up/Left/Right/Escape keys as needed. Wait for positive visible checkpoints rather than declaring success after a fixed delay. To resize use `resize-window -t target -x 40 -y 48`, then restore 150 columns. Follow ordinary visible UI instructions. Do not guess hidden actions from implementation details. You may read public CLI --help if required for launching. Retrying a missed key or reopening a menu is fine; don't change intended expectations.

Capture named plaintext pane checkpoints in your evidence directory using shell redirection or a small capture-only script. Preserve executed key/action history. Never log credentials or browser login codes: if a login code appears, redact it before saving a capture and omit it from your report. Ensure any echoed prompt is not mistaken for an assistant response. Brief harmless model requests are authorized. No full trace logging with live profiles. For native-only cases, you can exercise an explicitly labeled TUI subset but must not claim the original full case passed.

PUBLIC-UI NAVIGATION AID (mechanics only; the HUMAN criteria below remain authoritative):
- From the chat composer, type a supplied slash command literally, then press Enter separately. Capture the resulting screen before deciding the next action.
- Provider setup and credential recovery start at `/providers`, NOT `/model`. In `/providers`, select the named provider with Up/Down; Enter opens its management actions and `r` opens its recovery actions where advertised. API-key entry can be inspected through Anthropic's setup action and cancelled without supplying a key. OpenAI's account sign-in is under its recovery menu. `/model` selects models; it is not the credential-setup entry point.
- In a list, Up/Down moves the selection, Enter opens/confirms it, and Escape goes back or cancels. Follow the screen's footer if it specifies different keys.
- In the model picker, Left/Right changes provider tabs. Choose the model with Up/Down and Enter, then follow any visible effort selection. Do not assume a tab or model exists: its absence may be the finding.
- Use shortcuts such as `r` only where the visible UI or HUMAN instructions advertise them. Do not turn a navigation aid into an alternate route that bypasses the feature being tested.
- For a test message, send the COMPLETE literal text between backticks, excluding the backticks; send Enter separately. Never shorten an instruction to only its expected response marker. Confirm an assistant reply, not merely the echoed user prompt.
- Escape can interrupt an active response. Ctrl+D at an empty chat composer is the normal-exit shortcut; record whether it actually exits. Do not send it in a menu or during a required interaction.
- Plaintext captures omit styling: dim composer suggestions may look like typed text. Use `capture-pane -e -p` to retain ANSI colors when necessary. Do not repeatedly erase a placeholder. These are terminal captures, not native desktop screenshots.
- For cleanup only, use `tmux -S SOCKET kill-session -t target` on your dedicated target. Forced termination is not proof of normal exit. No need to discover unrelated quit commands.
- If a required native interface or prerequisite is absent, record blocked promptly. Do not explore unrelated features to fill time. If this aid disagrees with the visible UI, record the discrepancy and follow the UI; do not assume a product pass.

Begin with a prerequisite check for THIS case in the visible UI and write a short evidence/progress.md using the absolute evidence directory above. Save a preliminary results.json with status blocked if a required prerequisite is absent; preserve the reason even if you exercise an optional subset. Only OpenAI account and Claude managed token were provisioned: no environment/API-key, external auth-command, AWS, or local-model server fixture was provisioned. Inspection of an unconfigured API-key entry is still possible; successful external-source recovery is not. Never treat a working credential as an expired-token fixture. Write your final report before optional exploration; leave time to finish. Screens and actions already recorded are supporting evidence if the coordinator time limit ends the run.

At completion, exit ONLY the target session you created; never kill another server. Write report.md and results.json under the ABSOLUTE evidence directory given above (not work/evidence). JSON must include: case_number, status (passed/failed/blocked), summary, observed_checks (each with status and evidence path), blockers, actions, candidate_sha256, platform, limitations. A failed observable expectation is failed even if other checks are blocked; otherwise unresolved required checks mean blocked. Separately state any test-instruction ambiguity. Report no human acceptance and do not edit checklists. Finish with a concise summary.

HUMAN INSTRUCTIONS BEGIN
{instruction}
HUMAN INSTRUCTIONS END
'''
        prompt = root / 'instructions.md'
        prompt.write_text(guide)
        began = time.time()
        process = subprocess.Popen(['python3', str(SCRIPT), name, '--prompt', str(prompt)], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True)
        (ROOT / 'running.json').write_text(json.dumps({'case': number, 'pid': process.pid, 'started': began}))
        try:
            stdout, stderr = process.communicate(timeout=600)
            exit_code = process.returncode
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGTERM)
            stdout, stderr = process.communicate(timeout=20)
            exit_code = 'timeout'
        ledger.append({'case': number, 'run_name': name, 'started': began, 'ended': time.time(), 'exit_code': exit_code,
                       'stdout': stdout, 'stderr': stderr,
                       'report_exists': (root / 'evidence/report.md').exists(),
                       'results_exist': (root / 'evidence/results.json').exists()})
        (ROOT / 'ledger.json').write_text(json.dumps(ledger, indent=2))
        print(json.dumps(ledger[-1]), flush=True)
        # Ensure the agent's dedicated target cannot carry over to another case.
        subprocess.run(['/opt/homebrew/bin/tmux', '-S', str(sock), 'kill-server'], capture_output=True)
    (ROOT / 'running.json').write_text(json.dumps({'complete': True}))

if __name__ == '__main__':
    main()
