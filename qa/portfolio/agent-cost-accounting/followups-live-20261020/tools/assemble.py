"""Copy one group run's host-captured evidence into the record and write per-case isolation receipts.
From PF-60-S10's tools/assemble.py; acc-tart1 additions: the integrator's ledger read (with Request IDs), the
product-home manifest and config, and packing for the repository's blob policy (large JSONL/pty gzipped; the
executor's own folder of screen files packed as one .tar.gz).

usage: CANDIDATE_SHA=... python3 tools/assemble.py RECORD_DIR RUN_DIR LABEL CASE_ID [CASE_ID ...]
"""
import datetime
import gzip
import hashlib
import json
import pathlib
import shutil
import sqlite3
import sys
import tarfile
import tempfile

CANDIDATE = __import__("os").environ["CANDIDATE_SHA"]
GROUP_FIELDS = {
    "repo": "source_denied", "history": "history_denied", "symlink": "symlink_escape_denied",
    "credentials": "credentials_denied", "ipc": "cross_run_ipc_denied", "network": "network_restricted",
    "package": "package_readonly", "child": "children_confined",
}
KEEP = ("evidence", "fixture", "product-logs", "work")
LIMIT = 400 * 1024


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def ref(path, root):
    return {"path": str(pathlib.Path(path).relative_to(root)), "sha256": sha(path)}


def ns(stamp):
    return datetime.datetime.strptime(stamp, "%Y-%m-%dT%H:%M:%SZ").replace(
        tzinfo=datetime.timezone.utc).timestamp() * 1e9


def gz(path):
    with open(path, "rb") as src, gzip.GzipFile(str(path) + ".gz", "wb", mtime=0) as dst:
        shutil.copyfileobj(src, dst)
    path.unlink()


def ledger(home, out):
    """The integrator's read of the run's accounting ledger: one line per attempt, in dispatch order."""
    db = home / "pfterminal_state_5.sqlite"
    with tempfile.TemporaryDirectory() as tmp:
        for suffix in ("", "-wal", "-shm"):
            if (home / ("pfterminal_state_5.sqlite" + suffix)).exists():
                shutil.copy2(home / ("pfterminal_state_5.sqlite" + suffix), pathlib.Path(tmp) / ("s.sqlite" + suffix))
        con = sqlite3.connect(pathlib.Path(tmp) / "s.sqlite")
        rows = [json.loads(r[0]) for r in con.execute("select payload from draft_accounting_attempts")]
        con.close()
    rows.sort(key=lambda r: r.get("dispatched_at_ms", 0))
    lines = ["# thread | turn | provider | model | dispatched (UTC) | request_id | attempt_id (integrator's read of "
             "draft_accounting_attempts after the run)"]
    for r in rows:
        t = datetime.datetime.fromtimestamp(r.get("dispatched_at_ms", 0) / 1000, datetime.timezone.utc)
        lines.append(" | ".join([r.get("thread_id") or "-", r.get("turn") or "-", r.get("provider") or "-",
                                 r.get("model") or "-", t.strftime("%Y-%m-%dT%H:%M:%SZ"),
                                 r.get("request_id") or "-", r.get("attempt_id") or "-"]))
    out.write_text("\n".join(lines) + "\n")
    return len(rows)


