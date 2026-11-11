from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


class CliTests(unittest.TestCase):
    def test_cli_run_json(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            config = Path(tmp) / "config.json"
            config.write_text(json.dumps({"tasks": {"a": {"uses": "identity", "params": {"value": 9}}}}), encoding="utf-8")
            proc = subprocess.run([sys.executable, "-m", "pipeflow.cli", "run", str(config), "--json"], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, check=True)
        data = json.loads(proc.stdout)
        self.assertEqual(data["outputs"]["a"], 9)

    def test_public_import_contract(self) -> None:
        code = "from pipeflow import PipelineRunner; print(PipelineRunner({'tasks':{'a':{'params':{'value':'ok'}}}}).run()['outputs']['a'])"
        proc = subprocess.run([sys.executable, "-c", code], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, check=True)
        self.assertEqual(proc.stdout.strip(), "ok")


if __name__ == "__main__":
    unittest.main()
