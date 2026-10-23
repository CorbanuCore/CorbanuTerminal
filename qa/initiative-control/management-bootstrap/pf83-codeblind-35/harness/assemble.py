"""Assemble schema-2 records for campaign 35 from host-captured run directories.

usage: assemble.py RECORD_DIR RUNS_DIR CASE=RUN_DIR_NAME [...]
Copies evidence into RECORD_DIR/runs/CASE/, writes isolation.json per case, and
prints a JSON stub per case (dispositions are written separately by the integrator).
"""
import hashlib
import json
import pathlib
import shutil
import sys
import tarfile

DESIGN = "design.json"
CANDIDATE = "e552180a2dfc5457a93881cc4822054e18a4a90e8bf86f67008238e461e758f6"
GROUP_FIELDS = {
    "repo": "source_denied", "history": "history_denied", "symlink": "symlink_escape_denied",
    "credentials": "credentials_denied", "ipc": "cross_run_ipc_denied", "network": "network_restricted",
    "package": "package_readonly", "child": "children_confined",
}


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def ref(path, root):
    return {"path": str(pathlib.Path(path).relative_to(root)), "sha256": sha(path)}


def main(record, runs, pairs):
    record, runs = pathlib.Path(record), pathlib.Path(runs)
    design_hash = sha(record / DESIGN)
    out = {}
    for pair in pairs:
        label, name = pair.split("=", 1)
        case = label.split("@")[0]
        src = runs / name
        dst = record / "runs" / (label.replace("@", "-attempt"))
        if dst.exists():
            shutil.rmtree(dst)
        dst.mkdir(parents=True)
        with tarfile.open(src / "collected.tar") as tar:
            tar.extractall(dst / "collected")
        for extra in ("executor-events.jsonl", "executor.stderr", "executor-exit.txt",
                      "executor-start.txt", "executor-end.txt", "preflight-summary.stdout",
                      "wipe-before.txt", "wipe-after.txt"):
            if (src / extra).exists():
                shutil.copy2(src / extra, dst / extra)
        ev = dst / "collected" / "evidence"
        result = next((p for p in (ev / "result.md", dst / "collected/work/evidence/result.md",
                                   dst / "collected/work/result.md") if p.exists()), None)
        events = [json.loads(l) for l in (dst / "executor-events.jsonl").read_text().splitlines()
                  if l.startswith("{\"type\"")]
        thread = next(e["thread_id"] for e in events if e["type"] == "thread.started")
        docs = {}
        for name_ in ("preflight-launcher.json", "preflight-tmux-child.json", "preflight-executor.json"):
            p = ev / name_
            docs[name_] = json.loads(p.read_text()) if p.exists() else {"ok": False, "missing": True}
        probes = dst / "probes.json"
        probes.write_text(json.dumps(docs, indent=1) + "\n")
        groups = {}
        for doc in docs.values():
            for row in doc.get("probes", []):
                groups.setdefault(row["group"], []).append(row["ok"])
        positive = {r["probe"]: r["ok"] for r in docs["preflight-launcher.json"].get("probes", [])
                    if r["group"] == "positive"}
        pty = (ev / "preflight-pty-control.txt").read_text()
        isolation = {
            "enforcement": "os-enforced",
            "agent": thread,
            "run_id": name,
            "case_id": case,
            "design_sha256": design_hash,
            "candidate_sha256": CANDIDATE,
            "policy": ref(record / "common/effective-policy.txt", record),
            "tool_inventory": ref(record / "common/tool-inventory.json", record),
            "probe_evidence": ref(probes, record),
            "packet_readable": positive.get("packet readable and frozen", False),
            "candidate_launchable": positive.get("package launch --version", False)
                                    and positive.get("package binary digest", False),
            "actual_input_available": pty.count("pf83-pty-input-control") >= 2,
            "probe_documents_ok": {k: v.get("ok") for k, v in docs.items()},
        }
        for group, field in GROUP_FIELDS.items():
            isolation[field] = bool(groups.get(group)) and all(groups[group])
        start = (src / "executor-start.txt").read_text().strip()
        end = (src / "executor-end.txt").read_text().strip()
        import datetime
        t0 = datetime.datetime.strptime(start, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=datetime.timezone.utc).timestamp() * 1e9
        t1 = datetime.datetime.strptime(end, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=datetime.timezone.utc).timestamp() * 1e9
        window = [l for l in (record / "common/mediator-campaign.jsonl").read_text().splitlines()
                  if t0 - 60e9 <= json.loads(l)["time_ns"] <= t1 + 5e9]
        (dst / "mediator-window.jsonl").write_text("\n".join(window) + "\n")
        isolation["mediator_window"] = ref(dst / "mediator-window.jsonl", record)
        isolation["mediator_window_note"] = "all mediator rows from preflight to executor end; concurrent lanes share the mediator"
        (dst / "isolation.json").write_text(json.dumps(isolation, indent=1) + "\n")
        out[label] = {
            "run_id": name, "agent": thread, "result": str(result.relative_to(record)) if result else None,
            "isolation_ok": all(isolation[f] for f in list(GROUP_FIELDS.values()) +
                                ["packet_readable", "candidate_launchable", "actual_input_available"]),
        }
    print(json.dumps(out, indent=1))


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2], sys.argv[3:])
