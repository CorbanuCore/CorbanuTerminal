"""Lossless logs, exact gate counts, and scope/change inventory; rerunnable."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess

out=Path(__file__).resolve().parent
repo=out.parents[4]
for path in out.glob("*.log"):
    path.with_suffix(".log.gz").write_bytes(gzip.compress(path.read_bytes(),mtime=0))
# Preserve diagnostic lines and reconstruct the two earlier clock sources,
# accepting them only if they match their contemporaneous manifest hashes.
clock=(out/"fixture_clock.c").read_text()
clock2=clock.replace("// Keep gettimeofday real: parking_lot builds kernel absolute wait deadlines with it.",
                     "INTERPOSE(fixture_gettimeofday, gettimeofday)")
clock1=clock2.replace("// Each process receives an immutable timestamp from the driver.",
                     "// The driver changes this file only between stopped collection processes.")
clock1=clock1.replace('    const char *value = getenv("ACCT_QUALIFY_CLOCK_MS");\n    return value ? strtoll(value, NULL, 10) : 0;',
    '    const char *path = getenv("ACCT_QUALIFY_CLOCK_FILE");\n    FILE *file = path ? fopen(path, "r") : NULL;\n    long long ms = 0;\n    if (file) { if (fscanf(file, "%lld", &ms) != 1) ms = 0; fclose(file); }\n    return ms;')
for name,source in [("run-01",clock1),("run-02",clock2),("run-03",clock)]:
    manifest=json.loads((out/name/"manifest.json").read_text())
    assert hashlib.sha256(source.encode()).hexdigest()==manifest["clock_source_sha256"], name
    (out/name/"clock.c.gz").write_bytes(gzip.compress(source.encode(),mtime=0))
for name in ["run-01","run-02"]:
    raw=(out/name/"fixture/logs/codex-tui.log").read_text()
    errors=[line for line in raw.splitlines() if "request timed out after 30 seconds" in line]
    (out/name/"startup-errors.json").write_text(json.dumps(errors,indent=2)+"\n")
tests={}
for lane in ["core-default","core-feature","tui"]:
    text=(out/(lane+".log")).read_text()
    summary=re.findall(r"Summary .*",text)[-1]
    tests[lane]=dict(summary=summary,failures=re.findall(r"^\s*(?:FAIL|TIMEOUT) .*",text,re.MULTILINE),
                    slow=re.findall(r"^\s*SLOW .*",text,re.MULTILINE))
(out/"test-results.json").write_text(json.dumps(tests,indent=2)+"\n")
tracked=subprocess.check_output(["git","diff","--name-only","-z"],cwd=repo).decode().split("\0")
new=subprocess.check_output(["git","ls-files","--others","--exclude-standard","-z"],cwd=repo).decode().split("\0")
prefix="qa/portfolio/agent-cost-accounting/pf-60-s03/"
assert all(p.startswith(prefix) for p in tracked+new if p), (tracked,new)
inventory=[]
for name in sorted(filter(None,new)):
    p=repo/name
    if out not in p.parents or p.name=="inventory.json":
        continue
    data=p.read_bytes()
    inventory.append(dict(path=str(p.relative_to(out)),sha256=hashlib.sha256(data).hexdigest(),
                          bytes=len(data),lines=None if p.suffix==".gz" else len(data.splitlines())))
counts=dict(existing_diff_numstat=subprocess.check_output(["git","diff","--numstat"],cwd=repo,text=True).strip(),
            qa_program_lines=sum(x["lines"] for x in inventory if x["path"].endswith((".py",".c"))),
            rust_lines_changed=0,production_lines_changed=0,
            text_evidence_lines=sum(x["lines"] for x in inventory if x["lines"] is not None and not x["path"].endswith((".py",".c"))),
            files=len(inventory),outside_scope_paths=[])
(out/"inventory.json").write_text(json.dumps(dict(counts=counts,files=inventory),indent=2)+"\n")
print(json.dumps(counts))
