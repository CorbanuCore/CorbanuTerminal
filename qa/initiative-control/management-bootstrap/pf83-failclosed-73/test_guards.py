"""Local controls only: no guest transport, product launch, or functional cases."""
import contextlib
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import dispatch_guard
import prepare_payload
import stage

HERE = Path(__file__).resolve().parent
PRIOR = HERE.parent / "pf83-handoff-70"
RESULTS = {}
GOOD = 'Darwin arm64\n503\nagent\n26.2\n"IOPlatformUUID" = "' + stage.GUEST_UUID + '"\n'


class Guards(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(dir=HERE)
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.key = self.root / "synthetic-identity"
        self.key.write_text("not a private key; synthetic fixture\n")
        self.key.chmod(0o600)

    def run_stage(self, hosts):
        stage.stage(self.root / "nonexistent-archive", self.root / "nonexistent-receipt",
                    self.key, hosts, self.root / "evidence")

    def test_wrong_host_key_transfers_nothing(self):
        hosts = self.root / "wrong_known_hosts"
        # Valid known-host syntax with deliberately different public key bytes.
        hosts.write_text((HERE / "known_hosts").read_text().replace("VuRyCXyt", "VuRyCXyu"))
        with patch.object(stage.subprocess, "run") as run:
            with self.assertRaisesRegex(ValueError, "host key pin mismatch"):
                self.run_stage(hosts)
        run.assert_not_called()
        RESULTS["wrong_host_key"] = dict(transferred_bytes=0, transferred_files=[],
                                        ssh_calls=0, result="refused before transport")

    def test_wrong_uuid_transfers_nothing(self):
        wrong = GOOD.replace(stage.GUEST_UUID, "00000000-0000-0000-0000-000000000000")
        with patch.object(stage.subprocess, "run", return_value=subprocess.CompletedProcess(
                [], 0, wrong.encode())) as run:
            with self.assertRaisesRegex(ValueError, "UUID mismatch"):
                self.run_stage(HERE / "known_hosts")
        self.assertEqual(run.call_count, 1)
        self.assertEqual(run.call_args.kwargs["stdin"], subprocess.DEVNULL)
        RESULTS["wrong_uuid"] = dict(transferred_bytes=0, transferred_files=[],
                                    ssh_calls=1, result="refused after simulated identity reply",
                                    transport="local mock; no network connection")

    def test_ssh_host_rejection_aborts(self):
        with patch.object(stage.subprocess, "run", side_effect=subprocess.CalledProcessError(
                255, "ssh", stderr=b"Host key verification failed.")) as run:
            with self.assertRaises(subprocess.CalledProcessError):
                self.run_stage(HERE / "known_hosts")
        self.assertEqual(run.call_count, 1)
        self.assertTrue(run.call_args.kwargs["check"])
        self.assertEqual(run.call_args.kwargs["stdin"], subprocess.DEVNULL)

    def test_identity_positive_and_other_mismatches(self):
        stage.check_identity(GOOD)
        for old, new in [("26.2", "26.3"), ("503", "502"), ("agent", "root"),
                         ("arm64", "x86_64"), ('"IOPlatformUUID"', '"Missing"')]:
            with self.assertRaises(ValueError):
                stage.check_identity(GOOD.replace(old, new))
        cmd = stage.ssh_command(self.key, HERE / "known_hosts")
        for option in ["StrictHostKeyChecking=yes", "GlobalKnownHostsFile=/dev/null",
                       "HostKeyAlgorithms=ssh-ed25519", "BatchMode=yes"]:
            self.assertIn(option, cmd)

    def test_matching_identity_reaches_only_mock_upload(self):
        archive = self.root / "synthetic.tar"
        archive.write_bytes(b"SYNTHETIC TRANSPORT CONTROL")
        receipt = self.root / "receipt.json"
        receipt.write_text(json.dumps(dict(payload_kind="actor-assets-only-v1",
            archive_sha256=stage.sha(archive), inventory_sha256="a" * 64)))
        calls = []

        def transport(argv, **kwargs):
            calls.append(argv[-1])
            if len(calls) == 1:
                self.assertEqual(kwargs["stdin"], subprocess.DEVNULL)
                return subprocess.CompletedProcess(argv, 0, GOOD.encode())
            self.assertTrue(kwargs["check"])
            return subprocess.CompletedProcess(argv, 0)

        with patch.object(stage.subprocess, "run", side_effect=transport):
            stage.stage(archive, receipt, self.key, HERE / "known_hosts", self.root / "evidence")
        self.assertEqual(len(calls), 3)
        self.assertIn("tar -xpf", calls[1])
        RESULTS["positive_transport_control"] = dict(mock_calls=3, guest_contact=False)

    def test_stale_candidate_refuses_dispatch_and_matching_pin_passes(self):
        packets = PRIOR / "artifacts/rebound"
        bundle = PRIOR / "artifacts/handoff-root"
        expected = json.loads((packets / "packet-rebinding.json").read_bytes())["binding"]
        fixtures = self.root / "fixtures.py"
        fixtures.write_text("CANDIDATE = dict(commit='stale', tree='stale')\n")
        with patch.object(dispatch_guard.subprocess, "run") as run:
            with self.assertRaisesRegex(ValueError, "candidate disagreement; dispatch refused"):
                dispatch_guard.guarded(self.root, packets, bundle, ["never-execute"])
        run.assert_not_called()
        fixtures.write_text("CANDIDATE = " + repr(expected) + "\n")
        with contextlib.redirect_stdout(io.StringIO()):
            result = dispatch_guard.check(self.root, packets, bundle)
        self.assertEqual(result["checked_packets"], 288)
        RESULTS["candidate_disagreement"] = dict(dispatch_calls=0, result="refused",
                                                 matching_pin_checked_packets=288)

    def test_payload_allowlist_excludes_source_and_provenance(self):
        files = prepare_payload.selected(PRIOR / "artifacts/handoff-root")
        self.assertEqual(set(files), {
            "package/corbanu", "package/corbanu-acp", "package/corbanu-walletd",
            "package/codex-code-mode-host", "packet/original-F01-F11.md", "packet/navigation.md"})
        RESULTS["payload_allowlist"] = dict(files=sorted(files), product_staged=False,
                                             provenance_and_runtime="host only")

    def test_pin_parser_does_not_execute(self):
        fixtures = self.root / "fixtures.py"
        fixtures.write_text("CANDIDATE = __import__('os').system('false')\n")
        with self.assertRaisesRegex(ValueError, "nonliteral"):
            dispatch_guard.candidate_pin(fixtures)


if __name__ == "__main__":
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(Guards)
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    RESULTS["summary"] = dict(run=result.testsRun, failures=len(result.failures),
                              errors=len(result.errors), guest_contact=False,
                              functional_cases_executed=0)
    (HERE / "guard-controls.json").write_text(json.dumps(RESULTS, indent=2, sort_keys=True) + "\n")
    raise SystemExit(not result.wasSuccessful())
