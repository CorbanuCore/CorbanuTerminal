"""Write results.json (schema 2) for the targeted re-run: the 20 re-run cases from tools/final4.json, and the other
44 cases carried over from the third run (not re-run; their candidate is the third run's).

usage: python3 tools/make_results4.py RECORD_DIR THIRD_RECORD_DIR
"""
import hashlib
import json
import pathlib
import sys

root, third = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
CANDIDATE = "dc61bd6403b99a892b3db4ed59fef22f30a4ff65f1cd2a9ee165e994c2a50483"
THIRD_CANDIDATE = "ad288be98d80d5eb164c2fb34a040565dcc7590a12c58ab2ac3aaf25f0675043"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def ref(path):
    return {"path": str(path.relative_to(root)), "sha256": sha(path)}


def result_file(run):
    for p in (run / "collected/evidence/result.md", run / "collected/work/evidence/result.md"):
        if p.exists():
            return p
    raise FileNotFoundError(run)


def thread(run):
    for line in (run / "executor-events.jsonl").read_text().splitlines():
        if line.startswith('{"type":"thread.started"'):
            return json.loads(line)["thread_id"]


def attempt(label, case):
    run = root / "runs" / label
    iso = json.loads((run / f"isolation-{case}.json").read_text())
    return {
        "agent": thread(run),
        "run_id": iso["run_id"],
        "model": "executor: candidate corbanu exec, glm-5.3-flash (zai, default effort); product under test: glm-5.3-flash (zai)",
        "machine": "UTM guest \"macOS 3\" (macOS 26.2 arm64, agent@192.168.64.3), executor account " + iso["run_id"].split("-")[-2],
        "profile": "fresh per group: product-home/config.toml = harness/product-config.toml (security flags on, Permissive); "
                   "fixture.tar ledger-notes (targeted re-run harness, harness/)",
        "launcher": "sudo -u LANE /opt/agms/bin/agms-launch executor RUN GROUP -> /opt/agms/pkg/bin/corbanu exec; "
                    "product via agms-tui -> agms-product -> /opt/agms/pkg/bin/corbanu",
        "fresh_context": True,
        "code_blind": True,
        "results_blind": True,
        "packet": ref(root / "executor-packet" / "v5" / "prompts" / f"{label}.md"),
        "access_record": ref(run / "executor-events.jsonl"),
        "actions": ref(run / "collected/evidence/actions.jsonl"),
        "isolation_record": ref(run / f"isolation-{case}.json"),
    }


def evidence(label):
    run = root / "runs" / label
    refs = [ref(result_file(run))]
    for name in ("collected/evidence/screens.jsonl", "collected/evidence/fixture.jsonl", "collected/evidence/pty.raw.gz",
                 "mediator-window.jsonl", "guard-after.txt", "executor-start.txt", "executor-end.txt"):
        if (run / name).exists():
            refs.append(ref(run / name))
    return refs


final = json.loads((root / "tools/final4.json").read_text())
design = json.loads((root / "design.json").read_text())
old = {c["id"]: c for c in json.loads((third / "results.json").read_text())["cases"]}
third_results = {"path": "../aggressive-20261013-third/results.json", "sha256": sha(third / "results.json")}
cases = []
for c in design["cases"]:
    case = c["id"]
    if case in final:
        spec = final[case]
        cases.append({"id": case, "disposition": spec["disposition"], "summary": spec["summary"],
                      "raw_executor_verdicts": spec["raw"], "candidate_sha256": CANDIDATE, "method": "tmux",
                      "execution": attempt(spec["final"], case),
                      "evidence": evidence(spec["final"]),
                      "attempts": [{"label": a, "execution": attempt(a, case), "evidence": evidence(a)}
                                   for a in spec["attempts"]]})
    else:
        o = old[case]
        cases.append({"id": case, "disposition": o["disposition"],
                      "summary": "Not re-run in this targeted run; third-run result carried over unchanged (candidate "
                                 "4b6dd55c85, before #752 and #662). Third-run summary: " + o["summary"],
                      "raw_executor_verdicts": o.get("raw_executor_verdicts", {}), "candidate_sha256": THIRD_CANDIDATE,
                      "method": o.get("method", "tmux"), "execution": None, "evidence": [],
                      "carried_from": third_results})
results = {
    "schema_version": 2,
    "design_sha256": sha(root / "design.json"),
    "implementer": "Aggressive fixes merged to main after the third run (#752 for #736/#737/#740, #662 for #505, #724, #742); "
                   "integrator for this record: qualification lane (targeted re-run integrator; built the candidate and harness)",
    "candidate": {
        "version": "0.1.48",
        "source": "origin/main 4440d2d4ce3abce819603392278113993fd63374, clean tree",
        "platform": "macOS arm64 (aarch64-apple-darwin)",
        "binary_sha256": CANDIDATE,
        "package_manifest": ref(root / "candidate/candidate-manifest.md"),
    },
    "scope": "targeted: 20 cases executed on this candidate (tools/final4.json); 44 carried over from the third run",
    "dispositions": ref(root / "dispositions.md"),
    "cases": cases,
}
check = root / "review/evidence-check.md"
if check.exists():
    meta = json.loads((root / "review/evidence-check.json").read_text())
    results["evidence_check"] = {"agent": meta["agent"], "verdict": meta["verdict"], "artifact": ref(check)}
budget = json.loads((root / "tools/review-budget.json").read_text())
results["review_budget"] = {"used": budget["used"], "limit": 5, "ledger": ref(root / "review-ledger.md")}
(root / "results.json").write_text(json.dumps(results, indent=1, ensure_ascii=False) + "\n")
print(sha(root / "results.json"), len(cases))
