import http.client
import json
import socket
import threading
import unittest

import security_level_capture as capture
from security_level_evidence import EvidenceError


class CaptureTests(unittest.TestCase):
    def test_uppercase_and_mixed_case_hex_leakage_fails_in_all_consumers(self):
        canary = capture.new_canary()
        lower = canary.hex()
        mixed = "".join(
            char.upper() if index % 3 else char for index, char in enumerate(lower)
        )
        headers = [
            ("Host", "api.openai.com"),
            ("Authorization", "Bearer " + canary.decode()),
        ]
        for form in (lower.upper().encode(), mixed.encode()):
            self.assertEqual(
                capture.scan_surfaces({"log": b"hex=" + form}, canary, ["log"]),
                {"log": "failed"},
            )
            self.assertFalse(
                capture.Capture(canary).record("POST", "/v1/responses", headers, form)
            )
            self.assertFalse(
                capture.Capture(canary).record(
                    "POST",
                    "/v1/responses",
                    headers + [("X-Leak", form.decode())],
                    b"{}",
                )
            )

    def test_canaries_are_unique_and_all_encodings_are_scanned(self):
        canary = capture.new_canary()
        self.assertNotEqual(canary, capture.new_canary())
        for form in capture.canary_forms(canary):
            self.assertEqual(
                capture.scan_surfaces({"log": b"prefix" + form}, canary, ["log"]),
                {"log": "failed"},
            )
        with self.assertRaises(EvidenceError):
            capture.scan_surfaces({}, canary, ["log"])

    def test_loopback_request_capture_and_cleanup(self):
        canary = capture.new_canary()
        with capture.capture_proxy(canary) as (probe, address):
            self.assertEqual(address[0], "127.0.0.1")
            client = http.client.HTTPConnection(*address, timeout=3)
            client.request(
                "POST",
                "/v1/responses",
                body=b"{}",
                headers={
                    "Host": "api.openai.com",
                    "Authorization": "Bearer " + canary.decode(),
                },
            )
            response = client.getresponse()
            self.assertEqual(response.status, 200)
            self.assertNotIn(canary, response.read())
            client.close()
            report = probe.report()
            self.assertEqual(report["status"], "passed")
            self.assertEqual(report["tls_qualification"], "pending")
            self.assertNotIn(canary.decode(), json.dumps(report))
        with self.assertRaises(OSError):
            socket.create_connection(address, timeout=0.2)

    def test_duplicate_and_malformed_requests_fail_closed(self):
        canary = capture.new_canary()
        headers = [
            ("Host", "api.openai.com"),
            ("Authorization", "Bearer " + canary.decode()),
        ]
        good = capture.Capture(canary)
        self.assertTrue(good.record("POST", "/v1/responses", headers, b"{}"))
        self.assertFalse(good.record("POST", "/v1/responses", headers, b"{}"))
        self.assertEqual(good.report()["status"], "failed")
        attempts = [
            ("GET", "/v1/responses", headers, b"{}"),
            ("POST", "/v1/../private", headers, b"{}"),
            ("POST", "http://api.openai.com/v1/responses", headers, b"{}"),
            ("POST", "/v1/responses?redirect=1", headers, b"{}"),
            ("POST", "/v1/responses", headers + [headers[1]], b"{}"),
            ("POST", "/v1/responses", headers, canary),
        ]
        for attempt in attempts:
            with self.subTest(attempt=attempt[:2]):
                self.assertFalse(capture.Capture(canary).record(*attempt))

    def test_proxy_does_not_forward_connect_or_reflect_secret_errors(self):
        canary = capture.new_canary()
        with capture.capture_proxy(canary) as (probe, address):
            client = http.client.HTTPConnection(*address, timeout=3)
            client.request("CONNECT", canary.decode())
            response = client.getresponse()
            self.assertEqual(response.status, 405)
            self.assertNotIn(canary, response.read())
            client.close()
            self.assertEqual(probe.report()["status"], "failed")

    def test_fake_executor_records_concurrent_duplicates_without_side_effects(self):
        executor = capture.FakeExecutor()
        action = {"id": "fake", "broadcast": False}
        workers = [
            threading.Thread(target=executor.record, args=(action,)) for _ in range(2)
        ]
        for worker in workers:
            worker.start()
        for worker in workers:
            worker.join()
        self.assertFalse(executor.matches([action]))
        self.assertTrue(executor.matches([action, action]))
        self.assertFalse(executor.matches([{"id": "fake", "broadcast": 0}] * 2))


if __name__ == "__main__":
    unittest.main()
