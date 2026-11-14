"""Write results.json (schema 2) for the Moderate targeted re-run: the 17 re-run cases from tools/final-t.json, MOD-11
pending owner decision D5, and the other 16 cases carried over from the first run's record.

usage: python3 tools/make_results_t.py RECORD_DIR FIRST_RUN_RECORD_DIR
"""
import hashlib, json, pathlib, sys

root, prev = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
CANDIDATE = "cb42614c8cc194bf8fc5d598e61e49883384a6747051d92884d5c41f905b0d25"
GLM52 = {"MOD-09", "MOD-21"}
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
ref = lambda p: {"path": str(p.relative_to(root)), "sha256": sha(p)}


def thread(run):
    for line in (run / "executor-events.jsonl").read_text().splitlines():
        if line.startswith('{"type":"thread.started"'):
            return json.loads(line)["thread_id"]


def attempt(label, case):
    run = root / "runs" / label
    iso = json.loads((run / f"isolation-{case}.json").read_text())
    product = "glm-5.2 (zai; amendment v3 R-1)" if case in GLM52 else "glm-5.3-flash (zai; routes as each case requires)"
    return {
        "agent": thread(run), "run_id": iso["run_id"],
        "model": f"executor: candidate corbanu exec, glm-5.3-flash (zai, default effort); product under test: {product}",
        "machine": "UTM guest \"macOS 3\" (macOS 26.2 arm64, agent@192.168.64.3), executor account " + iso["run_id"].split("-")[-2],
        "profile": "fresh per group: product-home/config.toml = harness/product-config.toml (security flags on, Permissive); fixture.tar ledger-notes with the notes MCP fixture (harness/)",
        "launcher": "sudo -u LANE /opt/agms/bin/agms-launch executor RUN GROUP -> /opt/agms/pkg/bin/corbanu exec; product via agms-tui -> agms-product -> /opt/agms/pkg/bin/corbanu",
        "fresh_context": True, "code_blind": True, "results_blind": True,
        "packet": ref(root / "executor-packet/v3/prompts" / f"{label}.md"),
        "access_record": ref(run / "executor-events.jsonl"),
        "actions": ref(run / "collected/evidence/actions.jsonl"),
        "isolation_record": ref(run / f"isolation-{case}.json"),
    }


def evidence(label):
    run = root / "runs" / label
    refs = [ref(run / r) for r in ("collected/evidence/result.md", "collected/work/evidence/result.md") if (run / r).exists()]
    for name in ("collected/evidence/screens.jsonl", "collected/evidence/fixture.jsonl", "collected/evidence/pty.raw.gz",
                 "collected/evidence/pty-B.raw.gz", "collected/product-logs/codex-tui.log", "mediator-window.jsonl",
                 "guard-after.txt", "executor-start.txt", "executor-end.txt"):
        if (run / name).exists():
            refs.append(ref(run / name))
    return refs


final = json.loads((root / "tools/final-t.json").read_text())
design = json.loads((root / "design.json").read_text())
old = {c["id"]: c for c in json.loads((prev / "results.json").read_text())["cases"]}
prev_results = {"path": "../moderate-20261014/results.json", "sha256": sha(prev / "results.json")}
cases = []
for c in design["cases"]:
    case = c["id"]
    if case in final:
        s = final[case]
        cases.append({"id": case, "disposition": s["disposition"], "summary": s["summary"], "raw_executor_verdicts": s["raw"],
                      "previous_result": {"disposition": old[case]["disposition"], "from": prev_results},
                      "candidate_sha256": CANDIDATE, "method": "tmux", "execution": attempt(s["final"], case),
                      "evidence": evidence(s["final"]),
                      "attempts": [{"label": a, "execution": attempt(a, case), "evidence": evidence(a)} for a in s["attempts"]]})
    elif case == "MOD-11":
        o = dict(old[case])
        o.update({"disposition": "pending_owner_decision", "previous_result": {"disposition": old[case]["disposition"], "from": prev_results},
                  "summary": "Not re-run: pending owner decision D5 (#837, draft #875 'needs security sign-off'), per the coordinator's brief; recorded as pending, not failed. First-run summary: " + old[case]["summary"],
                  "carried_from": prev_results})
        cases.append(o)
    else:
        o = dict(old[case])
        o["summary"] = "Not re-run in this targeted run; the first run's record carried over unchanged. Its summary: " + o["summary"]
        o["carried_from"] = prev_results
        cases.append(o)
results = {
    "schema_version": 2,
    "design_sha256": sha(root / "design.json"),
    "implementer": "Moderate fixes merged to main after the first run (#813, #849, #850, #851, #854, #869, #870, #877, #881, #883, #892); integrator for this record: qualification lane qual-mod2 (staged the signed integration build and the harness, not the implementer)",
    "candidate": {"version": "0.1.48", "source": "signed integration build of main 4ef3be846789765e149c75a0cbe9af7c6bec6726 (INSTALL.json, built 2026-10-16T06:48:15Z) in the first run's candidate-v2 package",
                  "platform": "macOS arm64 (aarch64-apple-darwin)", "binary_sha256": CANDIDATE,
                  "package_manifest": ref(root / "candidate/candidate-manifest.md")},
    "scope": "targeted: 17 cases executed on this candidate under amendment v3; MOD-11 pending owner decision D5 (and MOD-06's git-commit part); 16 cases carried over from the first run's record",
    "dispositions": ref(root / "dispositions.md"),
    "cases": cases,
}
budget = json.loads((root / "tools/review-budget.json").read_text())
results["review_budget"] = {"used": budget["used"], "limit": 5, "ledger": ref(root / "review-ledger.md")}
(root / "results.json").write_text(json.dumps(results, indent=1, ensure_ascii=False) + "\n")
print(sha(root / "results.json"), len(cases))
