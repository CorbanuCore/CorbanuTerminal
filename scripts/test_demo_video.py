import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from scripts import demo_video as dv

SPEC = """
id = "sample-demo"
feature = "/status shows permissions"
expected = "The status card lists the active permissions."
model = "glm-5.2"
provider = "zai"
config = 'model = "glm-5.2"'

[credentials]
ZAI_API_KEY = "provider/zai_api_key"

[fixtures]
"README.md" = "workspace at {workspace}"

[[steps]]
wait = "Corbanu"
[[steps]]
type = "/status"
[[steps]]
key = "Enter"
[[steps]]
wait = "Permissions"
compress = 2
[[steps]]
pause = 1.5
"""


class DemoVideoTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = Path(self.tmp.name)
        self.spec_path = self.dir / "spec.toml"
        self.spec_path.write_text(SPEC)

    def tearDown(self):
        self.tmp.cleanup()

    def test_load_spec(self):
        spec = dv.load_spec(self.spec_path)
        self.assertEqual(
            [s.kind for s in spec.steps], ["wait", "type", "key", "wait", "pause"]
        )
        self.assertEqual(spec.steps[3].compress, 2)
        self.assertEqual(spec.credentials, {"ZAI_API_KEY": "provider/zai_api_key"})

    def test_tmux_literal_escapes_command_separator(self):
        self.assertEqual(dv.tmux_literal(";"), "\\;")
        self.assertEqual(dv.tmux_literal("a"), "a")

    def test_rejects_ambiguous_step(self):
        self.spec_path.write_text(SPEC + '[[steps]]\ntype = "x"\nkey = "Enter"\n')
        with self.assertRaisesRegex(dv.DemoError, "exactly one"):
            dv.load_spec(self.spec_path)

    def test_edit_events_trims_caps_compresses_and_holds(self):
        events = [
            [0.5, "o", "a"],
            [10.0, "o", "b"],
            [11.0, "o", "c"],
            [21.0, "o", "d"],
            [40.0, "o", "late"],
        ]
        edited = dv.edit_events(
            events, end=30.0, pauses=[(22.0, 25.0)], compress=[(11.0, 21.0, 2.0)]
        )
        data = [e[2] for e in edited]
        self.assertNotIn("late", data)
        times = {e[2]: e[0] for e in edited if e[2] != dv.KEEPALIVE}
        self.assertAlmostEqual(
            times["b"] - times["a"], dv.IDLE_LIMIT
        )  # idle gap capped
        self.assertAlmostEqual(
            times["d"] - times["c"], dv.IDLE_LIMIT
        )  # 10s window -> 2s, then capped
        holds = [e[0] for e in edited if e[2] == dv.KEEPALIVE]
        self.assertAlmostEqual(holds[1] - holds[0], 3.0)  # pause kept intact
        self.assertEqual(edited, sorted(edited, key=lambda e: e[0]))

    def test_compress_scales_dense_output(self):
        events = [[t / 10, "o", "."] for t in range(0, 201)]  # 20s of spinner
        edited = dv.edit_events(
            events, end=20.0, pauses=[], compress=[(0.0, 20.0, 4.0)]
        )
        self.assertAlmostEqual(edited[-1][0], 4.0, places=3)

    def test_title_card_contents(self):
        spec = dv.load_spec(self.spec_path)
        events = dv.title_events(spec, "abc123def456", "2026-10-06", "glm-5.2 (zai)")
        body = events[0][2]
        for text in (
            "/status shows permissions",
            "abc123def456",
            "2026-10-06",
            "glm-5.2 (zai)",
            "Expected result",
        ):
            self.assertIn(text, body)
        self.assertEqual(events[-1], [dv.TITLE_SECONDS, "o", "\x1bc"])

    def test_credentials_are_resolved_in_the_shell_not_inlined(self):
        spec = dv.load_spec(self.spec_path)
        prefix = dv.credential_prefix(spec, {"OTHER_KEY": "file:/tmp/k"})
        self.assertIn('ZAI_API_KEY="$(', prefix)
        self.assertIn("vault auth-helper provider/zai_api_key)", prefix)
        # The user's own vault is read outside the keyring isolation.
        self.assertIn("$(env -u CORBANU_TEST_NO_NATIVE_KEYRING ", prefix)
        self.assertIn('OTHER_KEY="$(cat /tmp/k)"', prefix)
        with self.assertRaises(dv.DemoError):
            dv.credential_prefix(spec, {"X": "literal:abc"})

    def test_refuses_without_keyring_isolation(self):
        with self.assertRaises(dv.DemoError):
            dv.require_keyring_isolation({})
        with self.assertRaises(dv.DemoError):
            dv.require_keyring_isolation({"CORBANU_TEST_NO_NATIVE_KEYRING": ""})
        dv.require_keyring_isolation({"CORBANU_TEST_NO_NATIVE_KEYRING": "1"})

    def test_prepare_run_and_launcher(self):
        spec = dv.load_spec(self.spec_path)
        run, places = dv.prepare_run(spec, self.dir / "out")
        self.assertIn(
            places["workspace"], (run / "workspace" / "README.md").read_text()
        )
        config = (run / "home" / "config.toml").read_text()
        self.assertIn('trust_level = "trusted"', config)
        launcher = dv.write_launcher(
            run, spec, Path("/bin/echo"), places, 'ZAI_API_KEY="$(true)"'
        )
        script = launcher.read_text()
        self.assertIn('ZAI_API_KEY="$(true)" exec env', script)
        self.assertIn("-m glm-5.2", script)
        self.assertIn("model_provider=", script)
        self.assertIn(f"CORBANU_HOME={places['home']}", script)
        self.assertIn("CORBANU_TEST_NO_NATIVE_KEYRING=1", script)

    def test_scan_blocks_published_and_redacts_private(self):
        cast = self.dir / "cast"
        cast.write_bytes(b"hello synthetic-canary-value and sk-" + b"a" * 30)
        log = self.dir / "log"
        log.write_bytes(b"env ZAI=synthetic-canary-value end")
        with mock.patch.dict(os.environ, {"CANARY": "synthetic-canary-value"}):
            hits = dv.scan_files([cast], [log], ["CANARY"])
        self.assertEqual(hits["published_literal"], 1)
        self.assertEqual(hits["published_pattern"], 1)
        self.assertEqual(hits["redacted_private_files"], [str(log)])
        self.assertEqual(log.read_bytes(), b"env ZAI=" + b"*" * 22 + b" end")

    def test_update_index_replaces_same_asset(self):
        manifest = {
            "sprint": "PF-1",
            "date": "2026-10-06",
            "id": "d",
            "feature": "F",
            "commit": "abc",
            "model": "glm-5.2",
            "seconds": 42.0,
        }
        index_dir = self.dir / "index"
        dv.update_index(manifest, "https://x/a.mp4", "https://x/a.cast", index_dir)
        path = dv.update_index(
            manifest, "https://x/a.mp4", "https://x/a.cast", index_dir
        )
        text = path.read_text()
        self.assertEqual(text.count("https://x/a.mp4"), 1)
        self.assertIn("# PF-1 demo videos", text)


if __name__ == "__main__":
    unittest.main()
