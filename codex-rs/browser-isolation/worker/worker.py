"""Fixed PF-30 worker. The container has no network; only the host brokers GETs.

No credentials, caller headers, host files, or browser profiles are accepted.
All results remain untrusted. This is acquisition, not content sanitization.
"""
import base64
import contextlib
import importlib.metadata
import json
import os
import signal
import socket
import sys
import tempfile
import time

MAX_LINE = 6 * 1024 * 1024
MAX_CONTENT = 2 * 1024 * 1024
WIRE = sys.stdout


def send(value):
    encoded = json.dumps(value, separators=(",", ":"))
    if len(encoded.encode()) > MAX_LINE:
        raise ValueError("limit")
    WIRE.write(encoded + "\n")
    WIRE.flush()


def receive():
    line = sys.stdin.buffer.readline(MAX_LINE + 1)
    if not line.endswith(b"\n") or len(line) > MAX_LINE:
        raise ValueError("protocol")
    value = json.loads(line)
    if not isinstance(value, dict):
        raise ValueError("protocol")
    return value


def probe():
    # Only run before loading untrusted content. Host inspect separately verifies
    # mounts/namespaces/caps/limits/image; these checks are not the sole boundary.
    if os.getuid() != 65532 or importlib.metadata.version("scrapling") != "0.4.15":
        raise ValueError("identity")
    try:
        with open("/corbanu-write-probe", "x"):
            pass
    except OSError:
        pass
    else:
        raise ValueError("writable root")
    with socket.socket() as sock:
        sock.settimeout(1)
        if sock.connect_ex(("1.1.1.1", 443)) == 0:
            raise ValueError("network")
    # Launch the actual packaged Chromium, not just an interpreter/version probe.
    from playwright.sync_api import sync_playwright
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(headless=True, args=["--no-sandbox"])
        page = browser.new_page()
        page.set_content("<title>corbanu-containment-probe</title>")
        if page.title() != "corbanu-containment-probe":
            raise ValueError("browser")
        browser.close()
    send({"type": "healthy", "version": 1})


def acquire():
    from scrapling.fetchers import DynamicFetcher
    job = receive()
    if set(job) != {"url"} or not isinstance(job["url"], str) or len(job["url"]) > 4096:
        raise ValueError("request")
    sequence = 0
    installed = False
    captured = None

    def route_request(route, request):
        nonlocal sequence
        # Non-GETs, websockets, auth and downloads get no alternate network path.
        if request.method != "GET":
            route.abort("blockedbyclient")
            return
        sequence += 1
        if sequence > 64:
            raise ValueError("limit")
        send({"type": "request", "id": sequence, "url": request.url})
        reply = receive()
        if reply.get("id") != sequence:
            raise ValueError("sequence")
        if reply.get("denied") is True:
            route.abort("blockedbyclient")
            return
        body = base64.b64decode(reply["body"], validate=True)
        if len(body) > MAX_CONTENT:
            raise ValueError("limit")
        route.fulfill(status=reply["status"], headers=reply["headers"], body=body)

    def setup(page):
        nonlocal installed
        page.context.route("**/*", route_request)
        page.context.route_web_socket("**/*", lambda ws: ws.close())
        installed = True

    def capture(page):
        nonlocal captured
        # Scrapling catches callback errors; explicitly require capture below.
        if not installed:
            raise ValueError("routing")
        content = page.content().encode()
        if len(content) > MAX_CONTENT:
            raise ValueError("limit")
        captured = (page.url, base64.b64encode(content).decode("ascii"))

    with tempfile.TemporaryDirectory(prefix="profile-") as profile:
        with contextlib.redirect_stdout(sys.stderr):
            DynamicFetcher.fetch(
                job["url"], headless=True, timeout=20000, google_search=False, retries=1,
                user_data_dir=profile, page_setup=setup, page_action=capture,
                additional_args={"service_workers": "block", "accept_downloads": False},
                extra_flags=["--no-sandbox", "--disable-background-networking",
                             "--disable-extensions", "--disable-sync"],
            )
    if not installed or captured is None:
        raise ValueError("acquisition")
    send({"type": "result", "url": captured[0], "body": captured[1]})


def main():
    signal.alarm(100)
    mode = sys.argv[1:] or ["idle"]
    if mode == ["idle"]:
        time.sleep(90)  # bounded lifetime even if the host is terminated abruptly
    elif mode == ["probe"]:
        with contextlib.redirect_stdout(sys.stderr):
            probe()
    elif mode == ["acquire"]:
        acquire()
    else:
        raise ValueError("mode")


if __name__ == "__main__":
    try:
        main()
    except Exception:
        # No raw URLs, response bodies, paths or exception text on the wire.
        send({"type": "failed"})
        sys.exit(1)
