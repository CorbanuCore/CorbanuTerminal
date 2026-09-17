"""Independent arithmetic and reader-state checks against saved real-key pages."""
import argparse
from decimal import Decimal
import gzip
import hashlib
import json
from pathlib import Path
import re
import sys
from p3_checks import assert_day

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("run", type=Path)
parser.add_argument("output", type=Path)
parser.add_argument("--semantics-only", action="store_true",
                    help="Mutation diagnostic only; cannot qualify capture integrity.")
args = parser.parse_args()
run, output = args.run, args.output
monetary_checks = {}
semantic_failures = []


def money(key, condition):
    """Evaluate every monetary invariant, including overlapping narrow checks."""
    assert key not in monetary_checks, key
    monetary_checks[key] = bool(condition)
    if not condition:
        semantic_failures.append(key)


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
money("arithmetic:known-attempt", expected == Decimal(".00071"))
money("arithmetic:emitted-usage", all((e["input"],e["cached"],e["output"])==(100,20,10) for e in emissions if e["usage_present"]))
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
# This auditor qualifies one frozen capture, not arbitrary future UI copy.
# Bind all content on every page, including priced/subtotal pages and IDs/times.
# A currency-keyword blacklist cannot establish absence of unexpected amounts.
page_digests = json.loads(
    Path(__file__).with_name("page-content-digests.json").read_text()
)
assert set(page_digests) == expected_pages
if not args.semantics_only:
    for name in sorted(expected_pages):
        digest = hashlib.sha256(
            json.dumps(selected_pages[name], ensure_ascii=True).encode()
        ).hexdigest()
        assert digest == page_digests[name], (
            "unexpected content on nonmonetary page" if name in nonmonetary_pages
            else "unexpected content on subtotal page", name,
        )
    viewport_digests = json.loads(
        Path(__file__).with_name("viewport-content-digests.json").read_text()
    )
    viewports = {p.name: p for p in run.glob("*.txt.gz")}
    assert set(viewports) == set(viewport_digests), "viewport identity coverage mismatch"
    for name, path in sorted(viewports.items()):
        digest = hashlib.sha256(gzip.decompress(path.read_bytes())).hexdigest()
        assert digest == viewport_digests[name], ("unexpected viewport content", name)
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
        money(f"exact-count:{name}", len(subtotal) == 1)
        money(f"exact-value:{name}", bool(subtotal) and all(Decimal(v) == amount for v in subtotal))
        displayed = re.findall(r"(?:Estimated token cost for recorded attempts|Known estimated token cost): \$([0-9.]+)",text)
        money(f"display-count:{name}", len(displayed) == (0 if name in zero_pages else 1))
        money(f"display-value:{name}", all(Decimal(v) == amount for v in displayed))
        numeric_pages.append(dict(page=name,expected_known_usd=str(amount),
            complete_recorded_estimate=known,complete_run_or_billed_amount=False))
    if name in unknown_pages:
        money(f"unknown-cost:{name}", "Estimated token cost: unknown" in text)
        money(f"incomplete-cost:{name}", "Full recorded estimate: unavailable (1 of 1 attempts incomplete)" in text)
    if name.endswith("Attempt-1") and known:
        for label,value in components.items():
            match = re.search(re.escape(label)+r" cost: \$([0-9.]+); exact USD ([0-9.]+)",text)
            money(f"component:{name}:{label}", match and Decimal(match[1]) == value and Decimal(match[2]) == value)
        for label,value in [("Noncached input","5"),("Cache read","0.5"),("Output","30")]:
            money(f"rate:{name}:{label}", f"{label} rate: {value} USD per million tokens" in text)
    if name.endswith("Attempt-1") and name.startswith("missing-price"):
        money(f"missing-price:{name}:snapshot", "Price: unavailable — no dispatch-time price snapshot" in text)
        money(f"missing-price:{name}:input", "Noncached input cost: unknown — rate unavailable" in text)
        money(f"missing-price:{name}:output", "Output cost: unknown — rate unavailable" in text)
    if name.endswith("Attempt-1") and name.startswith("no-usage"):
        money(f"no-usage:{name}:input", "Input: unknown — no retained numeric evidence" in text)
        money(f"no-usage:{name}:output", "Output cost: unknown — no retained numeric evidence" in text)
    if name in ("stale-estimate","stale-refresh","unavailable-backend"):
        money(f"refused-money:{name}", not subtotal and not re.search(r"\$[0-9]",text))
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
    money(f"narrow-qualifier:{phrase}", phrase in ntext)
