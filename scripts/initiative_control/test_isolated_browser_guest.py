"""Synthetic protocol/browser/clock tests only; no browser, install, model or host service."""
import asyncio
import importlib.util
import json
from pathlib import Path
import struct
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import AsyncMock, MagicMock, patch
import zlib

import isolated_browser_guest as g

PACKET = dict(run_id="run1", case_id="case1", entry="index.html", profile="desktop")


def encoded(**fields):
    return json.dumps(fields).encode()


def png(size=(1440, 900), color=6, bits=8, interlace=0):
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
    header = struct.pack(">IIBBBBB", *size, bits, color, 0, 0, interlace)
    data = zlib.compress((b"\0" * (size[0] * (4 if color == 6 else 3) + 1)) * size[1])
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", data) + chunk(b"IEND", b"")


class FakeWire:
    def __init__(self, *requests):
        self.requests, self.events = list(requests), []

    async def read(self):
        return self.requests.pop(0) if self.requests else None

    async def write(self, event):
        self.events.append(event)


class FakeBrowser:
    def __init__(self):
        self.violation, self.actions = None, []
        self.start, self.close = AsyncMock(), AsyncMock()

    async def observe(self):
        return dict(text="fixture", text_truncated=False, png_base64="fake", url=g.ORIGIN + "/index.html")

    async def act(self, request):
        self.actions.append(request)


