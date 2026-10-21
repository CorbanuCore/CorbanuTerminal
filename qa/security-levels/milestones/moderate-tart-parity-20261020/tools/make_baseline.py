"""Per-case UTM baseline for the Tart parity run (tart-par1), from the signed-off Moderate record's results.json
(moderate-targeted2-20261020 carries every case's latest UTM attempt).  usage: make_baseline.py MILESTONES_DIR OUT_JSON"""
import json
import pathlib
import sys

m = pathlib.Path(sys.argv[1])
rec = json.loads((m / "moderate-targeted2-20261020/results.json").read_text())
CAND = {"abd3e6ad": ("moderate-20261014", "df3d8c5d1c", "v2"),
        "cb42614c": ("moderate-targeted-20261016", "4ef3be8467", "v3"),
        "95ee54cb": ("moderate-targeted2-20261020", "8bf182de43", "v4")}
out = []
for c in rec["cases"]:
    sha = c["candidate_sha256"]
    record, commit, amend = CAND[sha[:8]]
    run_id = c["execution"]["run_id"]
    group = run_id.split("-")[1]
    model = "glm-5.2" if c["id"] in ("MOD-09", "MOD-21") and amend == "v3" else "glm-5.3-flash"
    out.append({
        "id": c["id"], "utm_record": record, "utm_group": group, "utm_run_id": run_id,
        "utm_candidate_commit": commit, "utm_candidate_sha256": sha, "judged_under": amend,
        "product_model": model, "utm_executor_verdicts": c.get("raw_executor_verdicts"),
        "utm_disposition": c["disposition"], "owner_decision": c.get("owner_decision"),
        "utm_summary": c["summary"],
    })
pathlib.Path(sys.argv[2]).write_text(json.dumps(out, indent=1) + "\n")
print(len(out))
