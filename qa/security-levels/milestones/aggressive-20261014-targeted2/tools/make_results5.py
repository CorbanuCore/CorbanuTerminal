"""Write results.json (schema 2) for the second targeted re-run: the 4 re-run cases from tools/final5.json, the 4 cases
the owner accepted without execution (#447, out_of_scope with the recorded approval), and the other 56 cases carried
over from the first targeted run's record (which itself carries 44 of them from the third run).

usage: python3 tools/make_results5.py RECORD_DIR TARGETED_RECORD_DIR
"""
import hashlib
import json
import pathlib
import sys

root, prev = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
CANDIDATE = "0deeb58c7882cc9bb246fe2b423b6f258d2eb39561917b3884221cab7e1049b3"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def ref(path):
    return {"path": str(path.relative_to(root)), "sha256": sha(path)}


def thread(run):
    for line in (run / "executor-events.jsonl").read_text().splitlines():
        if line.startswith('{"type":"thread.started"'):
            return json.loads(line)["thread_id"]


def attempt(label, case):
    run = root / "runs" / label
    iso = json.loads((run / f"isolation-{case}.json").read_text())
    out = {
        "agent": thread(run),
        "run_id": iso["run_id"],
        "model": "executor: candidate corbanu exec, glm-5.3-flash (zai, default effort); product under test: glm-5.3-flash (zai)",
        "machine": "UTM guest \"macOS 3\" (macOS 26.2 arm64, agent@192.168.64.3), executor account " + iso["run_id"].split("-")[-2],
        "profile": "fresh per group: product-home/config.toml = harness/product-config.toml (security flags on, Permissive); "
                   "fixture.tar ledger-notes (harness/)",
        "launcher": "sudo -u LANE /opt/agms/bin/agms-launch executor RUN GROUP -> /opt/agms/pkg/bin/corbanu exec; "
                    "product via agms-tui (or agms-drive -> agms-tui) -> agms-product -> /opt/agms/pkg/bin/corbanu",
        "fresh_context": True,
        "code_blind": True,
        "results_blind": True,
        "packet": ref(root / "executor-packet" / "v6" / "prompts" / f"{label}.md"),
        "access_record": ref(run / "executor-events.jsonl"),
        "actions": ref(run / "collected/evidence/actions.jsonl"),
        "isolation_record": ref(run / f"isolation-{case}.json"),
    }
    drv = root / "executor-packet/v6/drivers" / f"{case}.json"
    if drv.exists():
        out["keystroke_driver"] = ref(drv)
        out["driver_log"] = ref(run / "collected/evidence" / f"driver-{case}.log")
    return out


def evidence(label):
    run = root / "runs" / label
    refs = [ref(run / "collected/evidence/result.md")]
    for name in ("collected/evidence/screens.jsonl", "collected/evidence/fixture.jsonl", "collected/evidence/pty.raw.gz",
                 "mediator-window.jsonl", "guard-after.txt", "executor-start.txt", "executor-end.txt"):
        if (run / name).exists():
            refs.append(ref(run / name))
    refs += [ref(p) for p in sorted((run / "collected/evidence").glob("driver-*.log"))]
    return refs


final = json.loads((root / "tools/final5.json").read_text())
design = json.loads((root / "design.json").read_text())
old = {c["id"]: c for c in json.loads((prev / "results.json").read_text())["cases"]}
prev_results = {"path": "../aggressive-20261014-targeted/results.json", "sha256": sha(prev / "results.json")}
decision = root / "design/decisions/issue-447.json"
cases = []
for c in design["cases"]:
    case = c["id"]
    if case in final and final[case]["disposition"] == "out_of_scope":
        spec = final[case]
        cases.append({"id": case, "disposition": "out_of_scope", "summary": spec["summary"],
                      "approval": {"by": "Travis (owner), decision comment on #447 posted by the coordinator, 2026-10-14T22:58:14Z",
                                   "reason": "the four model-refusal blocks (AGG-18, R07, R10, 20) are accepted as covered by the "
                                             "product refusals shown elsewhere, AGG-20 recorded as a named limit",
                                   "artifact": ref(decision)},
                      "previous_result": {"disposition": old[case]["disposition"], "from": prev_results},
                      "execution": None, "evidence": []})
    elif case in final:
        spec = final[case]
        cases.append({"id": case, "disposition": spec["disposition"], "summary": spec["summary"],
                      "raw_executor_verdicts": spec["raw"], "candidate_sha256": CANDIDATE, "method": "tmux",
                      "execution": attempt(spec["final"], case),
                      "evidence": evidence(spec["final"]),
                      "attempts": [{"label": a, "execution": attempt(a, case), "evidence": evidence(a)}
                                   for a in spec["attempts"]]})
    else:
        o = dict(old[case])
        o["summary"] = ("Not re-run in this second targeted run; the first targeted run's record carried over unchanged. "
                        "Its summary: " + o["summary"])
        o["carried_from"] = prev_results
        cases.append(o)
results = {
    "schema_version": 2,
    "design_sha256": sha(root / "design.json"),
    "implementer": "Aggressive fixes merged to main after the first targeted run (#797 for #787, #798 for #789, #799 for #792, #801); "
                   "integrator for this record: qualification lane (second targeted re-run integrator; built the candidate and harness)",
    "candidate": {
        "version": "0.1.48",
        "source": "origin/main df3d8c5d1c521228a8dbb1035831af572afe1284, clean tree",
        "platform": "macOS arm64 (aarch64-apple-darwin)",
        "binary_sha256": CANDIDATE,
        "package_manifest": ref(root / "candidate/candidate-manifest.md"),
    },
    "scope": "second targeted: 4 cases executed on this candidate, 4 accepted by the owner without execution (tools/final5.json); "
             "56 carried over from the first targeted run's record",
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