class ProtocolTests(unittest.IsolatedAsyncioTestCase):
    async def execute(self, *requests, browser=None):
        wire, browser = FakeWire(*requests), browser or FakeBrowser()
        code = await g.run(wire, PACKET, {}, browser)
        browser.close.assert_awaited_once()
        return code, wire.events, browser

    async def test_observation_consumed_text_and_enter_separate(self):
        requests = [encoded(seq=1, op="observe"),
                    encoded(seq=2, op="text", frame_id="run1:case1:1", text="literal\nEnter ✨"),
                    encoded(seq=3, op="observe"),
                    encoded(seq=4, op="key", frame_id="run1:case1:3", key="Enter"),
                    encoded(seq=5, op="finish")]
        code, events, browser = await self.execute(*requests)
        self.assertEqual(code, 0)
        self.assertEqual([e["status"] for e in events], ["observed", "executed", "observed", "executed", "terminal"])
        self.assertEqual(browser.actions[0]["text"], "literal\nEnter ✨")
        self.assertEqual(browser.actions[1]["key"], "Enter")
        self.assertIsNone(events[1]["frame_id"])
        self.assertEqual(events[-1]["reason"], "finish")

    async def test_fail_closed_requests(self):
        cases = [b'{"seq":2,"op":"observe","op":"finish"}', b"[]", b"null", b"\xff", b"{",
                 encoded(seq=2, op="evaluate", code="x"), encoded(seq=2, op="observe", path="/tmp"),
                 encoded(seq=2, op="click", frame_id="old", x=0, y=0),
                 encoded(seq=2, op="click", frame_id="run1:case1:1", x=True, y=0),
                 encoded(seq=2, op="text", frame_id="run1:case1:1", text="✨" * 1366),
                 encoded(seq=2, op="text", frame_id="run1:case1:1", text="\ud800"),
                 encoded(seq=2, op="key", frame_id="run1:case1:1", key="Control+L"),
                 encoded(seq=True, op="observe"), encoded(seq=1, op="observe"),
                 encoded(seq=2, op="click", frame_id="run1:case1:1", x=float("nan"), y=0),
                 b" " * g.REQUEST]
        for raw in cases:
            with self.subTest(raw=raw[:100]):
                code, events, browser = await self.execute(encoded(seq=1, op="observe"), raw)
                self.assertEqual(code, 1)
                self.assertEqual(events[-1]["reason"], "error")
                self.assertEqual(browser.actions, [])
                self.assertIn("error", events[-1])

    async def test_fresh_observe_required_after_each_mutation(self):
        code, events, browser = await self.execute(encoded(seq=1, op="observe"),
            encoded(seq=2, op="reload", frame_id="run1:case1:1"),
            encoded(seq=3, op="back", frame_id="run1:case1:1"))
        self.assertEqual((code, len(browser.actions)), (1, 1))
        self.assertIn("stale_frame", events[-1]["error"])

    async def test_eof_and_exact_action_budget(self):
        code, events, _ = await self.execute()
        self.assertEqual((code, events[-1]["reason"]), (1, "eof"))
        code, events, _ = await self.execute(*(encoded(seq=i, op="observe") for i in range(1, 41)))
        self.assertEqual((code, len(events)), (1, 41))
        self.assertIn("action_budget", events[-1]["error"])

    async def test_failed_partial_action_and_browser_denial(self):
        browser = FakeBrowser()
        browser.act = AsyncMock(side_effect=RuntimeError("raw attempt failed"))
        code, events, _ = await self.execute(encoded(seq=1, op="observe"),
            encoded(seq=2, op="reload", frame_id="run1:case1:1"), browser=browser)
        self.assertEqual(code, 1)
        self.assertEqual(events[-1]["error"], "RuntimeError: raw attempt failed")
        self.assertNotIn("executed", [e["status"] for e in events])
        browser = FakeBrowser()
        browser.violation = "download"
        code, events, _ = await self.execute(encoded(seq=1, op="finish"), browser=browser)
        self.assertIn("download", events[-1]["error"])

    async def test_clock_and_startup_action_run_cleanup_timeouts(self):
        with self.assertRaisesRegex(ValueError, "run_budget"):
            await g.session(PACKET, FakeBrowser(), FakeWire(), clock=iter([0, 301]).__next__)
        async def stall(*_):
            await asyncio.Future()
        for field in ("start", "act", "close"):
            browser = FakeBrowser()
            setattr(browser, field, AsyncMock(side_effect=stall))
            with patch.object(g, "ACTION_SECONDS", .01):
                code, events, _ = await self.execute(encoded(seq=1, op="observe"),
                    encoded(seq=2, op="reload", frame_id="run1:case1:1"), browser=browser)
            self.assertEqual(code, 1)
            self.assertIn("cleanup_error" if field == "close" else "error", events[-1])
        wire = FakeWire()
        wire.read = stall
        with patch.object(g, "RUN_SECONDS", .01):
            self.assertEqual(await g.run(wire, PACKET, {}, FakeBrowser()), 1)
        self.assertEqual(wire.events[-1]["reason"], "timeout")

    def test_types_and_viewport_edges(self):
        for profile, size in g.PROFILES.items():
            for op, values in (("click", dict(x=size[0] - .1, y=0)),
                               ("hover", dict(x=0, y=size[1] - .1)),
                               ("scroll", dict(dx=-size[0], dy=size[1]))):
                request = dict(seq=2, op=op, frame_id="f", **values)
                self.assertEqual(g.validate(request, 1, "f", size), op)
                for bad in (-size[0] - 1, 2**10000, float("inf"), None, "1", [], True):
                    request[next(iter(values))] = bad
                    with self.assertRaises((ValueError, OverflowError)):
                        g.validate(request, 1, "f", size)

    async def test_wire_eof_oversize_partial_and_slow(self):
        for chunks, expected in (([b""], None), ([b"a\nb\n"], b"a"),
                                 ([b"x" * 8191 + b"\n"], b"x" * 8191)):
            wire = object.__new__(g.Wire)
            wire.buffer, wire.chunk = b"", AsyncMock(side_effect=chunks)
            self.assertEqual(await wire.read(), expected)
        for chunks in ([b"a", b""], [b"x" * 8193], [b"x" * 8192 + b"\n"]):
            wire = object.__new__(g.Wire)
            wire.buffer, wire.chunk = b"", AsyncMock(side_effect=chunks)
            with self.assertRaises(ValueError):
                await wire.read()
        wire.buffer = b"a"
        wire.chunk = AsyncMock(side_effect=lambda: None)
        async def stall():
            await asyncio.Future()
        wire.chunk = stall
        with patch.object(g, "ACTION_SECONDS", .01), self.assertRaises(TimeoutError):
            await wire.read()

    async def test_wire_nonblocking_output_and_failures(self):
        wire = object.__new__(g.Wire)
        wire.ready = AsyncMock()
        with patch.object(g.os, "read", side_effect=[BlockingIOError(), b"x"]):
            self.assertEqual(await wire.chunk(), b"x")
        with patch.object(g.os, "write", side_effect=[BlockingIOError(), 2, 999]) as write:
            await wire.write({"ok": 1})
            self.assertEqual(write.call_args_list[1].args[1], b'{"ok":1}\n')
        with self.assertRaisesRegex(ValueError, "output_budget"):
            await wire.write({"x": "x" * g.OUTPUT})
        async def stall(*_):
            await asyncio.Future()
        wire.ready = stall
        with patch.object(g.os, "write", side_effect=BlockingIOError()), patch.object(g, "ACTION_SECONDS", .01):
            with self.assertRaises(TimeoutError):
                await wire.write({})
        fake = FakeWire(encoded(seq=1, op="finish"))
        fake.write = AsyncMock(side_effect=BrokenPipeError())
        self.assertEqual(await g.run(fake, PACKET, {}, FakeBrowser()), 2)

    async def test_fd_registration_removed_on_cancellation(self):
        wire, loop = object.__new__(g.Wire), asyncio.get_running_loop()
        for writing in (False, True):
            add, remove = ("add_writer", "remove_writer") if writing else ("add_reader", "remove_reader")
            with patch.object(loop, add) as register, patch.object(loop, remove) as unregister:
                task = asyncio.create_task(wire.ready(99, writing))
                await asyncio.sleep(0)
                register.assert_called_once()
                task.cancel()
                with self.assertRaises(asyncio.CancelledError):
                    await task
                unregister.assert_called_once_with(99)

    async def test_error_truncation_and_post_action_denial(self):
        browser = FakeBrowser()
        browser.act = AsyncMock(side_effect=RuntimeError("x" * 5000))
        _, events, _ = await self.execute(encoded(seq=1, op="observe"),
            encoded(seq=2, op="reload", frame_id="run1:case1:1"), browser=browser)
        self.assertEqual(len(events[-1]["error"]), 4096)
        self.assertTrue(events[-1]["error_truncated"])
        browser = FakeBrowser()
        async def deny(_):
            browser.violation = "popup"
        browser.act = deny
        _, events, _ = await self.execute(encoded(seq=1, op="observe"),
            encoded(seq=2, op="reload", frame_id="run1:case1:1"), browser=browser)
        self.assertNotIn("executed", [e["status"] for e in events])


