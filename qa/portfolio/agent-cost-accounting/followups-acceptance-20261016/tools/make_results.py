"""Write results.json (schema 2) and dispositions.md for the PF-60-S10 record from tools/final.json.
usage: python3 make_results.py RECORD_DIR
"""
import hashlib, json, pathlib, re, sys

root = pathlib.Path(sys.argv[1])
CANDIDATE = "5b805b9c244e55c12c6f61fc3d31bddd760a415554cbabff8cbcb4c3ba0531a1"
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
ref = lambda p: {"path": str(p.relative_to(root)), "sha256": sha(p)}
final = json.loads((root / "tools/final.json").read_text())
design = json.loads((root / "design.json").read_text())
PACKET = {"A": "executor-packet", "B": "executor-packet", "C": "executor-packet", "D": "executor-packet",
          "E": "executor-packet-v2", "F": "executor-packet-v2", "C2": "executor-packet-v2", "E2": "executor-packet-v3"}
GROUP_CASES = {}
for g in PACKET:
    GROUP_CASES[g] = sorted(p.name[10:-5] for p in (root / "runs" / g).glob("isolation-*.json"))


def thread(run):
    import gzip
    for line in gzip.open(run / "executor-events.jsonl.gz", "rt").read().splitlines():
        if line.startswith('{"type":"thread.started"'):
            return json.loads(line)["thread_id"]


def raw(label, case):
    text = (root / "runs" / label / "collected/evidence/result.md").read_text()
    m = re.findall(r"VERDICT %s: (passed|failed|blocked)" % case, text)
    return m[-1] if m else "missing"


def attempt(label, case):
    run = root / "runs" / label
    iso = json.loads((run / f"isolation-{case}.json").read_text())
    return {
        "agent": thread(run), "run_id": iso["run_id"],
        "model": "executor: candidate corbanu exec, glm-5.3-flash (zai, default effort); product under test: the case's route/model",
        "machine": "UTM guest \"macOS 3\" (macOS 26.2 arm64, agent@192.168.64.3), executor account " + iso["run_id"].split("-")[-2],
        "profile": "fresh per group: product-home/config.toml = harness/product-config.toml + trust entry for the run's project; fixture ledger-notes",
        "launcher": "sudo -u LANE /opt/agms/bin/agms-launch executor RUN GROUP -> /opt/agms/pkg/bin/corbanu exec; product via agms-tui -> agms-product -> /opt/agms/pkg/bin/corbanu",
        "fresh_context": True, "code_blind": True, "results_blind": True,
        "packet": ref(root / PACKET[label] / "prompts" / f"{label}.md"),
        "access_record": ref(run / "executor-events.jsonl.gz"),
        "actions": ref(run / "collected/evidence/actions.jsonl"),
        "isolation_record": ref(run / f"isolation-{case}.json"),
        "raw_executor_verdict": raw(label, case),
    }


def evidence(label):
    run = root / "runs" / label
    names = ["collected/evidence/result.md", "collected/evidence/screens.jsonl.gz", "collected/evidence/fixture.jsonl",
             "collected/evidence/pty.raw.gz", "mediator-window.jsonl", "ledger-attempts.txt", "guard-after.txt",
             "executor-start.txt", "executor-end.txt"]
    return [ref(run / n) for n in names if (run / n).exists()]


cases = []
for c in design["cases"]:
    f = final[c["id"]]
    row = {"id": c["id"], "priority": f.get("priority", c["priority"]), "disposition": f["d"], "summary": f["s"],
           "candidate_sha256": CANDIDATE, "method": "tmux", "execution": attempt(f["run"], c["id"]),
           "evidence": evidence(f["run"])}
    others = [g for g, ids in GROUP_CASES.items() if c["id"] in ids and g != f["run"]]
    if others:
        row["other_attempts"] = [{"label": g, "execution": attempt(g, c["id"]), "evidence": evidence(g)} for g in others]
    cases.append(row)
results = {
    "schema_version": 2,
    "design_sha256": sha(root / "design.json"),
    "implementer": "PF-60-S06..S09 implementers (Codex workers pf60-s06, pf60-next; #859 pf60-moonshot); integrator for this record: pf60-s10 (built the harness, staged the coordinator's signed build)",
    "candidate": {"version": "0.1.48 (developer build with developer-accounting)", "source": "origin/main 395ac0adcd, signed developer build installed by the coordinator 2026-10-16T03:08:51Z (candidate/INSTALL.json)",
                  "platform": "macOS arm64 (aarch64-apple-darwin)", "binary_sha256": CANDIDATE,
                  "package_manifest": ref(root / "candidate/candidate-manifest.md")},
    "dispositions": ref(root / "dispositions.md"),
    "cases": cases,
}
ledger = root / "review-ledger.md"
budget = json.loads((root / "tools/review-budget.json").read_text())
results["review_budget"] = {"used": budget["used"], "limit": 5, "ledger": ref(ledger)}
if budget.get("evidence_check"):
    ec = budget["evidence_check"]
    results["evidence_check"] = {"agent": ec["agent"], "verdict": ec["verdict"], "artifact": ref(root / ec["artifact"])}
(root / "results.json").write_text(json.dumps(results, indent=1, ensure_ascii=False) + "\n")
counts = {}
for r in cases:
    counts[r["disposition"]] = counts.get(r["disposition"], 0) + 1
print(sha(root / "results.json"), counts)
