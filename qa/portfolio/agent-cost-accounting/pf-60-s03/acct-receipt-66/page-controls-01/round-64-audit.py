"""Independent arithmetic and reader-state checks against saved real-key pages."""
from decimal import Decimal
import gzip
import json
from pathlib import Path
import re
import sys
from p3_checks import assert_day

run = Path(sys.argv[1])
output = Path(sys.argv[2])
emissions = json.loads((run/"emissions.json").read_text())
identities = json.loads((run/"identities.json").read_text())
cases = json.loads((run/"reader-cases.json").read_text())
assert len(emissions)==5
assert [(e["provider"],e["model"],e["usage_present"]) for e in emissions] == [
    ("openai","gpt-5.6-sol",True),("openai","gpt-5.6-sol",True),
    ("reader_local","reader-local-model",True),("openai","gpt-6-astra",True),
    ("openai","gpt-5.6-sol",False)]
# Frozen tariff input: independent of the persisted quote/estimate.
rates = {"noncached":Decimal(5),"read":Decimal(".5"),"output":Decimal(30)}
row = emissions[0]
components = {
    "Noncached input":Decimal(row["input"]-row["cached"])*rates["noncached"]/1000000,
    "Cache read":Decimal(row["cached"])*rates["read"]/1000000,
    "Cache write":Decimal(0),
    "Output":Decimal(row["output"])*rates["output"]/1000000}
expected = sum(components.values())
assert expected==Decimal(".00071")
assert all((e["input"],e["cached"],e["output"])==(100,20,10) for e in emissions if e["usage_present"])
known_prefixes = ("historical","narrow","mixed-provider","stale-restored-control","backend-reopened","backend-refresh")
unknown_prefixes = ("missing-price","no-usage")
# Frozen from the intended navigation, never inferred from amounts that survive.
priced_pages = {
    "backend-reopened", "historical", "historical-Request-1-Attempt-1",
    "mixed-provider", "mixed-provider-Request-1-Attempt-1",
    "mixed-provider-Provider-openai", "narrow", "narrow-Request-1-Attempt-1",
    "stale-restored-control",
}
unknown_pages = {
    "missing-price", "missing-price-Request-1-Attempt-1",
    "no-usage", "no-usage-Request-1-Attempt-1",
}
zero_pages = {"zero-attempt-day", "never-prompted"}
expected_numeric_pages = priced_pages | unknown_pages | zero_pages
selected_pages = {p.name.removesuffix("-selected.json"): json.loads(p.read_text())
                  for p in run.glob("*-selected.json")}
nonmonetary_pages = {
    "backend-refresh", "historical-Request-1", "missing-price-Request-1",
    "mixed-provider-Request-1", "narrow-Request-1", "no-usage-Request-1",
    "stale-estimate", "stale-refresh", "unavailable-backend",
}
expected_pages = expected_numeric_pages | nonmonetary_pages
assert set(selected_pages) == expected_pages, (
    "page identity coverage mismatch",
    {"missing": sorted(expected_pages - set(selected_pages)),
     "unexpected": sorted(set(selected_pages) - expected_pages)},
)
# Navigation/refusal pages have no numeric money, regardless of the label used.
for name in sorted(nonmonetary_pages):
    text = " ".join(selected_pages[name])
    assert not re.search(
        r"\$\s*[+-]?[0-9]|\bUSD\s*:?\s*[+-]?[0-9]|[0-9]\s*USD\b",
        text,
    ), ("unexpected monetary amount on nonmonetary page", name)
numeric_pages = []
for file in sorted(run.glob("*-selected.json")):
    name = file.name.removesuffix("-selected.json")
    lines = json.loads(file.read_text())
    text = " ".join(lines)
    known = name.startswith(known_prefixes)
    unknown = name.startswith(unknown_prefixes)
    empty = name in ("zero-attempt-day","never-prompted")
    subtotal = re.findall(r"Known (?:subtotal|estimate) exact USD: ([0-9.]+)",text)
    if name in expected_numeric_pages:
        amount = expected if name in priced_pages else Decimal(0)
        assert known or unknown or empty, name
        assert len(subtotal)==1, ("numeric page coverage mismatch", name, subtotal)
        assert Decimal(subtotal[0])==amount, (name,subtotal,amount)
        displayed = re.findall(r"(?:Estimated token cost for recorded attempts|Known estimated token cost): \$([0-9.]+)",text)
        assert len(displayed)==(0 if name in zero_pages else 1), (name,displayed)
        assert all(Decimal(v)==amount for v in displayed), (name,displayed,amount)
        numeric_pages.append(dict(page=name,expected_known_usd=str(amount),
            complete_recorded_estimate=known,complete_run_or_billed_amount=False))
    if name in unknown_pages:
        assert "Estimated token cost: unknown" in text
        assert "Full recorded estimate: unavailable (1 of 1 attempts incomplete)" in text
    if name.endswith("Attempt-1") and known:
        for label,value in components.items():
            match = re.search(re.escape(label)+r" cost: \$([0-9.]+); exact USD ([0-9.]+)",text)
            assert match and Decimal(match[1])==value and Decimal(match[2])==value, (name,label)
        for label,value in [("Noncached input","5"),("Cache read","0.5"),("Output","30")]:
            assert f"{label} rate: {value} USD per million tokens" in text, (name,label)
    if name.endswith("Attempt-1") and name.startswith("missing-price"):
        assert "Price: unavailable — no dispatch-time price snapshot" in text
        assert "Noncached input cost: unknown — rate unavailable" in text
        assert "Output cost: unknown — rate unavailable" in text
    if name.endswith("Attempt-1") and name.startswith("no-usage"):
        assert "Input: unknown — no retained numeric evidence" in text
        assert "Output cost: unknown — no retained numeric evidence" in text
    if name in ("stale-estimate","stale-refresh","unavailable-backend"):
        assert not subtotal and not re.search(r"\$[0-9]",text),name
        assert "Refresh" in lines

