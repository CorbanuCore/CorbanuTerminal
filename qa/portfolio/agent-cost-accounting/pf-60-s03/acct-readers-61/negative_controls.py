"""Preserve counterexamples for each round-61 P3 assertion."""
from datetime import datetime
import gzip
import json
import os
from pathlib import Path
import subprocess
import sys
from p3_checks import assert_day, assert_compact_hour, assert_timezone, COMPACT_HOUR

here = Path(__file__).resolve().parent
run = Path(sys.argv[1]).resolve()
output = Path(sys.argv[2]).resolve()
assert not output.exists()
checks = []
def rejects(name, call, old_check_passes=None):
    try:
        call()
    except AssertionError as error:
        result = dict(case=name,rejected=True,error=str(error))
        if old_check_passes is not None:
            result["old_check_passes"] = old_check_passes
        checks.append(result)
    else:
        raise AssertionError("Negative control accepted: "+name)

for name,day in [("mixed-day","2026-11-01"),("deleted-history-after-retention","2026-03-08")]:
    text = gzip.decompress((run/(name+".txt.gz")).read_bytes()).decode()
    assert_day(text,day)
    wrong = text.replace("Requested UTC day: "+day,"Requested UTC day: 2026-11-02")
    # Prior assertion only required the refusal and absence of a subtotal.
    assert "compacted history lost request/provider attribution" in wrong
    assert "Known subtotal exact USD:" not in wrong
    rejects(name+"-wrong-day",lambda:assert_day(wrong,day),old_check_passes=True)
    missing = text.replace("Requested UTC day: "+day,"Requested day omitted")
    assert "compacted history lost request/provider attribution" in missing
    assert "Known subtotal exact USD:" not in missing
    rejects(name+"-missing-day",lambda:assert_day(missing,day),old_check_passes=True)

lines = json.loads((run/"mixed-compact-hour-selected.json").read_text())
text = " ".join(lines)
assert_compact_hour(text)
wrong = text.replace(COMPACT_HOUR,"History unavailable.")
assert "Known subtotal exact USD:" not in wrong
rejects("compact-hour-explanation-removed",lambda:assert_compact_hour(wrong),old_check_passes=True)

# Deliberately launch the same product with TZ removed. The constructor runs
# within the version process too; no Python-local conversion supplies its result.
manifest = json.loads((run/"manifest.json").read_text())
binary = manifest["binary"]
clock = here/"tz_clock.dylib"
fixture = output.parent/"fixture"
fixture.mkdir(exist_ok=True)
probe = fixture/"tz-removed"
env = {"PATH":os.environ["PATH"],"HOME":str(fixture),
       "CODEX_HOME":str(fixture),"CODEX_TESTS_DISABLE_KEYRING":"1",
       "DYLD_INSERT_LIBRARIES":str(clock),"ACCT_TZ_PROBE":str(probe),
       "ACCT_QUALIFY_CLOCK_MS":"1773144000000"}
process = subprocess.Popen([binary,"--version"],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
stdout,stderr = process.communicate(timeout=15)
assert process.returncode == 0, stderr
observed = json.loads(Path(str(probe)+f".{process.pid}.json").read_text())
assert observed["pid"] == process.pid and observed["tz"] == ""
rejects("TZ-stripped-from-actual-product-process",
        lambda:assert_timezone(probe,process.pid,"America/New_York","2026-03-10T12:00:00Z"))
controls = json.loads((run/"timezone-controls.json").read_text())
assert len(controls)==4 and len({c["local"] for c in controls})==4
checks.append(dict(case="four-positive-product-process-controls",controls=controls,passed=True))
with output.open("x") as file:
    json.dump(dict(checks=checks,negative_count=6,positive_count=4,
        removed_tz_process=dict(binary=binary,pid=process.pid,version=stdout.strip(),observed=observed)),file,indent=2)
    file.write("\n")
print("6 counterexamples rejected; 4 distinct product-local-time controls passed")