class AdapterTests(unittest.IsolatedAsyncioTestCase):
    async def test_runtime_pins_routes_events_and_cleanup(self):
        page = MagicMock(url=g.ORIGIN + "/index.html")
        page.goto, page.close = AsyncMock(), AsyncMock()
        context = MagicMock(new_page=AsyncMock(return_value=page), route=AsyncMock(), route_web_socket=AsyncMock())
        browser = MagicMock(new_context=AsyncMock(return_value=context), close=AsyncMock())
        driver = SimpleNamespace(chromium=SimpleNamespace(launch=AsyncMock(return_value=browser)), stop=AsyncMock())
        module = SimpleNamespace(async_playwright=lambda: SimpleNamespace(start=AsyncMock(return_value=driver)))
        server = MagicMock()
        with patch.dict(sys.modules, {"playwright.async_api": module}), patch.object(g.importlib.metadata, "version", return_value="1.59.0"), patch.object(g.asyncio, "start_server", AsyncMock(return_value=server)):
            guest = g.Browser(PACKET)
            await guest.start({"/index.html": ("text/html", b"fixture"), "/app.js": ("text/javascript", b"")})
        launch = driver.chromium.launch.call_args.kwargs
        self.assertEqual(launch["executable_path"], g.CHROME)
        self.assertFalse(launch["chromium_sandbox"])
        self.assertNotIn("HTTP_PROXY", launch["env"])
        self.assertEqual(browser.new_context.call_args.kwargs["service_workers"], "block")
        self.assertEqual(page.goto.call_args.args, (g.ORIGIN + "/index.html",))
        for url, nav, method, allowed in ((g.ORIGIN + "/index.html", True, "GET", True),
             (g.ORIGIN + "/app.js", False, "GET", True), (g.ORIGIN + "/app.js", True, "GET", False),
             (g.ORIGIN + "/missing.html", True, "GET", False), (g.ORIGIN + "/app.js.map", False, "GET", False),
             ("file:///packet/case.json", True, "GET", False), ("https://external/", False, "GET", False),
             (g.ORIGIN + "/index.html", False, "POST", False)):
            route = SimpleNamespace(request=SimpleNamespace(url=url, method=method, is_navigation_request=lambda: nav),
                                    continue_=AsyncMock(), abort=AsyncMock())
            await guest.route(route)
            (route.continue_ if allowed else route.abort).assert_awaited_once()
        callbacks = dict(page.on.call_args_list[i].args for i in range(len(page.on.call_args_list)))
        callbacks["download"](None)
        await asyncio.sleep(0)
        self.assertEqual(guest.violation, "download")
        await guest.close()
        browser.close.assert_awaited_once()
        driver.stop.assert_awaited_once()
        server.close.assert_called_once()

    async def test_literal_adapter_and_bounded_observation(self):
        guest = g.Browser(PACKET)
        page = SimpleNamespace(mouse=SimpleNamespace(click=AsyncMock(), move=AsyncMock(), wheel=AsyncMock()),
            keyboard=SimpleNamespace(insert_text=AsyncMock(), press=AsyncMock()), go_back=AsyncMock(), reload=AsyncMock(),
            locator=MagicMock(return_value=SimpleNamespace(evaluate=AsyncMock(return_value=("✨" * 16384, True)))),
            screenshot=AsyncMock(return_value=png()), url=g.ORIGIN + "/index.html")
        guest.page = page
        for op, fields in (("text", {"text": " x\nEnter "}), ("key", {"key": "Enter"}),
            ("click", {"x": 1, "y": 2}), ("hover", {"x": 2, "y": 3}), ("scroll", {"dx": 1, "dy": -2}),
            ("back", {}), ("reload", {})):
            await guest.act(dict(op=op, **fields))
        page.keyboard.insert_text.assert_awaited_once_with(" x\nEnter ")
        page.keyboard.press.assert_awaited_once_with("Enter")
        result = await guest.observe()
        self.assertLessEqual(len(result["text"].encode()), g.BODY)
        self.assertTrue(result["text_truncated"])
        self.assertFalse(page.screenshot.call_args.kwargs["full_page"])

    def test_png_and_origin_rejections(self):
        for size in g.PROFILES.values():
            for color in (2, 6):
                g.check_png(png(size, color), size)
        for data in (b"", png((1, 1)), png(bits=16), png(color=0), png(interlace=1), png() + b"x", png()[:-1]):
            with self.assertRaises(ValueError):
                g.check_png(data, g.PROFILES["desktop"])
        for url in ("file:///tmp", "http://localhost:8768/", g.ORIGIN + "@evil/x", g.ORIGIN + "/\\evil",
                    "http://127.0.0.1:87680/", "data:text/html,x", g.ORIGIN + "/\nx"):
            self.assertFalse(g.local_url(url))

    async def test_http_exact_asset_no_listing_or_traversal(self):
        for target, allowed in (("/index.html", True), ("/index.html?x=1", True), ("/", False),
                                ("/../case.json", False), ("/%2e%2e/case.json", False), ("/index.html.map", False)):
            reader = asyncio.StreamReader()
            reader.feed_data(f"GET {target} HTTP/1.1\r\nHost: 127.0.0.1:8768\r\n\r\n".encode())
            writer = MagicMock(drain=AsyncMock())
            await g.serve(reader, writer, {"/index.html": ("text/html", b"fixture")})
            self.assertEqual(writer.write.called, allowed)
            writer.close.assert_called_once()


