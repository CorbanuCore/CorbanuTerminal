import pathlib
import unittest
from unittest.mock import patch

import facility_control
from facilities import FACILITIES


class FacilityControlTests(unittest.TestCase):
    def test_status_groups_units_by_host_and_fails_closed(self):
        key = pathlib.Path("/tmp/owner-only-key")
        states = {
            "comfyui-ui.service": "active",
            "yue2-ui.service": "active",
            "ace-step-ui.service": "active",
            "minimax-music3-api.service": "inactive",
            "minimax-music3-ui.service": "active",
            "minimax-music3-download.service": "active",
            "minimax-music3-activate.service": "activating",
        }

        def fake_run(host, _key, args):
            if host == "100.81.145.102":
                return None, "unreachable", 255
            units = args[3:]
            return [states[unit] for unit in units], None, 3

        with patch.object(facility_control, "_run_remote", side_effect=fake_run) as remote:
            records = facility_control.get_statuses(key)

        self.assertEqual(remote.call_count, 2)
        self.assertEqual(records["comfyui"]["status"], "running")
        self.assertEqual(records["minimax-music3"]["status"], "initializing")
        self.assertFalse(records["minimax-music3"]["actions"]["start"])
        self.assertEqual(records["rvc"]["status"], "unreachable")
        self.assertFalse(records["rvc"]["actions"]["stop"])
        self.assertNotIn("no action was attempted", records["rvc"]["detail"])

    def test_action_rejects_unknown_and_unreachable_without_remote_mutation(self):
        key = pathlib.Path("/tmp/owner-only-key")
        with patch.object(facility_control, "_run_remote", side_effect=AssertionError("unknown id must not probe")):
            code, payload = facility_control.action("missing", "start", key)
        self.assertEqual(code, 404)
        self.assertFalse(payload["ok"])

        unreachable = {"rvc": {"status": "unreachable", "detail": "offline"}}
        with patch.object(facility_control, "get_statuses", return_value=unreachable), \
             patch.object(facility_control, "_perform_action", side_effect=AssertionError("unreachable must not mutate")):
            code, payload = facility_control.action("rvc", "stop", key)
        self.assertEqual(code, 503)
        self.assertFalse(payload["ok"])

    def test_action_requires_zero_exit_from_start_stop_command(self):
        item = next(item for item in FACILITIES if item["id"] == "comfyui")
        key = pathlib.Path("/tmp/owner-only-key")
        with patch.object(facility_control, "_run_remote", return_value=([], None, 1)):
            ok, message = facility_control._perform_action(item, "start", key)
        self.assertFalse(ok)
        self.assertIn("action failed", message)

    def test_stop_accepts_systemctl_is_active_inactive_exit_code(self):
        item = next(item for item in FACILITIES if item["id"] == "comfyui")
        key = pathlib.Path("/tmp/owner-only-key")
        with patch.object(facility_control, "_run_remote", side_effect=[([], None, 0), (["inactive"], None, 3)]):
            ok, message = facility_control._perform_action(item, "stop", key)
        self.assertTrue(ok)
        self.assertIn("verified", message)

    def test_ssh_command_is_batch_mode_and_does_not_embed_passwords(self):
        command = facility_control._ssh_command("100.99.88.49", pathlib.Path("/tmp/key"),
                                                ["systemctl", "--user", "is-active", "comfyui-ui.service"])
        rendered = " ".join(command)
        self.assertIn("BatchMode=yes", rendered)
        self.assertIn("IdentitiesOnly=yes", rendered)
        self.assertIn("systemctl --user is-active comfyui-ui.service", rendered)
        self.assertNotIn("password", rendered.lower())

    def test_incomplete_or_failed_probe_cannot_verify_action(self):
        item = next(item for item in FACILITIES if item["id"] == "minimax-music3")
        for verb, values in (("start", ["active"]), ("stop", ["inactive"])):
            wrong_code = 3 if verb == "start" else 0
            for probe, code in (([], 1), (values, 0), (values * 2, 1), (values * 3, 0), (values * 2, wrong_code)):
                with self.subTest(verb=verb, probe=probe, code=code), patch.object(
                    facility_control, "_run_remote", side_effect=[([], None, 0), (probe, None, code)]
                ):
                    ok, message = facility_control._perform_action(item, verb, pathlib.Path("/tmp/key"))
                    self.assertFalse(ok)
                    self.assertIn("without a verifiable", message)

    def test_dispatched_connection_failure_does_not_deny_possible_action(self):
        item = FACILITIES[0]
        for code in (None, 255):
            with self.subTest(code=code), patch.object(
                facility_control, "_run_remote", return_value=(None, "unreachable", code)
            ):
                ok, message = facility_control._perform_action(item, "start", pathlib.Path("/tmp/key"))
            self.assertFalse(ok)
            self.assertIn("uncertain", message)
            self.assertNotIn("no action was attempted", message)


if __name__ == "__main__":
    unittest.main()
