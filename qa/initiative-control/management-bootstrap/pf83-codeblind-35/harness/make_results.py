"""Write results.json (schema 2) for campaign 35 from the assembled record and final dispositions.

usage: make_results.py RECORD_DIR [REVIEWER_THREAD VERDICT ARTIFACT]
"""
import hashlib
import json
import pathlib
import sys

record = pathlib.Path(sys.argv[1]).resolve()
CANDIDATE = "e552180a2dfc5457a93881cc4822054e18a4a90e8bf86f67008238e461e758f6"
LANES = {"pf83x1": 602, "pf83x2": 603, "pf83x3": 604}
FINAL = {  # case: (attempt dir used for the disposition, disposition, summary)
    "F01": ("F01", "passed", "Restricted probe gated; Full Access applied for next turn via confirmation; next-turn probe ran without approval; fixture shows both markers."),
    "F02": ("F02", "passed", "Restriction applied for next turn; the pending interval was idle, so its probe was the first turn after the boundary; that probe and the one after application were both gated, declined, markers absent."),
    "F03": ("F03-attempt2", "blocked", "Core outcome not demonstrated: attempt 1 selected while idle and the product model refused the out-of-workspace probe; attempt 2 pressed Enter on the default Cancel each time, so Full Access never applied (its 'failed' is a misread)."),
    "F04": ("F04-attempt2", "blocked", "Post-effective gating passed twice; the active-turn admission window was exercised only with a non-discriminating workspace probe, which the product steered into the running Full Access turn; attempt 2's turn ended before selection."),
    "F05": ("F05", "failed", "Declined request rendered as both 'You canceled' and 'Ran … (no output)', contradicting an understandable disposition; core 'select Full Access while approval pending' not performed; typing into the approval modal approved with persistent scope."),
    "F06": ("F06", "passed", "Both directions: in-flight slow op completed with both markers; change disclosed as next-turn; post-boundary probe followed the new level."),
    "F07": ("F07", "blocked", "Starting state (active harmless turn) never established; all selections idle; conflicting selection was typed text into a modal."),
    "F08": ("F08", "blocked", "Cancellation exercised and accurate from restricted only (next probe gated); cancellation from Full Access not exercised; failure branch substituted with a command denial."),
    "F09": ("F09", "blocked", "All selections idle; continuation through the application path not exercised."),
    "F10": ("F10-attempt2", "blocked", "Attempt 2 met every observed outcome (resume after restart for variants a-d; levels persisted/resolved truthfully; pending approval restored as interrupted, never accepted; post-restart probes matched). Blocked because required actions deviate without product-authority acceptance: variants chained in one session, restart by tmux kill not in-app quit, the offered Esc recovery route for pending states not exercised, (c) pre-stop header claim unscreened. Attempt 1 (no resume) does not qualify."),
    "F11": ("F11", "blocked", "zai-anthropic route errors before any request (coverage limit); zai repeats inherit F03-F05 gaps; route switch happened after Full Access was already effective."),
}
ATTEMPTS = {"F03": ["F03", "F03-attempt2"], "F04": ["F04", "F04-attempt2"], "F10": ["F10", "F10-attempt2"]}


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def ref(path):
    path = record / path
    return {"path": str(path.relative_to(record)), "sha256": sha(path)}


