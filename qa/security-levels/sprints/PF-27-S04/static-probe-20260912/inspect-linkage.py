#!/usr/bin/env python3
"""Read-only artifact inspection; never executes either target or calls ldd."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

out = Path(sys.argv[1])
static = out / 'candidate/codex-protected-root-probe'
dynamic = Path('/home/travis/security-round5/evidence/pf27-sealed-20260912/verified/candidate/codex-protected-root-probe')
reports = []
for name, path in [('static', static), ('dynamic-control', dynamic)]:
    size = path.stat().st_size
    if not 64 <= size <= 512 * 1024 * 1024:
        raise SystemExit(f'{name}: invalid size')
    with path.open('rb') as handle:
        header = handle.read(64)
        handle.seek(0)
        digest = hashlib.file_digest(handle, 'sha256').hexdigest()
    if not (header[:7] == b'\x7fELF\x02\x01\x01'
            and int.from_bytes(header[16:18], 'little') in (2, 3)
            and header[18:20] == b'\x3e\x00'):
        raise SystemExit(f'{name}: unsupported ELF identity')
    text = subprocess.run(['readelf', '-W', '-h', '-l', '-d', str(path)],
                          check=True, capture_output=True, text=True)
    (out / f'{name}-readelf.txt').write_text(text.stdout)
    (out / f'{name}-readelf-stderr.txt').write_text(text.stderr)
    if text.stderr.strip():
        raise SystemExit(f'{name}: readelf diagnostic needs disposition')
    interp = any(line.strip().startswith('INTERP ') for line in text.stdout.splitlines())
    needed = '(NEEDED)' in text.stdout
    reports.append({'name': name, 'path': str(path), 'size': size,
                    'sha256': digest, 'pt_interp': interp, 'dt_needed': needed})
    if name == 'static' and (interp or needed):
        raise SystemExit('static: external loader or dependencies remain')
    if name == 'dynamic-control' and not (interp and needed):
        raise SystemExit('negative dynamic control missing expected linkage')
(out / 'linkage.json').write_text(json.dumps({
    'linkage_passed': True, 'artifacts_executed': False,
    'runtime_or_privileged_qualification': False, 'artifacts': reports,
}, indent=2) + '\n')
print('PF27_STATIC_LINKAGE_PASS; artifacts not executed')
