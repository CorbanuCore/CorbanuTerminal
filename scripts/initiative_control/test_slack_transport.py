import copy
import functools
import json
import multiprocessing
import os
import signal
import socket
import subprocess
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from unittest.mock import patch
from urllib.parse import parse_qs

from slack_sdk import WebClient
from slack_sdk.errors import SlackApiError
from slack_sdk.socket_mode import SocketModeClient
from slack_sdk.socket_mode.request import SocketModeRequest
from slack_sdk.socket_mode.builtin.connection import Connection

import decisions as d
import decision_alerts as a
import decision_replies as r
import slack_transport as s
import decision_manager as m
from test_decisions import NOW, revision
from test_decision_alerts import SlackFixture, PIN, OWNER, REMOTE


def payload(event_id="Ev001", **changes):
    event = dict(type="message", channel=PIN["channel"], user=PIN["human"], thread_ts="100.000001",
                 ts="101.000001", event_ts="101.000001", text="Five testers")
    event.update(changes)
    return dict(type="event_callback", team_id=PIN["team"], api_app_id=PIN["app"], event_id=event_id, event=event)


def ui_evidence():
    return dict(binding=PIN, observed_at=NOW, receipt="owner-supported-ui-fixture", checks=s.UI_CHECKS)


def sdk_socket(transport, acknowledge):
    client = SocketModeClient(app_token="fixture-app", web_client=WebClient(token="fixture-bot", retry_handlers=[]),
                              logger=s.QUIET, auto_reconnect_enabled=False, concurrency=1)
    client.send_socket_mode_response = acknowledge
    transport.socket, transport.active = client, True
    return client


def crash_post(root, key, endpoint):
    store = a.Store(root)
    transport = s.Transport(store, PIN, lambda: ("fixture-bot", "fixture-app"), live=True, now=lambda: NOW,
                            web_factory=lambda **kw: WebClient(base_url=endpoint, **kw))
    write = store.write
    def checkpoint(name, value):
        if name == "transport" and any(p["receipt"] for p in value["posts"].values()):
            os._exit(31)
        write(name, value)
    store.write = checkpoint
    a.send(store, key, PIN, transport.exchange)


def crash_ack(root):
    transport = s.Transport(a.Store(root), PIN, lambda: None, live=True, now=lambda: NOW)
    client = sdk_socket(transport, lambda _: os._exit(32))
    transport.callback(client, SocketModeRequest(type="events_api", envelope_id="envelope-1", payload=payload()))
    client.close()


def hold_offline(root, feed, ready, release):
    with a.Store(root).lock(), r.feed_lock(feed):
        ready.set()
        release.wait(10)


def hold_transport(root, ready, release):
    with s.locked(a.Store(root)):
        ready.set()
        release.wait(10)


def crash_cancelled_drain(root, after):
    store = a.Store(root)
    write = store.write
    def checkpoint(name, value):
        if name == "transport" and any(e.get("disposition") for e in value["events"].values()):
            if after:
                write(name, value)
            os._exit(36 if after else 37)
        write(name, value)
    store.write = checkpoint
    s.drain(store)


def cancel_under_lock(root, key, ready, release):
    store = a.Store(root)
    with store.lock():
        rows = store.read("alerts")
        ready.set()
        if not release.wait(10):
            raise TimeoutError("fixture release missing")
        rows[key].update(cancelled=True, reason="cancelled")
        store.write("alerts", rows)


def listener_process(root, ready, stop, mode="connected"):
    store = a.Store(root)
    fd = s.runtime_file(store, PIN)
    runtime = s._ChildRuntime(fd, store, PIN)  # Actual spawned child; fd remains owned until process death.
    transport = s.Transport(store, PIN, lambda: ("fixture-bot", "fixture-app"), live=True, now=lambda: NOW)
    updates, update = [0], s.Session.update
    def observed(owner, phase):
        update(owner, phase)
        if phase == "connected":
            updates[0] += 1
            if updates[0] >= 2:
                ready.set()  # Actual durable renewal, not merely connect() entry.
    def connect(client):
        if mode == "starting":
            ready.set()
            stop.wait(10)
        session = Connection("wss://fixture.invalid", s.QUIET)
        session.sock, session.fixture_peer = socket.socketpair()
        client.current_session = session
    with patch.object(SocketModeClient, "connect", autospec=True, side_effect=connect), \
            patch.object(s.Session, "update", observed):
        try:
            transport.listen(ongoing=True, stop=stop, runtime=runtime)
        finally:
            if transport.socket.current_session:
                transport.socket.current_session.fixture_peer.close()


def failed_lifecycle_process(root, phase, ready, release):
    store = a.Store(root)
    owner = None if phase == "starting" else s.Session(store)
    if owner:
        owner.update("connected")
    try:
        with patch.object(store, "write", side_effect=OSError("fixture lifecycle disk failure")):
            if phase == "starting":
                s.Session(store)
            elif phase == "renewal":
                owner.update("connected")
            else:
                owner.close()
    except OSError:
        ready.set()
        release.wait(10)  # Stay alive: denial must use released ownership, not process exit.
    finally:
        if owner:
            owner.release()


def joined_callbacks():
    stopped = threading.Event()
    callback = threading.Thread(target=stopped.wait)
    callback.start()
    stopped.set()
    callback.join(2)
    assert not callback.is_alive()  # Owned runtime fixture, not a production supervisor qualification.


def crash_fence_repair(root, case, stage):
    manager = m.ManagedListener(a.Store(root), PIN)
    with manager.quiesced() as witness:
        s.recover_missing_fence(manager.store, PIN, expected_digest=case, evidence="fixture-loss-reviewed", now=NOW,
            quiesce_listener=witness, checkpoint=lambda where: os._exit(61) if where == stage else None)


def old_inode_callback(root, ready, stop):
    runtime = s._ChildRuntime(s.runtime_file(a.Store(root), PIN), a.Store(root), PIN)
    owner = s.Session(a.Store(root))
    owner.update("connected")
    fd = os.open(os.path.join(root, ".ingress.fence"), os.O_WRONLY | os.O_APPEND)
    ready.set()
    assert stop.wait(10)
    os.write(fd, b"!")  # Complete an already-open callback against the lost inode before join returns.
    os.fsync(fd)
    os.close(fd)
    owner.close()


def runtime_case(function):
    """Preserve literal assertions in a fresh exec, with a real lifetime guard, never a patched gate."""
    @functools.wraps(function)
    def wrapped(self):
        if hasattr(self, "runtime"):
            return function(self)
        reader, writer = os.pipe()
        try:
            command = [sys.executable, "-B", "-c", "from test_slack_transport import runtime_case_child; import sys; runtime_case_child(sys.argv[1], int(sys.argv[2]))",
                       function.__name__, str(reader)]
            env = dict(os.environ, PYTHONPATH=os.path.dirname(__file__) + os.pathsep + os.environ.get("PYTHONPATH", ""))
            with subprocess.Popen(command, pass_fds=(reader,), stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                  stderr=subprocess.PIPE, env=env) as process:
                output, errors = process.communicate(d.canonical(dict(binding=PIN, seconds=60, ongoing=True)) + b"\n", timeout=30)
                self.assertEqual(process.returncode, 0, errors.decode())
                self.assertEqual(json.loads(output), dict(type="runtime-owned"))
        finally:
            os.close(reader)
            os.close(writer)
    return wrapped


def runtime_case_child(name, control):
    import traceback
    case = TransportTests(name)
    case.setUp()
    guard = s.runtime_file(case.store, PIN)
    def run(runtime, stop, data):
        case.runtime = runtime
        try:
            getattr(case, name)()
        except BaseException:
            traceback.print_exc()
            raise
        finally:
            case.tearDown()
            case.doCleanups()
    m.listener_child(str(case.root), guard, control, run=run)


def legacy_sdk_before_session(ready, stop):
    client = SocketModeClient(app_token="fixture-app", web_client=WebClient(token="fixture-bot"), logger=s.QUIET)
    ready.set()  # Real SDK threads exist; no Session or runtime pin was ever claimed.
    stop.wait(10)
    client.close()


class LiveFixture(SlackFixture):
    def setUp(self):
        super().setUp()
        s.initialize(self.store)
        self.calls, self.messages, self.options, self.acks = [], [], [], []
        self.reply = None
        self.scopes = s.SCOPES.copy()
        fixture = self
        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_):
                pass
            def do_POST(self):
                raw = self.rfile.read(int(self.headers.get("Content-Length", 0)))
                data = json.loads(raw or b"{}") if self.headers.get("Content-Type", "").startswith("application/json") else {
                    k: v[0] for k, v in parse_qs(raw.decode()).items()}
                method = self.path.rsplit("/", 1)[-1]
                fixture.calls.append((method, data))
                result, status, headers = {"ok": True}, 200, {}
                if method == "auth.test":
                    result.update(team_id=PIN["team"], bot_id=PIN["bot"], user_id="UBOTTEST")
                    headers["x-oauth-scopes"] = ",".join(fixture.scopes)
                elif method == "apps.connections.open":
                    result["url"] = "wss://fixture.invalid"
                elif method in ("conversations.history", "conversations.replies"):
                    result.update(messages=fixture.messages, response_metadata={"next_cursor": ""})
                elif method == "chat.postMessage":
                    message = dict(data, ts=f"100.{len(fixture.messages) + 1:06d}", bot_id=PIN["bot"], app_id=PIN["app"])
                    fixture.messages.append(message)
                    result.update(channel=PIN["channel"], ts=message["ts"])
                if fixture.reply is not None:
                    result, status, headers = fixture.reply(method, result)
                if result is None:
                    self.connection.shutdown(socket.SHUT_RDWR)
                    self.connection.close()
                    return
                encoded = result if isinstance(result, bytes) else json.dumps(result).encode()
                self.send_response(status)
                for key, value in headers.items():
                    self.send_header(key, value)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(encoded)))
                self.end_headers()
                try:
                    self.wfile.write(encoded)
                except BrokenPipeError:
                    pass
        self.server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self.server.daemon_threads = True
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        def close_server():
            self.server.shutdown()
            self.server.server_close()
            self.thread.join(5)
        self.addCleanup(close_server)
        self.endpoint = f"http://127.0.0.1:{self.server.server_port}/api/"
        def factory(**options):
            self.options.append({k: v for k, v in options.items() if k != "token"})
            return WebClient(base_url=self.endpoint, **options)
        self.transport = s.Transport(self.store, PIN, lambda: ("fixture-bot", "fixture-app"), live=True,
                                     now=lambda: NOW, web_factory=factory)
        self.transport.qualify(ui_evidence())
        self.owner = s.Session(self.store)
        self.owner.update("connected")  # Genuine lifetime owner, never a patched gate or fabricated lease.
        self.addCleanup(self.owner.release)

    def orphan(self, attempt="pf83-vm-key-20260915T103614Z"):
        request = dict(a.request_for(self.key, a.inspect(self.store, self.key), "parent"), attempt=attempt)
        with s.locked(self.store) as value:
            value["posts"][attempt] = dict(request=copy.deepcopy(request), receipt=None, retry_after=None)
            self.store.write("transport", value)
        return request

    def sending(self):
        row = a.send(self.store, self.key, PIN, self.transport.exchange)
        self.transport.bind_alert(self.key, row)
        self.client = sdk_socket(self.transport, lambda response: self.acks.append(response.envelope_id))
        self.addCleanup(self.client.close)
        return row

    def session_review(self):
        with s.locked(self.store) as journal:
            session, epoch = s.observe_session_locked(self.store, journal)
            return dict(session=session, epoch=epoch)

    def start_listener(self, mode="connected"):
        self.owner.close()
        ctx = multiprocessing.get_context("spawn")
        ready, stop = ctx.Event(), ctx.Event()
        process = ctx.Process(target=listener_process, args=(str(self.root), ready, stop, mode))
        process.start()
        def cleanup():
            if process.is_alive():
                os.kill(process.pid, signal.SIGCONT)
                stop.set()
                process.join(3)
            if process.is_alive():
                process.kill()
            process.join(3)
        self.addCleanup(cleanup)
        self.assertTrue(ready.wait(5))
        process.fixture_ready = ready
        return process, stop

    def review_gap(self):
        with s.locked(self.store) as journal:
            review = dict(watermark=journal["watermark"], ingress=s.ingress_count(self.store),
                          binding=d.digest(PIN), evidence="fixture-reviewed-gap")
        self.transport.qualify(ui_evidence(), dict(review, **self.session_review()))

    def quiesce(self):
        self.transport.active = False
        if self.transport.socket:
            self.transport.socket.close()
        self.owner.close()

    def lose_fence(self, store=None):
        store = store or self.store
        covered = s.ingress_count(store, mark=True)
        with s.locked(store) as journal:
            journal["ingress"] = covered
            store.write("transport", journal)
        s.ingress_count(store, mark=True)  # An uncovered arrival existed before the destructive failure.
        with patch.object(s.os, "write", side_effect=OSError("fixture append-before-bytes")), self.assertRaises(OSError):
            s.ingress_count(store, mark=True)
        self.assertFalse((store.root / ".ingress.fence").exists())
        return s.inspect_fence_loss(store, PIN)["case_digest"]

    def callback(self, value=None, envelope_id="envelope-1"):
        self.transport.callback(self.client, SocketModeRequest(type="events_api", envelope_id=envelope_id, payload=value or payload()))


