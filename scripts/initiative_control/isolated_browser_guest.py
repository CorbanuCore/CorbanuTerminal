"""Fixed guest runtime. Requires parent-enforced containment; provides no host isolation."""

import argparse
import asyncio
import base64
import importlib.metadata
import json
import math
import os
from pathlib import Path
import re
import signal
import stat
import struct
import time
from urllib.parse import urlsplit
import zlib

ORIGIN = "http://127.0.0.1:8768"
CASE = Path("/packet/case.json")
SITE = Path("/packet/site")
CHROME = "/browser/chrome-linux/chrome"
PROFILES = {"desktop": (1440, 900), "phone": (390, 844)}
REQUEST, TEXT, BODY, PNG = 8192, 4096, 16384, 2 * 1024 * 1024
ACTION_SECONDS, RUN_SECONDS, ACTIONS = 5, 300, 40
OUTPUT = 3 * 1024 * 1024
KEYS = {"Enter", "Tab", "Escape", "Backspace", "Delete", "ArrowUp", "ArrowDown",
        "ArrowLeft", "ArrowRight", "Home", "End", "PageUp", "PageDown", "Space"}
MIME = {".html": "text/html", ".css": "text/css", ".js": "text/javascript",
        ".png": "image/png", ".jpg": "image/jpeg", ".jpeg": "image/jpeg",
        ".svg": "image/svg+xml", ".webp": "image/webp", ".ico": "image/x-icon",
        ".woff": "font/woff", ".woff2": "font/woff2", ".json": "application/json"}


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def strict_json(raw):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, "duplicate_key")
            result[key] = value
        return result
    return json.loads(raw.decode("utf-8"), object_pairs_hook=pairs,
                      parse_constant=lambda _: require(False, "nonfinite_json"))


def relative(value):
    require(type(value) is str and len(value) <= 512 and all(
        re.fullmatch(r"[A-Za-z0-9_-][A-Za-z0-9_.-]*", p) and p not in
        {"src", "source", "history", "auth", "debug", "credentials"}
        for p in value.split("/")), "invalid_relative_path")
    return value


def load_packet():
    for path in (CASE.parent, SITE):
        require(stat.S_ISDIR(path.lstat().st_mode), "packet_directory")
    info = CASE.lstat()
    require(stat.S_ISREG(info.st_mode) and info.st_size <= REQUEST and
            info.st_uid in {0, os.getuid()} and not info.st_mode & 0o022,
            "packet_not_owner_controlled")
    packet = strict_json(CASE.read_bytes())
    require(type(packet) is dict and set(packet) ==
            {"run_id", "case_id", "entry", "profile"}, "packet_fields")
    for key in ("run_id", "case_id"):
        require(type(packet[key]) is str and
                re.fullmatch(r"[A-Za-z0-9_-]{1,64}", packet[key]), "packet_identity")
    require(type(packet["profile"]) is str and packet["profile"] in PROFILES,
            "packet_profile")
    entry = relative(packet["entry"])
    assets, total, entries = {}, 0, 0
    # Owner freezes this read-only tree; reject links/special files before any read.
    for root, dirs, files in os.walk(SITE, followlinks=False):
        for name in dirs + files:
            entries += 1
            require(entries <= 1024, "site_entry_budget")
            path = Path(root) / name
            rel = relative(path.relative_to(SITE).as_posix())
            info = path.lstat()
            require(stat.S_ISDIR(info.st_mode) or stat.S_ISREG(info.st_mode), "asset_type")
            require(not info.st_mode & 0o022, "asset_writable")
            if stat.S_ISREG(info.st_mode):
                require(path.suffix in MIME and info.st_size <= 8 * 1024 * 1024,
                        "asset_format_or_size")
                total += info.st_size
                require(len(assets) < 512 and total <= 32 * 1024 * 1024, "site_budget")
                assets["/" + rel] = (MIME[path.suffix], path.read_bytes())
    require("/" + entry in assets and entry.endswith(".html"), "entry_not_html")
    return packet, assets


def local_url(url):
    return (type(url) is str and len(url) <= 2048 and url.startswith(ORIGIN + "/")
            and not any(ord(c) < 33 or c == "\\" for c in url)
            and urlsplit(url).netloc == "127.0.0.1:8768")


