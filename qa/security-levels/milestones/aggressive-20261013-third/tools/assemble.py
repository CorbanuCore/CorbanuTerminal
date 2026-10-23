"""Copy one group run's host-captured evidence into the record and write per-case isolation receipts.

usage: python3 tools/assemble.py RECORD_DIR RUN_DIR LABEL CASE_ID [CASE_ID ...]
  RUN_DIR: host run directory written by run_group.sh; LABEL: record folder name under runs/ (e.g. G01 or
  G03-attempt2). Writes runs/LABEL/... and runs/LABEL/isolation-<CASE>.json; prints a JSON summary.
"""
import datetime
import hashlib
import json
import pathlib
import shutil
import sys
import tarfile

CANDIDATE = "ad288be98d80d5eb164c2fb34a040565dcc7590a12c58ab2ac3aaf25f0675043"
GROUP_FIELDS = {
    "repo": "source_denied", "history": "history_denied", "symlink": "symlink_escape_denied",
    "credentials": "credentials_denied", "ipc": "cross_run_ipc_denied", "network": "network_restricted",
    "package": "package_readonly", "child": "children_confined",
}
KEEP = ("evidence", "fixture", "product-logs", "work")


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def ref(path, root):
    return {"path": str(pathlib.Path(path).relative_to(root)), "sha256": sha(path)}


def ns(stamp):
    return datetime.datetime.strptime(stamp, "%Y-%m-%dT%H:%M:%SZ").replace(
        tzinfo=datetime.timezone.utc).timestamp() * 1e9


def main(record, src, label, cases):
    record, src = pathlib.Path(record), pathlib.Path(src)
    dst = record / "runs" / label
    assert not dst.exists(), f"{dst} exists; runs are write-once"
    dst.mkdir(parents=True)
    with tarfile.open(src / "collected.tar") as tar:
        members = [m for m in tar.getmembers()
                   if m.name.split("/")[0] in KEEP and "/.git/objects/" not in m.name
                   and (m.isfile() or m.isdir())]
        tar.extractall(dst / "collected", members=members)
    for extra in ("executor-events.jsonl", "executor.stderr", "executor-exit.txt", "executor-start.txt",
                  "executor-end.txt", "preflight-summary.stdout", "wipe-before.txt", "wipe-after.txt",
                  "guard-before.txt", "guard-after.txt", "BLOCKED"):
        if (src / extra).exists():
            shutil.copy2(src / extra, dst / extra)
    ev = dst / "collected" / "evidence"
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
    design_hash = sha(record / "design.json")
    summary = {"label": label, "run_id": src.name, "agent": thread, "cases": {}}
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
            "note": "group run: cases share one executor session and one preflight; lanes ran one at a time",
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
