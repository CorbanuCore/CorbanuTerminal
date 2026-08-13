#!/usr/bin/env python3

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
BUILD_SCRIPT = REPO_ROOT / "codex-cli" / "scripts" / "build_npm_package.py"


def load_build_module():
    spec = importlib.util.spec_from_file_location("build_npm_package", BUILD_SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load module from {BUILD_SCRIPT}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class NpmMetadataTest(unittest.TestCase):
    def test_staged_cli_publishes_corbanu_and_legacy_commands(self) -> None:
        build_module = load_build_module()

        with tempfile.TemporaryDirectory() as temp_dir:
            staging_dir = Path(temp_dir)
            build_module.stage_sources(staging_dir, "1.2.3", "codex")
            package_json = json.loads(
                (staging_dir / "package.json").read_text(encoding="utf-8")
            )

            self.assertTrue((staging_dir / "mkdocs.yml").is_file())
            self.assertTrue((staging_dir / "docs" / "index.md").is_file())

        self.assertEqual(
            package_json["bin"],
            {
                "corbanu": "bin/codex.js",
                "pfterminal": "bin/codex.js",
            },
        )
        self.assertEqual(package_json["name"], "@corbanucore/terminal")
        self.assertTrue(
            all(
                name.startswith("@corbanucore/terminal-")
                for name in package_json["optionalDependencies"]
            )
        )
        self.assertIn("Corbanu Terminal", package_json["description"])
        self.assertEqual(
            package_json["repository"]["url"],
            "git+https://github.com/CorbanuCore/CorbanuTerminal.git",
        )
        self.assertEqual(package_json["files"], ["bin/codex.js", "mkdocs.yml", "docs"])


if __name__ == "__main__":
    unittest.main()