class PacketTests(unittest.TestCase):
    def test_packet_and_assets_validate_before_runtime(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            site, case = root / "site", root / "case.json"
            site.mkdir()
            (site / "index.html").write_text("fixture")
            case.write_text(json.dumps(PACKET))
            with patch.object(g, "CASE", case), patch.object(g, "SITE", site):
                self.assertEqual(g.load_packet()[0], PACKET)
                for change in ({"entry": "../case.json"}, {"entry": "/index.html"}, {"entry": "%69ndex.html"},
                               {"profile": ["desktop"]}, {"run_id": "../run"}, {"extra": "field"}):
                    case.write_text(json.dumps(dict(PACKET, **change)))
                    with self.assertRaises(ValueError):
                        g.load_packet()
                case.write_text(json.dumps(PACKET))
                for name in ("app.map", ".git", "source.py"):
                    asset = site / name
                    asset.write_text("excluded")
                    with self.assertRaises(ValueError):
                        g.load_packet()
                    asset.unlink()
                (site / "escape.html").symlink_to(case)
                with self.assertRaisesRegex(ValueError, "asset_type"):
                    g.load_packet()

    def test_import_help_inert(self):
        with patch.object(g.importlib.metadata, "version", side_effect=AssertionError("runtime touched")):
            spec = importlib.util.spec_from_file_location("inert_guest", g.__file__)
            spec.loader.exec_module(importlib.util.module_from_spec(spec))
            with patch.object(g, "Wire", side_effect=AssertionError("stdio touched")), patch.object(sys, "argv", ["guest", "--help"]):
                with self.assertRaises(SystemExit) as stopped:
                    g.main()
                self.assertEqual(stopped.exception.code, 0)


if __name__ == "__main__":
    unittest.main()
