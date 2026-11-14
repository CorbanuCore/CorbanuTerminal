"""Copy one group run's host-captured evidence into the Tart parity record and write per-case isolation receipts.
From qual-mod3's assemble.py. Product homes and product logs are not exported (tester copies inside evidence/ are listed
by size and SHA-256 only); each fixture's .git is renamed dot-git without objects; pty.raw and the executor events are
gzipped.   usage: python3 tools/assemble_par.py RECORD_DIR RUN_DIR CLONE_DIR LABEL CASE_ID [CASE_ID ...]
"""
import datetime
import gzip
import hashlib
import json
import pathlib
import shutil
import sys
import tarfile

CANDIDATE = "15cb32e9407cc346286d39f24514bb391027db10d2aceb12c1c2798e482c4ae6"
GROUP_FIELDS = {
    "repo": "source_denied", "history": "history_denied", "symlink": "symlink_escape_denied",
    "credentials": "credentials_denied", "ipc": "cross_run_ipc_denied", "network": "network_restricted",
    "package": "package_readonly", "child": "children_confined",
}
KEEP = ("evidence", "fixture", "work")


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def ref(path, root):
    return {"path": str(pathlib.Path(path).relative_to(root)), "sha256": sha(path)}


def ns(stamp):
    return datetime.datetime.strptime(stamp, "%Y-%m-%dT%H:%M:%SZ").replace(
        tzinfo=datetime.timezone.utc).timestamp() * 1e9


def gz(src, dst):
    with open(src, "rb") as i, gzip.GzipFile(dst, "wb", mtime=0) as o:
        shutil.copyfileobj(i, o)


def main(record, src, clone, label, cases):
    record, src, clone = pathlib.Path(record), pathlib.Path(src), pathlib.Path(clone)
    dst = record / "runs" / label
    assert not dst.exists(), f"{dst} exists; runs are write-once"
    (dst / "collected").mkdir(parents=True)
    withheld = []
    with tarfile.open(src / "collected.tar") as tar:
        for m in tar.getmembers():
            parts = m.name.split("/")
            if parts[0] not in KEEP or any("product-home" in p for p in parts):
                if m.isfile():
                    data = tar.extractfile(m).read()
                    withheld.append(f"{m.size} {hashlib.sha256(data).hexdigest()} {m.name}")
                continue
            if "/.git/objects/" in m.name + "/" or not (m.isfile() or m.isdir()):
                continue
            out = dst / "collected" / "/".join("dot-git" if p == ".git" else p for p in parts)
            if m.isdir():
                out.mkdir(parents=True, exist_ok=True)
                continue
            out.parent.mkdir(parents=True, exist_ok=True)
            data = tar.extractfile(m).read()
            if out.name == "pty.raw":
                with gzip.GzipFile(str(out) + ".gz", "wb", mtime=0) as o:
                    o.write(data)
            else:
                out.write_bytes(data)
    (dst / "withheld-files.txt").write_text(
        "# Not exported (product homes, product logs, tester copies of product homes): size sha256 path\n"
        + "\n".join(sorted(withheld)) + "\n")
    for extra in ("executor.stderr", "executor-exit.txt", "executor-start.txt", "executor-end.txt",
                  "preflight-summary.stdout", "wipe-before.txt", "wipe-after.txt", "guard-before.txt",
                  "guard-after.txt", "memory-before.txt", "memory-after.txt", "gate-check.txt", "BLOCKED"):
        if (src / extra).exists():
            shutil.copy2(src / extra, dst / extra)
    gz(src / "executor-events.jsonl", dst / "executor-events.jsonl.gz")
    ev = dst / "collected" / "evidence"
    events = [json.loads(l) for l in (src / "executor-events.jsonl").read_text().splitlines()
              if l.startswith("{\"type\"")]
    thread = next(e["thread_id"] for e in events if e["type"] == "thread.started")
    docs = {}
    for name in ("preflight-launcher.json", "preflight-tmux-child.json", "preflight-executor.json"):
        try:
            docs[name] = json.loads((ev / name).read_text())
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
    summary = {"label": label, "run_id": src.name, "clone": clone.name, "agent": thread, "cases": {}}
    for case in cases:
        isolation = {
            "enforcement": "os-enforced",
            "agent": thread,
            "run_id": src.name,
            "case_id": case,
            "group": label,
            "machine": f"Tart clone of corbanu-qa-base (evidence folder common/{clone.name})",
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
            "note": "group run: cases share one executor session and one preflight; one fresh Tart clone per group",
        }
        for group, field in GROUP_FIELDS.items():
            isolation[field] = bool(groups.get(group)) and all(groups[group])
        (dst / f"isolation-{case}.json").write_text(json.dumps(isolation, indent=1) + "\n")
        summary["cases"][case] = all(isolation[f] for f in list(GROUP_FIELDS.values()) +
                                     ["packet_readable", "candidate_launchable", "actual_input_available"])
    print(json.dumps(summary))


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5:])
