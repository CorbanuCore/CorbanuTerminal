"""Protocol regressions only; these do not certify a container or Chromium."""
import base64
import importlib.util
import io
import json
from pathlib import Path
import sys
import types
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("worker", Path(__file__).with_name("worker.py"))
worker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(worker)


class Page:
    url = "https://example.com/"

    def __init__(self, broken_setup=False):
        self.context = self
        self.broken_setup = broken_setup
        self.handler = None
        self.websocket_handler = None

    def route(self, pattern, handler):
        self.handler = handler

    def route_web_socket(self, pattern, handler):
        if self.broken_setup:
            raise ValueError("fixture setup failure")
        self.websocket_handler = handler

    def content(self):
        return "<html>untrusted</html>"


class WorkerTests(unittest.TestCase):
    def run_acquisition(self, page):
        def fetch(url, **options):
            self.assertEqual(options["retries"], 1)
            self.assertEqual(options["additional_args"], {"service_workers": "block", "accept_downloads": False})
            # Match pinned Scrapling's callback-error handling.
            for callback in [options["page_setup"], options["page_action"]]:
                try:
                    callback(page)
                except Exception:
                    pass

        fetchers = types.ModuleType("scrapling.fetchers")
        fetchers.DynamicFetcher = types.SimpleNamespace(fetch=fetch)
        wire = io.StringIO()
        stdin = types.SimpleNamespace(buffer=io.BytesIO(b'{"url":"https://example.com/"}\n'))
        with patch.dict(sys.modules, {"scrapling.fetchers": fetchers}), patch.object(sys, "stdin", stdin), patch.object(worker, "WIRE", wire):
            worker.acquire()
        return json.loads(wire.getvalue())

    def test_raw_content_requires_routing_and_capture(self):
        page = Page()
        result = self.run_acquisition(page)
        self.assertIsNotNone(page.handler)
        self.assertIsNotNone(page.websocket_handler)
        self.assertEqual(result, {"type": "result", "url": page.url, "body": base64.b64encode(page.content().encode()).decode()})

    def test_swallowed_scrapling_setup_error_cannot_be_a_success(self):
        with self.assertRaises(ValueError):
            self.run_acquisition(Page(broken_setup=True))

    def test_receiver_rejects_missing_newline_and_non_object(self):
        for line in [b'{}', b'[]\n', b'x' * (worker.MAX_LINE + 1)]:
            with patch.object(sys, "stdin", types.SimpleNamespace(buffer=io.BytesIO(line))):
                with self.assertRaises(ValueError):
                    worker.receive()

    def test_sender_bounds_before_writing(self):
        wire = io.StringIO()
        with patch.object(worker, "WIRE", wire), self.assertRaises(ValueError):
            worker.send({"body": "x" * worker.MAX_LINE})
        self.assertEqual(wire.getvalue(), "")


if __name__ == "__main__":
    unittest.main()