class TransportTests(LiveFixture):
    def test_runtime_guard_unknown_legacy_binding_inode_and_direct_entry_refuse(self):
        self.quiesce()
        before = self.store.read("transport")
        with patch.object(s.Transport, "web", side_effect=AssertionError("unguarded SDK")):
            for runtime in (None, True, object()):
                with self.assertRaises(d.Invalid):
                    self.transport.listen(runtime=runtime)
            with self.assertRaises(d.Invalid):
                with m.ManagedListener(self.store, dict(PIN, human="wrong-human")).quiesced():
                    self.fail("wrong identity admitted")
        path = self.root / ".listener.runtime.lock"
        path.rename(self.root / "retained-runtime")
        for replacement in ("missing", "symlink", "new-inode"):
            if replacement == "symlink":
                path.symlink_to(self.root / "retained-runtime")
            elif replacement == "new-inode":
                path.unlink()
                path.touch(mode=0o600)
            with self.assertRaises((d.Invalid, OSError)), m.ManagedListener(self.store, PIN).quiesced():
                self.fail("unknown runtime admitted")
            self.assertEqual(self.store.read("transport"), before)
        legacy = copy.deepcopy(before)
        del legacy["runtime_guard"]
        legacy["lifecycle"].update(session=None, epoch=0, history=[])
        self.store.write("transport", legacy)
        ctx = multiprocessing.get_context("spawn")
        ready, stop = ctx.Event(), ctx.Event()
        child = ctx.Process(target=legacy_sdk_before_session, args=(ready, stop))
        child.start()
        try:
            self.assertTrue(ready.wait(5))
            with self.assertRaises(d.Invalid), m.ManagedListener(self.store, PIN).quiesced():
                self.fail("epoch zero is not a quiescence proof")
            self.assertEqual(self.store.read("transport"), legacy)
        finally:
            stop.set()
            child.join(5)
            if child.is_alive():
                child.kill()
                child.join(3)

    def test_runtime_birth_partial_fsync_failure_never_reinitializes(self):
        root = self.root / "partial-birth"
        root.mkdir(mode=0o700)
        store = a.Store(root)
        store.initialize()
        with patch.object(s.os, "fsync", side_effect=OSError("fixture birth sync")), self.assertRaises(d.Invalid):
            s.initialize(store)
        self.assertTrue((root / ".listener.runtime.lock").exists())
        self.assertFalse((root / "transport.json").exists())
        with self.assertRaises(d.Invalid):
            s.initialize(store)
        with self.assertRaises((OSError, d.Invalid)), m.ManagedListener(store, PIN).quiesced():
            self.fail("partial birth became ready")

    def test_fence_repair_real_crash_boundaries_preserve_loss_and_all_old_data(self):
        for stage in ("prepared", "cancelled", "partial", "staged", "linked", "synced", "restored"):
            with self.subTest(stage=stage):
                transport = self.initial_transport(stage)
                transport.qualify(ui_evidence())
                store = transport.store
                key = a.enqueue(store, self.feed, "choice-1", REMOTE, PIN, OWNER, NOW)
                with store.lock():
                    rows = store.read("alerts")
                    rows[key]["parent"].update(state="sending", request=a.request_for(key, rows[key], "parent"))
                    store.write("alerts", rows)
                case = self.lose_fence(store)
                before, replies = store.read("transport"), (store.root / "replies.json").read_bytes()
                worker = multiprocessing.get_context("spawn").Process(target=crash_fence_repair, args=(str(store.root), case, stage))
                worker.start()
                worker.join(5)
                self.assertEqual(worker.exitcode, 61)
                after = store.read("transport")
                if stage != "restored":
                    with self.assertRaises(d.Invalid):
                        s.Session(store)
                    missing = not os.path.lexists(store.root / ".ingress.fence")
                    with self.assertRaises(FileNotFoundError if missing else d.Invalid):
                        transport.qualify(ui_evidence())
                    with self.assertRaises(d.Invalid):
                        s.initialize(store)
                    with s.locked(store) as journal, self.assertRaises(d.Invalid):
                        s.observe_session_locked(store, journal)
                with m.ManagedListener(a.Store(store.root), PIN).quiesced() as witness:
                    result = s.recover_missing_fence(a.Store(store.root), PIN, expected_digest=case,
                        evidence="fixture-loss-reviewed", now=NOW, quiesce_listener=witness)
                self.assertEqual(result, dict(loss_id=case, state="held", retired_count=1))
                after = store.read("transport")
                self.assertEqual(after["ingress"], before["ingress"])
                self.assertGreater(s.ingress_count(store), before["ingress"])
                self.assertEqual(len(after["fence_losses"]), 1)
                prior = after["fence_losses"][0]["intent"]["retired"][key]
                self.assertEqual(prior["digest"], d.digest(rows[key]))
                self.assertEqual(prior["cancelled"], rows[key]["cancelled"])
                for field in before:
                    if field != "hold":
                        self.assertEqual(after[field], before[field])
                self.assertEqual(store.read("alerts")[key], dict(rows[key], cancelled=True, reason="cancelled"))
                self.assertEqual((store.root / "replies.json").read_bytes(), replies)
                self.assertEqual(d.load_fixture(self.feed_root, NOW), self.feed)
                with self.assertRaises(d.Invalid):
                    transport.gate()
                with m.ManagedListener(store, PIN).quiesced() as witness:
                    s.recover_missing_fence(store, PIN, expected_digest=case, evidence="fixture-loss-reviewed", now=NOW,
                                           quiesce_listener=witness)
                self.assertEqual(store.read("transport"), after)

    def test_repair_waits_for_real_old_inode_callback_and_rejects_stale_preview(self):
        self.quiesce()
        ctx = multiprocessing.get_context("spawn")
        ready, stop = ctx.Event(), ctx.Event()
        process = ctx.Process(target=old_inode_callback, args=(str(self.root), ready, stop))
        process.start()
        self.assertTrue(ready.wait(5))
        case = self.lose_fence()
        def join_runtime():
            stop.set()
            process.join(5)
            self.assertEqual(process.exitcode, 0)
        try:
            before = self.store.read("transport")
            with self.assertRaises(d.Invalid):
                s.recover_missing_fence(self.store, PIN, expected_digest=case, evidence="fixture-loss-reviewed", now=NOW,
                                       quiesce_listener=joined_callbacks)
            self.assertEqual(self.store.read("transport"), before)  # Another real process still owns the runtime.
            with self.assertRaises(d.Invalid):
                s.recover_missing_fence(self.store, PIN, expected_digest=case, evidence="fixture-loss-reviewed", now=NOW,
                                       quiesce_listener=join_runtime)
            self.assertFalse((self.root / ".ingress.fence").exists())
            case = s.inspect_fence_loss(self.store, PIN)["case_digest"]
            s.recover_missing_fence(self.store, PIN, expected_digest=case, evidence="fixture-loss-reviewed", now=NOW,
                                   quiesce_listener=join_runtime)
            self.assertEqual(self.store.read("transport")["fence_losses"][0]["phase"], "restored")
        finally:
            if process.is_alive():
                process.kill()
                process.join(3)

    def test_repair_refusals_and_failed_publication_keep_incident_held(self):
        self.quiesce()
        case = self.lose_fence()
        before = self.store.read("transport")
        def repair(**kwargs):
            return s.recover_missing_fence(self.store, PIN, expected_digest=case, evidence="fixture-loss-reviewed", now=NOW, **kwargs)
        for hook in (None, True, lambda: True):
            with self.assertRaises(d.Invalid):
                repair(quiesce_listener=hook)
            self.assertEqual(self.store.read("transport"), before)
        with patch.object(self.store, "write", side_effect=OSError("fixture disk full")), self.assertRaises(d.Invalid):
            repair(quiesce_listener=self.quiesce)
        self.assertEqual(self.store.read("transport"), before)
        with patch.object(s.os, "link", side_effect=OSError("fixture link failure")), self.assertRaises(d.Invalid):
            repair(quiesce_listener=self.quiesce)
        pending = self.store.read("transport")
        self.assertEqual(pending["fence_losses"][0]["phase"], "prepared")
        path = self.root / ".ingress.fence"
        path.symlink_to(self.root / "alerts.json")
        with self.assertRaises(d.Invalid):
            repair(quiesce_listener=self.quiesce)
        path.unlink()
        path.write_bytes(b"unrelated fixture")
        path.chmod(0o600)
        with self.assertRaises(d.Invalid):
            repair(quiesce_listener=self.quiesce)
        self.assertEqual(path.read_bytes(), b"unrelated fixture")
        self.assertEqual(self.store.read("transport"), pending)

    def test_repair_snapshot_binding_inode_capacity_and_sync_failure(self):
        self.quiesce()
        case = self.lose_fence()
        before = self.store.read("transport")
        args = dict(expected_digest=case, evidence="fixture-loss-reviewed", now=NOW, quiesce_listener=self.quiesce)
        for binding, extra in ((dict(PIN, human="another-human"), {}), (PIN, {"expected_digest": "0" * 64})):
            with self.assertRaises(d.Invalid):
                s.recover_missing_fence(self.store, binding, **dict(args, **extra))
            self.assertEqual(self.store.read("transport"), before)
        with s.locked(self.store) as journal:
            journal["ingress"] = d.MAX_BYTES - 65
            self.store.write("transport", journal)
        args["expected_digest"] = s.inspect_fence_loss(self.store, PIN)["case_digest"]
        with self.assertRaises(d.Invalid):
            s.recover_missing_fence(self.store, PIN, **args)
        self.assertNotIn("fence_losses", self.store.read("transport"))
        self.store.write("transport", before)  # Fixture restoration only; no production reset API.
        args["expected_digest"] = case
        def fail_after_link(stage):
            if stage == "linked":
                raise OSError("fixture directory sync failure")
        with self.assertRaises(d.Invalid):
            s.recover_missing_fence(self.store, PIN, checkpoint=fail_after_link, **args)
        with patch.object(s.os, "fsync", side_effect=OSError("fixture fsync")), self.assertRaises(d.Invalid):
            s.recover_missing_fence(self.store, PIN, **args)
        self.assertEqual(self.store.read("transport")["fence_losses"][0]["phase"], "prepared")
        owner = self.root / ".listener.owner.lock"
        owner.rename(self.root / "retained-owner")
        owner.touch(mode=0o600)
        with self.assertRaises(d.Invalid):
            s.recover_missing_fence(self.store, PIN, **args)
        self.assertEqual(self.store.read("transport")["fence_losses"][0]["phase"], "prepared")

    def initial_transport(self, name):
        root = self.root / name
        root.mkdir(mode=0o700)
        store = a.Store(root)
        store.initialize()
        s.initialize(store)
        return s.Transport(store, PIN, lambda: ("fixture-bot", "fixture-app"), live=True, now=lambda: NOW,
                           web_factory=lambda **kw: WebClient(base_url=self.endpoint, **kw))

    def test_initial_qualification_failures_retry_after_real_reload_without_reset(self):
        for failure in ("timeout", "credentials", "scope"):
            with self.subTest(failure=failure):
                transport = self.initial_transport(failure)
                store = transport.store
                before = store.read("transport")
                if failure == "credentials":
                    transport.credentials = lambda: (_ for _ in ()).throw(KeyError("fixture missing credential"))
                elif failure == "scope":
                    self.scopes = set()
                else:
                    self.reply = lambda *_: (time.sleep(0.1) or {"ok": True}, 200, {})
                    transport.web().timeout = 0.02
                with self.assertRaises((TimeoutError, KeyError, d.Invalid)):
                    transport.qualify(ui_evidence())
                self.assertEqual(a.Store(store.root).read("transport"), dict(before, hold="qualifying"))
                self.reply, self.scopes = None, s.SCOPES.copy()
                reopened = s.Transport(a.Store(store.root), PIN, lambda: ("fixture-bot", "fixture-app"),
                    live=True, now=lambda: NOW, web_factory=lambda **kw: WebClient(base_url=self.endpoint, **kw))
                self.assertEqual(reopened.qualify(ui_evidence()), PIN)
                saved = a.Store(store.root).read("transport")
                self.assertEqual(saved, dict(before, binding=PIN, last_verified=NOW, hold=None, ui_evidence=ui_evidence()))
                self.assertEqual(s.ingress_count(store), 0)
                with self.assertRaises(d.Invalid):
                    reopened.gate()  # Successful qualification alone is not listener/work authority.
        self.assertFalse(any(method == "chat.postMessage" for method, _ in self.calls))

    def test_initial_retry_exception_denies_prior_evidence_and_missing_fence(self):
        changes = [dict(binding=PIN), dict(last_verified=NOW), dict(ui_evidence=ui_evidence()), dict(hold="outage-gap"),
                   dict(watermark=1), dict(ingress=1)]
        changes += [{key: ["retained"] if key == "gap_reviews" else {"retained": {}}}
                    for key in ("events", "posts", "routes", "bridges", "gap_reviews")]
        changes += [{"lifecycle": dict(session=None, epoch=1, history=[])},
                    {"lifecycle": dict(session=None, epoch=0, history=[{"id": "retained"}])},
                    {"lifecycle": dict(session={"id": "retained"}, epoch=0, history=[])}]
        for index, change in enumerate(changes + [dict(fence="missing"), dict(fence="uncovered")]):
            with self.subTest(change=change):
                transport = self.initial_transport("adverse" + str(index))
                with s.locked(transport.store) as journal:
                    journal["hold"] = "qualifying"
                    if "lifecycle" in change:
                        journal["lifecycle"].update(change["lifecycle"])
                    else:
                        journal.update({k: v for k, v in change.items() if k != "fence"})
                    transport.store.write("transport", journal)
                if change.get("fence") == "missing":
                    (transport.store.root / ".ingress.fence").unlink()
                elif change.get("fence") == "uncovered":
                    s.ingress_count(transport.store, mark=True)
                before = transport.store.read("transport")
                for _ in range(2):
                    with self.assertRaises((d.Invalid, OSError)):
                        transport.qualify(ui_evidence())
                    self.assertEqual(transport.store.read("transport"), before)

    def test_initial_retry_rechecks_evidence_after_actual_auth_exchange(self):
        transport = self.initial_transport("auth-race")
        def response(method, result):
            self.assertEqual(method, "auth.test")
            with s.locked(transport.store) as journal:
                journal["posts"]["retained-uncertain-attempt"] = dict(receipt=None)
                transport.store.write("transport", journal)
            return result, 200, {"x-oauth-scopes": ",".join(s.SCOPES)}
        self.reply = response
        with self.assertRaises(d.Invalid):
            transport.qualify(ui_evidence())
        saved = transport.store.read("transport")
        self.assertIsNone(saved["binding"])
        self.assertIn("retained-uncertain-attempt", saved["posts"])
        self.reply = None
        with self.assertRaises(d.Invalid):
            transport.qualify(ui_evidence())
        self.assertEqual(a.Store(transport.store.root).read("transport"), saved)

    def test_owner_loss_during_durable_post_request_does_not_send_or_replay(self):
        request = a.request_for(self.key, a.inspect(self.store, self.key), "parent")
        write = self.store.write
        def boundary(name, value):
            write(name, value)
            if name == "transport" and value["posts"]:
                self.owner.release()
        with patch.object(self.store, "write", side_effect=boundary), self.assertRaises(d.Invalid):
            self.transport.exchange(request)
        self.assertEqual(len(self.messages), 0)
        self.assertIsNone(a.Store(self.root).read("transport")["posts"][request["attempt"]]["receipt"])
        with self.assertRaises(d.Invalid):
            self.transport.exchange(request)

    def test_stopped_listener_expires_then_resumes_held_until_exact_gap_review(self):
        process, _ = self.start_listener()
        self.review_gap()
        before = self.session_review()
        os.kill(process.pid, signal.SIGSTOP)
        os.waitpid(process.pid, os.WUNTRACED)
        time.sleep(5.2)
        with self.assertRaises(d.Invalid):
            self.transport.gate()
        process.fixture_ready.clear()
        os.kill(process.pid, signal.SIGCONT)
        self.assertTrue(process.fixture_ready.wait(3))
        after = self.session_review()
        self.assertEqual(after["session"], before["session"])
        self.assertGreater(after["epoch"], before["epoch"])
        with self.assertRaises(d.Invalid):
            self.transport.gate()
        with self.assertRaises(d.Invalid):
            self.transport.qualify(ui_evidence(), dict(watermark=0, ingress=s.ingress_count(self.store),
                binding=d.digest(PIN), evidence="stale-session-review", **before))
        self.review_gap()
        self.transport.gate()
        self.assertTrue(process.is_alive())

    def test_startup_crash_and_competing_owner_never_establish_authority(self):
        process, _ = self.start_listener("starting")
        original = self.store.read("transport")
        self.assertEqual(original["lifecycle"]["session"]["phase"], "starting")
        with self.assertRaises(BlockingIOError):
            s.Session(a.Store(self.root))
        self.assertEqual(a.Store(self.root).read("transport"), original)
        with self.assertRaises(d.Invalid):
            self.transport.gate()
        process.kill()
        process.join(3)
        self.assertEqual(process.exitcode, -signal.SIGKILL)
        with self.assertRaises(d.Invalid):
            self.transport.gate()
        self.assertEqual(a.Store(self.root).read("transport"), original)

    def test_startup_renewal_and_shutdown_write_failures_release_lifetime_ownership(self):
        self.owner.close()
        ctx = multiprocessing.get_context("spawn")
        for phase in ("starting", "renewal", "shutdown"):
            with self.subTest(phase=phase):
                ready, release = ctx.Event(), ctx.Event()
                process = ctx.Process(target=failed_lifecycle_process, args=(str(self.root), phase, ready, release))
                process.start()
                try:
                    self.assertTrue(ready.wait(5))
                    self.assertTrue(process.is_alive())
                    with self.assertRaises(d.Invalid):
                        self.transport.gate()
                    with s.locked(self.store) as journal:
                        fd = s.owner_file(self.store, journal)
                        try:
                            s.fcntl.flock(fd, s.fcntl.LOCK_EX | s.fcntl.LOCK_NB)
                        finally:
                            os.close(fd)
                finally:
                    release.set()
                    process.join(3)
                    if process.is_alive():
                        process.kill()
                        process.join(3)
                self.assertEqual(process.exitcode, 0)
        self.assertEqual(len(self.messages), 0)

    def test_missing_replaced_owner_lock_and_legacy_journal_stay_held(self):
        self.sending()
        self.callback()
        original = self.store.read("transport")
        path = self.root / ".listener.owner.lock"
        retained = self.root / "fixture-retained-owner"
        path.rename(retained)
        with self.assertRaises(d.Invalid):
            self.transport.gate()
        fd = os.open(path, os.O_CREAT | os.O_EXCL | os.O_RDWR, 0o600)
        os.close(fd)
        with self.assertRaises(d.Invalid):
            self.transport.gate()
        with self.assertRaises(d.Invalid):
            s.Session(a.Store(self.root))
        self.assertEqual(self.store.read("transport"), original)
        legacy = copy.deepcopy(original)
        del legacy["schema"], legacy["lifecycle"]
        with s.locked(self.store):
            self.store.write("transport", legacy)
        with self.assertRaises(d.Invalid):
            self.transport.gate()
        with self.assertRaises(d.Invalid):
            self.transport.qualify(ui_evidence())
        with self.assertRaises(d.Invalid):
            s.initialize(self.store)
        self.assertEqual(s.drain(a.Store(self.root), PIN), 1)
        self.assertEqual(self.store.read("transport")["posts"], original["posts"])
        self.assertEqual(len(self.messages), 2)

    def transformed_history(self, emoji=False):
        if emoji:
            feed = revision(self.feed)
            feed["decisions"][0]["revisions"][-1]["summary"] += " 🎉"
            self.key = self.enqueue(feed)
        worker = multiprocessing.get_context("spawn").Process(target=crash_post, args=(str(self.root), self.key, self.endpoint))
        worker.start()
        worker.join(10)
        self.assertEqual(worker.exitcode, 31)
        request = a.inspect(a.Store(self.root), self.key)["parent"]["request"]
        original = copy.deepcopy(request)
        text = self.messages[0]["text"]
        if emoji:
            self.assertIn("🎉", text)
            self.messages[0]["text"] = text.replace("🎉", ":tada:")
        else:
            url = next(word for word in text.split() if word.startswith("https://"))
            self.messages[0]["text"] = text.replace(url, "<" + url + ">")
        self.assertNotEqual(self.messages[0]["text"], text)
        evidence = self.transport.reconcile(request)
        self.assertEqual(evidence["payload_digest"], original["payload_digest"])
        a.reconcile(self.store, self.key, "parent", evidence)
        self.assertEqual(a.inspect(a.Store(self.root), self.key)["parent"]["request"], original)
        self.assertEqual(sum(method == "chat.postMessage" for method, _ in self.calls), 1)

    def test_history_url_autolink_recovers_exact_attempt_after_real_crash(self):
        self.transformed_history()

    def test_history_unicode_colon_emoji_recovers_exact_attempt_after_real_crash(self):
        self.transformed_history(emoji=True)

    def test_history_requires_unique_exact_attempt_identity_channel_thread_and_local_request(self):
        request = a.request_for(self.key, a.inspect(self.store, self.key), "parent")
        self.reply = lambda *_: ({"ok": False, "error": "fatal_error"}, 200, {})
        with self.assertRaises(d.Invalid):
            self.transport.exchange(request)
        self.reply = None
        original = copy.deepcopy(self.messages[0])
        for change in (dict(client_msg_id=None), dict(client_msg_id=original["client_msg_id"] + "x"),
                       dict(app_id="AOTHER"), dict(bot_id="BOTHER"), dict(channel="COTHER"), dict(thread_ts="999.000001")):
            with self.subTest(change=change), self.assertRaises(d.Invalid):
                self.messages[:] = [dict(original, **change)]
                self.transport.reconcile(request)
        for messages in ([], [original, copy.deepcopy(original)], [original, dict(original, ts="999.000002")]):
            with self.subTest(messages=len(messages)), self.assertRaises(d.Invalid):
                self.messages[:] = messages
                self.transport.reconcile(request)
        pages = iter([dict(ok=True, messages=[original], response_metadata=dict(next_cursor="second")),
                      dict(ok=True, messages=[original], response_metadata=dict(next_cursor=""))])
        self.reply = lambda *_: (next(pages), 200, {})
        with self.assertRaises(d.Invalid):
            self.transport.reconcile(request)
        self.reply = None
        altered = copy.deepcopy(request)
        altered["payload"]["text"] += " altered locally"
        altered["payload_digest"] = d.digest(altered["payload"])
        with self.assertRaises(d.Invalid):
            self.transport.reconcile(altered)
        self.messages[:] = [dict(original, text="Different remote rendering is not an integrity receipt")]
        evidence = self.transport.reconcile(request)
        self.assertEqual(evidence["payload_digest"], request["payload_digest"])
        self.assertEqual(sum(method == "chat.postMessage" for method, _ in self.calls), 1)
        self.assertTrue(all(data["channel"] == PIN["channel"] and int(data["limit"]) == 15
                            for method, data in self.calls if method == "conversations.history"))

    def cancelled_disposition_crash(self, after):
        self.sending()
        self.callback()
        cancelled = a.send(self.store, self.key, PIN, lambda _: self.fail("cancel sent"), cancelled=True)
        event = self.store.read("transport")["events"]["Ev001"]["envelope"]
        worker = multiprocessing.get_context("spawn").Process(target=crash_cancelled_drain, args=(str(self.root), after))
        worker.start()
        worker.join(10)
        self.assertEqual(worker.exitcode, 36 if after else 37)
        with patch.object(r, "intake", side_effect=AssertionError("cancelled intake")):
            self.assertEqual(s.drain(a.Store(self.root)), 0 if after else 1)
            self.assertEqual(s.drain(a.Store(self.root)), 0)
        saved = a.Store(self.root).read("transport")["events"]["Ev001"]
        self.assertTrue(saved["drained"])
        self.assertEqual(saved["envelope"], event)
        self.assertEqual(saved["disposition"], dict(kind="cancelled", alert_digest=d.digest(cancelled), event_digest=d.digest(event)))
        self.assertEqual(self.store.read("replies"), dict(events={}, aliases={}, intents={}))
        self.assertTrue(a.inspect(self.store, self.key)["cancelled"])

    def test_cancelled_disposition_crash_before_write_recovers_without_intake(self):
        self.cancelled_disposition_crash(False)

    def test_cancelled_disposition_crash_after_write_is_idempotent_on_restart(self):
        self.cancelled_disposition_crash(True)

    def test_cancel_process_wins_store_lock_before_drain_without_offline_intake(self):
        self.sending()
        self.callback()
        ctx = multiprocessing.get_context("spawn")
        ready, release = ctx.Event(), ctx.Event()
        worker = ctx.Process(target=cancel_under_lock, args=(str(self.root), self.key, ready, release))
        worker.start()
        attempted, results, errors = threading.Event(), [], []
        original_lock = a.Store.lock
        def observed_lock(store, *args, **kwargs):
            attempted.set()
            return original_lock(store, *args, **kwargs)
        def drain():
            try:
                results.append(s.drain(a.Store(self.root)))
            except Exception as error:
                errors.append(error)
        try:
            self.assertTrue(ready.wait(5))
            with patch.object(a.Store, "lock", observed_lock), patch.object(r, "intake", side_effect=AssertionError("cancelled intake")):
                contender = threading.Thread(target=drain)
                contender.start()
                self.assertTrue(attempted.wait(5))
                release.set()
                contender.join(5)
                self.assertFalse(contender.is_alive())
            self.assertEqual(errors, [])
            self.assertEqual(results, [1])
        finally:
            release.set()
            worker.join(10)
        self.assertEqual(worker.exitcode, 0)
        self.assertTrue(a.Store(self.root).read("transport")["events"]["Ev001"]["drained"])
        self.assertEqual(self.store.read("replies")["events"], {})

    def test_unproven_cancellation_or_alert_binding_does_not_discard_ingress(self):
        self.sending()
        self.callback()
        original = self.store.read("alerts")
        for mutate in (lambda row: row.update(cancelled="true"),
                       lambda row: row.update(cancelled=True, reason=None),
                       lambda row: row.update(intent_digest="unverified")):
            damaged = copy.deepcopy(original)
            mutate(damaged[self.key])
            with self.store.lock():
                self.store.write("alerts", damaged)
            with patch.object(r, "intake", side_effect=AssertionError("unverified intake")), self.assertRaises(d.Invalid):
                s.drain(a.Store(self.root))
            event = self.store.read("transport")["events"]["Ev001"]
            self.assertFalse(event["drained"])
            self.assertNotIn("disposition", event)
        with self.store.lock():
            self.store.write("alerts", original)
        self.assertEqual(s.drain(a.Store(self.root)), 1)

    def test_cancelled_pending_events_are_audited_without_intake_and_valid_sibling_drains(self):
        self.sending()
        feed = copy.deepcopy(self.feed)
        feed["decisions"].append(dict(copy.deepcopy(feed["decisions"][0]), id="choice-2"))
        other = a.enqueue(self.store, feed, "choice-2", "https://dashboard.example.test", PIN, OWNER, NOW)
        row = a.send(self.store, other, PIN, self.transport.exchange)
        self.transport.bind_alert(other, row)
        self.callback()  # Retained before cancellation.
        cancelled = a.send(self.store, self.key, PIN, lambda _: self.fail("cancel sent"), cancelled=True)
        self.callback(payload("Ev002", ts="102.000001", event_ts="102.000001"))  # After cancellation.
        self.callback(payload("Ev003", thread_ts=row["parent"]["receipt"]["ts"], ts="103.000001", event_ts="103.000001"))
        original = self.store.read("transport")["events"]
        intake = r.intake
        def checked_intake(store, alert, *args):
            self.assertEqual(alert, other)
            return intake(store, alert, *args)
        with patch.object(r, "intake", side_effect=checked_intake) as calls:
            self.assertEqual(s.drain(a.Store(self.root), PIN), 3)
            self.assertEqual(calls.call_count, 1)
        saved = a.Store(self.root).read("transport")
        for event in ("Ev001", "Ev002"):
            self.assertEqual(saved["events"][event]["envelope"], original[event]["envelope"])
            self.assertEqual(saved["events"][event]["disposition"], dict(kind="cancelled", alert_digest=d.digest(cancelled),
                             event_digest=d.digest(original[event]["envelope"])))
        self.assertEqual(set(self.store.read("replies")["events"]), {"Ev003"})
        self.assertEqual(s.drain(a.Store(self.root)), 0)
        self.assertTrue(a.inspect(self.store, self.key)["cancelled"])
        self.assertEqual(d.load_fixture(self.feed_root, NOW), self.feed)
        s.hold(self.store, "outage-gap")
        self.transport.qualify(ui_evidence(), dict(watermark=saved["watermark"], binding=d.digest(PIN),
            evidence="fixture-cancel-gap-reviewed", ingress=s.ingress_count(self.store), **self.session_review()))
        self.transport.gate()

    @runtime_case
    def test_sdk_silent_health_disconnect_is_polled_fenced_and_reconnected(self):
        transport, sessions, callbacks, pauses = self.transport, [], [], []
        test = self
        def connect(client):
            client.issue_new_wss_url()  # Actual pinned SDK HTTP against the controlled local endpoint.
            session = Connection("wss://fixture.invalid", s.QUIET, on_close_listener=callbacks.append,
                                 on_error_listener=callbacks.append)
            session.sock, peer = socket.socketpair()
            self.addCleanup(peer.close)
            self.addCleanup(session.close)
            client.current_session = session
            sessions.append(session)
            if len(sessions) == 2:
                with s.locked(self.store) as journal:
                    self.assertEqual(journal["hold"], "outage-gap")
                    self.assertGreater(s.ingress_count(self.store), journal["ingress"])
        class Control:
            stopped = False
            def is_set(self):
                return self.stopped
            def wait(self, seconds):
                pauses.append(seconds)
                if seconds == 0.1 and len(pauses) == 1:
                    sessions[0].last_ping_pong_time = 0
                    sessions[0].check_state()  # Real SDK stale-pong path sends a ping and silently closes.
                    test.assertFalse(transport.socket.is_connected())
                    test.assertTrue(transport.active)
                    test.assertEqual(callbacks, [])
                elif seconds == 0.1:
                    self.stopped = True
                return self.stopped
        self.owner.close()
        with patch.object(SocketModeClient, "connect", autospec=True, side_effect=connect):
            transport.listen(ongoing=True, stop=Control(), runtime=self.runtime)
        self.assertEqual(len(sessions), 2)
        self.assertIn(1, pauses)
        self.assertFalse(transport.active)
        self.assertEqual(len(self.messages), 0)
        with self.assertRaises(d.Invalid):
            transport.gate()

    def test_failed_append_before_bytes_missing_fence_has_no_reset_route(self):
        self.sending()
        with patch.object(s.os, "write", side_effect=OSError("fixture disk full before append")):
            self.callback()
        self.assertFalse((self.root / ".ingress.fence").exists())
        self.assertEqual(self.acks, [])
        with self.assertRaises(d.Invalid):
            self.transport.gate()
        with self.assertRaises(FileNotFoundError):
            self.transport.qualify(ui_evidence())
        with self.assertRaises(d.Invalid):
            s.initialize(a.Store(self.root))

    def test_minimal_scopes_require_explicit_owner_ui_evidence(self):
        self.assertEqual(s.SCOPES, {"chat:write", "groups:history"})
        self.assertEqual([method for method, _ in self.calls], ["auth.test"])
        for evidence in (None, dict(ui_evidence(), checks=[]), dict(ui_evidence(), binding=dict(PIN, human="UOTHER"))):
            with self.assertRaises(d.Invalid):
                self.transport.qualify(evidence)
        with s.locked(self.store) as value:
            self.assertEqual(value["ui_evidence"], ui_evidence())

    @runtime_case
    def test_ongoing_reception_passes_sixty_seconds_reconnects_without_gap_clear(self):
        self.sending()
        transport, pauses, replies = self.transport, [], []
        monotonic, elapsed = time.monotonic, [0]
        class Control:
            stopped = False
            def is_set(self):
                return self.stopped
            def wait(self, seconds):
                pauses.append(seconds)
                if seconds == 0.1:
                    elapsed[0] += 61  # Cross the finite-proof deadline without a real overnight wait.
                    transport.now = lambda: "2026-09-12T13:00:00Z"  # One hour beyond initial qualification.
                    index = len(replies) + 1
                    transport.callback(transport.socket, SocketModeRequest(type="events_api", envelope_id="ongoing" + str(index),
                        payload=payload("EvLive" + str(index), ts=f"120.{index:06d}", event_ts=f"120.{index:06d}")))
                    replies.append(index)
                    if index == 1:
                        transport.socket.connect_to_new_endpoint(force=True)
                    else:
                        self.stopped = True
                return self.stopped
        self.owner.close()
        with patch.object(SocketModeClient, "connect", autospec=True, side_effect=lambda client: client.issue_new_wss_url()) as connect, \
                patch.object(SocketModeClient, "is_connected", return_value=True), \
                patch.object(SocketModeClient, "send_socket_mode_response") as acknowledge, \
                patch.object(s.time, "monotonic", side_effect=lambda: monotonic() + elapsed[0]):
            transport.listen(ongoing=True, stop=Control(), runtime=self.runtime)
        self.assertEqual(connect.call_count, 2)
        self.assertEqual(acknowledge.call_count, 2)
        self.assertIn(1, pauses)
        self.assertEqual(s.drain(a.Store(self.root), PIN), 2)
        with self.assertRaises(d.Invalid):
            transport.gate()
        with s.locked(self.store) as value:
            self.assertIsNotNone(value["hold"])
        self.assertEqual(len(self.messages), 2)

    @runtime_case
    def test_ongoing_reconnect_failure_streak_is_bounded_and_shutdown_is_explicit(self):
        self.reply = lambda *_: ({"ok": False, "error": "internal_error"}, 503, {})
        pauses = []
        class Control:
            def is_set(self):
                return False
            def wait(self, seconds):
                pauses.append(seconds)
                return False
        self.owner.close()
        with patch.object(SocketModeClient, "connect", autospec=True, side_effect=lambda client: client.issue_new_wss_url()):
            with self.assertRaises(SlackApiError):
                self.transport.listen(ongoing=True, stop=Control(), runtime=self.runtime)
        self.assertEqual(sum(method == "apps.connections.open" for method, _ in self.calls), 3)
        self.assertEqual(pauses, [1, 2])
        self.assertFalse(self.transport.active)
        self.assertEqual(len(self.messages), 0)

    def test_outbound_parent_details_notice_echoes_do_not_stop_human_intake(self):
        self.sending()
        a.notice(self.store, self.key, "clarification", d.digest(["audit"]), PIN, self.transport.exchange)
        for index, message in enumerate(self.messages):
            self.callback(payload("EvBot" + str(index), **dict(message, subtype="bot_message", user="UBOTTEST",
                          event_ts=message["ts"])), envelope_id="echo" + str(index))
        self.assertTrue(self.transport.active)
        self.callback()
        self.assertEqual(s.drain(a.Store(self.root)), 1)
        self.assertEqual(len(self.acks), 4)
        self.assertEqual(len(r.snapshot(self.store, self.key)["replies"]), 1)

    def test_parent_history_with_self_thread_recovers_but_reply_thread_remains_exact(self):
        request = a.request_for(self.key, a.inspect(self.store, self.key), "parent")
        self.reply = lambda *_: ({"ok": False, "error": "fatal_error"}, 200, {})
        with self.assertRaises(d.Invalid):
            self.transport.exchange(request)
        self.reply = None
        self.messages[0]["thread_ts"] = self.messages[0]["ts"]
        evidence = self.transport.reconcile(request)
        self.assertEqual(evidence["thread_ts"], None)
        row = a.inspect(self.store, self.key)
        row["parent"].update(state="sent", receipt=evidence)
        reply = a.request_for(self.key, row, "details")
        self.reply = lambda *_: ({"ok": False, "error": "fatal_error"}, 200, {})
        with self.assertRaises(d.Invalid):
            self.transport.exchange(reply)
        self.reply = None
        self.messages[-1]["thread_ts"] = self.messages[-1]["ts"]
        with self.assertRaises(d.Invalid):
            self.transport.reconcile(reply)

    @runtime_case
    def test_sdk_socket_rate_limit_and_forced_disconnect_cannot_retry(self):
        self.reply = lambda *_: ({"ok": False, "error": "ratelimited"}, 429, {"Retry-After": "1"})
        self.owner.close()
        with patch.object(SocketModeClient, "connect", autospec=True, side_effect=lambda client: client.issue_new_wss_url()):
            with self.assertRaises(SlackApiError):
                self.transport.listen(seconds=0.2, runtime=self.runtime)
        self.assertEqual(sum(method == "apps.connections.open" for method, _ in self.calls), 1)
        self.transport.socket.connect_to_new_endpoint(force=True)
        self.assertEqual(sum(method == "apps.connections.open" for method, _ in self.calls), 1)
        self.assertFalse(self.transport.active)

    def test_session_generation_duplicate_conflict_and_unbound_thread_hold(self):
        self.sending()
        request = SocketModeRequest(type="events_api", envelope_id="bad-session", payload=payload())
        self.transport.callback(object(), request)
        self.assertEqual(self.acks, [])
        self.transport.active = True
        with s.locked(self.store) as value:
            value["binding"] = dict(PIN, generation="rotated")
            self.store.write("transport", value)
        self.callback()
        self.assertEqual(self.acks, [])
        with s.locked(self.store) as value:
            value["binding"] = PIN
            self.store.write("transport", value)
        self.transport.active = True
        self.callback()
        self.callback(payload(text="Conflicting event ID"))
        self.assertEqual(len(self.acks), 1)
        self.transport.active = True
        self.callback(payload("EvOther", thread_ts="199.000001"))
        self.assertEqual(len(self.acks), 2)
        self.assertTrue(self.transport.active)
        with s.locked(self.store) as value:
            self.assertEqual(value["held_human"]["EvOther"]["envelope"]["thread_ts"], "199.000001")
            self.assertEqual(value["hold"], "ingress-held")
            self.assertEqual(value["watermark"], 1)
            self.assertEqual(value["events"]["Ev001"]["envelope"]["text"], "Five testers")

    def test_receipt_write_then_lost_return_and_mib_limit_retain_prior_state(self):
        write = self.store.write
        def lost(name, value):
            write(name, value)
            if name == "transport" and any(p["receipt"] for p in value["posts"].values()):
                raise SystemExit(35)
        with patch.object(self.store, "write", side_effect=lost), self.assertRaises(SystemExit):
            a.send(self.store, self.key, PIN, self.transport.exchange)
        row = a.inspect(a.Store(self.root), self.key)
        self.assertEqual(row["parent"]["state"], "uncertain")
        count = len(self.calls)
        evidence = self.transport.reconcile(row["parent"]["request"])
        self.assertEqual(len(self.calls), count)
        a.reconcile(self.store, self.key, "parent", evidence)
        before = (self.root / "transport.json").read_bytes()
        with s.locked(self.store) as value:
            value["hold"] = "x" * d.MAX_BYTES
            with self.assertRaises(d.Invalid):
                self.store.write("transport", value)
        self.assertEqual((self.root / "transport.json").read_bytes(), before)
        self.assertEqual(len(self.messages), 1)

    def test_outage_review_is_explicit_watermark_pinned_and_retained(self):
        self.sending()
        self.callback()
        s.hold(self.store, "outage-gap")
        review = dict(watermark=1, binding=d.digest(PIN), evidence="manager-gap-receipt", ingress=s.ingress_count(self.store),
                      **self.session_review())
        with self.assertRaises(d.Invalid):
            self.transport.qualify(ui_evidence(), review)
        s.drain(self.store)
        with self.assertRaises(d.Invalid):
            self.transport.qualify(ui_evidence(), dict(review, watermark=0))
        self.transport.qualify(ui_evidence(), review)
        with s.locked(self.store) as value:
            self.assertEqual(value["gap_reviews"], [dict(review, at=NOW)])
            self.assertIsNone(value["hold"])

    def test_attempt_validation_precedes_all_writes_and_http(self):
        request = a.request_for(self.key, a.inspect(self.store, self.key), "parent")
        for attempt in ("pf83-vm-key-20260915T103614Z", "", "a" * 31, "a" * 32, "a" * 63,
                        "a" * 65, "A" * 64, "g" * 64, "a" * 64 + "\n", None, 3, []):
            with self.subTest(attempt=attempt):
                before = (self.root / "transport.json").read_bytes()
                calls = len(self.calls)
                with patch.object(self.store, "write", side_effect=AssertionError("state written")), \
                        patch.object(self.transport, "gate", side_effect=AssertionError("gate reached")), \
                        self.assertRaises(d.Invalid):
                    self.transport.exchange(dict(request, attempt=attempt))
                self.assertEqual((self.root / "transport.json").read_bytes(), before)
                self.assertEqual(len(self.calls), calls)

    def test_digest_attempt_retains_uuid_receipt_and_duplicate_semantics(self):
        request = a.request_for(self.key, a.inspect(self.store, self.key), "parent")
        receipt = self.transport.exchange(request)
        self.assertEqual(self.messages[0]["client_msg_id"], str(s.uuid.UUID(request["attempt"][:32])))
        self.assertEqual(self.transport.exchange(request), receipt)
        self.assertEqual(self.transport.reconcile(request), receipt)
        self.assertEqual(len(self.messages), 1)

    def test_reconcile_orphan_refuses_non_string_attempt(self):
        # Owner input off the pipe must refuse like every other owner path.
        for attempt in ([], {}, 7, None):
            with self.subTest(attempt=attempt), self.assertRaises(d.Invalid):
                self.transport.reconcile_orphan(attempt, request_digest="a" * 64, evidence="inspection")

    def test_malformed_orphan_classifies_without_http_and_owner_retains_evidence(self):
        request = self.orphan()
        before = self.store.read("transport")
        with patch.object(self.transport, "web", side_effect=AssertionError("HTTP accessed")):
            outcome = self.transport.reconcile(request)
            self.assertEqual(outcome["state"], "never-sent")
            self.assertEqual(self.store.read("transport"), before)
            result = self.transport.reconcile_orphan(request["attempt"], request_digest=d.digest(request),
                                                     evidence="owner-inspection-sha256-" + "a" * 64)
        self.assertEqual(result, outcome)
        post = self.store.read("transport")["posts"][request["attempt"]]
        self.assertEqual({k: post[k] for k in before["posts"][request["attempt"]]}, before["posts"][request["attempt"]])
        self.assertEqual(post["reconciliation"], dict(outcome=outcome, at=NOW,
                         evidence="owner-inspection-sha256-" + "a" * 64))
        self.assertTrue(s.reconciled_never_sent(post))
        raw = (self.root / "transport.json").read_bytes()
        reopened = s.Transport(a.Store(self.root), PIN, lambda: self.fail("credentials"), live=True, now=lambda: NOW)
        self.assertEqual(reopened.reconcile_orphan(request["attempt"], request_digest=d.digest(request),
                         evidence=post["reconciliation"]["evidence"]), outcome)
        self.assertEqual((self.root / "transport.json").read_bytes(), raw)
        with self.assertRaises(d.Invalid):
            reopened.reconcile_orphan(request["attempt"], request_digest=d.digest(request), evidence="different")
        self.assertEqual((self.root / "transport.json").read_bytes(), raw)

    def test_orphan_repair_requires_exact_evidence_request_and_uncertain_receipt(self):
        request = self.orphan()
        for digest, evidence in ((d.digest(request), ""), ("0" * 64, "inspected"), (d.digest(request), None)):
            before = (self.root / "transport.json").read_bytes()
            with self.assertRaises(d.Invalid):
                self.transport.reconcile_orphan(request["attempt"], request_digest=digest, evidence=evidence)
            self.assertEqual((self.root / "transport.json").read_bytes(), before)
        with s.locked(self.store) as value:
            value["posts"][request["attempt"]]["receipt"] = {"retained": True}
            self.store.write("transport", value)
        before = (self.root / "transport.json").read_bytes()
        with self.assertRaises(d.Invalid):
            self.transport.reconcile_orphan(request["attempt"], request_digest=d.digest(request), evidence="inspected")
        self.assertEqual((self.root / "transport.json").read_bytes(), before)

    def test_owner_cannot_clear_any_legacy_encodable_attempt_or_missing_history(self):
        for attempt in ("a" * 64, "a" * 32, "A" * 64, "a" * 32 + "-legacy-suffix"):
            with self.subTest(attempt=attempt):
                request = self.orphan(attempt)
                before = (self.root / "transport.json").read_bytes()
                with self.assertRaises(d.Invalid):
                    self.transport.reconcile_orphan(attempt, request_digest=d.digest(request), evidence="inspected")
                with self.assertRaises(d.Invalid):
                    self.transport.reconcile(request)  # Empty complete history is still uncertainty.
                self.assertEqual((self.root / "transport.json").read_bytes(), before)
        self.assertEqual(len(self.messages), 0)

    def test_orphan_repair_preserves_fence_and_session_refusals(self):
        request = self.orphan()
        self.owner.close()
        before = (self.root / "transport.json").read_bytes()
        with self.assertRaises(d.Invalid):
            self.transport.reconcile_orphan(request["attempt"], request_digest=d.digest(request), evidence="inspected")
        self.assertEqual((self.root / "transport.json").read_bytes(), before)
        self.owner = s.Session(self.store)
        self.owner.update("connected")
        self.addCleanup(self.owner.release)
        s.ingress_count(self.store, mark=True)
        before = (self.root / "transport.json").read_bytes()
        with self.assertRaises(d.Invalid):
            self.transport.reconcile_orphan(request["attempt"], request_digest=d.digest(request), evidence="inspected")
        self.assertEqual((self.root / "transport.json").read_bytes(), before)

    def test_real_sdk_qualification_posts_and_restart_receipts(self):
        row = self.sending()
        self.assertEqual((row["parent"]["state"], row["details"]["state"]), ("sent", "sent"))
        self.assertEqual(self.options[0]["timeout"], 5)
        self.assertEqual(self.options[0]["retry_handlers"], [])
        self.assertEqual(self.messages[1]["thread_ts"], "100.000001")
        for _ in range(2):
            self.assertEqual(self.transport.reconcile(row["parent"]["request"]), row["parent"]["receipt"])
            a.send(a.Store(self.root), self.key, PIN, self.transport.exchange)
        self.assertEqual(len(self.messages), 2)
        self.assertNotIn("fixture-bot", (self.root / "transport.json").read_text())

    def test_off_missing_scopes_wrong_identity_and_stale_binding(self):
        transport = s.Transport(self.store, PIN, lambda: self.fail("credentials read while OFF"), now=lambda: NOW)
        with self.assertRaises(d.Invalid):
            transport.qualify(ui_evidence())
        self.scopes.remove("chat:write")
        with self.assertRaises(d.Invalid):
            self.transport.qualify(ui_evidence())
        with self.assertRaises(d.Invalid):
            self.transport.gate()
        with s.locked(self.store) as value:
            value["binding"] = dict(PIN, generation="rotated")
            self.store.write("transport", value)
        with self.assertRaises(d.Invalid):
            self.transport.gate()
        self.assertEqual(len(self.messages), 0)

    def test_no_retry_for_ambiguous_sdk_responses(self):
        cases = [(500, {"ok": False, "error": "internal_error"}), (200, {"ok": False, "error": "fatal_error"}),
                 (200, {"ok": True, "channel": "CWRONG", "ts": "100.000001"}),
                 (200, {"ok": False, "error": "unknown_error"}), (200, b"not-json"), (200, {"ok": True}), (200, None)]
        request = a.request_for(self.key, a.inspect(self.store, self.key), "parent")
        for index, (status, result) in enumerate(cases):
            with self.subTest(index=index):
                request["attempt"] = d.digest([index])
                self.reply = lambda method, normal: (result, status, {})
                with self.assertRaises(d.Invalid):
                    self.transport.exchange(copy.deepcopy(request))
                with self.assertRaises(d.Invalid):
                    self.transport.exchange(copy.deepcopy(request))
                self.assertEqual(len(self.messages), index + 1)

    def test_rate_limit_is_retained_without_retry(self):
        self.reply = lambda *_: ({"ok": False, "error": "ratelimited"}, 429, {"Retry-After": "42"})
        row = a.send(self.store, self.key, PIN, self.transport.exchange)
        self.assertEqual(row["parent"]["state"], "failed")
        with s.locked(self.store) as value:
            self.assertEqual(value["posts"][row["parent"]["request"]["attempt"]]["retry_after"], 42)
        a.send(self.store, self.key, PIN, self.transport.exchange)
        self.assertEqual(len(self.messages), 1)

    def test_http_timeout_and_wrong_channel_hold(self):
        self.transport.web().timeout = 0.03  # Real SDK timeout, shortened only for this controlled endpoint.
        def slow(*_):
            time.sleep(0.1)
            return {"ok": True, "channel": "CWRONG", "ts": "100.000001"}, 200, {}
        self.reply = slow
        row = a.send(self.store, self.key, PIN, self.transport.exchange)
        self.assertEqual(row["parent"]["state"], "uncertain")
        self.assertEqual(len(self.messages), 1)

    def test_process_crash_after_http_acceptance_reconciles_positive_history(self):
        worker = multiprocessing.get_context("spawn").Process(target=crash_post, args=(str(self.root), self.key, self.endpoint))
        worker.start()
        worker.join(10)
        self.assertEqual(worker.exitcode, 31)
        row = a.inspect(a.Store(self.root), self.key)
        self.assertEqual(row["parent"]["state"], "uncertain")
        evidence = self.transport.reconcile(row["parent"]["request"])
        a.reconcile(self.store, self.key, "parent", evidence)
        self.assertEqual(a.send(self.store, self.key, PIN, self.transport.exchange)["details"]["state"], "sent")
        self.assertEqual(len(self.messages), 2)

    def test_missing_or_ambiguous_history_never_resends(self):
        request = a.request_for(self.key, a.inspect(self.store, self.key), "parent")
        self.reply = lambda *_: ({"ok": False, "error": "fatal_error"}, 200, {})
        with self.assertRaises(d.Invalid):
            self.transport.exchange(request)
        self.reply = lambda *_: ({"ok": True, "messages": [], "response_metadata": {"next_cursor": "more"}}, 200, {})
        with self.assertRaises(d.Invalid):
            self.transport.reconcile(request)
        self.assertEqual(sum(method == "conversations.history" for method, _ in self.calls), 3)
        self.reply = None
        self.messages.append(copy.deepcopy(self.messages[0]))
        with self.assertRaises(d.Invalid):
            self.transport.reconcile(request)
        self.assertEqual(sum(method == "chat.postMessage" for method, _ in self.calls), 1)

    def test_callback_edit_delete_event_dedup_and_restart_drain(self):
        self.sending()
        self.callback()
        self.callback(envelope_id="different-envelope")
        self.assertEqual(s.drain(a.Store(self.root)), 1)
        edit = payload("Ev002", subtype="message_changed", message=dict(user=PIN["human"], ts="101.000001",
                       thread_ts="100.000001", text="Ten testers", edited={"user": PIN["human"]}), event_ts="102.000001")
        self.callback(edit)
        self.callback(payload("Ev003", subtype="message_deleted", deleted_ts="101.000001", event_ts="103.000001"))
        self.assertEqual(s.drain(a.Store(self.root)), 2)
        self.assertEqual(s.drain(a.Store(self.root)), 0)
        audit = r.snapshot(self.store, self.key)["replies"]
        self.assertEqual([e["envelope"]["kind"] for e in audit.values()], ["message", "edit", "delete"])
        self.assertEqual(audit["Ev003"]["envelope"]["user"], PIN["human"])
        self.assertEqual(len(self.acks), 4)
        with s.locked(self.store) as value:
            self.assertEqual(value["watermark"], 3)

    def unbind_fixture(self):
        row = self.sending()
        with s.locked(self.store) as value:
            value["routes"].clear()
            self.store.write("transport", value)
        return row

    def test_unbound_reply_survives_restart_and_binds_once_to_exact_thread(self):
        row = self.unbind_fixture()
        self.callback()
        retained = self.store.read("transport")["held_human"]["Ev001"]
        self.assertEqual(len(self.acks), 1)
        self.assertEqual(self.store.read("transport")["events"], {})
        self.assertEqual((d.stamp(retained["expires_at"]) - d.stamp(retained["arrived_at"])).total_seconds(), 900)
        self.transport.now = lambda: "2026-09-12T12:01:00Z"
        self.callback(envelope_id="redelivery")
        self.assertEqual(self.store.read("transport")["held_human"]["Ev001"], retained)
        restarted = s.Transport(a.Store(self.root), PIN, lambda: None, now=lambda: NOW)
        restarted.bind_alert(self.key, row)
        self.assertEqual(self.store.read("transport")["held_human"], {})
        self.assertEqual(self.store.read("transport")["events"]["Ev001"]["alert"], self.key)
        restarted.bind_alert(self.key, row)
        self.assertEqual(s.drain(a.Store(self.root), now=NOW), 1)
        self.assertEqual(s.drain(a.Store(self.root), now=NOW), 0)
        self.assertEqual(self.store.read("transport")["watermark"], 1)
        self.assertEqual(r.snapshot(self.store, self.key)["replies"]["Ev001"]["envelope"]["text"], "Five testers")

    def test_unbound_replay_write_failure_retains_reply_and_conflicting_retry_is_not_acked(self):
        row = self.unbind_fixture()
        self.callback()
        self.callback(payload(text="conflicting answer"))
        self.assertEqual(len(self.acks), 1)
        self.assertEqual(self.store.read("transport")["held_human"]["Ev001"]["envelope"]["text"], "Five testers")
        with patch.object(self.store, "write", side_effect=OSError("fixture")):
            with self.assertRaises(OSError):
                self.transport.bind_alert(self.key, row)
        journal = self.store.read("transport")
        self.assertIn("Ev001", journal["held_human"])
        self.assertEqual(journal["events"], {})
        self.assertEqual(journal["routes"], {})
        self.transport.bind_alert(self.key, row)
        self.assertEqual(s.drain(self.store, now=NOW), 1)
        self.assertEqual(s.fence_gap(self.store, self.store.read("transport")), 1)

    def test_unbound_late_bound_details_message_is_quarantined_not_delivered(self):
        row = self.unbind_fixture()
        self.callback(payload(ts=row["details"]["receipt"]["ts"]))
        self.transport.now = lambda: self.store.read("transport")["held_human"]["Ev001"]["expires_at"]
        self.transport.bind_alert(self.key, row)
        journal = self.store.read("transport")
        self.assertEqual(journal["held_human"], {})
        self.assertEqual(journal["events"], {})
        self.assertEqual(journal["quarantine"]["records"][-1]["reason"], "invalid-message")

    def test_unbound_edits_and_deletes_survive_until_binding(self):
        row = self.unbind_fixture()
        self.callback(payload("EvZ"))
        self.callback(payload("EvA", subtype="message_changed", message=dict(user=PIN["human"], ts="101.000001",
            thread_ts="100.000001", text="Ten testers", edited={"user": PIN["human"]}), event_ts="102.000001"))
        self.callback(payload("EvB", subtype="message_deleted", user=None,
                              deleted_ts="101.000001", event_ts="103.000001"))
        self.assertEqual(len(self.store.read("transport")["held_human"]), 3)
        self.transport.bind_alert(self.key, row)
        self.assertEqual(s.drain(self.store, now=NOW), 3)
        self.assertEqual({e["envelope"]["kind"] for e in self.store.read("replies")["events"].values()},
                         {"message", "edit", "delete"})

    def test_unbound_expiry_without_route_is_durable_and_cannot_be_resurrected(self):
        row = self.unbind_fixture()
        self.callback(payload(text="private held answer"))
        deadline = self.store.read("transport")["held_human"]["Ev001"]["expires_at"]
        self.transport.now = lambda: deadline
        with patch.object(self.store, "write", side_effect=OSError("fixture")):
            with self.assertRaises(OSError):
                s.drain(self.store, now=deadline)
        self.assertIn("Ev001", self.store.read("transport")["held_human"])
        self.assertNotIn("quarantine", self.store.read("transport"))
        self.assertEqual(s.drain(self.store, now=deadline), 0)
        self.transport.bind_alert(self.key, row)
        value = self.store.read("transport")
        self.assertEqual(value["held_human"], {})
        self.assertEqual(value["events"], {})
        self.assertEqual(s.outstanding_quarantine(value), dict(count=0, oldest_at=None))
        record = value["quarantine"]["records"][-1]
        self.assertEqual((record["reason"], record["channel"], record["message_ts"], record["at"]),
                         ("unbound-expired", PIN["channel"], "101.000001", deadline))
        self.assertNotIn("private held answer", d.canonical(value).decode())
        self.assertEqual(s.drain(self.store, now=deadline), 0)
        self.assertEqual(self.store.read("transport")["quarantine"]["total"], 1)

    def test_route_at_or_after_deadline_delivers_retained_reply_once(self):
        row = self.unbind_fixture()
        for index, delay in enumerate((0, 60)):
            with self.subTest(delay=delay):
                with s.locked(self.store) as journal:
                    journal["routes"].clear()
                    self.store.write("transport", journal)
                key = "EvLate" + str(index)
                self.callback(payload(key, ts=f"101.{index + 1:06d}"))
                deadline = self.store.read("transport")["held_human"][key]["expires_at"]
                at = (d.stamp(deadline) + s.dt.timedelta(seconds=delay)).strftime("%Y-%m-%dT%H:%M:%SZ")
                self.transport.now = lambda: at
                with patch.object(self.store, "write", side_effect=OSError("fixture")):
                    with self.assertRaises(OSError):
                        self.transport.bind_alert(self.key, row)
                self.assertIn(key, self.store.read("transport")["held_human"])
                self.transport.bind_alert(self.key, row)
                self.assertEqual(s.drain(self.store, now=at), 1)
                self.assertEqual(s.drain(self.store, now=at), 0)
                self.assertIn(key, self.store.read("replies")["events"])
                self.assertNotIn("quarantine", self.store.read("transport"))

    def test_routed_reply_waits_for_queue_capacity_past_deadline(self):
        row = self.unbind_fixture()
        self.callback()
        journal = self.store.read("transport")
        journal["routes"]["100.000001"] = dict(alert=self.key, details=row["details"]["receipt"]["ts"])
        journal["events"] = {str(index): dict(drained=False) for index in range(100)}
        deadline = journal["held_human"]["Ev001"]["expires_at"]
        s.settle_held(journal, deadline)
        self.assertIn("Ev001", journal["held_human"])
        self.assertNotIn("quarantine", journal)
        journal["events"].clear()
        s.settle_held(journal, deadline)
        self.assertIn("Ev001", journal["events"])
        self.assertFalse(journal["held_human"])

    def test_outstanding_quarantine_survives_pruning_then_exact_review_clears_it(self):
        self.sending()
        for index in range(130):
            self.callback(payload("EvPoison" + str(index), subtype="message_deleted",
                                  deleted_ts="109.000001"))
        journal = self.store.read("transport")
        self.assertEqual(len(journal["quarantine"]["records"]), 128)
        self.assertEqual(s.outstanding_quarantine(journal), dict(count=130, oldest_at=NOW))
        write = self.store.write
        def fail_review(name, value):
            if name == "transport" and value["gap_reviews"]:
                raise OSError("fixture final review write")
            return write(name, value)
        with patch.object(self.store, "write", side_effect=fail_review):
            with self.assertRaises(d.Invalid):
                self.review_gap()
        self.assertEqual(s.outstanding_quarantine(self.store.read("transport"))["count"], 130)
        self.review_gap()
        journal = self.store.read("transport")
        self.assertEqual(s.outstanding_quarantine(journal), dict(count=0, oldest_at=None))
        self.assertEqual(journal["quarantine"]["total"], 130)
        later = (d.stamp(NOW) + s.dt.timedelta(seconds=60)).strftime("%Y-%m-%dT%H:%M:%SZ")
        self.transport.now = lambda: later
        self.callback(payload("EvNew", subtype="message_deleted", deleted_ts="110.000001"))
        self.assertEqual(s.outstanding_quarantine(self.store.read("transport")), dict(count=1, oldest_at=later))

    def test_legacy_unknown_is_durable_and_exact_review_does_not_rearm(self):
        self.sending()
        self.callback(payload("EvOld", subtype="message_deleted", deleted_ts="109.000001"))
        self.review_gap()
        with s.locked(self.store) as journal:
            journal["quarantine"] = dict(total=129, records=journal["quarantine"]["records"])
            self.store.write("transport", journal)
        expected = dict(count=0, oldest_at=None, unknown=128)
        self.assertEqual(s.outstanding_quarantine(self.store.read("transport")), expected)
        self.callback(payload("EvBot", user=PIN["bot"], bot_id=PIN["bot"], app_id=PIN["app"]))
        self.assertEqual(s.outstanding_quarantine(self.store.read("transport")), expected)
        self.assertIsNone(self.store.read("transport")["hold"])
        self.callback(payload("EvNew", subtype="message_deleted", deleted_ts="110.000001"))
        expected = dict(count=1, oldest_at=NOW, unknown=128)
        self.assertEqual(self.store.read("transport")["quarantine"]["outstanding"], expected)
        write = self.store.write
        def fail_review(name, value):
            if name == "transport" and len(value["gap_reviews"]) == 2:
                raise OSError("fixture final legacy review write")
            return write(name, value)
        with patch.object(self.store, "write", side_effect=fail_review), self.assertRaises(d.Invalid):
            self.review_gap()
        self.assertEqual(s.outstanding_quarantine(self.store.read("transport")), expected)
        self.review_gap()
        # Reopen the durable store; unrelated arrivals and a fresh rejection
        # must not resurrect the pruned prefix after successful review.
        reopened = a.Store(self.root)
        self.assertEqual(reopened.read("transport")["quarantine"]["outstanding"],
                         dict(count=0, oldest_at=None))
        self.callback(payload("EvAnotherBot", user=PIN["bot"], bot_id=PIN["bot"], app_id=PIN["app"]))
        self.assertEqual(s.outstanding_quarantine(reopened.read("transport")), dict(count=0, oldest_at=None))
        later = (d.stamp(NOW) + s.dt.timedelta(seconds=60)).strftime("%Y-%m-%dT%H:%M:%SZ")
        self.transport.now = lambda: later
        self.callback(payload("EvLater", subtype="message_deleted", deleted_ts="111.000001"))
        self.assertEqual(s.outstanding_quarantine(reopened.read("transport")), dict(count=1, oldest_at=later))
        self.assertEqual(reopened.read("transport")["quarantine"]["total"], 131)

    def quiet_legacy_unknown(self):
        self.sending()
        self.callback(payload("EvOld", subtype="message_deleted", deleted_ts="109.000001"))
        self.review_gap()
        with s.locked(self.store) as journal:
            journal["quarantine"] = dict(total=129, records=journal["quarantine"]["records"])
            self.store.write("transport", journal)
        self.assertIsNone(journal["hold"])
        self.assertEqual(s.fence_gap(self.store, journal), 0)
        return dict(count=0, oldest_at=None, unknown=128)

    def test_quiet_legacy_unknown_requires_explicit_review_and_clears_durably(self):
        expected = self.quiet_legacy_unknown()
        self.transport.qualify(ui_evidence())
        self.assertEqual(s.outstanding_quarantine(self.store.read("transport")), expected)
        self.assertEqual(len(self.store.read("transport")["gap_reviews"]), 1)
        self.review_gap()
        reopened = a.Store(self.root)
        reviewed = reopened.read("transport")
        self.assertEqual(len(reviewed["gap_reviews"]), 2)
        self.assertEqual(reviewed["quarantine"]["outstanding"], dict(count=0, oldest_at=None))
        self.assertEqual(reviewed["quarantine"]["total"], 129)
        self.assertIsNone(reviewed["hold"])
        self.callback(payload("EvBot", user=PIN["bot"], bot_id=PIN["bot"], app_id=PIN["app"]))
        self.assertEqual(s.outstanding_quarantine(reopened.read("transport")), dict(count=0, oldest_at=None))
        self.callback(payload("EvNew", subtype="message_deleted", deleted_ts="110.000001"))
        self.assertEqual(s.outstanding_quarantine(reopened.read("transport")), dict(count=1, oldest_at=NOW))

    def test_quiet_legacy_unknown_failed_review_write_preserves_unknown(self):
        expected = self.quiet_legacy_unknown()
        write = self.store.write
        def fail_review(name, value):
            if name == "transport" and len(value["gap_reviews"]) == 2:
                raise OSError("fixture quiet legacy review write")
            return write(name, value)
        with patch.object(self.store, "write", side_effect=fail_review), self.assertRaises(d.Invalid):
            self.review_gap()
        reopened = a.Store(self.root)
        self.assertEqual(s.outstanding_quarantine(reopened.read("transport")), expected)
        self.assertEqual(len(reopened.read("transport")["gap_reviews"]), 1)
        self.review_gap()
        self.assertEqual(s.outstanding_quarantine(reopened.read("transport")), dict(count=0, oldest_at=None))

    def test_quiet_legacy_unknown_rejects_inexact_explicit_reviews(self):
        expected = self.quiet_legacy_unknown()
        original = self.store.read("transport")
        review = dict(watermark=original["watermark"], ingress=s.ingress_count(self.store),
                      binding=d.digest(PIN), evidence="fixture-quiet-review", **self.session_review())
        for field, invalid in (("watermark", review["watermark"] + 1),
                               ("ingress", review["ingress"] + 1),
                               ("binding", d.digest("wrong-binding")),
                               ("session", "wrong-session"), ("epoch", review["epoch"] + 1),
                               ("evidence", "")):
            with self.subTest(field=field):
                # Each attempt starts quiet, so an earlier refusal cannot make
                # later attempts enter the pre-existing held-journal path.
                self.store.write("transport", copy.deepcopy(original))
                with self.assertRaises(d.Invalid):
                    self.transport.qualify(ui_evidence(), dict(review, **{field: invalid}))
                self.assertEqual(s.outstanding_quarantine(self.store.read("transport")), expected)
                self.assertEqual(len(self.store.read("transport")["gap_reviews"]), 1)

    def test_legacy_unknown_summary_rejects_invalid_counts(self):
        self.sending()
        self.callback(payload("EvPoison", subtype="message_deleted", deleted_ts="109.000001"))
        original = self.store.read("transport")
        for unknown in (-1, True, "1", 1, 2):
            with self.subTest(unknown=unknown):
                journal = copy.deepcopy(original)
                journal["quarantine"]["outstanding"]["unknown"] = unknown
                self.store.write("transport", journal)
                with self.assertRaises(d.Invalid), s.locked(self.store):
                    pass
        self.store.write("transport", original)

    def test_unbound_idle_renewal_records_expiry_without_new_callbacks(self):
        self.unbind_fixture()
        self.callback()
        deadline = self.store.read("transport")["held_human"]["Ev001"]["expires_at"]
        self.owner.now = lambda: deadline
        self.owner.update("connected")
        self.assertEqual(self.store.read("transport")["held_human"], {})
        self.assertEqual(self.store.read("transport")["quarantine"]["records"][-1]["reason"], "unbound-expired")

    def test_unbound_queue_full_and_write_failure_do_not_ack_or_drop_held_replies(self):
        self.unbind_fixture()
        with patch.object(self.store, "write", side_effect=OSError("fixture")):
            self.callback()
        self.assertEqual(self.acks, [])
        self.assertNotIn("held_human", self.store.read("transport"))
        self.transport.active = True
        for index in range(s.HELD_HUMAN_LIMIT):
            self.callback(payload("EvHeld" + str(index)))
        retained = self.store.read("transport")["held_human"]
        self.assertEqual(len(retained), 100)
        self.callback(payload("EvOverflow"))
        self.assertEqual(len(self.acks), 100)
        self.assertFalse(self.transport.active)
        self.assertEqual(self.store.read("transport")["held_human"], retained)
        self.assertEqual(s.fence_gap(self.store, self.store.read("transport")), 1)

    def test_unbound_invalid_author_and_content_are_never_retained_for_replay(self):
        self.unbind_fixture()
        self.callback(payload(bot_id="BOTHER"))
        self.callback(payload(text="x" * 4001))
        self.assertFalse(self.store.read("transport").get("held_human"))
        self.assertEqual(self.store.read("transport")["quarantine"]["total"], 2)
        self.assertEqual(len(self.acks), 2)

    def test_poison_shapes_are_quarantined_and_following_human_reply_is_delivered(self):
        self.sending()
        cases = [
            (payload(user="UOTHER"), "nonhuman-author", "other"),
            (payload(subtype="message_deleted", deleted_ts="109.000001"), "unknown-delete", "unknown"),
            (payload(subtype="message_changed", message=dict(user=PIN["human"], ts="101.000001",
                thread_ts="100.000001", text="private-body", edited={"user": "UOTHER"})),
                "invalid-message", "pinned-human"),
            (payload(subtype="message_replied"), "unsupported-subtype", "pinned-human"),
            (dict(payload(), event=None), "invalid-message", "unknown"),
            (dict(payload(), event=[]), "invalid-message", "unknown"),
            (payload(subtype="message_changed", message=None), "invalid-message", "unknown"),
            (payload(subtype=[]), "invalid-message", "pinned-human"),
            (payload(text="private-body" * 500), "invalid-message", "pinned-human"),
        ]
        for index, (event, reason, hint) in enumerate(cases):
            with self.subTest(index=index):
                self.callback(event, envelope_id="poison-" + str(index))
                journal = self.store.read("transport")
                self.assertTrue(self.transport.active)
                self.assertEqual(len(self.acks), index + 1)
                self.assertEqual(s.fence_gap(self.store, journal), 0)
                record = journal["quarantine"]["records"][-1]
                self.assertEqual((record["reason"], record["author_hint"]), (reason, hint))
                self.assertEqual(record["ingress"], index + 1)
                self.assertNotIn("private-body", d.canonical(journal["quarantine"]).decode())
        self.callback(payload("EvAfterPoison"))
        self.assertEqual(s.drain(self.store), 1)
        self.assertEqual(list(r.snapshot(self.store, self.key)["replies"]), ["EvAfterPoison"])
        journal = self.store.read("transport")
        self.assertEqual(journal["quarantine"]["total"], len(cases))
        self.assertEqual(journal["hold"], "ingress-held")
        with self.assertRaises(d.Invalid):
            self.transport.gate()  # Continuing intake does not grant decision/work authority.
        self.review_gap()
        self.transport.gate()
        expected = dict(journal["quarantine"], outstanding=dict(count=0, oldest_at=None))
        self.assertEqual(self.store.read("transport")["quarantine"], expected)

    def test_quarantine_is_bounded_and_redelivery_does_not_starve_valid_intake(self):
        self.sending()
        event = payload("EvPoison", subtype="message_deleted", deleted_ts="109.000001")
        for _ in range(131):
            self.callback(event)
        journal = self.store.read("transport")
        self.assertEqual(journal["quarantine"]["total"], 131)
        self.assertEqual(len(journal["quarantine"]["records"]), 128)
        self.assertEqual(journal["quarantine"]["records"][0]["ingress"], 4)
        self.assertEqual(len(self.acks), 131)
        self.assertTrue(self.transport.active)
        self.callback(payload("EvSurvived"))
        self.assertEqual(s.drain(self.store), 1)

    def test_quarantine_write_failure_never_acknowledges_or_covers_callback(self):
        self.sending()
        with patch.object(self.store, "write", side_effect=OSError("private-body")):
            self.callback(payload(subtype="message_deleted", deleted_ts="109.000001"))
        self.assertEqual(self.acks, [])
        self.assertFalse(self.transport.active)
        journal = self.store.read("transport")
        self.assertNotIn("quarantine", journal)
        self.assertEqual(s.fence_gap(self.store, journal), 1)
        self.assertEqual(self.transport.failure_snapshot(),
                         s.failure_record("transport-io", "callback-store", callbacks=1, disconnect_marks=0, quarantined=0))

    @runtime_case
    def test_three_fence_marks_can_be_cleanup_with_zero_callbacks(self):
        self.owner.close()
        before = s.ingress_count(self.store)
        write = self.store.write
        def fail_connected(name, value):
            if name == "transport" and value["lifecycle"]["session"]["phase"] == "connected":
                raise BlockingIOError("private fixture lock failure")
            return write(name, value)
        with patch.object(SocketModeClient, "connect"), \
                patch.object(SocketModeClient, "is_connected", return_value=True), \
                patch.object(self.store, "write", side_effect=fail_connected):
            with self.assertRaises(BlockingIOError):
                self.transport.listen(ongoing=True, runtime=self.runtime)
        self.assertEqual(s.ingress_count(self.store) - before, 3)
        self.assertEqual(self.store.read("transport")["ingress"], 0)
        self.assertEqual(self.transport.failure_snapshot(),
                         s.failure_record("transport-busy", "session-renew", callbacks=0, disconnect_marks=3, quarantined=0))
        self.assertEqual(self.acks, [])

    def test_wrong_envelopes_are_held_without_ack(self):
        self.sending()
        cases = [dict(payload(), team_id="TWRONG"), dict(payload(), api_app_id="AWRONG"),
                 payload(text="x" * 17000)]
        for value in cases:
            self.transport.active = True  # Independent authenticated callback-seam trial.
            self.callback(value)
            self.assertEqual(self.acks, [])
        self.assertEqual(s.drain(self.store), 0)
        with s.locked(self.store) as value:
            self.assertEqual(value["hold"], "ingress-held")

    def test_ingress_process_crash_before_ack_and_independent_locks(self):
        self.sending()
        ctx = multiprocessing.get_context("spawn")
        ready, release = ctx.Event(), ctx.Event()
        worker = ctx.Process(target=hold_offline, args=(str(self.root), str(self.feed_root), ready, release))
        worker.start()
        self.assertTrue(ready.wait(5))
        started = time.monotonic()
        self.callback()
        self.assertLess(time.monotonic() - started, 3)
        release.set()
        worker.join(5)
        self.assertEqual(worker.exitcode, 0)
        crash = ctx.Process(target=crash_ack, args=(str(self.root),))
        crash.start()
        crash.join(10)
        self.assertEqual(crash.exitcode, 32)
        self.assertEqual(s.drain(a.Store(self.root)), 1)
        self.assertEqual(len(r.snapshot(self.store, self.key)["replies"]), 1)

    def test_drain_crash_after_intake_deduplicates_and_full_state_holds(self):
        self.sending()
        self.callback()
        with patch.object(self.store, "write", side_effect=OSError("fixture storage unavailable")), self.assertRaises(d.Invalid):
            s.drain(self.store)
        self.assertEqual(s.drain(a.Store(self.root)), 1)
        self.assertEqual(len(r.snapshot(self.store, self.key)["replies"]), 1)
        for index in range(100):
            self.callback(payload("Ev" + str(index + 10), ts=f"110.{index + 1:06d}", event_ts=f"110.{index + 1:06d}"))
        count = len(self.acks)
        self.callback(payload("EvFull", ts="111.000001", event_ts="111.000001"))
        self.assertEqual(len(self.acks), count)
        with s.locked(self.store) as value:
            self.assertEqual(sum(not e["drained"] for e in value["events"].values()), 100)

    @runtime_case
    def test_fsync_corruption_missing_extension_and_socket_shutdown_hold(self):
        self.sending()
        with patch.object(a.os, "fsync", side_effect=OSError("fixture disk failure")):
            self.callback()
        self.assertEqual(self.acks, [])
        with self.assertRaises(d.Invalid):
            s.initialize(self.store)
        self.transport.active = False
        stop = threading.Event()
        class StopAfterConnect:
            def is_set(self):
                return stop.is_set()
            def wait(self, _):
                stop.set()
                return True
        self.owner.close()
        with patch.object(SocketModeClient, "connect") as connect, patch.object(SocketModeClient, "is_connected", return_value=True):
            with s.locked(self.store) as value:
                value["ingress"] = s.ingress_count(self.store)
                self.store.write("transport", value)
            self.transport.listen(seconds=0.1, stop=StopAfterConnect(), runtime=self.runtime)
        self.assertEqual(connect.call_count, 1)
        self.assertFalse(self.transport.socket.auto_reconnect_enabled)
        with s.locked(self.store) as value:
            self.assertEqual(value["hold"], "outage-gap")
        (self.root / "transport.json").write_bytes(b"broken fixture")
        with self.assertRaises((d.Invalid, ValueError)):
            self.transport.gate()
