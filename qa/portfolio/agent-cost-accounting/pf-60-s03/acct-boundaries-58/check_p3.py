"""Non-destructive negative controls; synthetic mutations live only in a temp QA directory."""
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import sqlite3
import subprocess
import tempfile

out = Path(__file__).resolve().parent
prior = out.parent / "acct-qualify-55"
spec = importlib.util.spec_from_file_location("audit", prior/"audit.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
run = out/"p3-run-01"
proof = {}
with tempfile.TemporaryDirectory(prefix="audit-control-", dir=out) as temp:
    copy = Path(temp)
    for path in list(run.glob("*-selected.json"))+[run/"results.json",run/"independent-arithmetic.json"]:
        shutil.copyfile(path, copy/path.name)
    results = json.loads((copy/"results.json").read_text())
    results["passed"] = False
    (copy/"results.json").write_text(json.dumps(results))
    proof["false_driver_pass_flag_does_not_gate_audit"] = module.audit(copy)
    target = copy/"week-edge-hours-0-selected.json"
    original = target.read_text()
    # A mislabelled bucket with its corresponding, internally consistent amount
    # passed the old product-derived membership check. The driver-window check rejects it.
    changed = json.loads(original)
    changed = [s.replace("Bucket: [2026-08-30T23:00:00.000Z, 2026-08-31T00:00:00.000Z)",
                         "Bucket: [2026-08-31T00:00:00.000Z, 2026-08-31T01:00:00.000Z)")
               .replace("Known subtotal exact USD: 0.00196","Known subtotal exact USD: 0.00072")
               for s in changed]
    assert changed != json.loads(original)
    target.write_text(json.dumps(changed))
    try:
        module.audit(copy)
        raise RuntimeError("mislabelled bucket escaped")
    except AssertionError as error:
        assert "wrong bucket label" in str(error)
        proof["consistent_mislabel_and_amount_rejected"] = str(error).replace(str(copy),"<synthetic-copy>")
    target.write_text(original.replace("Known subtotal exact USD: 0.00196","Known subtotal exact USD: 0.00197"))
    try:
        module.audit(copy)
        raise RuntimeError("wrong amount escaped")
    except AssertionError as error:
        proof["wrong_amount_rejected"] = str(error).replace(str(copy),"<synthetic-copy>")

causes = json.loads((run/"results.json").read_text())["unknown_causes"]
assert len(causes)==5
assert len({c["readback"]["inspect_pid"] for c in causes})==5
assert len({(c["readback"]["source"],tuple(c["readback"]["edge_parents"])) for c in causes})==5
for case in causes:
    actual = case["readback"]
    assert actual["source"]==case["source"]
    assert actual["edge_parents"]==([case["edge_parent"]] if case["edge_parent"] else [])
proof["five_causes"] = [dict(cause=c["cause"],pid=c["readback"]["inspect_pid"],
    sha256=hashlib.sha256((run/("unknown-"+c["cause"]+"-store-readback.json")).read_bytes()).hexdigest())
    for c in causes]
# Counterfactual: reusing the first readback cannot pass the cause match.
proof["stale_first_cause_readback_rejected"] = [
    c["cause"] for c in causes[1:] if
    (causes[0]["readback"]["source"],causes[0]["readback"]["edge_parents"]) !=
    (c["source"],[c["edge_parent"]] if c["edge_parent"] else [])]
assert len(proof["stale_first_cause_readback_rejected"])==4

historical = out.parent/"acct-qualify-50/inspect-01/utc-alignment.json"
before = historical.read_bytes(), historical.stat().st_mtime_ns
subprocess.run(["python3",str(prior/"verify_utc_alignment.py")],check=True)
after = historical.read_bytes(), historical.stat().st_mtime_ns
assert before==after
proof["historical_receipt_unchanged"] = dict(sha256=hashlib.sha256(before[0]).hexdigest(),mtime_ns=before[1])
# Execute the producer against synthetic history, then tamper that historical
# receipt to prove byte mismatch is fatal. No real historical artifact is touched.
with tempfile.TemporaryDirectory(prefix="utc-control-",dir=out) as temp:
    root = Path(temp)
    old = root/"acct-qualify-50/inspect-01"
    old.mkdir(parents=True)
    new = root/"acct-qualify-55"
    new.mkdir()
    for name in ["hour-overview-selected.json","week-overview-selected.json","month-overview-selected.json","utc-alignment.json"]:
        shutil.copyfile(historical.parent/name,old/name)
    shutil.copyfile(prior/"verify_utc_alignment.py",new/"verify_utc_alignment.py")
    (old/"utc-alignment.json").write_text("[]\n")
    result = subprocess.run(["python3",str(new/"verify_utc_alignment.py")],capture_output=True,text=True)
    assert result.returncode!=0 and "Historical receipt mismatch" in result.stderr
    proof["historical_mismatch_rejected"] = dict(exit=result.returncode, error="Historical receipt mismatch")

wire = json.loads((run/"requests.json").read_text())
checkpoint = [r for r in wire if r["kind"]=="checkpoint"]
assert len(checkpoint)==1 and checkpoint[0]["fixture_utc"]=="2026-10-02T12:00:00Z"
assert len([r for r in wire if r["kind"]=="orphan"])==1
identity = json.loads((run/"identities.json").read_text())
with sqlite3.connect(next((run/"fixture/profile").glob("*state_*.sqlite"))) as db:
    rows = [json.loads(r[0]) for r in db.execute("SELECT payload FROM draft_accounting_attempts")]
check = [r for r in rows if r["dispatched_at_ms"]==1790942400000]
assert len(check)==1 and check[0]["thread_id"]==identity["root"]
proof["checkpoint"] = dict(role=checkpoint[0]["kind"],selected_root=identity["root"],
                          stored_attempt=check[0],outside_all_compared_ranges=True)
with (out/"p3-negative-controls.json").open("x") as f:
    f.write(json.dumps(proof,indent=2)+"\n")
print("P3: independent label/amount rejection, five cause reads/PIDs, immutable history/mismatch, selected-root checkpoint verified")
