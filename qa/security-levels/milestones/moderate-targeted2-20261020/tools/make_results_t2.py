"""Write results.json (schema 2) for the Moderate second targeted re-run: MOD-15 and MOD-28 from tools/final-t2.json,
the owner's 2026-10-19 dispositions (tools/owner-dispositions.json: D39, D41, D43) and the other cases carried over
from the first targeted re-run's record.

usage: python3 tools/make_results_t2.py RECORD_DIR PREVIOUS_TARGETED_RECORD_DIR
"""
import hashlib, json, pathlib, sys

root, prev = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
CANDIDATE = "95ee54cb7ce1bcd3566da613cb415e8677227cb5e11baac63d5a1bbfecf9e509"
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
ref = lambda p: {"path": str(p.relative_to(root)), "sha256": sha(p)}


def thread(run):
    for line in (run / "executor-events.jsonl").read_text().splitlines():
        if line.startswith('{"type":"thread.started"'):
            return json.loads(line)["thread_id"]


def attempt(label, case):
    run = root / "runs" / label
    iso = json.loads((run / f"isolation-{case}.json").read_text())
    return {
        "agent": thread(run), "run_id": iso["run_id"],
        "model": "executor: candidate corbanu exec, glm-5.3-flash (zai, default effort); product under test: glm-5.3-flash (zai)",
        "machine": "UTM guest \"macOS 3\" (macOS 26.2 arm64, agent@192.168.64.3), executor account " + iso["run_id"].split("-")[-2],
        "profile": "fresh per group: product-home/config.toml = harness/product-config.toml (security flags on, Permissive); fixture.tar ledger-notes with the notes MCP fixture (harness/)",
        "launcher": "sudo -u LANE /opt/agms/bin/agms-launch executor RUN GROUP -> /opt/agms/pkg/bin/corbanu exec; product via agms-tui -> agms-product -> /opt/agms/pkg/bin/corbanu",
        "fresh_context": True, "code_blind": True, "results_blind": True,
        "packet": ref(root / "executor-packet/v4/prompts" / f"{label}.md"),
        "access_record": ref(run / "executor-events.jsonl"),
        "actions": ref(run / "collected/evidence/actions.jsonl"),
        "isolation_record": ref(run / f"isolation-{case}.json"),
    }


def evidence(label):
    run = root / "runs" / label
    refs = [ref(run / "collected/evidence/result.md")]
    for name in ("collected/evidence/screens.jsonl", "collected/evidence/fixture.jsonl", "collected/evidence/pty.raw.gz",
                 "collected/product-logs/codex-tui.log", "mediator-window.jsonl", "guard-after.txt",
                 "executor-start.txt", "executor-end.txt"):
        if (run / name).exists():
            refs.append(ref(run / name))
    return refs


final = json.loads((root / "tools/final-t2.json").read_text())
owner = json.loads((root / "tools/owner-dispositions.json").read_text())
design = json.loads((root / "design.json").read_text())
old = {c["id"]: c for c in json.loads((prev / "results.json").read_text())["cases"]}
prev_results = {"path": "../moderate-targeted-20261016/results.json", "sha256": sha(prev / "results.json")}
owner_ref = ref(root / "tools/owner-dispositions.json")
def carry(o, prev_case):
    """Carried cases: keep the earlier record's own carried_from (first run) and say where the evidence paths resolve."""
    o["carried_via"] = prev_results
    if "carried_from" in prev_case:
        o["evidence_paths_relative_to"] = "../moderate-20261014/"
    else:
        o["carried_from"] = prev_results
        o["evidence_paths_relative_to"] = "../moderate-targeted-20261016/"


cases = []
for c in design["cases"]:
    case = c["id"]
    if case in final:
        s = final[case]
        cases.append({"id": case, "disposition": s["disposition"], "summary": s["summary"], "raw_executor_verdicts": s["raw"],
                      "previous_result": {"disposition": old[case]["disposition"], "from": prev_results},
                      "owner_decision": "D38" if case == "MOD-15" else "D40",
                      "candidate_sha256": CANDIDATE, "method": "tmux", "execution": attempt(s["final"], case),
                      "evidence": evidence(s["final"]), "attempts": []})
    elif case in owner:
        o = dict(old[case])
        o.update({"disposition": owner[case]["disposition"], "owner_decision": owner[case]["decision"],
                  "previous_result": {"disposition": old[case]["disposition"], "from": prev_results},
                  "summary": owner[case]["summary"] + " Previous record's summary: " + old[case]["summary"],
                  "owner_disposition_record": owner_ref})
        carry(o, old[case])
        cases.append(o)
    else:
        o = dict(old[case])
        carry(o, old[case])
        cases.append(o)
results = {
    "schema_version": 2,
    "design_sha256": sha(root / "design.json"),
    "implementer": "Fixes merged to main after the first targeted re-run (#949 for #941, #962 for #942; #900 already merged); integrator for this record: qualification lane qual-mod3 (staged the signed integration build and the harness, not the implementer)",
    "candidate": {"version": "0.1.48", "source": "signed integration build of main 8bf182de43f412e1571858ad4b5d747f5420461a (INSTALL.json, built 2026-10-20T03:20:00Z) in the first run's candidate-v2 package",
                  "platform": "macOS arm64 (aarch64-apple-darwin)", "binary_sha256": CANDIDATE,
                  "package_manifest": ref(root / "candidate/candidate-manifest.md")},
    "scope": "targeted: MOD-15 and MOD-28 executed on this candidate under amendment v4; MOD-09, MOD-21, MOD-11 (and MOD-06's git part), MOD-33 dispositioned by owner decisions D39, D43, D41; the other cases carried over from ../moderate-targeted-20261016/results.json",
    "dispositions": ref(root / "dispositions.md"),
    "cases": cases,
}
budget = json.loads((root / "tools/review-budget.json").read_text())
results["review_budget"] = {"used": budget["used"], "limit": 5, "ledger": ref(root / "review-ledger.md")}
(root / "results.json").write_text(json.dumps(results, indent=1, ensure_ascii=False) + "\n")
print(sha(root / "results.json"), len(cases))
