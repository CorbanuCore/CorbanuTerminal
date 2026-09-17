"""Build compact, hash-checked receipts; retain larger raw evidence losslessly."""
import gzip
import hashlib
import json
from pathlib import Path
import subprocess

here = Path(__file__).resolve().parent
repo = here.parents[4]
base = "17371d192e52de71ad7efde9088cb47b6ae3a049"


def load(path):
    if path.exists():
        return json.loads(path.read_text())
    return json.loads(gzip.decompress(path.with_suffix(path.suffix + ".gz").read_bytes()))


def write(name, value):
    (here / name).write_text(json.dumps(value, indent=2) + "\n")


lanes = []
for name in ("prerequisites", "core-default", "core-feature", "tui"):
    row = load(here / "gates-01" / (name + ".json"))
    data = gzip.decompress((here / "gates-01" / (name + ".log.gz")).read_bytes())
    assert hashlib.sha256(data).hexdigest() == row["log_sha256"]
    assert row["base"] == base and row["exit"] == 0
    if name != "prerequisites":
        assert row["run"] == row["passed"] > 0
        assert not row["failure_lines"]
        assert b"Test isolation: disposable profile; native keyring disabled (debug lane)" in data
    lanes.append(row)
assert [r["run"] for r in lanes[1:]] == [124, 127, 91]
write("test-results.json", dict(lanes=lanes, total_executions=342,
                               failure_names=[], unique_test_count_claimed=False))
semantic = load(here / "controls-04/semantic-results.json")
coverage = {}
for index, row in enumerate(semantic["controls"]):
    key = row["mutation"]["check"]
    diagnostic = json.loads(row["stderr"].splitlines()[0])
    assert key in diagnostic["semantic_failures"]
    assert key in diagnostic["monetary_checks_evaluated"]
    assert row["exit"] != 0 and not row["receipt_written"]
    coverage.setdefault(key, []).append(index)
assert len(coverage) == semantic["checks"] == 147
write("monetary-coverage.json", dict(
    checks=147, mutations=159, all_reached_and_failed_for_named_reason=True,
    plan="controls-04/plan.json.gz", raw_results="controls-04/semantic-results.json.gz",
    check_to_zero_based_mutation_indices=coverage))
prior = here.parent / "acct-readers-61"
source = prior / "reader-run-05"
bindings = []
for file in sorted(source.glob("*.txt.gz")):
    original = subprocess.check_output(["git", "show", base + ":" + str(file.relative_to(repo))], cwd=repo)
    assert file.read_bytes() == original
    digest = hashlib.sha256(gzip.decompress(original)).hexdigest()
    assert digest == load(prior / "viewport-content-digests.json")[file.name]
    bindings.append(dict(file=file.name, content_sha256=digest))
assert len(bindings) == 45
write("viewport-provenance.json", dict(base=base, unchanged_capture_count=45, bindings=bindings))
# Only direct evidence files in run directories, never fixture symlinks.
# Preserve the original bytes inside gzip; do not revise historical attempts.
for directory in sorted(here.iterdir()):
    if directory.is_dir():
        for file in sorted(directory.iterdir()):
            if file.is_file() and not file.is_symlink() and file.suffix in (".json", ".jsonl"):
                data = file.read_bytes()
                if len(data) > 32768 or file.suffix == ".jsonl":
                    target = file.with_suffix(file.suffix + ".gz")
                    assert not target.exists(), target
                    target.write_bytes(gzip.compress(data, mtime=0))
                    assert gzip.decompress(target.read_bytes()) == data
                    file.unlink()
print(json.dumps(dict(rust=342, monetary_checks=147, semantic_mutations=159,
                      viewport_bindings=45)))
