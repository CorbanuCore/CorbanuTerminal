"""Independent Decimal/window audit of saved boundary pages; no driver-pass flag."""
from datetime import datetime
from decimal import Decimal
import json
from pathlib import Path
import re
import sys

run = Path(sys.argv[1])
rows = json.loads((run/"independent-arithmetic.json").read_text())
cases = json.loads((run/"boundary-results.json").read_text())
rates = {"gpt-5.6-sol":("5",".5","30"),"gpt-5.6-terra":("2.5",".25","15")}
stamp = lambda s: datetime.fromisoformat(s.replace("Z","+00:00"))
for row in rows:
    a,b,c = map(Decimal,rates[row["model"]])
    row["usd"] = ((row["input"]-row["cached"])*a+row["cached"]*b+row["output"]*c)/Decimal(1000000)
    assert row["usd"]==Decimal(row["exact_usd"])
receipt = []
for case in cases:
    members = [r for r in rows if stamp(case["start"])<=stamp(r["fixture_utc"])<stamp(case["end"])
               and not (case.get("deleted_child") and r["kind"]=="child")]
    total = sum((r["usd"] for r in members),Decimal(0))
    assert [r["emission"] for r in members]==case["emissions"]
    assert total==Decimal(case["exact_usd"])
    metrics = {
        "Input":sum(r["input"] for r in members),
        "Noncached input (derived for inclusive input)":sum(r["input"]-r["cached"] for r in members),
        "Cache read":sum(r["cached"] for r in members),
        "Cache write":0,
        "Output":sum(r["output"] for r in members),
        "Reasoning (subset, not separately billed)":sum(r["reasoning"] for r in members),
        "Total (not separately billed)":sum(r["input"]+r["output"] for r in members)}
    numeric = 0
    for name,lines in case["rendered"].items():
        assert lines==json.loads((run/(name+"-selected.json")).read_text())
        exact = [s for s in lines if s.startswith("Known subtotal exact USD: ")]
        if case["expected_state"].startswith("unavailable"):
            assert not exact, (name,exact)
        elif exact:
            assert len(exact)==1 and Decimal(exact[0].split(": ",1)[1])==total
            for key,value in metrics.items():
                assert f"{key}: {value} known + unknown in 0 attempts" in lines, (name,key)
            numeric += 1
        admission = next((s for s in lines if s.startswith("UTC admission interval: ")), None)
        if admission:
            a,b = map(int,admission.split("[",1)[1].split(")",1)[0].split(","))
            assert (a,b)==(int(stamp(case["start"]).timestamp()*1000),int(stamp(case["end"]).timestamp()*1000))
        if name==case["case"]:
            interval = next((s for s in lines if s.startswith("Bucket: [")),None)
            if interval:
                start,end = re.search(r"Bucket: \[([^,]+), ([^)]+)\)",interval).groups()
                assert (stamp(start),stamp(end))==(stamp(case["start"]),stamp(case["end"]))
    if case["expected_state"].startswith("available"):
        assert numeric>=1,case["case"]
    receipt.append(dict(case=case["case"],emissions=case["emissions"],exact_usd=str(total),
                        metrics=metrics,expected_state=case["expected_state"],numeric_pages=numeric))
# DST wall-clock jumps do not alter UTC bucket duration/membership.
spring = [c for c in receipt if c["case"].startswith("spring-") and not c["case"].endswith("day")]
assert len(spring)==8
assert len({tuple(c["emissions"]) for c in spring if c["case"].endswith("-0")})==1
assert len({tuple(c["emissions"]) for c in spring if c["case"].endswith("-1")})==1
offset = json.loads((run/"offset-input-result.json").read_text())["rendered"]
assert "Range refused: timestamps must use UTC Z" in offset
# Store values are checked against wire arithmetic; they never define it.
state = json.loads((run/"mixed-store-readback.json").read_text())
checkpoint = state["checkpoint"][0][1]
cutoff = checkpoint-90*86400000
retained = [r for r in rows if r["kind"]!="child"]
raw_expected = [r for r in retained if int(stamp(r["fixture_utc"]).timestamp()*1000)>cutoff]
raw_actual = [json.loads(r[1]) for r in state["raw"]]
assert sorted((r["model"],int(stamp(r["fixture_utc"]).timestamp()*1000)) for r in raw_expected)==sorted((r["model"],r["dispatched_at_ms"]) for r in raw_actual)
compact_checks = []
for thread, day, payload in state["compact"]:
    actual = json.loads(payload)
    expected = [r for r in retained if int(stamp(r["fixture_utc"]).timestamp()*1000)<=cutoff
                and int(stamp(r["fixture_utc"]).timestamp())//86400==day]
    usd = sum((r["usd"] for r in expected),Decimal(0))
    known = [sum(r["input"] for r in expected),sum(r["input"]-r["cached"] for r in expected),
             sum(r["cached"] for r in expected),0,sum(r["output"] for r in expected),
             sum(r["reasoning"] for r in expected),sum(r["input"]+r["output"] for r in expected)]
    assert Decimal(actual["known_usd"])==usd and actual["known"]==known
    assert actual["attempts"]==len(expected) and actual["unknown"]==[0]*7
    compact_checks.append(dict(day=day,emissions=[r["emission"] for r in expected],exact_usd=str(usd),metrics=known))
expected_days = {int(stamp(r["fixture_utc"]).timestamp())//86400 for r in retained if int(stamp(r["fixture_utc"]).timestamp()*1000)<=cutoff}
assert {r[1] for r in state["compact"]}==expected_days
with Path(sys.argv[2]).open("x") as output:
    output.write(json.dumps(dict(cases=receipt,passed=True,raw_attempts_checked=len(raw_actual),compact_days=compact_checks,
        membership="Driver windows + emitted timestamp/model/count tuples; deletion controlled through native API.",
        amount_source="Independent fixed-rate Decimal calculations; no persisted subtotal supplies an expectation.",
        timezone="UTC hours stay 3600s and UTC days 86400s; local spring gap/fall repeat never redefines a bucket.",
        offset_input="Range refused: timestamps must use UTC Z"),indent=2)+"\n")
print(f"{len(receipt)} independent boundary cases audited")