def check_png(png, size):
    require(45 <= len(png) <= PNG and png[:8] == b"\x89PNG\r\n\x1a\n", "screenshot_bounds")
    offset, kinds = 8, []
    while offset < len(png):
        require(offset + 12 <= len(png), "png_chunk")
        length, kind = struct.unpack(">I4s", png[offset:offset + 8])
        end = offset + 12 + length
        require(end <= len(png) and kind in {b"IHDR", b"IDAT", b"IEND"}, "png_chunk")
        require(zlib.crc32(png[offset + 4:end - 4]) == struct.unpack(">I", png[end - 4:end])[0], "png_crc")
        kinds.append(kind)
        offset = end
    require(kinds[0] == b"IHDR" and kinds[-1] == b"IEND" and len(kinds) >= 3
            and all(k == b"IDAT" for k in kinds[1:-1]) and png[-12:-8] == b"\0\0\0\0"
            and png[8:12] == b"\0\0\0\r" and struct.unpack(">II", png[16:24]) == size
            and png[24] == 8 and png[25] in {2, 6} and png[26:29] == b"\0\0\0", "png_format")


async def serve(reader, writer, assets):
    try:
        async with asyncio.timeout(ACTION_SECONDS):
            header = await reader.readuntil(b"\r\n\r\n")
            lines = header.decode("ascii").split("\r\n")
            method, target, version = lines[0].split(" ")
            require(len(header) <= REQUEST and method == "GET" and version == "HTTP/1.1"
                    and lines[1:].count("Host: 127.0.0.1:8768") == 1, "http_request")
            path = urlsplit(target).path
            require(target.startswith("/") and path in assets, "http_asset")
            mime, data = assets[path]
            writer.write((f"HTTP/1.1 200 OK\r\nContent-Type: {mime}\r\n"
                          f"Content-Length: {len(data)}\r\nConnection: close\r\n"
                          "X-Content-Type-Options: nosniff\r\n\r\n").encode() + data)
            await writer.drain()
    except Exception:
        pass  # Invalid HTTP is closed; never provide listings, paths or diagnostics.
    finally:
        writer.close()


class Wire:
    """Finite nonblocking stdio; idle waits use run budget, partial lines get five seconds."""
    def __init__(self):
        self.buffer = b""
        for fd in (0, 1):
            os.set_blocking(fd, False)

    async def ready(self, fd, writing=False):
        loop, future = asyncio.get_running_loop(), asyncio.get_running_loop().create_future()
        add = loop.add_writer if writing else loop.add_reader
        remove = loop.remove_writer if writing else loop.remove_reader
        add(fd, lambda: None if future.done() else future.set_result(None))
        try:
            await future
        finally:
            remove(fd)

    async def chunk(self):
        while True:
            try:
                return os.read(0, REQUEST + 1)
            except BlockingIOError:
                await self.ready(0)

    async def read(self):
        if not self.buffer:
            self.buffer = await self.chunk()
        if not self.buffer:
            return None
        async with asyncio.timeout(ACTION_SECONDS):
            while b"\n" not in self.buffer:
                require(len(self.buffer) <= REQUEST, "oversize_request")
                chunk = await self.chunk()
                require(chunk, "partial_eof")
                self.buffer += chunk
            line, self.buffer = self.buffer.split(b"\n", 1)
            require(len(line) + 1 <= REQUEST, "oversize_request")
            return line

    async def write(self, event):
        data = json.dumps(event, ensure_ascii=True, allow_nan=False, separators=(",", ":")).encode() + b"\n"
        require(len(data) <= OUTPUT, "output_budget")
        async with asyncio.timeout(ACTION_SECONDS):
            while data:
                try:
                    count = os.write(1, data)
                    require(count > 0, "output_closed")
                    data = data[count:]
                except BlockingIOError:
                    await self.ready(1, True)