money("narrow-exact", "Known subtotal exact USD: 0.00071" in ntext)
metrics = [("Input",100),("Noncached input (derived for inclusive input)",80),
           ("Cache read",20),("Cache write",0),("Output",10),
           ("Reasoning (subset, not separately billed)",4),("Total (not separately billed)",110)]
for name in ("historical","narrow","mixed-provider","missing-price","no-usage","zero-attempt-day"):
    text = " ".join(json.loads((run/(name+"-selected.json")).read_text()))
    for label,value in metrics:
        val,unknown = (0,1) if name=="no-usage" else (0,0) if name=="zero-attempt-day" else (value,0)
        money(f"usage:{name}:{label}", f"{label}: {val} known + unknown in {unknown} attempts" in text)

bindings = [(json.loads(a),None if s is None else json.loads(s))
            for a,s in json.loads((run/"native-bindings.json").read_text())]
assert len(bindings)==4, len(bindings)
assert all(a["provider"]=="openai" for a,_ in bindings)
assert sum(a["thread_id"]==identities["historical_and_mixed"] for a,_ in bindings)==2
money("native:missing-price", all(s is None for a,s in bindings if a["model"]=="gpt-6-astra"))
money("native:rates", all(tuple(Decimal(s["rates"][k]) for k in ("noncached","read","output")) ==
           tuple(rates[k] for k in ("noncached","read","output"))
           for a,s in bindings if s is not None))
assert json.loads((run/"stale-injected.json").read_text())==json.loads((run/"stale-after-refresh.json").read_text())
assert json.loads((run/"stale-before.json").read_text())!=json.loads((run/"stale-injected.json").read_text())
backend = " ".join(json.loads((run/"unavailable-backend-selected.json").read_text()))
assert "Unavailable — accounting evidence is corrupt, incompatible or could not be read. Refresh to retry; no repair performed." in backend
assert json.loads((run/"backend-fault.json").read_text())["tables"]==[["qa_held_estimates"]]
assert json.loads((run/"backend-restored.json").read_text())["tables"]==[["draft_accounting_estimates"]]
if semantic_failures:
    print(json.dumps(dict(semantic_failures=semantic_failures,
                          monetary_checks_evaluated=sorted(monetary_checks))), file=sys.stderr)
    raise AssertionError("monetary semantic checks failed")
receipt = dict(passed=True, mode="semantics-only" if args.semantics_only else "digest-bound",
    capture_qualified=not args.semantics_only, monetary_checks=sorted(monetary_checks),
    viewport_bound_count=0 if args.semantics_only else len(viewport_digests),
    emissions=len(emissions),native_attempts=len(bindings),
    numeric_pages=numeric_pages,numeric_page_count=len(numeric_pages),
    priced_reconciliation_pages=sorted(priced_pages),
    priced_reconciliation_count=len(priced_pages),
    unknown_cost_pages=sorted(unknown_pages),unknown_cost_page_count=len(unknown_pages),
    zero_recorded_pages=sorted(zero_pages),zero_recorded_page_count=len(zero_pages),
    nonmonetary_pages=sorted(nonmonetary_pages),page_identity_count=len(expected_pages),
    content_bound_pages=[] if args.semantics_only else sorted(expected_pages),
    content_bound_page_count=0 if args.semantics_only else len(page_digests),
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
print(f"{receipt['mode']}: {len(priced_pages)} priced page reconciliations; {len(unknown_pages)} unknown-cost pages; {len(zero_pages)} zero-recorded pages; {len(monetary_checks)} monetary checks")
