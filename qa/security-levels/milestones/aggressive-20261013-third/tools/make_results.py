"""Write results.json (schema 2) from the assembled runs and the integrator's dispositions.

usage: python3 tools/make_results.py RECORD_DIR
Reads tools/final.json (case -> disposition, final attempt label, all attempt labels, summary).
"""
import hashlib
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
CANDIDATE = "ad288be98d80d5eb164c2fb34a040565dcc7590a12c58ab2ac3aaf25f0675043"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def ref(path):
    return {"path": str(path.relative_to(root)), "sha256": sha(path)}


def result_file(run):
    for p in (run / "collected/evidence/result.md", run / "collected/work/evidence/result.md"):
        if p.exists():
            return p
    nested = sorted(run.glob("collected/work/runs/*/evidence/result.md"))
    if nested:
        return nested[0]
    raise FileNotFoundError(run)


def thread(run):
    for line in (run / "executor-events.jsonl").read_text().splitlines():
        if line.startswith('{"type":"thread.started"'):
            return json.loads(line)["thread_id"]


def attempt(label, case):
    run = root / "runs" / label
    packet = root / "executor-packet" / "v4" / "prompts" / f"{label}.md"
    product_model = "glm-5.3-flash" if label != "G17" else "glm-5.3-flash (AGG-27); glm-5.2 on route zai-anthropic (AGG-A01, as the case requires)"
    return {
        "agent": thread(run),
        "run_id": json.loads((run / f"isolation-{case}.json").read_text())["run_id"],
        "model": f"executor: candidate corbanu exec, glm-5.3-flash (zai, default effort); product under test: {product_model}",
        "machine": "UTM guest \"macOS 3\" (macOS 26.2 arm64, agent@192.168.64.3), executor account " +
                   json.loads((run / f"isolation-{case}.json").read_text())["run_id"].split("-")[-2],
        "profile": "fresh per group: product-home/config.toml = harness/product-config.toml (security flags on, Permissive); "
                   "fixture.tar ledger-notes (third-run harness, harness/)",
        "launcher": "sudo -u LANE /opt/agms/bin/agms-launch executor RUN GROUP -> /opt/agms/pkg/bin/corbanu exec; "
                    "product via agms-tui -> agms-product -> /opt/agms/pkg/bin/corbanu",
        "fresh_context": True,
        "code_blind": True,
        "results_blind": True,
        "packet": ref(packet),
        "access_record": ref(run / "executor-events.jsonl"),
        "actions": ref(run / "collected/evidence/actions.jsonl") if (run / "collected/evidence/actions.jsonl").exists() else None,
        "isolation_record": ref(run / f"isolation-{case}.json"),
    }


def evidence(label):
    run = root / "runs" / label
    refs = [ref(result_file(run))]
    for name in ("collected/evidence/screens.jsonl", "collected/evidence/fixture.jsonl", "collected/evidence/pty.raw.gz", "mediator-window.jsonl",
                 "guard-after.txt", "executor-start.txt", "executor-end.txt"):
        if (run / name).exists():
            refs.append(ref(run / name))
    return refs


final = json.loads((root / "tools/final.json").read_text())
design = json.loads((root / "design.json").read_text())
assert [c["id"] for c in design["cases"]] == list(final), "every frozen case, in order"
cases = []
for case, spec in final.items():
    row = {"id": case, "disposition": spec["disposition"], "summary": spec["summary"],
           "raw_executor_verdicts": spec["raw"], "candidate_sha256": CANDIDATE, "method": "tmux",
           "execution": attempt(spec["final"], case) if spec["final"] else None,
           "evidence": (evidence(spec["final"]) if spec["final"] else []) + [ref(root / x) for x in spec.get("extra_evidence", [])],
           "attempts": [{"label": a, "execution": attempt(a, case), "evidence": evidence(a)} for a in spec["attempts"]]}
    cases.append(row)
results = {
    "schema_version": 2,
    "design_sha256": sha(root / "design.json"),
    "implementer": "Aggressive decision and defect fixes merged to main after the second run (#637 #638 #639 #640 #644 #655 #657 #661 #664 and others); "
                   "integrator for this record: qualification lane (third-run integrator; built the candidate and harness)",
    "candidate": {
        "version": "0.1.48",
        "source": "origin/main 4b6dd55c85291dcf4458c285f0fc675a0121fd75, clean tree",
        "platform": "macOS arm64 (aarch64-apple-darwin)",
        "binary_sha256": CANDIDATE,
        "package_manifest": ref(root / "candidate/candidate-manifest.md"),
    },
    "dispositions": ref(root / "dispositions.md"),
    "cases": cases,
}
check = root / "review/evidence-check.md"
if check.exists():
    meta = json.loads((root / "review/evidence-check.json").read_text())
    results["evidence_check"] = {"agent": meta["agent"], "verdict": meta["verdict"], "artifact": ref(check)}
ledger = root / "review-ledger.md"
budget = json.loads((root / "tools/review-budget.json").read_text())
results["review_budget"] = {"used": budget["used"], "limit": 5, "extension": budget.get("extension", 0), "ledger": ref(ledger)}
(root / "results.json").write_text(json.dumps(results, indent=1, ensure_ascii=False) + "\n")
print(sha(root / "results.json"), len(cases))