def execution(attempt, case):
    base = record / "runs" / attempt
    iso = json.loads((base / "isolation.json").read_text())
    lane = iso["run_id"].split("-")[2]
    result = next(p for p in (base / "collected/evidence/result.md", base / "collected/work/evidence/result.md") if p.exists())
    evidence = [result.relative_to(record)] + [p.relative_to(record) for p in sorted((base / "collected/evidence").glob("*"))
                                               if p.name != "result.md"]
    evidence += [p.relative_to(record) for p in sorted((base / "collected/product-home").rglob("*.jsonl"))]
    evidence += [(base / "mediator-window.jsonl").relative_to(record), (base / "executor.stderr").relative_to(record)]
    return {
        "agent": iso["agent"],
        "run_id": iso["run_id"],
        "model": "glm-5.2 via provider zai (campaign mediator), corbanu exec default reasoning effort",
        "machine": "UTM VM 'macOS 3' 4B1BAF7F-F655-4BF9-81DC-7DEFABA23BA6, IOPlatformUUID F9AABE1C-BA2C-55D7-9859-A6ED560D3218, macOS 26.2 (25C56) arm64, 192.168.64.3",
        "profile": "fresh per-run product CORBANU_HOME and executor-home under /Users/%s/runs/%s; account %s uid %d; lane wiped before and after" % (lane, iso["run_id"], lane, LANES[lane]),
        "launcher": "agent -> sudo -n -u %s -H /opt/pf83/bin/pf83-launch executor %s %s -> /opt/pf83/pkg/corbanu exec; product via pf83-tui -> /opt/pf83/bin/pf83-product -> /opt/pf83/pkg/corbanu" % (lane, iso["run_id"], case),
        "fresh_context": True,
        "code_blind": True,
        "results_blind": True,
        "packet": ref("packet/prompts/%s.md" % case),
        "access_record": ref("runs/%s/executor-events.jsonl" % attempt),
        "actions": ref("runs/%s/collected/evidence/actions.jsonl" % attempt),
        "isolation_record": ref("runs/%s/isolation.json" % attempt),
    }, [ {"path": str(p), "sha256": sha(record / p)} for p in evidence ]


cases = []
for case, (attempt, disposition, summary) in FINAL.items():
    run, evidence = execution(attempt, case)
    row = {"id": case, "disposition": disposition, "summary": summary, "candidate_sha256": CANDIDATE,
           "method": "tmux", "execution": run, "evidence": evidence,
           "disposition_record": ref("dispositions.md")}
    others = [a for a in ATTEMPTS.get(case, []) if a != attempt]
    if others:
        row["other_attempts"] = []
        for a in others:
            _, other_evidence = execution(a, case)
            row["other_attempts"].append({"attempt": a, "isolation_record": ref("runs/%s/isolation.json" % a),
                                          "access_record": ref("runs/%s/executor-events.jsonl" % a),
                                          "evidence": other_evidence})
    cases.append(row)

results = {
    "schema_version": 2,
    "design_sha256": sha(record / "design.json"),
    "implementer": "Astra High permission-confirmation worker (PF-83-S01 owner)",
    "candidate": {
        "version": "0.1.42",
        "source": "commit b4513f6cc699773a7ca922d08b67683f47586a96 tree d6a3d723e0591cdd65f5a13e41fc8b9f92ff2c49 (package attested in increment-27; archive 7296e9b7bf45413465e3df1dc17ea6f87720e52dcaa862f05d9aa94d43c73995)",
        "platform": "macOS arm64 (guest macOS 26.2)",
        "binary_sha256": CANDIDATE,
        "package_manifest": ref("candidate-manifest.json"),
    },
    "cases": cases,
    "aborted_attempts": [ref(str(p.relative_to(record))) for p in sorted((record / "runs-aborted").glob("*/ABORTED.txt"))],
    "review_budget": {
        "used": 7,
        "limit": 5,
        "ledger": ref("review-ledger.md"),
        "extension": {"by": "PF-83 integrator-side worker under the September 12 root delegation (manager to confirm)",
                      "reason": "Evidence checks r1 and r2 failed on fixable evidence/representation defects; corrective re-checks r2 and r3.",
                      "artifact": ref("review-ledger.md")},
    },
}
if len(sys.argv) > 4:
    results["evidence_check"] = {"agent": sys.argv[2], "verdict": sys.argv[3], "artifact": ref(sys.argv[4])}
(record / "results.json").write_text(json.dumps(results, indent=1) + "\n")
print(sha(record / "results.json"), {c["id"]: c["disposition"] for c in cases})
