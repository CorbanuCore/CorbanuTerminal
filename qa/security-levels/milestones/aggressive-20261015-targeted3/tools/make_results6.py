"""Write results.json (schema 2) for the third targeted re-run: the re-run cases from tools/final6.json (executed here,
or judged on the second targeted run's recorded runs where the owner decided so), the 4 cases the owner accepted without
execution (#447, carried), and the other cases carried over from the second targeted run's record.

usage: python3 tools/make_results6.py RECORD_DIR TARGETED2_RECORD_DIR
"""
import hashlib, json, pathlib, sys

root, prev = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
CANDIDATE = "d8fc95094766e489ab319a110dc8b943d16a9fd6467c4f27a3f6ec739251fdf4"
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
ref = lambda p: {"path": str(p.relative_to(root)), "sha256": sha(p)}


def thread(run):
    for line in (run / "executor-events.jsonl").read_text().splitlines():
        if line.startswith('{"type":"thread.started"'):
            return json.loads(line)["thread_id"]


def attempt(label, case):
    run = root / "runs" / label
    iso = json.loads((run / f"isolation-{case}.json").read_text())
    out = {
        "agent": thread(run), "run_id": iso["run_id"],
        "model": "executor: candidate corbanu exec, glm-5.3-flash (zai, default effort); product under test: glm-5.3-flash (zai)",
        "machine": "UTM guest \"macOS 3\" (macOS 26.2 arm64, agent@192.168.64.3), executor account " + iso["run_id"].split("-")[-2],
        "profile": "fresh per group: product-home/config.toml = harness/product-config.toml (security flags on, Permissive); fixture.tar ledger-notes (harness/)",
        "launcher": "sudo -u LANE /opt/agms/bin/agms-launch executor RUN GROUP -> /opt/agms/pkg/bin/corbanu exec; product via agms-tui (or agms-drive -> agms-tui) -> agms-product -> /opt/agms/pkg/bin/corbanu",
        "fresh_context": True, "code_blind": True, "results_blind": True,
        "packet": ref(root / "executor-packet/v7/prompts" / f"{label.split('-')[0]}.md"),
        "access_record": ref(run / "executor-events.jsonl"),
        "actions": ref(run / "collected/evidence/actions.jsonl"),
        "isolation_record": ref(run / f"isolation-{case}.json"),
    }
    drv = root / "executor-packet/v7/drivers" / f"{case}.json"
    if drv.exists():
        out["keystroke_driver"] = ref(drv)
        log = run / "collected/evidence" / f"driver-{case}.log"
        if log.exists():
            out["driver_log"] = ref(log)
    return out


def evidence(label):
    run = root / "runs" / label
    refs = [ref(run / r) for r in ("collected/evidence/result.md", "collected/work/evidence/result.md") if (run / r).exists()]
    for name in ("collected/evidence/screens.jsonl", "collected/evidence/fixture.jsonl", "collected/evidence/pty.raw.gz",
                 "collected/evidence/pty-2.raw.gz", "mediator-window.jsonl", "guard-after.txt", "executor-start.txt", "executor-end.txt"):
        if (run / name).exists():
            refs.append(ref(run / name))
    refs += [ref(p) for p in sorted((run / "collected/evidence").glob("driver-*.log"))]
    if (run / "product-log-excerpt.txt").exists():
        refs.append(ref(run / "product-log-excerpt.txt"))
    return refs


final = json.loads((root / "tools/final6.json").read_text())
design = json.loads((root / "design.json").read_text())
old = {c["id"]: c for c in json.loads((prev / "results.json").read_text())["cases"]}
prev_results = {"path": "../aggressive-20261014-targeted2/results.json", "sha256": sha(prev / "results.json")}
cases = []
for c in design["cases"]:
    case = c["id"]
    if case in final:
        spec = final[case]
        row = {"id": case, "disposition": spec["disposition"], "summary": spec["summary"],
               "raw_executor_verdicts": spec["raw"], "previous_result": {"disposition": old[case]["disposition"], "from": prev_results}}
        if spec.get("judged_on_previous_run"):
            o = old[case]
            row.update({"candidate_sha256": o.get("candidate_sha256"), "method": o.get("method"),
                        "execution": o.get("execution"), "evidence": o.get("evidence", []),
                        "judged_on": {"record": prev_results, "run": spec["judged_on_previous_run"],
                                      "note": "not re-run (owner decision, #816/#447 2026-10-15): judged against amendment 6 on the second targeted run's recorded screens; paths are relative to that record"}})
        else:
            row.update({"candidate_sha256": CANDIDATE, "method": "tmux", "execution": attempt(spec["final"], case),
                        "evidence": evidence(spec["final"]),
                        "attempts": [{"label": a, "execution": attempt(a, case), "evidence": evidence(a)} for a in spec["attempts"]]})
            if spec.get("parts"):
                row["parts"] = {k: {"label": v, "execution": attempt(v, case), "evidence": evidence(v)} for k, v in spec["parts"].items()}
        cases.append(row)
    else:
        o = dict(old[case])
        o["summary"] = "Not re-run in this third targeted run; the second targeted run's record carried over unchanged. Its summary: " + o["summary"]
        o["carried_from"] = prev_results
        cases.append(o)
results = {
    "schema_version": 2,
    "design_sha256": sha(root / "design.json"),
    "implementer": "Aggressive fixes merged to main after the second targeted run (#813 for #739, #834 for #816, #835 for #817); integrator for this record: qualification lane (third targeted re-run integrator; built the candidate and harness)",
    "candidate": {"version": "0.1.48", "source": "origin/main 4c9b0a0cf038e70add1c5ec4df511bfcfe4ac2c5, clean tree",
                  "platform": "macOS arm64 (aarch64-apple-darwin)", "binary_sha256": CANDIDATE,
                  "package_manifest": ref(root / "candidate/candidate-manifest.md")},
    "scope": "third targeted: AGG-T01 (parts A and B), AGG-T02, AGG-R03, AGG-R14 executed on this candidate; AGG-04 and AGG-37 judged on the second targeted run's recorded runs (owner decision); others carried over from the second targeted run's record",
    "dispositions": ref(root / "dispositions.md"),
    "cases": cases,
}
budget = json.loads((root / "tools/review-budget.json").read_text())
results["review_budget"] = {"used": budget["used"], "limit": 5, "ledger": ref(root / "review-ledger.md")}
(root / "results.json").write_text(json.dumps(results, indent=1, ensure_ascii=False) + "\n")
print(sha(root / "results.json"), len(cases))
