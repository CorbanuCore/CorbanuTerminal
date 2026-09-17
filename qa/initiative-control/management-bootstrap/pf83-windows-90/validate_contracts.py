"""Retain fresh immutable receipts for meta-validation and six deliberate reds."""
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
CACHE = HERE.parent / "pf83-f09gap-83" / "artifacts"
ENV = {
    "PATH": "/opt/homebrew/bin:/usr/bin:/bin",
    "UV_CACHE_DIR": str(CACHE / "uv-cache"),
    "UV_PYTHON_INSTALL_DIR": str(CACHE / "python"),
    "PYTHONDONTWRITEBYTECODE": "1",
}
BASE = ["uv", "run", "--offline", "--no-project", "--no-config",
        "--with", "jsonschema==4.25.1", "python", "-B"]
stages = [
    ("schema-meta-validation",
     [str(HERE.parent / "pf83-f09gap-83" / "check_contract.py")], 0),
    ("contract-controls", [str(HERE / "run_controls.py")], 0),
]
stages.extend(
    ("weakened-" + kind, [str(HERE / "run_controls.py"), "--weaken", kind], 1)
    for kind in ("capture-required", "capture-range", "capture-enum",
                 "results-required", "results-range", "results-enum")
)
sources = [
    HERE / "test_strength.py", HERE / "run_controls.py",
    HERE.parent / "pf83-controls-86" / "frozen_fixtures.py",
    *(HERE.parent / "pf83-f09gap-83" / name for name in (
        "check_contract.py", "test_contract.py",
        "evidence-contract.schema.json", "results.schema.json")),
]
with (HERE / "source-freeze.json").open("x") as stream:
    json.dump({str(p.relative_to(REPO)): hashlib.sha256(p.read_bytes()).hexdigest()
               for p in sources}, stream, indent=2)
    stream.write("\n")
for stage, command, expected in stages:
    with (HERE / (stage + ".log")).open("xb") as stream:
        result = subprocess.run(BASE + command, cwd=REPO, env=ENV,
                                stdout=stream, stderr=subprocess.STDOUT)
    with (HERE / (stage + "-command.json")).open("x") as stream:
        json.dump(dict(argv=BASE + command, cwd=str(REPO), environment=ENV,
                       exit_code=result.returncode, expected_exit_code=expected),
                  stream, indent=2)
        stream.write("\n")
    print(stage + ": exit " + str(result.returncode), flush=True)
    if result.returncode != expected:
        raise SystemExit("Unexpected gate outcome; preserve attempt and inspect log")