class Browser:
    def __init__(self, packet):
        self.packet, self.violation = packet, None
        self.driver = self.browser = self.server = None
        self.connections = set()

    async def start(self, assets):
        self.assets = assets
        require(importlib.metadata.version("playwright") == "1.59.0", "playwright_version")
        from playwright.async_api import async_playwright  # Runtime only, never import/help.
        async def connection(reader, writer):
            if len(self.connections) >= 16:
                writer.close()
                return
            self.connections.add(writer)
            try:
                await serve(reader, writer, assets)
            finally:
                self.connections.discard(writer)
        self.server = await asyncio.start_server(connection,
                                                "127.0.0.1", 8768, limit=REQUEST)
        self.driver = await async_playwright().start()
        self.browser = await self.driver.chromium.launch(
            executable_path=CHROME, headless=True, chromium_sandbox=False,
            args=["--no-sandbox", "--no-proxy-server", "--disable-extensions",
                  "--disable-background-networking", "--disable-component-update", "--disable-sync"],
            env={"PATH": "/usr/bin:/bin", "HOME": "/tmp", "TMPDIR": "/tmp", "LANG": "C.UTF-8"})
        width, height = PROFILES[self.packet["profile"]]
        self.context = await self.browser.new_context(viewport={"width": width, "height": height},
            device_scale_factor=1, accept_downloads=False, service_workers="block")
        await self.context.route("**/*", self.route)
        await self.context.route_web_socket("**/*", lambda socket: socket.close())
        self.page = await self.context.new_page()
        self.context.on("page", lambda page: self.deny("popup", page))
        self.page.on("download", lambda _: self.deny("download", self.page))
        self.page.on("dialog", lambda _: self.deny("dialog", self.page))
        self.page.on("crash", lambda _: self.deny("crash", self.page))
        self.page.on("close", lambda _: setattr(self, "violation", self.violation or "page_closed"))
        self.page.on("framenavigated", lambda frame: None if local_url(frame.url)
                     else self.deny("navigation", self.page))
        self.page.set_default_timeout(ACTION_SECONDS * 1000)
        await self.page.goto(ORIGIN + "/" + self.packet["entry"], wait_until="domcontentloaded")

    def deny(self, reason, page):
        self.violation = reason
        asyncio.create_task(page.close())

    async def route(self, route):
        request = route.request
        if (local_url(request.url) and urlsplit(request.url).path in self.assets and request.method == "GET" and
                (not request.is_navigation_request() or urlsplit(request.url).path.endswith(".html"))):
            await route.continue_()
        else:
            self.violation = "request_denied"
            await route.abort("blockedbyclient")

    async def observe(self):
        # Fixed script extracts rendered text only; no caller-supplied evaluation/selector.
        text, truncated = await self.page.locator("body").evaluate(
            "el => {const t=el.innerText; return [t.slice(0,16384),t.length>16384]}")
        raw = text.encode("utf-8")
        text = raw[:BODY].decode("utf-8", errors="ignore")
        png = await self.page.screenshot(type="png", full_page=False, timeout=5000)
        check_png(png, PROFILES[self.packet["profile"]])
        require(local_url(self.page.url), "observation_url")
        return dict(text=text, text_truncated=truncated or len(raw) > BODY,
                    png_base64=base64.b64encode(png).decode(), url=self.page.url)

    async def act(self, request):
        op = request["op"]
        if op in {"click", "hover"}:
            method = self.page.mouse.click if op == "click" else self.page.mouse.move
            await method(request["x"], request["y"])
        elif op == "text":
            await self.page.keyboard.insert_text(request["text"])
        elif op == "key":
            await self.page.keyboard.press(request["key"])
        elif op == "scroll":
            await self.page.mouse.wheel(request["dx"], request["dy"])
        elif op == "back":
            await self.page.go_back(wait_until="domcontentloaded")
        elif op == "reload":
            await self.page.reload(wait_until="domcontentloaded")

    async def close(self):
        if self.server:
            self.server.close()
        for writer in list(self.connections):
            writer.close()
        results = await asyncio.gather(*(getattr(resource, method)() for resource, method in
            ((self.browser, "close"), (self.driver, "stop")) if resource), return_exceptions=True)
        require(not any(isinstance(r, BaseException) for r in results), "browser_cleanup:" + str(results))


