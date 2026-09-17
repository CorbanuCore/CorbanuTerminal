"""Exact numeric audit of persisted TUI selections against independent wire arithmetic."""
from decimal import Decimal
import json
from pathlib import Path
import sys

run=Path(sys.argv[1])
load=lambda name: json.loads((run/name).read_text())
results=load("results.json")
rows=load("independent-arithmetic.json")
rates={"gpt-5.6-sol":("5",".5","30"),"gpt-5.6-terra":("2.5",".25","15")}
for row in rows:
    a,b,c=map(Decimal,rates[row["model"]])
    exact=((row["input"]-row["cached"])*a+row["cached"]*b+row["output"]*c)/Decimal(1000000)
    assert exact==Decimal(row["exact_usd"])
assert results["passed"]
checked=0
for path in sorted(run.glob("*-selected.json")):
    selected=json.loads(path.read_text())
    exact=[s for s in selected if s.startswith(("Known subtotal exact USD: ","Known estimate exact USD: "))]
    if not exact:
        continue
    if any(s.startswith("Attempt: ") for s in selected):
        model=next(s.removeprefix("Model: ") for s in selected if s.startswith("Model: "))
        inp=int(next(s.removeprefix("Input: ") for s in selected if s.startswith("Input: ")))
        candidates=[r for r in rows if r["model"]==model and r["input"]==inp]
        assert len(candidates)==1,(path,candidates)
        expected=Decimal(candidates[0]["exact_usd"])
    elif path.name.startswith("group-"):
        index=int(path.name.split("-")[1])
        expected=sum((Decimal(rows[i]["exact_usd"]) for i in results["groups"][index]["emissions"]),Decimal(0))
    else:
        # Full bucket headers identify membership independently of results.json.
        import re
        from datetime import datetime
        interval=next((s for s in selected if s.startswith("Bucket: [")),None)
        if interval:
            start,end=re.search(r"Bucket: \[([^,]+), ([^)]+)\)",interval).groups()
            stamp=lambda x: datetime.fromisoformat(x.replace("Z","+00:00"))
            members=[r for r in rows if r["kind"]!="orphan" and stamp(start)<=stamp(r["fixture_utc"])<stamp(end)]
        elif path.name.startswith("unknown-"):
            members=[r for r in rows if r["kind"]=="root" and r["fixture_utc"]=="2026-09-01T00:00:00Z"]
        else:
            raise AssertionError(("Unmapped total",path))
        expected=sum((Decimal(r["exact_usd"]) for r in members),Decimal(0))
    assert all(Decimal(s.split(": ",1)[1])==expected for s in exact),(path,exact,expected)
    checked+=1
receipt=dict(exact_numeric_pages_checked=checked,buckets=len(results["buckets"]),
             groups=len(results["groups"]),unknown_causes=len(results["unknown_causes"]),
             passed=True,expected_source="loopback emitted counts and hardcoded independent rates")
(run/"numeric-audit.json").write_text(json.dumps(receipt,indent=2)+"\n")
print(json.dumps(receipt))