for case in cases:
    name = case["case"]
    assert case["rendered"]==json.loads((run/(name+"-selected.json")).read_text())
    assert_day(gzip.decompress((run/(name+".txt.gz")).read_bytes()).decode(),case["day"])
narrow = next(c for c in cases if c["case"]=="narrow")
assert (narrow["columns"],narrow["rows"])==(64,24)
ntext = " ".join(narrow["rendered"])
for phrase in ["Estimated token cost for recorded attempts: $0.000710",
               "Billed cost: unavailable — no settlement evidence",
               "this UTC day is not their complete lifetime."]:
    assert phrase in ntext
assert "Known subtotal exact USD: 0.00071" in ntext
metrics = [("Input",100),("Noncached input (derived for inclusive input)",80),
           ("Cache read",20),("Cache write",0),("Output",10),
           ("Reasoning (subset, not separately billed)",4),("Total (not separately billed)",110)]
for name in ("historical","narrow","mixed-provider","missing-price","no-usage","zero-attempt-day"):
    text = " ".join(json.loads((run/(name+"-selected.json")).read_text()))
    for label,value in metrics:
        val,unknown = (0,1) if name=="no-usage" else (0,0) if name=="zero-attempt-day" else (value,0)
        assert f"{label}: {val} known + unknown in {unknown} attempts" in text,(name,label)

bindings = [(json.loads(a),None if s is None else json.loads(s))
            for a,s in json.loads((run/"native-bindings.json").read_text())]
assert len(bindings)==4, len(bindings)
assert all(a["provider"]=="openai" for a,_ in bindings)
assert sum(a["thread_id"]==identities["historical_and_mixed"] for a,_ in bindings)==2
assert all(s is None for a,s in bindings if a["model"]=="gpt-6-astra")
assert all(tuple(Decimal(s["rates"][k]) for k in ("noncached","read","output"))==
           tuple(rates[k] for k in ("noncached","read","output"))
           for a,s in bindings if s is not None)
assert json.loads((run/"stale-injected.json").read_text())==json.loads((run/"stale-after-refresh.json").read_text())
assert json.loads((run/"stale-before.json").read_text())!=json.loads((run/"stale-injected.json").read_text())
backend = " ".join(json.loads((run/"unavailable-backend-selected.json").read_text()))
assert "Unavailable — accounting evidence is corrupt, incompatible or could not be read. Refresh to retry; no repair performed." in backend
assert json.loads((run/"backend-fault.json").read_text())["tables"]==[["qa_held_estimates"]]
assert json.loads((run/"backend-restored.json").read_text())["tables"]==[["draft_accounting_estimates"]]
receipt = dict(passed=True,emissions=len(emissions),native_attempts=len(bindings),
    numeric_pages=numeric_pages,numeric_page_count=len(numeric_pages),
    priced_reconciliation_pages=sorted(priced_pages),
    priced_reconciliation_count=len(priced_pages),
    unknown_cost_pages=sorted(unknown_pages),unknown_cost_page_count=len(unknown_pages),
    zero_recorded_pages=sorted(zero_pages),zero_recorded_page_count=len(zero_pages),
    nonmonetary_pages=sorted(nonmonetary_pages),page_identity_count=len(expected_pages),
    coverage_assertion="Exact 24 page identities required: 15 subtotal pages and 9 pages that forbid numeric money.",
    count_interpretation="9 priced page reconciliations (repeated views, not 9 attempts); 4 unknown-cost pages; 2 zero-recorded pages. No measured/billed-dollar reconciliation.",
    independent_components={k:str(v) for k,v in components.items()},known_attempt_usd=str(expected),
    rates_per_million={k:str(v) for k,v in rates.items()},
    missing_price="Known subtotal 0 is an empty known component, not a zero-price quote.",
    no_usage="usage_present=false means numeric handler constants were not transmitted and cannot supply a measurement.",
    mixed_provider="OpenAI + reader_local emitted; only OpenAI collected. .00071 is the known recorded component; full mixed-provider cost unknown.",
    stale="Stale evidence refused, Refresh did not repair injected evidence, restored control recovers .00071.",
    backend_refresh=json.loads((run/"backend-refresh-result.json").read_text()),
    narrow="64x24 actual resize; amount, billing/estimate/day qualifiers and drill-down retained.")
with output.open("x") as file:
    json.dump(receipt,file,indent=2)
    file.write("\n")
print(f"{len(priced_pages)} priced page reconciliations; {len(unknown_pages)} unknown-cost pages; {len(zero_pages)} zero-recorded pages; exact 15-page subtotal coverage bound")
