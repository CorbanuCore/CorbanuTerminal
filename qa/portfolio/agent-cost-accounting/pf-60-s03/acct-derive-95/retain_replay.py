"""Retain actual replay streams outside ignored scratch; verify receipt bindings."""
import hashlib
import json
from pathlib import Path
import shutil

here = Path(__file__).resolve().parent
record = json.loads((here / "replay-checks.json").read_bytes())
source = Path(record["run_directory"])
out = here / "replay-streams"
out.mkdir(exist_ok=True)
for case in record["cases"]:
    name = case["case"]
    for stream in ("stdout", "stderr"):
        shutil.copyfile(source / (name + "." + stream + ".txt"),
                        out / (name + "." + stream + ".txt"))
    receipt = json.loads((out / (name + ".stdout.txt")).read_bytes())
    if receipt != case["receipt"]:
        raise RuntimeError("outer receipt mismatch")
    destination = out / name
    destination.mkdir(exist_ok=True)
    for command in receipt["commands"]:
        for stream in ("stdout", "stderr"):
            filename = command["name"] + "." + stream + ".txt"
            raw = (Path(receipt["run_directory"]) / filename).read_bytes()
            if hashlib.sha256(raw).hexdigest() != command[stream + "_sha256"]:
                raise RuntimeError("stream digest mismatch: " + filename)
            (destination / filename).write_bytes(raw)
    controls = json.loads((destination / "controls.stdout.txt").read_text().split("\n", 1)[1])
    if len(controls["cases"]) != 13 or not all(row["passed"] for row in controls["cases"]):
        raise RuntimeError("verifier controls failed")
print("Retained four outer stdout/stderr pairs and sixteen nested streams; each control run passed all 13 cases.")
