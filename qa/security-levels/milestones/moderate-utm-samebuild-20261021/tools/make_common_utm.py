"""utm-par2: copy each evidence folder (UTM: common/utm; Tart clones: common/T*-cur-*) into the record's common/
(mediator logs gzipped, the clone's IP dropped), the host relay log gzipped into common/host/, and write
common/tool-inventory.json and common/evidence-folders.json.   usage: make_common_utm.py WORK_DIR RECORD_DIR"""
import collections
import gzip
import json
import pathlib
import shutil
import sys

work, rec = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
out = rec / "common"
out.mkdir(exist_ok=True)
sets = collections.Counter()
refused = collections.Counter()
folders = []
for c in sorted(p for p in (work / "common").iterdir() if p.is_dir() and p.name != "host"):
    dst = out / c.name
    if dst.exists():
        shutil.rmtree(dst)
    dst.mkdir()
    for f in sorted(c.rglob("*")):
        if f.is_dir() or f.name == "vm-ip.txt":
            continue
        rel = f.relative_to(c)
        (dst / rel).parent.mkdir(parents=True, exist_ok=True)
        if f.name == "mediator-final.jsonl":
            with open(f, "rb") as i, gzip.GzipFile(dst / (str(rel) + ".gz"), "wb", mtime=0) as o:
                shutil.copyfileobj(i, o)
        else:
            shutil.copy2(f, dst / rel)
    med = c / "mediator-final.jsonl"
    fwd = non200 = errs = 0
    for line in med.read_text().splitlines() if med.exists() else []:
        try:
            e = json.loads(line)
        except ValueError:
            continue
        if e.get("event") == "forwarded":
            fwd += 1
            non200 += str(e.get("status")) != "200"
            if e.get("tools"):
                sets[(e.get("role"), tuple(e["tools"]))] += 1
        elif e.get("event") == "session_error":
            errs += 1
        elif e.get("event") == "denied_proxy_request":
            refused[e.get("request_line", "").split(" ")[1] if " " in e.get("request_line", "") else "?"] += 1
    tunnel = (c / "tunnel.log").read_text() if (c / "tunnel.log").exists() else ""
    folders.append({"evidence_folder": c.name, "forwarded": fwd, "non_200": non200, "session_error": errs,
                    "tunnel_connects": tunnel.count(" connect agent@"), "tunnel_ssh_exits": tunnel.count("ssh exited")})
host = out / "host"
host.mkdir(exist_ok=True)
with open(work / "common/host/host-zai-relay.jsonl", "rb") as i, gzip.GzipFile(host / "host-zai-relay.jsonl.gz", "wb", mtime=0) as o:
    shutil.copyfileobj(i, o)
relay = [json.loads(l) for l in (work / "common/host/host-zai-relay.jsonl").read_text().splitlines() if l.strip()]
(out / "tool-inventory.json").write_text(json.dumps({
    "note": "Model-visible tool sets observed by the guest mediators (UTM guest and Tart clones), by request role "
            "(executor = the tester's corbanu exec; product = the TUI under test). Names only; no bodies or headers.",
    "observed_tool_sets": [{"role": r, "tools": list(t), "requests": n} for (r, t), n in sorted(sets.items())],
    "refused_connects": dict(sorted(refused.items())),
}, indent=1) + "\n")
(out / "evidence-folders.json").write_text(json.dumps({
    "folders": folders,
    "host_relay": {"requests": sum("status" in e for e in relay),
                   "non_200": sum("status" in e and str(e["status"]) != "200" for e in relay),
                   "other_events": collections.Counter(e.get("event") for e in relay if "status" not in e)},
}, indent=1, default=dict) + "\n")
print(json.dumps(folders), sum("status" in e for e in relay))