def main(record, src, label, cases):
    record, src = pathlib.Path(record), pathlib.Path(src)
    dst = record / "runs" / label
    assert not dst.exists(), f"{dst} exists; runs are write-once"
    dst.mkdir(parents=True)
    with tempfile.TemporaryDirectory() as tmp:
        with tarfile.open(src / "collected.tar") as tar:
            tar.extractall(tmp, filter="fully_trusted")
        home = pathlib.Path(tmp) / "product-home"
        n = ledger(home, dst / "ledger-attempts.txt")
        manifest = []
        for p in sorted(home.rglob("*")):
            if p.is_file():
                manifest.append("%d %s ./%s" % (p.stat().st_size, sha(p), p.relative_to(home)))
        (dst / "product-home.manifest.txt").write_text("\n".join(manifest) + "\n")
        shutil.copy2(home / "config.toml", dst / "product-home-config.toml")
        for name in KEEP:
            if (pathlib.Path(tmp) / name).exists():
                shutil.copytree(pathlib.Path(tmp) / name, dst / "collected" / name, symlinks=True,
                                ignore=shutil.ignore_patterns("objects"))
    ev = dst / "collected" / "evidence"
    for folder in sorted(p for p in ev.iterdir() if p.is_dir()):
        with tarfile.open(str(folder) + ".tar.gz", "w:gz") as tar:
            tar.add(folder, arcname=folder.name)
        shutil.rmtree(folder)
    for extra in ("executor-events.jsonl", "executor.stderr", "executor-exit.txt", "executor-start.txt",
                  "executor-end.txt", "preflight-summary.stdout", "wipe-before.txt", "wipe-after.txt",
                  "guard-before.txt", "guard-after.txt", "BLOCKED"):
        if (src / extra).exists():
            shutil.copy2(src / extra, dst / extra)
    events = [json.loads(l) for l in (dst / "executor-events.jsonl").read_text().splitlines()
              if l.startswith("{\"type\"")]
    thread = next(e["thread_id"] for e in events if e["type"] == "thread.started")
    docs = {}
    for name in ("preflight-launcher.json", "preflight-tmux-child.json", "preflight-executor.json"):
        p = ev / name
        try:
            docs[name] = json.loads(p.read_text())
        except (OSError, ValueError):
            docs[name] = {"ok": False, "missing_or_invalid": True}
    probes = dst / "probes.json"
    probes.write_text(json.dumps(docs, indent=1) + "\n")
    groups = {}
    for doc in docs.values():
        for row in doc.get("probes", []):
            groups.setdefault(row["group"], []).append(row["ok"])
    positive = {r["probe"]: r["ok"] for r in docs["preflight-launcher.json"].get("probes", [])
                if r["group"] == "positive"}
    pty = (ev / "preflight-pty-control.txt").read_text()
    t0, t1 = ns((src / "executor-start.txt").read_text().strip()), ns((src / "executor-end.txt").read_text().strip())
    window = [l for l in (src / "mediator-at-end.jsonl").read_text().splitlines()
              if l.strip() and t0 - 120e9 <= json.loads(l)["time_ns"] <= t1 + 5e9]
    (dst / "mediator-window.jsonl").write_text("\n".join(window) + "\n")
    # Blob policy: gzip anything large (the record README lists what is stored gzipped).
    for p in sorted(dst.rglob("*")):
        if p.is_file() and not p.name.endswith(".gz") and p.stat().st_size > LIMIT:
            gz(p)
    design_hash = sha(record / "design.json")
    summary = {"label": label, "run_id": src.name, "agent": thread, "ledger_attempts": n, "cases": {}}
    for case in cases:
        isolation = {
            "enforcement": "os-enforced",
            "agent": thread,
            "run_id": src.name,
            "case_id": case,
            "group": label,
            "design_sha256": design_hash,
            "candidate_sha256": CANDIDATE,
            "policy": ref(record / "common/effective-policy.txt", record),
            "tool_inventory": ref(record / "common/tool-inventory.json", record),
            "probe_evidence": ref(probes, record),
            "mediator_window": ref(dst / "mediator-window.jsonl", record),
            "packet_readable": positive.get("packet readable and frozen", False),
            "candidate_launchable": positive.get("package launch --version", False)
                                    and positive.get("package binary digest", False),
            "actual_input_available": pty.count("agms-pty-input-control") >= 2,
            "probe_documents_ok": {k: v.get("ok") for k, v in docs.items()},
            "note": "group run: cases share one executor session and one preflight; lanes ran one at a time "
                    "on one Tart clone",
        }
        for group, field in GROUP_FIELDS.items():
            isolation[field] = bool(groups.get(group)) and all(groups[group])
        path = dst / f"isolation-{case}.json"
        path.write_text(json.dumps(isolation, indent=1) + "\n")
        summary["cases"][case] = all(isolation[f] for f in list(GROUP_FIELDS.values()) +
                                     ["packet_readable", "candidate_launchable", "actual_input_available"])
    print(json.dumps(summary, indent=1))


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4:])
