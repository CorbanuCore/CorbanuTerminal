"""Independent same-store/day and actual-key drill-down reconciliation."""
from datetime import datetime, timezone
from decimal import Decimal
import gzip
import json
from pathlib import Path
import re
import sys

here = Path(__file__).resolve().parent
prior = here.parent / "acct-readers-61"
sys.path.insert(0, str(prior))
from p3_checks import assert_day

run = Path(sys.argv[1]).resolve()
output = Path(sys.argv[2]).resolve()
def read(name):
    return json.loads((run/name).read_text())

result = read("scope-zero-result.json")
before = read("store-before-fresh-reader.json")
assert before == read("store-during-fresh-reader.json")
emissions = read("emissions.json")
assert len(emissions) == 4
assert all((e["provider"],e["model"],e["fixture_utc"],e["usage_present"],
            e["input"],e["cached"],e["cache_write"],e["output"],e["reasoning"]) ==
           ("openai","gpt-5.6-sol","2026-08-10T12:00:00Z",True,100,20,0,10,4)
           for e in emissions)
# Frozen tariff and wire usage, not a sum taken on trust from the driver/DB.
expected = (Decimal(80)*5 + Decimal(20)*Decimal(".5") + Decimal(10)*30)/1000000
assert expected == Decimal(".00071")
attempts = [json.loads(a["payload"]) for a in before["draft_accounting_attempts"]]
assert len(attempts) == 4 and len({a["attempt_id"] for a in attempts}) == 4
assert len({a["thread_id"] for a in attempts}) == 3
assert all(a["dispatched_at_ms"] == 1786363200000 for a in attempts)
contributions = before["draft_accounting_contributions"]
assert {c["attempt_id"] for c in contributions} == {a["attempt_id"] for a in attempts}
assert all(c["utc_day"] == 20675 for c in contributions)
for contribution in contributions:
    # Eight quote revisions exist: select the contribution's actual evidence key,
    # not the earlier missing-usage revision and not a sum of all revisions.
    matches = [q for q in before["draft_accounting_estimates"]
               if (q["attempt_id"],q["evidence"]) ==
                  (contribution["attempt_id"],contribution["evidence"])]
    assert len(matches) == 1
    q = json.loads(matches[0]["payload"])
    assert q["attempt"] in attempts
    assert q["usage"] == dict(input=100,noncached=80,read=20,write=0,output=10,reasoning=4,total=110)
    assert q["snapshot"]["rates"] == dict(noncached="5",read="0.5",write=None,output="30")
    assert Decimal(q["known_subtotal"]) == Decimal(q["all_buckets_priced"]) == expected
    assert q["observations"] == json.loads(contribution["evidence"])

pages = {p.name.removesuffix("-selected.json"): json.loads(p.read_text())
         for p in run.glob("*-selected.json")}
assert result["rendered"] == pages["fresh-scope-zero"]
for phrase in ("No recorded attempts in this day; collection coverage unknown.",
               "Known subtotal exact USD: 0", "oldest recorded day: None"):
    assert phrase in " ".join(pages["fresh-scope-zero"])
assert_day(gzip.decompress((run/"fresh-scope-zero.txt.gz").read_bytes()).decode(),"2026-08-10")
expected_priced = {"root-a-control","root-b-control","root-c-control",
                  "root-a-control-Request-1-Attempt-1","root-a-control-Request-2-Attempt-1",
                  "root-b-control-Request-1-Attempt-1","root-c-control-Request-1-Attempt-1"}
actual_priced = {name for name,lines in pages.items()
                 if "Estimated token cost for recorded attempts:" in " ".join(lines)}
assert actual_priced == expected_priced, (expected_priced,actual_priced)
seen_attempts = set()
for name in expected_priced:
    text = " ".join(pages[name])
    amount = expected * (2 if name == "root-a-control" else 1)
    assert re.findall(r"Known subtotal exact USD: ([0-9.]+)", text) == [str(amount)]
    assert re.findall(r"Estimated token cost for recorded attempts: \$([0-9.]+)", text) == [format(amount, ".6f")]
    assert "Billed cost: unavailable — no settlement evidence" in text
    if "Attempt-1" in name:
        identity = next(line.removeprefix("Attempt: ") for line in pages[name] if line.startswith("Attempt: "))
        assert identity not in seen_attempts
        seen_attempts.add(identity)
assert seen_attempts == {a["attempt_id"] for a in attempts}

old = prior/"reader-run-05"
old_bindings = [(json.loads(a), None if s is None else json.loads(s))
                for a,s in json.loads((old/"native-bindings.json").read_text())]
old_rows = []
for a,s in old_bindings:
    day = datetime.fromtimestamp(a["dispatched_at_ms"]/1000,timezone.utc).date().isoformat()
    # The known no-usage thread is identified from the original fixture receipt.
    identity = json.loads((old/"identities.json").read_text())
    has_usage = a["thread_id"] != identity["no_usage"]
    old_rows.append(dict(attempt=a["attempt_id"],thread=a["thread_id"],day=day,
                         model=a["model"],price_snapshot_present=s is not None,
                         usage_present=has_usage,
                         complete_estimate_usd=str(expected) if s and has_usage else None))
assert len(old_rows) == 4 and sum(r["day"]=="2026-08-10" for r in old_rows)==3
assert sum(r["day"]=="2026-08-10" and r["complete_estimate_usd"] is not None for r in old_rows)==1
receipt = dict(passed=True,new_attempts=attempts,priced_attempt_count=4,root_count=3,
               per_attempt_usd=str(expected),store_day_known_usd=str(expected*4),
               priced_page_count=len(expected_priced),priced_pages=sorted(expected_priced),
               zero_recorded_page_count=1,measured_dollar_comparisons=0,
               scope_zero_reproduced=True,store_unchanged=True,
               original_run_collected_attempts=old_rows,
               original_run_correction="4 store attempts total; 3 on August 10; only 1 priced with usage on that day. Fourth August-10 emission is uncollected reader_local.")
with output.open("x") as file:
    json.dump(receipt,file,indent=2)
    file.write("\n")
print("Scope-zero reproduced; 4 priced attempts across 3 roots = USD 0.00284; 7 priced pages reconcile; 1 scope-zero page.")
