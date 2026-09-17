"""Explicit, one-shot maintenance; verification never invokes this refresh."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys

here = Path(__file__).resolve().parent
repo = here.parents[4]
scope = here.parent
base = "6a7128ccc4e10c92dd583ca65b62e70d84483012"
verifier = scope / "acct-inventory-79/verify_acceptance.py"
acceptance = scope / "acct-acceptance-75/acceptance.md"
inventory = scope / "acct-acceptance-75/scope.json"
correction = scope / "acct-inventory-79/inventory-correction.json"
reference = scope / "acct-reference-88/expected.stdout.txt"
manifest = scope / "acct-reference-88/reference.json"
environment = {k: v for k, v in os.environ.items()
               if not k.startswith("GIT_")}
environment.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
                   GIT_TERMINAL_PROMPT="0")


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def git(*args, cwd=repo):
    return subprocess.check_output(["git", *args], cwd=cwd, env=environment)


def save(name, raw):
    with (here / name).open("xb") as stream:
        stream.write(raw)


def save_json(name, data):
    save(name, (json.dumps(data, indent=2) + "\n").encode())


def run(name, cwd, optimized=False):
    command = [sys.executable, "-B", *(["-O"] if optimized else []),
               str(verifier.relative_to(repo))]
    result = subprocess.run(command, cwd=cwd, env=environment, capture_output=True)
    save(name + ".stdout.txt", result.stdout)
    save(name + ".stderr.txt", result.stderr)
    require(not result.stderr, name + " stderr")
    return result


require(git("rev-parse", "HEAD").decode().strip() == base, "source base changed")
prior = {}
for label, path in (("inventory", inventory), ("correction", correction),
                    ("reference", reference), ("reference-manifest", manifest),
                    ("acceptance", acceptance), ("verifier", verifier)):
    raw = git("show", base + ":" + str(path.relative_to(repo)))
    save("before-" + label + (".py" if label == "verifier" else ".txt"), raw)
    prior[label] = raw

clone = here / "target/before-checkout"
clone.parent.mkdir(parents=True, exist_ok=True)
git("clone", "--shared", "--no-checkout", "--quiet", str(repo), str(clone))
git("sparse-checkout", "set", str(scope.relative_to(repo)), cwd=clone)
git("checkout", "--quiet", "--detach", base, cwd=clone)
require(git("status", "--porcelain", cwd=clone) == b"", "before checkout dirty")
before = run("before", clone)
require(before.returncode == 3 and
        b"RESULT agreement=18 disagreement=2 unavailable=3 exit=3" in before.stdout,
        "unexpected starting disagreements")
require(git("status", "--porcelain", cwd=clone) == b"", "before run changed checkout")

# Metadata exclusions must not hide new quantities, changed model names, or
# identifier prefixes extended with another digit.
spec = importlib.util.spec_from_file_location("acceptance_verifier", verifier)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
cases = []
for name, text, expected in (
    ("reported-identifiers", "round-103 revision at `claude-opus-5-plan`", True),
    ("adjacent-measurement", "round-103 revision at `claude-opus-5-plan`: 7 reviews", False),
    ("other-model", "`claude-opus-6-plan`", False),
    ("extended-revision", "round-1039", False),
    ("unmapped-money", "USD 0.003", False),
):
    module.DOCUMENT, module.CLAIMS = text, []
    try:
        detail = module.coverage()
        passed = True
    except ValueError as exc:
        detail, passed = str(exc), False
    cases.append(dict(case=name, text=text, accepted=passed, expected=expected, detail=detail))
    require(passed == expected, name)
save_json("identifier-controls.json", cases)

# Refresh only after the evidence comparison. Preserve the historical membership.
subprocess.run([sys.executable, "-B", str(scope / "acct-selfcheck-84/refresh_inventory.py")],
               cwd=repo, env=environment, check=True)
document = json.loads(inventory.read_bytes())
document["reconcile_107_refresh"] = dict(
    source_commit=base,
    record=str((here / "RETURN.md").relative_to(repo)),
    previous_inventory_sha256=sha(prior["inventory"]),
    reason="Review disclosure changed acceptance bytes; exact metadata identifiers were unmapped. "
           "Claims, historical membership, expected counts and qualification gaps are unchanged.")
inventory.write_text(json.dumps(document, indent=2) + "\n")
record = json.loads(correction.read_bytes())
record["reconcile_107_refresh"] = dict(
    source_commit=base, previous_after_sha256=json.loads(prior["correction"])["after_sha256"],
    record=str((here / "RETURN.md").relative_to(repo)))
record["after_sha256"] = sha(inventory.read_bytes())
correction.write_text(json.dumps(record, indent=2) + "\n")
old_rows = {r["path"]: r for r in json.loads(prior["inventory"])["files"]}
new_rows = {r["path"]: r for r in document["files"]}
require(old_rows.keys() == new_rows.keys(), "membership changed")
changed = [name for name in old_rows if old_rows[name] != new_rows[name]]
require(changed == [str(acceptance.relative_to(repo))], "unexpected inventory changes")

# Run the intentional newline drift against refreshed current bytes in both modes.
original = acceptance.read_bytes()
try:
    acceptance.write_bytes(original + b"\n")
    drift = run("drift", repo)
    optimized = run("drift-optimized", repo, True)
finally:
    acceptance.write_bytes(original)
require(drift.returncode == optimized.returncode == 3, "drift not refused")
require(drift.stdout == optimized.stdout, "optimized drift differs")
require(b"RESULT agreement=19 disagreement=1 unavailable=3 exit=3" in drift.stdout,
        "unexpected drift counts")
fences = re.findall(rb"```text\n(.*?)```", original, re.S)
require(drift.stdout in fences, "documented drift differs; reconcile before refreshing")
save_json("simulation.json", dict(source_commit=base,
    mutation="Append one newline to refreshed acceptance.md, run, restore exact original bytes.",
    stdout="drift.stdout.txt", optimized_stdout="drift-optimized.stdout.txt",
    stdout_sha256=sha(drift.stdout), normal_exit=drift.returncode,
    optimized_exit=optimized.returncode, stderr_empty=True, restored=True))

healthy = run("refreshed", repo)
require(healthy.returncode == 2 and
        b"RESULT agreement=20 disagreement=0 unavailable=3 exit=2" in healthy.stdout,
        "refreshed evidence does not match documented baseline")
reference.write_bytes(healthy.stdout)
binding = json.loads(manifest.read_bytes())
binding["sha256"] = sha(healthy.stdout)
binding["refresh"] = dict(source_commit=base, record=str((here / "RETURN.md").relative_to(repo)),
                         previous_sha256=sha(prior["reference"]))
manifest.write_text(json.dumps(binding, indent=2) + "\n")
save_json("refresh.json", dict(source_commit=base, classification="routine evidence maintenance",
    disagreement_verdict="documentation drift; no changed monetary/functional evidence claim",
    prior_exit=before.returncode, refreshed_exit=healthy.returncode,
    baseline_counts_unchanged=[20, 0, 3, 2], membership_unchanged=True,
    changed_inventory_entries=[dict(before=old_rows[name], after=new_rows[name]) for name in changed],
    previous_totals={k: v for k, v in json.loads(prior["inventory"]).items() if k.startswith("total_")},
    refreshed_totals={k: v for k, v in document.items() if k.startswith("total_")},
    previous_reference_sha256=sha(prior["reference"]), reference_sha256=sha(healthy.stdout),
    changed_reference_lines=dict(
        removed=[s for s in prior["reference"].decode().splitlines()
                 if s not in healthy.stdout.decode().splitlines()],
        added=[s for s in healthy.stdout.decode().splitlines()
               if s not in prior["reference"].decode().splitlines()])))
print("Refresh complete; final clean-checkout verification remains required.")