def validate(request, last_seq, frame, size):
    require(type(request) is dict, "request_object")
    op = request.get("op")
    require(type(op) is str and op in {"observe", "click", "hover", "text", "key",
                                     "scroll", "back", "reload", "finish"}, "unknown_operation")
    fields = {"click": {"x", "y"}, "hover": {"x", "y"}, "text": {"text"},
              "key": {"key"}, "scroll": {"dx", "dy"}}.get(op, set())
    mutate = op not in {"observe", "finish"}
    require(set(request) == {"seq", "op"} | fields | ({"frame_id"} if mutate else set()), "request_fields")
    require(type(request["seq"]) is int and last_seq < request["seq"] <= 2**53 - 1, "sequence")
    if mutate:
        require(frame is not None and type(request["frame_id"]) is str and
                request["frame_id"] == frame, "stale_frame")
    for key, limit in zip(("x", "y") if op in {"click", "hover"} else ("dx", "dy"), size):
        if key in request:
            value = request[key]
            require(type(value) in {int, float} and math.isfinite(value) and
                    (0 <= value < limit if key in {"x", "y"} else -limit <= value <= limit), "coordinate")
    if op == "text":
        require(type(request["text"]) is str and len(request["text"].encode("utf-8")) <= TEXT, "text_bounds")
    if op == "key":
        require(type(request["key"]) is str and request["key"] in KEYS, "key")
    return op


async def session(packet, browser, wire, clock=time.monotonic):
    deadline, seq, frame, count = clock() + RUN_SECONDS, 0, None, 0
    while True:
        require(clock() < deadline, "run_budget")
        require(count < ACTIONS, "action_budget")
        raw = await wire.read()
        require(not browser.violation, "browser_denied:" + str(browser.violation))
        if raw is None:
            return "eof"
        require(clock() < deadline and count < ACTIONS, "run_or_action_budget")
        require(len(raw) + 1 <= REQUEST, "oversize_request")
        request = strict_json(raw)
        op = validate(request, seq, frame, PROFILES[packet["profile"]])
        seq, count = request["seq"], count + 1
        if op == "finish":
            return "finish"
        async with asyncio.timeout(ACTION_SECONDS):
            if op == "observe":
                observation = await browser.observe()
                frame = f"{packet['run_id']}:{packet['case_id']}:{seq}"
                event = dict(status="observed", frame_id=frame, **observation)
            else:
                frame = None  # A failed/partial mutation cannot reuse its observation.
                await browser.act(request)
                event = dict(status="executed", frame_id=None)
            require(not browser.violation, "browser_denied:" + str(browser.violation))
        await wire.write(dict(seq=seq, run_id=packet["run_id"], case_id=packet["case_id"], **event))


async def run(wire, packet, assets, browser, budget=RUN_SECONDS):
    reason, error, cleanup, phase = "error", None, None, "startup"
    try:
        async with asyncio.timeout(min(budget, RUN_SECONDS)):
            await asyncio.wait_for(browser.start(assets), ACTION_SECONDS)
            phase = "protocol"
            reason = await session(packet, browser, wire)
    except Exception as exc:
        reason = "timeout" if isinstance(exc, TimeoutError) else "error"
        error = f"{type(exc).__name__}: {exc}"
    finally:
        try:
            await asyncio.wait_for(browser.close(), ACTION_SECONDS)
        except Exception as exc:
            cleanup = f"{type(exc).__name__}: {exc}"
    event = dict(status="terminal", reason=reason, phase=phase, run_id=packet["run_id"], case_id=packet["case_id"])
    for key, value in (("error", error), ("cleanup_error", cleanup)):
        if value is not None:
            event[key], event[key + "_truncated"] = value[:4096], len(value) > 4096
    try:
        await wire.write(event)
    except Exception:
        return 2  # Broken/stalled output cannot acknowledge a terminal receipt.
    return int(reason != "finish" or error is not None or cleanup is not None)


def main():
    argparse.ArgumentParser(description=__doc__).parse_args()
    started = time.monotonic()
    wire = Wire()
    def packet_timeout(*_):
        raise TimeoutError("packet_timeout")
    signal.signal(signal.SIGALRM, packet_timeout)
    signal.setitimer(signal.ITIMER_REAL, ACTION_SECONDS)
    try:
        packet, assets = load_packet()
    except Exception as exc:
        signal.setitimer(signal.ITIMER_REAL, 0)
        error = f"{type(exc).__name__}: {exc}"
        try:
            asyncio.run(wire.write(dict(status="terminal", reason="packet_error", error=error[:4096], error_truncated=len(error) > 4096)))
        except Exception:
            return 2
        return 1
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
    return asyncio.run(run(wire, packet, assets, Browser(packet), max(0, RUN_SECONDS - (time.monotonic() - started))))


if __name__ == "__main__":
    raise SystemExit(main())
