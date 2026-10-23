"""Stage an exact local developer package; do not launch it or claim isolation."""
import gzip
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import time

here = Path(__file__).resolve().parent
repo = here.parents[4]
prerequisites = json.loads((here / "gates-01/prerequisites.json").read_text())
assert prerequisites["exit"] == 0
settings = prerequisites["environment"]
command = ["cargo", "build", "--locked", "--offline", "-p", "codex-cli", "--bin", "codex",
           "-p", "codex-rmcp-client", "-p", "codex-code-mode-host", "--bins",
           "--features", "codex-core/developer-accounting"]
log = here / "package-build.log"
start = time.time()
with log.open("x") as output:
    result = subprocess.run(command, cwd=repo / "codex-rs", env=dict(os.environ, **settings),
                            stdout=output, stderr=subprocess.STDOUT)
data = log.read_bytes()
with (here / "package-build.log.gz").open("xb") as output:
    output.write(gzip.compress(data, mtime=0))
log.unlink()
receipt = dict(command=command, environment=settings, exit=result.returncode,
               elapsed_seconds=time.time() - start, log_sha256=hashlib.sha256(data).hexdigest(),
               base=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip(),
               platform=platform.platform(), files=[], launched=False,
               status="Local developer package only; not an isolated executor enclosure, signed distribution, or independent qualification.")
if result.returncode == 0:
    target = Path(settings["CARGO_TARGET_DIR"]) / "debug"
    package = here / "package"
    package.mkdir(exist_ok=False)
    for name in ("codex", "codex-code-mode-host", "rmcp_test_server"):
        source = target / name
        destination = package / name
        shutil.copyfile(source, destination)
        destination.chmod(0o555)
        digest = hashlib.file_digest(destination.open("rb"), "sha256").hexdigest()
        receipt["files"].append(dict(path=str(destination.relative_to(repo)), bytes=destination.stat().st_size,
                                     sha256=digest, mode="0555"))
    package.chmod(0o555)
with (here / "package-manifest.json").open("x") as output:
    json.dump(receipt, output, indent=2)
    output.write("\n")
print(json.dumps(receipt))
raise SystemExit(result.returncode)
