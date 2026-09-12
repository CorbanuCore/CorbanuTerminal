"""Coordinator-only isolation pilot. Never include this file in an agent packet."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path('/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911')
PACKAGE = Path('/Volumes/CorbanuDrive/Corbanu/.codex-work/provider-reauth-health/macos-candidate-final9-20260910')
AUTH = Path('/Volumes/CorbanuDrive/Corbanu/.codex-work/provider-reauth-health/tmp/live-fresh-0910/home')
PACKET = Path(__file__).parent / 'packet'

def prepare(name):
    root = ROOT / name
    root.mkdir(parents=True, exist_ok=False)
    for directory in ('agent-home', 'target-home', 'tmp', 'work', 'evidence'):
        (root / directory).mkdir(mode=0o700)
    # Both documented absolute paths and accidental work-relative evidence
    # writes resolve to the same run-owned directory without widening access.
    (root / 'work/evidence').symlink_to(root / 'evidence', target_is_directory=True)
    subprocess.run(['/bin/cp', '-cR', str(PACKAGE), str(root / 'package')], check=True)
    shutil.copy2(AUTH / 'auth.json', root / 'agent-home/auth.json')
    for filename in ('auth.json', 'config.toml', 'provider-eligibility.json'):
        if (AUTH / filename).exists():
            shutil.copy2(AUTH / filename, root / 'target-home' / filename)
    shutil.copytree(AUTH / 'secrets', root / 'target-home/secrets')
    # Coordinator-only fixture relocation: fallback filenames are home-bound.
    # Copy the existing encrypted-vault key under the new private home's identity.
    # No credential plaintext is decrypted, logged, or sent to an executor.
    def fallback_name(home):
        account = 'secrets|' + hashlib.sha256(str(home.resolve()).encode()).hexdigest()[:16]
        return hashlib.sha256(b'codex\0' + account.encode()).hexdigest() + '.key'
    donor = AUTH / 'secrets/keyring-fallback' / fallback_name(AUTH)
    if not donor.is_file():
        raise RuntimeError('Expected private fixture fallback key is unavailable')
    shutil.copy2(donor, root / 'target-home/secrets/keyring-fallback' / fallback_name(root / 'target-home'))
    policy = '''(version 1)
(allow default)
(deny file-read* file-write*)
(allow file-read-metadata)
(allow file-read* (literal "/") (subpath "/Library/Apple") (subpath "/System/Library") (subpath "/System/Cryptexes") (subpath "/System/Volumes/Preboot") (subpath "/usr") (subpath "/bin")
  (subpath "/sbin") (subpath "/dev") (subpath "/private/etc")
  (subpath "/private/var/db") (subpath "/opt/homebrew"))
(allow file-read* (subpath "ROOT"))
(allow file-write* (subpath "ROOT") (literal "/dev/null") (literal "/dev/ptmx") (regex #"^/dev/ttys[0-9]+$") (regex #"^/dev/pty.*$"))
(deny file-write* (subpath "ROOT/package") (literal "ROOT/instructions.md"))
(deny appleevent-send)
(deny process-info* (target others))
'''.replace('ROOT', str(root))
    (ROOT / (name + '.sb')).write_text(policy)
    env = {k: v for k, v in os.environ.items() if k in ('LANG', 'LC_ALL')}
    env.update(PATH='/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin', HOME=str(root),
               TMPDIR=str(root / 'tmp'), XDG_CACHE_HOME=str(root / 'tmp/cache'),
               CODEX_HOME=str(root / 'agent-home'), CORBANU_HOME=str(root / 'agent-home'),
               PFTERMINAL_HOME=str(root / 'agent-home'), TERM='xterm-256color', RUST_LOG='warn')
    (ROOT / (name + '-env.json')).write_text(json.dumps(env))
    return root, env

def main():
    args = argparse.ArgumentParser()
    args.add_argument('name')
    args.add_argument('--prepare', action='store_true')
    args.add_argument('--prompt', type=Path)
    ns = args.parse_args()
    root, env = prepare(ns.name) if ns.prepare else (ROOT / ns.name, json.loads((ROOT / (ns.name + '-env.json')).read_text()))
    prefix = ['/usr/bin/sandbox-exec', '-f', str(ROOT / (ns.name + '.sb'))]
    if not ns.prompt:
        probes = []
        link = root / 'work/forbidden-link'
        if not link.exists():
            link.symlink_to('/Volumes/CorbanuDrive/Corbanu/worktrees/provider-reauth-health/AGENTS.md')
        for label, command, expected in [
            ('repository-denied', ['/bin/cat', '/Volumes/CorbanuDrive/Corbanu/worktrees/provider-reauth-health/AGENTS.md'], False),
            ('other-repository-denied', ['/bin/ls', '/Volumes/CorbanuDrive/Corbanu/CorbanuTerminal'], False),
            ('prior-results-denied', ['/bin/cat', '/Volumes/CorbanuDrive/Corbanu/worktrees/provider-reauth-health/qa/provider-auth/pf-58/finish-20260910/dispositions.md'], False),
            ('symlink-escape-denied', ['/bin/cat', str(link)], False),
            ('package-runs', [str(root / 'package/bin/corbanu'), '--version'], True),
            ('tmux-runs', ['/opt/homebrew/bin/tmux', '-V'], True),
        ]:
            result = subprocess.run(prefix + command, cwd=root / 'work', env=env, capture_output=True, text=True)
            passed = result.returncode == 0 if expected else result.returncode != 0 and 'Operation not permitted' in result.stderr
            probes.append(dict(label=label, command=command, returncode=result.returncode, stdout=result.stdout, stderr=result.stderr, passed=passed))
        receipt = dict(probes=probes, passed=all(p['passed'] for p in probes))
        (ROOT / (ns.name + '-isolation.json')).write_text(json.dumps(receipt, indent=2))
        print(json.dumps(receipt, indent=2))
        return
    command = prefix + [str(root / 'package/bin/corbanu'), 'exec', '--skip-git-repo-check', '--ignore-user-config', '--ignore-rules', '--ephemeral', '--dangerously-bypass-approvals-and-sandbox', '-C', str(root / 'work'), '-m', 'gpt-5.6-luna', '-c', 'model_reasoning_effort="max"', '-c', 'cli_auth_credentials_store="file"', '-c', 'features.apps=false', '-c', 'check_for_update_on_startup=false', '--json', '-o', str(root / 'evidence/final.md'), '-']
    with (ROOT / (ns.name + '-events.jsonl')).open('w') as out, (ROOT / (ns.name + '-stderr.log')).open('w') as err:
        result = subprocess.run(command, input=ns.prompt.read_text(), text=True, cwd=root / 'work', env=env, stdout=out, stderr=err)
    print(json.dumps({'exit_code': result.returncode, 'name': ns.name}))

if __name__ == '__main__':
    main()
