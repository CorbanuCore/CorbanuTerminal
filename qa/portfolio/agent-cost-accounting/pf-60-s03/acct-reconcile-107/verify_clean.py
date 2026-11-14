"""Check proposed QA from a clean local Git snapshot; retain exact streams."""
import gzip
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

here = Path(__file__).resolve().parent
repo = here.parents[4]
scope = here.parent.relative_to(repo)
base = "6a7128ccc4e10c92dd583ca65b62e70d84483012"
environment = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
environment.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
                   GIT_TERMINAL_PROMPT="0")


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def git(*args, cwd=repo):
    return subprocess.check_output(["git", *args], cwd=cwd, env=environment)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def save(name, raw):
    with (here / name).open("xb") as stream:
        stream.write(raw)


require(git("rev-parse", "HEAD").decode().strip() == base, "source base changed")
subprocess.run(["git", "diff", "--check"], cwd=repo, check=True, env=environment)
lanes = []
for name in ("prerequisites", "core-default", "core-feature", "tui"):
    row = json.loads((here / "gates-01" / (name + ".json")).read_bytes())
    raw = gzip.decompress((here / "gates-01" / (name + ".log.gz")).read_bytes())
    require(sha(raw) == row["log_sha256"] and row["exit"] == 0 and row["base"] == base,
            name + " failed/mismatched receipt")
    require(row["environment"]["NEXTEST_TEST_THREADS"] == "4", "thread setting")
    if name != "prerequisites":
        require(row["command"][:2] == ["just", "test"] and row["isolation_banner"], "unguarded lane")
        require(row["passed"] == row["run"] > 0 and not row["failure_lines"], "test count/failures")
        require(all(row[k] == 0 for k in ("failed", "timed_out", "flaky", "leaky")), "nonpassing lane")
        summaries = re.findall(r"^\s*Summary .*?(\d+) tests? run: (.+)$", raw.decode(), re.M)
        require(len(summaries) == 1 and int(summaries[0][0]) == row["run"], "raw run count")
        for key, label in (("passed", "passed"), ("skipped", "skipped")):
            require(int(re.search(r"(\d+) " + label, summaries[0][1])[1]) == row[key], key)
    lanes.append(row)
require(len({r["environment"]["CARGO_TARGET_DIR"] for r in lanes}) == 1, "targets differ")

# Existing adversarial controls include wrong monetary claim, missing log,
# corrupted viewport, baseline mutations and optional-package refusals.
controls = subprocess.run([sys.executable, "-B", str(scope / "acct-inventory-79/check_verifier.py")],
                          cwd=repo, env=environment, capture_output=True)
save("controls.stdout.txt", controls.stdout)
save("controls.stderr.txt", controls.stderr)
require(controls.returncode == 0 and not controls.stderr, "acceptance controls failed")
control_record = json.loads(controls.stdout[controls.stdout.index(b"{"):])
require(len(control_record["cases"]) == 13 and all(r["passed"] for r in control_record["cases"]),
        "control count/results")

# Materialize tracked base plus only this allocation's proposed files, never
# ignored target/package/profile data. The temporary commit is not integration.
changed = set(git("diff", "--name-only", "-z", base).decode().split("\0")[:-1])
changed.update(git("ls-files", "--others", "--exclude-standard", "-z").decode().split("\0")[:-1])
require(changed and all(p.startswith(str(scope) + "/") for p in changed), "outside writable scope")
clone = here / "target/final-checkout"
git("clone", "--shared", "--no-checkout", "--quiet", str(repo), str(clone))
git("checkout", "--quiet", "--detach", base, cwd=clone)
for name in sorted(changed):
    source, destination = repo / name, clone / name
    if source.is_file():
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)
    elif destination.exists():
        destination.unlink()
git("add", "--", str(scope), cwd=clone)
git("-c", "user.name=Evidence Check", "-c", "user.email=evidence.invalid@example.invalid",
    "-c", "commit.gpgsign=false", "commit", "--quiet", "--no-verify",
    "-m", "acct-reconcile-107 proposed QA snapshot; not integration", cwd=clone)
snapshot = git("rev-parse", "HEAD", cwd=clone).decode().strip()
before = git("status", "--porcelain", "--untracked-files=all", cwd=clone).decode()
require(before == "", "final checkout starts dirty")
require(not (clone / scope / "acct-fitness-76/package").exists(), "local package leaked")
reference = (clone / scope / "acct-reference-88/expected.stdout.txt").read_bytes()
binding = json.loads((clone / scope / "acct-reference-88/reference.json").read_bytes())
require(sha(reference) == binding["sha256"], "reference digest differs")
runs = []
# Ordinary Python is last: the final acceptance invocation requested by the brief.
for name, flags in (("final-optimized", ["-O"]), ("final", [])):
    command = [sys.executable, "-B", *flags, str(scope / "acct-inventory-79/verify_acceptance.py")]
    result = subprocess.run(command, cwd=clone, env=environment, capture_output=True)
    save(name + ".stdout.txt", result.stdout)
    save(name + ".stderr.txt", result.stderr)
    runs.append(dict(name=name, command=command, exit=result.returncode,
                     stderr_empty=not result.stderr, stdout_sha256=sha(result.stdout),
                     exact_reference_match=result.stdout == reference))
after = git("status", "--porcelain", "--untracked-files=all", cwd=clone).decode()
receipt = dict(source_commit=base, snapshot_commit=snapshot,
    snapshot_tree=git("rev-parse", "HEAD^{tree}", cwd=clone).decode().strip(),
    snapshot_kind="Full clean local Git checkout of proposed QA; not an integration commit.",
    checkout=str(clone), status_before=before, status_after=after,
    no_network=True, package_absent=True, package_launched=False,
    proposed_file_sha256={name: sha((repo / name).read_bytes()) for name in sorted(changed)
                          if (repo / name).is_file()},
    controls=control_record, gates=lanes, runs=runs)
save("final-check.json", (json.dumps(receipt, indent=2) + "\n").encode())
require(before == after == "", "verification changed checkout")
require(all(r["exit"] == 2 and r["stderr_empty"] and r["exact_reference_match"] for r in runs),
        "final verification differs from refreshed reference")
print(result.stdout.decode(), end="")
print("Process exit:", result.returncode)
print("Clean snapshot:", snapshot)
