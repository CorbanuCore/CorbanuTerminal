import copy
from contextlib import contextmanager
import io
import json
import multiprocessing
import os
from pathlib import Path
import select
import subprocess
import sys
import tempfile
from types import SimpleNamespace
from unittest.mock import patch

import decisions as d
import decision_alerts as a
import decision_replies as r
import decision_manager as m
import slack_transport as s
import test_slack_transport as fixtures
from test_decisions import NOW, revision
from test_decision_alerts import PIN, OWNER


@contextmanager
def fixture_child(mode, endpoint):
    popen = subprocess.Popen
    def launch(command, **kwargs):
        env = dict(os.environ, PYTHONPATH=str(Path(__file__).parent) + os.pathsep + os.environ.get("PYTHONPATH", ""))
        command = [command[0], "-B", "-c", "from test_decision_manager import supervised_child; import sys; supervised_child(*sys.argv[1:])",
                   *command[4:], mode, endpoint]
        return popen(command, env=env, **kwargs)
    with patch.object(m.subprocess, "Popen", side_effect=launch):
        yield


def supervised_child(root, guard, control, mode, endpoint):
    if mode == "ignore-term":
        fixtures.signal.signal(fixtures.signal.SIGTERM, fixtures.signal.SIG_IGN)
    def run(runtime, stop, data):
        assert not os.get_inheritable(runtime.fd)
        try:
            subprocess.Popen([sys.executable, "-c", "pass"], pass_fds=(runtime.fd,))
        except d.Invalid:
            pass
        else:
            raise AssertionError("SDK child could create a guard-inheriting descendant")
        transport = s.Transport(a.Store(root), PIN, lambda: ("fixture-bot", "fixture-app"), live=True, now=lambda: NOW,
                                web_factory=lambda **kw: fixtures.WebClient(base_url=endpoint, **kw))
        update, mark_original = s.Session.update, s.ingress_count
        owners, peers = [], []
        def connect(client):
            client.issue_new_wss_url()
            session = fixtures.Connection("wss://fixture.invalid", s.QUIET)
            session.sock, peer = fixtures.socket.socketpair()
            session.sock.settimeout(0.05)  # Match SDK connection timeout; a blocking fixture socket strands close.
            peers.append(peer)
            client.current_session = session
        def renewed(owner, phase):
            update(owner, phase)
            if phase == "connected" and len(owners) == 1:
                owners.append(owner)
                m.Stdio().emit(dict(type="renewed"))
            if phase == "connected" and not owners:
                owners.append(owner)
                m.Stdio().emit(dict(type="connected"))
                if mode == "hung":
                    transport.socket.enqueue_message(json.dumps(dict(type="events_api", envelope_id="fixture", payload=fixtures.payload())))
        def blocked(store, mark=False):
            if mark and fixtures.threading.current_thread() is not fixtures.threading.main_thread() and mode == "hung":
                fd = os.open(store.root / ".ingress.fence", os.O_WRONLY | os.O_APPEND)
                owners[0].release()  # Real SDK executor survives release of the old lease.
                m.Stdio().emit(dict(type="stopped"))  # Deliberately false frame, never death evidence.
                fixtures.threading.Event().wait()
                os.close(fd)
            return mark_original(store, mark=mark)
        with (patch.object(fixtures.SocketModeClient, "connect", autospec=True, side_effect=connect),
              patch.object(s.Session, "update", renewed), patch.object(s, "ingress_count", blocked)):
            transport.listen(seconds=data["seconds"], ongoing=data["ongoing"], stop=stop, runtime=runtime)
    m.listener_child(root, int(guard), int(control), run=run)


def orphan_controller(root, endpoint):
    manager = m.ManagedListener(a.Store(root), PIN, live=True)
    with fixture_child("hung", endpoint):
        manager.start(ongoing=True)
        channel = m.Stdio(manager.process.stdout, manager.process.stdin)
        assert channel.read() == dict(type="connected")
        assert channel.read() == dict(type="stopped")
        m.Stdio().emit(dict(type="owned-child-ready"))
        fixtures.threading.Event().wait()


def controller_boundary(root, endpoint, phase):
    manager = m.ManagedListener(a.Store(root), PIN, live=True)
    def boundary():
        m.Stdio().emit(dict(type="boundary", phase=phase))
        fixtures.threading.Event().wait()  # Controller killed here, never an invented successful wait.
    with fixture_child("hung" if phase == "stop" else "connected", endpoint):
        if phase == "before-ready":
            emit = m.Stdio.emit
            def before_config(channel, value):
                if "binding" in value:
                    boundary()
                return emit(channel, value)
            with patch.object(m.Stdio, "emit", before_config):
                manager.start(ongoing=True)
        else:
            manager.start(ongoing=True)
            channel = m.Stdio(manager.process.stdout, manager.process.stdin)
            assert channel.read() == dict(type="connected")
            if phase == "renewal":
                assert channel.read() == dict(type="renewed")
            elif phase == "stop":
                assert channel.read() == dict(type="stopped")
                with patch.object(manager.process, "wait", side_effect=lambda **_: boundary()):
                    manager.stop()  # EOF already closed; no store lock is held at the injected boundary.
            elif phase == "reap":
                manager.stop()
            boundary()


def bridge_child(root, feed, key, mode):
    store = a.Store(root)
    transport = s.Transport(store, PIN, lambda: None, live=True, now=lambda: NOW)
    channel = m.Stdio(timeout=0.3 if mode == "timeout" else 5)
    write = store.write
    def checkpoint(name, value):
        write(name, value)
        if name == "transport" and value["bridges"]:
            bridge = value["bridges"][key]
            if mode == "request-crash":
                os._exit(33)
            if mode == "acceptance-crash" and bridge["submission_id"]:
                os._exit(34)
    store.write = checkpoint
    result = m.dispatch(store, feed, key, transport, channel, lambda: OWNER, NOW)
    channel.emit(dict(type="result", result=result))


def accepted(outbound):
    request = outbound["request"]
    return dict(type="accepted", tool="multi_agent_v1.send_input", agent=request["owner"]["agent"],
                handoff=request["handoff"], payload_digest=request["payload_digest"], submission_id="actual-fixture-submission-1")


def assistant(outbound):
    return dict(type="assistant", tool="multi_agent_v1.wait_agent", agent=outbound["request"]["owner"]["agent"],
                submission_id="actual-fixture-submission-1", text=json.dumps(outbound["expected_ack"]))


def contended_delete(root):
    transport = s.Transport(a.Store(root), PIN, lambda: None, live=True, now=lambda: NOW)
    acknowledgments = []
    client = fixtures.sdk_socket(transport, acknowledgments.append)
    try:
        event = fixtures.payload("EvFinalWrite", subtype="message_deleted", deleted_ts="101.000001", event_ts="102.000001")
        transport.callback(client, fixtures.SocketModeRequest(type="events_api", envelope_id="final-write", payload=event))
        assert not transport.active and not acknowledgments
    finally:
        client.close()


def cli_child(root, feed, endpoint):
    with patch("slack_sdk.WebClient", side_effect=lambda **kw: fixtures.WebClient(base_url=endpoint, **kw)):
        m.main(["dispatch", "--store", root, "--feed", feed, "--live"],
               credentials=lambda: ("fixture-bot", "fixture-app"), observe_owner=lambda: OWNER, now=lambda: NOW)


class ManagerTests(fixtures.LiveFixture):
    def test_unacknowledged_answer_projects_until_real_ack_and_sent_notice(self):
        import decision_feed
        from test_decision_replies import receive
        key = self.queue()
        saved = d.load_fixture(self.feed_root, NOW)
        self.assertEqual(saved["decisions"][0]["revisions"][-1]["status"], "resolved")
        calls = len(self.calls)
        before = {name: (self.root / (name + ".json")).read_bytes() for name in ("alerts", "replies")}
        with patch.object(s.Transport, "web", side_effect=AssertionError("projection posted")):
            status = m.project_status(self.store, NOW, True)
            self.assertEqual(status["unacknowledged_answers"], 1)
            output = io.StringIO()
            cli = m.main(["project-status", "--store", str(self.root), "--live"],
                         now=lambda: NOW, stdout=output)
            self.assertEqual(json.loads(output.getvalue()), cli)
            self.assertEqual(cli["unacknowledged_answers"], 1)
            projected = decision_feed.project_slack(self.feed_root, self.root, NOW, True)
        self.assertEqual(projected["status"]["unacknowledged_answers"], 1)
        self.assertEqual(projected["decisions"][0]["replies"]["unacknowledged_answers"], 1)
        snapshot = dict(schema=2, status="valid", feed=saved, slack_status="valid", slack=projected)
        page = decision_feed.render(snapshot, NOW, [], {})
        self.assertIn("unacknowledged answers: 1", page)
        self.assertEqual(len(self.calls), calls)
        for name, raw in before.items():
            self.assertEqual((self.root / (name + ".json")).read_bytes(), raw)
        intent = r.dispatch(self.store, self.feed_root, key, OWNER, receive, NOW)
        self.assertEqual(m.project_status(self.store, NOW, True)["unacknowledged_answers"], 1)
        ack = dict(handoff=key, agent=OWNER["agent"], allocation=OWNER["allocation"],
                   payload_digest=intent["request"]["payload_digest"], receipt_id=intent["receipt"]["receipt_id"])
        with self.assertRaises(d.Invalid):
            r.reconcile_handoff(self.store, key, intent["receipt"], dict(ack, allocation="wrong"))
        self.assertEqual(m.project_status(self.store, NOW, True)["unacknowledged_answers"], 1)
        r.reconcile_handoff(self.store, key, intent["receipt"], ack)
        self.assertEqual(m.project_status(self.store, NOW, True)["unacknowledged_answers"], 1)
        requests = []
        def timeout(request):
            requests.append(request)
            raise TimeoutError()
        notice = a.notice(self.store, self.key, "acknowledged", d.digest(ack), PIN, timeout)
        self.assertEqual(m.project_status(self.store, NOW, True)["unacknowledged_answers"], 1)
        evidence = dict(attempt=requests[0]["attempt"], identity=PIN,
                        payload_digest=requests[0]["payload_digest"], thread_ts=requests[0]["thread_ts"], ts="109.000001")
        a.reconcile(self.store, self.key, notice["request"]["attempt"], evidence)
        self.assertEqual(m.project_status(self.store, NOW, True)["unacknowledged_answers"], 0)
        self.assertEqual(decision_feed.project_slack(self.feed_root, self.root, NOW, True)
                         ["decisions"][0]["replies"]["unacknowledged_answers"], 0)
        self.assertEqual(d.load_fixture(self.feed_root, NOW), saved)

    def test_recorded_resolution_crash_and_held_transport_keep_outstanding_count(self):
        key = self.queue()
        # Recreate the durable crash boundary: resolution saved, queue write lost.
        ledger = self.store.read("replies")
        ledger["intents"][key]["state"] = "recorded"
        ledger["events"]["Ev001"]["state"] = "recorded"
        self.store.write("replies", ledger)
        self.assertEqual(m.project_status(self.store, NOW, True)["unacknowledged_answers"], 1)
        self.owner.release()
        status = m.project_status(self.store, NOW, True)
        self.assertEqual(status["state"], "held")
        self.assertEqual(status["unacknowledged_answers"], 1)
        legacy = m.project_status(None, NOW)
        del legacy["unacknowledged_answers"]
        self.assertEqual(m.validate_status(legacy), legacy)
        for bad in (True, -1, "1", None):
            with self.assertRaises(d.Invalid):
                m.validate_status(dict(legacy, unacknowledged_answers=bad))

    def follower(self, *, parent=True):
        if parent:
            self.sending()
        feed = copy.deepcopy(self.feed)
        feed["revision"] += 1
        child = copy.deepcopy(feed["decisions"][0])
        child.update(id="choice-2", follows="choice-1")
        feed["decisions"].append(child)
        d.save_fixture(self.feed_root, feed, d.digest(self.feed), NOW)
        return a.enqueue(self.store, feed, "choice-2", fixtures.REMOTE, PIN, OWNER, NOW)

    def assert_parent_answerable(self, *, already_received=False):
        if not already_received:
            self.callback(fixtures.payload("EvParent"))
        self.assertTrue(self.transport.active)
        self.assertEqual(self.acks, ["envelope-1"])
        self.assertIsNone(self.store.read("transport")["hold"])
        self.assertEqual(self.store.read("transport")["events"]["EvParent"]["alert"], self.key)
        self.assertEqual(s.drain(self.store), 1)
        self.assertEqual(self.store.read("replies")["events"]["EvParent"]["alert"], self.key)
        row = a.inspect(self.store, self.key)
        manager = dict(actor="manager-fixture", answer="Five testers", scope="Synthetic parent only", alert=self.key,
                       context_digest=row["intent"]["context_digest"], audit=r.snapshot(self.store, self.key)["audit"],
                       owner=OWNER, interpretation="answer")
        guarded = m.ResolutionStore(self.transport)
        handoff = r.interpret(guarded, self.feed_root, "EvParent", manager, OWNER, NOW)
        self.assertIsNotNone(handoff)
        self.assertEqual(r.resume(guarded, self.feed_root, handoff, OWNER, NOW), "queued")
        self.assertEqual(d.load_fixture(self.feed_root, NOW)["decisions"][0]["revisions"][-1]["status"], "resolved")

    def test_follower_sdk_each_thread_attributes_only_its_own_question(self):
        key = self.follower()
        original = copy.deepcopy(self.store.read("alerts")[self.key])
        calls = len(self.calls)
        row = a.send(self.store, key, PIN, self.transport.exchange)
        self.transport.bind_alert(key, row)
        self.assertNotEqual(row["parent"]["receipt"]["ts"], original["parent"]["receipt"]["ts"])
        self.assertEqual(len(self.calls), calls + 3)
        self.assertEqual(row["details"]["receipt"]["thread_ts"], row["parent"]["receipt"]["ts"])
        pointer = row["notices"][d.digest([key, "follow-up", self.key])]
        self.assertEqual(pointer["receipt"]["thread_ts"], original["parent"]["receipt"]["ts"])
        self.assertEqual(pointer["request"]["payload"]["text"],
            "A follow-up decision has been raised. Find it in its own thread: Open follow-up thread.")
        mention = pointer["request"]["payload"]["blocks"][0]["elements"][0]["elements"][1]
        self.assertEqual(mention, dict(type="message_mention", channel_id="CTEST", message_ts="100.000003"))
        self.assertEqual(self.calls[-1][1]["blocks"], pointer["request"]["payload"]["blocks"])
        self.assertEqual(self.store.read("alerts")[self.key], original)
        a.send(a.Store(self.root), key, PIN, lambda _: self.fail("duplicate pointer"))
        self.callback(fixtures.payload("EvFollower", thread_ts=row["parent"]["receipt"]["ts"], ts="102.000001"))
        self.assertEqual(s.drain(self.store), 1)
        self.assertEqual(self.store.read("replies")["events"]["EvFollower"]["alert"], key)
        manager = dict(actor="manager-fixture", answer="Five testers", scope="Synthetic follower only", alert=key,
                       context_digest=row["intent"]["context_digest"], audit=r.snapshot(self.store, key)["audit"],
                       owner=OWNER, interpretation="answer")
        handoff = r.interpret(self.store, self.feed_root, "EvFollower", manager, OWNER, NOW)
        self.assertEqual(r.resume(self.store, self.feed_root, handoff, OWNER, NOW), "queued")
        self.assertEqual(d.load_fixture(self.feed_root, NOW)["decisions"][0], self.feed["decisions"][0])
        self.acks.clear()
        self.assert_parent_answerable()

    def test_pending_follower_leaves_parent_answerable(self):
        self.follower()
        self.assert_parent_answerable()

    def test_sent_unbound_follower_leaves_parent_answerable(self):
        key = self.follower()
        a.send(self.store, key, PIN, self.transport.exchange)
        self.assert_parent_answerable()

    def test_follower_post_in_progress_leaves_parent_answerable(self):
        key = self.follower()
        exchange = self.transport.exchange
        def post(request):
            evidence = exchange(request)
            if request["thread_ts"] is None:
                self.callback(fixtures.payload("EvParent"))
            return evidence
        a.send(self.store, key, PIN, post)
        self.assert_parent_answerable(already_received=True)

    def uncertain_follower(self):
        key = self.follower()
        # Lose only the follower details response, after its own root receipt.
        self.reply = lambda method, result: ((None, 200, {})
            if method == "chat.postMessage" and result["ts"] == "100.000004" else (result, 200, {}))
        row = a.send(self.store, key, PIN, self.transport.exchange)
        self.assertEqual(row["details"]["state"], "uncertain")
        self.reply = None
        return key, row

    def test_uncertain_follower_leaves_parent_answerable(self):
        self.uncertain_follower()
        self.assert_parent_answerable()

    def test_failed_follower_leaves_parent_answerable(self):
        key = self.follower()
        self.reply = lambda method, result: ((dict(ok=False, error="ratelimited"), 429, {"Retry-After": "1"})
            if method == "chat.postMessage" and result["ts"] == "100.000004" else (result, 200, {}))
        row = a.send(self.store, key, PIN, self.transport.exchange)
        self.assertEqual(row["details"]["state"], "failed")
        self.assertEqual(row.get("notices", {}), {})
        a.send(self.store, key, PIN, lambda _: self.fail("terminal failure retried"))
        self.assert_parent_answerable()

    def test_reconciled_unbound_follower_leaves_parent_answerable(self):
        key, row = self.uncertain_follower()
        a.reconcile(self.store, key, "details", self.transport.reconcile(row["details"]["request"]))
        self.assert_parent_answerable()

    def test_unbound_follower_reply_is_never_attributed_to_parent(self):
        key, row = self.uncertain_follower()
        self.callback(fixtures.payload("EvUnbound", thread_ts=row["parent"]["receipt"]["ts"]))
        self.assertFalse(self.transport.active)
        self.assertEqual(self.acks, [])
        self.assertEqual(self.store.read("transport")["events"], {})
        self.assertEqual(self.store.read("replies")["events"], {})

    def test_failed_pointer_leaves_parent_answerable(self):
        key = self.follower()
        self.reply = lambda method, result: ((dict(ok=False, error="ratelimited"), 429, {"Retry-After": "1"})
            if method == "chat.postMessage" and result["ts"] == "100.000005" else (result, 200, {}))
        row = a.send(self.store, key, PIN, self.transport.exchange)
        self.assertEqual(row["details"]["state"], "sent")
        self.assertEqual(row["notices"][d.digest([key, "follow-up", self.key])]["state"], "failed")
        self.transport.bind_alert(key, row)
        a.send(self.store, key, PIN, lambda _: self.fail("failed pointer retried"))
        self.assert_parent_answerable()

    def test_uncertain_pointer_leaves_parent_answerable_and_reconciles_without_reposting(self):
        key = self.follower()
        self.reply = lambda method, result: ((None, 200, {})
            if method == "chat.postMessage" and result["ts"] == "100.000005" else (result, 200, {}))
        row = a.send(self.store, key, PIN, self.transport.exchange)
        slot = d.digest([key, "follow-up", self.key])
        self.assertEqual(row["notices"][slot]["state"], "uncertain")
        self.transport.bind_alert(key, row)
        self.reply = None
        self.assert_parent_answerable()
        a.send(a.Store(self.root), key, PIN, lambda _: self.fail("uncertain pointer retried"))
        a.reconcile(self.store, key, slot, self.transport.reconcile(row["notices"][slot]["request"]))
        a.send(a.Store(self.root), key, PIN, lambda _: self.fail("reconciled pointer reposted"))
        self.assertEqual(a.inspect(self.store, key)["notices"][slot]["state"], "sent")
        self.assertEqual(self.store.read("transport")["routes"]["100.000001"]["alert"], self.key)

    def test_refused_pointer_leaves_parent_answerable(self):
        key, row = self.uncertain_follower()
        a.reconcile(self.store, key, "details", self.transport.reconcile(row["details"]["request"]))
        with self.assertRaises(d.Invalid):
            a.notice(self.store, key, "follow-up", self.key, dict(PIN, generation="wrong"),
                     lambda _: self.fail("refused pointer posted"))
        self.assertEqual(a.inspect(self.store, key).get("notices", {}), {})
        self.assert_parent_answerable()

    def test_old_parent_answer_edit_stays_with_parent_after_follower(self):
        handoff = self.retained_ack()
        feed = d.load_fixture(self.feed_root, NOW)
        child = copy.deepcopy(self.feed["decisions"][0])
        child.update(id="choice-2", follows="choice-1")
        changed = copy.deepcopy(feed)
        changed["revision"] += 1
        changed["decisions"].append(child)
        d.save_fixture(self.feed_root, changed, d.digest(feed), NOW)
        key = a.enqueue(self.store, changed, "choice-2", fixtures.REMOTE, PIN, OWNER, NOW)
        row = a.send(self.store, key, PIN, self.transport.exchange)
        self.transport.bind_alert(key, row)
        self.callback(fixtures.payload("EvPriorEdit", subtype="message_changed",
            message=dict(user=PIN["human"], thread_ts="100.000001",
                         ts="101.000001", text="Ten testers", edited=dict(user=PIN["human"])),
            previous_message=dict(user=PIN["human"]), event_ts="105.000001"))
        self.assertTrue(self.transport.active)
        self.assertEqual(self.store.read("transport")["events"]["EvPriorEdit"]["alert"], self.key)
        self.assertEqual(s.drain(self.store), 1)
        self.assertFalse(m.finish(self.store, self.feed_root, handoff, self.transport, lambda: OWNER, NOW)["work_ready"])
        self.assertEqual(d.load_fixture(self.feed_root, NOW)["decisions"][0], feed["decisions"][0])

    def assert_reconcile_defers_pointer(self, *, held, sessionless):
        key, row = self.uncertain_follower()
        original = copy.deepcopy(self.store.read("alerts")[self.key])
        if sessionless:
            self.owner.close()
        with s.locked(self.store) as journal:
            if sessionless:
                journal["lifecycle"]["session"] = None  # Synthetic sessionless recovery fixture.
            journal["hold"] = "outage-gap" if held else None
            self.store.write("transport", journal)
        posts = copy.deepcopy(self.store.read("transport")["posts"])
        with (tempfile.TemporaryFile(mode="w+") as source,
              tempfile.TemporaryFile(mode="w+") as sink,
              patch("slack_sdk.WebClient", side_effect=lambda **kw: fixtures.WebClient(base_url=self.endpoint, **kw))):
            source.write(json.dumps(dict(binding=PIN, key=key, phase="details")) + "\n")
            source.seek(0)
            result = m.main(["reconcile", "--store", str(self.root), "--live"],
                            stdin=source, stdout=sink, credentials=lambda: ("fixture-bot", "fixture-app"), now=lambda: NOW)
        self.assertEqual(result, dict(reconciled=True))
        slot = d.digest([key, "follow-up", self.key])
        pointer = a.inspect(self.store, key)["notices"][slot]
        self.assertEqual(pointer["state"], "pending")
        self.assertIsNone(pointer["receipt"])
        self.assertNotIn(slot, self.store.read("transport")["posts"])
        self.assertEqual(set(self.store.read("transport")["posts"]), set(posts))
        self.assertEqual(len(self.messages), 4)
        self.assertEqual(self.store.read("transport")["routes"][row["parent"]["receipt"]["ts"]]["alert"], key)
        self.assertEqual(self.store.read("alerts")[self.key], original)
        if sessionless:
            self.owner = s.Session(self.store)
            self.addCleanup(self.owner.release)
            self.owner.update("connected")
        self.review_gap()
        sent = a.send(a.Store(self.root), key, PIN, self.transport.exchange)
        self.assertEqual(sent["notices"][slot]["state"], "sent")
        self.assertEqual(sent["notices"][slot]["request"], pointer["request"])
        a.send(a.Store(self.root), key, PIN, lambda _: self.fail("duplicate deferred pointer"))
        self.assertEqual(len(self.messages), 5)
        self.assertEqual(m.project_status(self.store, NOW, True)["state"], "last-verified")
        self.assertEqual(self.store.read("alerts")[self.key], original)

    def test_cli_reconcile_under_hold_leaves_pointer_pending_until_admitted_send(self):
        self.assert_reconcile_defers_pointer(held=True, sessionless=False)

    def test_cli_reconcile_without_session_leaves_pointer_pending_until_admitted_send(self):
        self.assert_reconcile_defers_pointer(held=False, sessionless=True)

    def test_cli_reconcile_held_and_sessionless_leaves_pointer_pending_until_admitted_send(self):
        self.assert_reconcile_defers_pointer(held=True, sessionless=True)

    def test_pointer_pre_admission_refusal_never_writes_uncertain(self):
        key, row = self.uncertain_follower()
        a.reconcile(self.store, key, "details", self.transport.reconcile(row["details"]["request"]))
        slot = d.digest([key, "follow-up", self.key])
        gate = self.transport.gate
        def hold_after_gate():
            admitted = gate()
            with s.locked(self.store) as journal:
                journal["hold"] = "outage-gap"
                self.store.write("transport", journal)
            return admitted
        writes = []
        write = self.store.write
        def capture(name, value):
            if name == "alerts" and slot in value[key].get("notices", {}):
                writes.append(value[key]["notices"][slot]["state"])
            write(name, value)
        with patch.object(self.transport, "gate", side_effect=hold_after_gate), patch.object(self.store, "write", side_effect=capture):
            row = a.send(self.store, key, PIN, self.transport.exchange)
        self.assertNotIn("uncertain", writes)
        self.assertEqual(row["notices"][slot]["state"], "pending")
        self.assertNotIn(slot, self.store.read("transport")["posts"])
        self.assertEqual(len(self.messages), 4)
        self.review_gap()
        self.assertEqual(a.send(self.store, key, PIN, self.transport.exchange)["notices"][slot]["state"], "sent")
        a.send(self.store, key, PIN, lambda _: self.fail("duplicate pointer"))
        self.assertEqual(len(self.messages), 5)

    def test_pointer_message_mention_contains_only_documented_fields(self):
        key = self.follower()
        row = a.send(self.store, key, PIN, self.transport.exchange)
        mention = self.calls[-1][1]["blocks"][0]["elements"][0]["elements"][1]
        self.assertEqual(mention, dict(type="message_mention", channel_id="CTEST",
                                      message_ts=row["parent"]["receipt"]["ts"]))

    def test_cli_reconcile_binds_own_thread_posts_pointer_and_cannot_repoint_parent(self):
        key, row = self.uncertain_follower()
        original = copy.deepcopy(self.store.read("alerts")[self.key])
        with (tempfile.TemporaryFile(mode="w+") as source,
              tempfile.TemporaryFile(mode="w+") as sink,
              patch("slack_sdk.WebClient", side_effect=lambda **kw: fixtures.WebClient(base_url=self.endpoint, **kw))):
            source.write(json.dumps(dict(binding=PIN, key=key, phase="details")) + "\n")
            source.seek(0)
            result = m.main(["reconcile", "--store", str(self.root), "--live"],
                            stdin=source, stdout=sink, credentials=lambda: ("fixture-bot", "fixture-app"), now=lambda: NOW)
        self.assertEqual(result, dict(reconciled=True))
        pointer = a.inspect(self.store, key)["notices"][d.digest([key, "follow-up", self.key])]
        self.assertEqual(pointer["receipt"]["thread_ts"], "100.000001")
        self.transport.bind_alert(self.key, original)  # Old reconciliation cannot change the follower route.
        self.callback(fixtures.payload("EvReconciledFollower", thread_ts=row["parent"]["receipt"]["ts"]))
        self.assertEqual(s.drain(self.store), 1)
        self.assertEqual(self.store.read("replies")["events"]["EvReconciledFollower"]["alert"], key)
        self.assertEqual(self.store.read("alerts")[self.key], original)
        self.acks.clear()
        self.assert_parent_answerable()

    def test_route_binding_refuses_another_alert_on_an_existing_thread(self):
        key = self.follower()
        row = a.send(self.store, key, PIN, self.transport.exchange)
        before = self.store.read("transport")
        collision = copy.deepcopy(row)
        collision["parent"]["receipt"]["ts"] = "100.000001"
        with self.assertRaises(d.Invalid):
            self.transport.bind_alert(key, collision)
        self.assertEqual(self.store.read("transport"), before)
        self.assert_parent_answerable()

    def test_fallback_surfaces_without_mutating_legacy_alerts(self):
        import decision_feed as f
        key = self.follower(parent=False)
        before = [(self.root / (name + ".json")).read_bytes() for name in ("alerts", "replies")]
        with patch.object(s.Transport, "web", side_effect=AssertionError("projection posted")):
            self.assertEqual(m.project_status(self.store, NOW, True)["new_thread_fallbacks"], 1)
            projected = f.project_slack(self.feed_root, self.root, NOW, True)
        self.assertEqual(projected["status"]["new_thread_fallbacks"], 1)
        rows = {row["id"]: row for row in projected["decisions"]}
        self.assertEqual(rows["choice-1"]["replies"]["new_thread_fallbacks"], 0)
        self.assertEqual(rows["choice-2"]["replies"]["new_thread_fallbacks"], 1)
        snapshot = dict(schema=2, status="valid", feed=d.load_fixture(self.feed_root, NOW), slack_status="valid", slack=projected)
        self.assertIn("new thread fallbacks: 1", f.render(snapshot, NOW, [], {}))
        self.assertEqual([(self.root / (name + ".json")).read_bytes() for name in ("alerts", "replies")], before)
        legacy = copy.deepcopy(projected)
        del legacy["status"]["new_thread_fallbacks"]
        for row in legacy["decisions"]:
            del row["replies"]["new_thread_fallbacks"]
        f.validate_slack(legacy, d.load_fixture(self.feed_root, NOW), NOW)
        for bad in (True, -1, "1", None):
            with self.subTest(bad=bad), self.assertRaises(d.Invalid):
                m.validate_status(dict(projected["status"], new_thread_fallbacks=bad))
            changed = copy.deepcopy(projected)
            changed["decisions"][1]["replies"]["new_thread_fallbacks"] = bad
            with self.subTest(bad=bad), self.assertRaises(d.Invalid):
                f.validate_slack(changed, d.load_fixture(self.feed_root, NOW), NOW)

    def supervised(self, mode="connected", **options):
        self.quiesce()
        manager = m.ManagedListener(self.store, PIN, live=True)
        self.addCleanup(manager.stop)
        with fixture_child(mode, self.endpoint):
            manager.start(**options)
        self.assertEqual(self.line(manager.process), dict(type="connected"))
        return manager

    def test_supervisor_waits_for_real_executor_old_inode_despite_false_stopped_frame(self):
        self.sending()
        manager = self.supervised("hung", ongoing=True)
        self.assertEqual(self.line(manager.process), dict(type="stopped"))
        self.assertIsNone(manager.process.poll())
        case = self.lose_fence()
        before = self.store.read("transport")
        completed, errors = [], []
        def repair():
            try:
                with manager.quiesced() as witness:
                    completed.append(s.recover_missing_fence(self.store, PIN, expected_digest=case,
                        evidence="fixture-loss-reviewed", now=NOW, quiesce_listener=witness))
            except BaseException as error:
                errors.append(error)
        worker = fixtures.threading.Thread(target=repair)
        process = manager.process
        worker.start()
        fixtures.time.sleep(0.2)
        self.assertTrue(worker.is_alive())
        self.assertEqual(self.store.read("transport"), before)
        self.assertFalse((self.root / ".ingress.fence").exists())
        with self.assertRaises(BlockingIOError), m.ManagedListener(self.store, PIN).quiesced():
            self.fail("unowned callback admitted")
        process.kill()  # Retained handle only, never the false stopped frame or a journal PID.
        worker.join(5)
        self.assertFalse(worker.is_alive())
        self.assertEqual(errors, [])
        self.assertEqual(completed[0]["state"], "held")
        self.assertEqual(process.returncode, -fixtures.signal.SIGKILL)
        self.assertEqual(self.store.read("transport")["posts"], before["posts"])

    def test_supervisor_finite_graceful_eof_hung_close_and_expiring_scoped_hook(self):
        manager = self.supervised(seconds=0.2)
        process = manager.process
        process.wait(5)
        self.assertEqual(process.returncode, 0)
        manager.stop()
        with manager.quiesced() as expired:
            expired()
            with self.assertRaises(d.Invalid):
                manager.start()
        with manager.quiesced():
            with self.assertRaises(d.Invalid):
                expired()
        manager = self.supervised("hung", ongoing=True)
        self.assertEqual(self.line(manager.process), dict(type="stopped"))
        process = manager.process
        started = fixtures.time.monotonic()
        manager.stop()
        self.assertIsNotNone(process.returncode)
        self.assertLess(fixtures.time.monotonic() - started, 9.5)
        with manager.quiesced() as witness:
            witness()

    def test_controller_sigkill_orphan_eof_excludes_replacement_until_process_death(self):
        self.sending()
        self.quiesce()
        command = [sys.executable, "-B", "-c", "from test_decision_manager import orphan_controller; import sys; orphan_controller(*sys.argv[1:])",
                   str(self.root), self.endpoint]
        env = dict(os.environ, PYTHONPATH=str(Path(__file__).parent) + os.pathsep + os.environ.get("PYTHONPATH", ""))
        process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env)
        try:
            self.assertEqual(self.line(process), dict(type="owned-child-ready"))
            process.kill()
            process.wait(3)
            replacement = m.ManagedListener(self.store, PIN)
            with self.assertRaises(BlockingIOError), replacement.quiesced():
                self.fail("orphan callback still exists")
            deadline = fixtures.time.monotonic() + 8
            while True:
                try:
                    with replacement.quiesced() as witness:
                        witness()
                    break
                except BlockingIOError:
                    self.assertLess(fixtures.time.monotonic(), deadline)
                    fixtures.time.sleep(0.1)
            self.assertEqual(m.project_status(self.store, NOW, True)["state"], "held")
        finally:
            if process.poll() is None:
                process.kill()
            process.communicate(timeout=3)

    def test_controller_crashes_before_ready_after_renewal_during_stop_and_after_reap(self):
        self.quiesce()
        old = self.store.read("transport")
        env = dict(os.environ, PYTHONPATH=str(Path(__file__).parent) + os.pathsep + os.environ.get("PYTHONPATH", ""))
        for phase in ("before-ready", "renewal", "stop", "reap"):
            with self.subTest(phase=phase):
                command = [sys.executable, "-B", "-c", "from test_decision_manager import controller_boundary; import sys; controller_boundary(*sys.argv[1:])",
                           str(self.root), self.endpoint, phase]
                process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env)
                try:
                    self.assertEqual(self.line(process), dict(type="boundary", phase=phase))
                    if phase != "reap":
                        with self.assertRaises(BlockingIOError), m.ManagedListener(self.store, PIN).quiesced():
                            self.fail("live runtime admitted")
                    process.kill()
                    process.wait(3)
                    deadline = fixtures.time.monotonic() + 8
                    while True:
                        try:
                            with m.ManagedListener(self.store, PIN).quiesced() as witness:
                                witness()
                            break
                        except BlockingIOError:
                            self.assertLess(fixtures.time.monotonic(), deadline)
                            fixtures.time.sleep(0.1)
                finally:
                    if process.poll() is None:
                        process.kill()
                    process.communicate(timeout=3)
        current = self.store.read("transport")
        for key in ("posts", "bridges", "runtime_guard", "routes", "events"):
            self.assertEqual(current[key], old[key])

    def test_supervisor_sigstop_unknown_owner_and_real_forced_kill(self):
        manager = self.supervised("ignore-term", ongoing=True)
        process = manager.process
        os.kill(process.pid, fixtures.signal.SIGSTOP)  # Exact owned fixture handle, not a discovered PID.
        try:
            with self.assertRaises(BlockingIOError), m.ManagedListener(self.store, PIN).quiesced():
                self.fail("stopped process is not dead")
            with (patch.object(process, "wait", side_effect=subprocess.TimeoutExpired("fixture wait", 2)),
                  patch.object(process, "kill", side_effect=OSError("fixture kill failure")), self.assertRaises(OSError)):
                manager.stop()
            self.assertIs(manager.process, process)
            with self.assertRaises(BlockingIOError), m.ManagedListener(self.store, PIN).quiesced():
                self.fail("failed kill admitted recovery")
            with patch.object(process, "terminate", wraps=process.terminate) as terminate, patch.object(process, "kill", wraps=process.kill) as kill:
                manager.stop()
            self.assertEqual(terminate.call_count, 1)
            self.assertEqual(kill.call_count, 1)
            self.assertEqual(process.returncode, -fixtures.signal.SIGKILL)
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(3)
        with m.ManagedListener(self.store, PIN).quiesced() as witness:
            witness()

    def test_supervisor_spawn_handshake_and_wait_failures_keep_exclusion_or_reap(self):
        self.quiesce()
        manager = m.ManagedListener(self.store, PIN, live=True)
        self.addCleanup(manager.stop)
        before = self.store.read("transport")
        for call in ("pipe", "spawn", "write", "read"):
            target = (patch.object(m.os, "pipe", side_effect=OSError("fixture pipe")) if call == "pipe" else
                      patch.object(m.subprocess, "Popen", side_effect=OSError("fixture spawn")) if call == "spawn" else
                      patch.object(m.Stdio, "emit" if call == "write" else "read", side_effect=OSError("fixture frame")))
            with fixture_child("connected", self.endpoint), target, self.assertRaises(OSError):
                manager.start(ongoing=True)
            self.assertIsNone(manager.process)
            with manager.quiesced() as witness:
                witness()
        self.assertEqual(self.store.read("transport")["bridges"], before["bridges"])
        manager = self.supervised(ongoing=True)
        process = manager.process
        with patch.object(process, "wait", side_effect=OSError("fixture wait failure")), self.assertRaises(OSError):
            manager.stop()
        self.assertIs(manager.process, process)
        manager.stop()  # Retry the same handle; no replacement/PID adoption.
        self.assertIsNotNone(process.returncode)

    def test_foreground_local_commands_repair_and_eof_without_live_sdk(self):
        self.quiesce()
        case = self.lose_fence()
        incoming, writer = os.pipe()
        reader, outgoing = os.pipe()
        commands = [dict(binding=PIN), dict(operation="start", seconds=60, ongoing=True),
                    dict(operation="inspect-fence-loss"), dict(operation="recover-missing-fence", case_digest=case,
                    evidence="fixture-loss-reviewed"), dict(operation="status")]
        try:
            with os.fdopen(incoming, "rb") as source, os.fdopen(outgoing, "wb") as sink:
                os.write(writer, b"".join(d.canonical(c) + b"\n" for c in commands))
                os.close(writer)
                writer = None
                with patch.object(s.Transport, "web", side_effect=AssertionError("SDK on local operation")):
                    result = m.main(["supervise-listener", "--store", str(self.root)], stdin=source, stdout=sink, now=lambda: NOW)
                self.assertEqual(result["state"], "held")
            frames = [json.loads(line)["result"] for line in os.read(reader, 16384).splitlines()]
            self.assertEqual(frames[0], dict(state="held"))
            self.assertEqual(frames[1]["case_digest"], case)
            self.assertEqual(frames[2]["loss_id"], case)
            self.assertEqual(frames[3]["state"], "off")
        finally:
            if writer is not None:
                os.close(writer)
            os.close(reader)

    def test_dedicated_exec_rejects_wrong_root_and_inherited_descriptor_before_ready(self):
        self.quiesce()
        before = self.store.read("transport")
        for wrong in ("root", "descriptor"):
            with self.subTest(wrong=wrong):
                guard = os.open(self.root / (".transport.lock" if wrong == "descriptor" else ".listener.runtime.lock"), os.O_RDWR)
                reader, writer = os.pipe()
                try:
                    root = self.root / "not-initialized" if wrong == "root" else self.root
                    command = [sys.executable, "-B", str(Path(m.__file__).resolve()), "_listen-child", str(root), str(guard), str(reader)]
                    process = subprocess.Popen(command, pass_fds=(guard, reader), close_fds=True,
                                               stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                    output, error = process.communicate(d.canonical(dict(binding=PIN, seconds=1, ongoing=False)) + b"\n", timeout=7)
                    self.assertEqual((process.returncode, output, error), (1, b"", b""))
                finally:
                    for fd in (guard, reader, writer):
                        os.close(fd)
                with m.ManagedListener(self.store, PIN).quiesced() as witness:
                    witness()
                self.assertEqual(self.store.read("transport"), before)

    def fence_recovery_flow(self, consumed):
        key = self.retained_ack()
        if consumed:
            self.assertTrue(m.finish(self.store, self.feed_root, key, self.transport, lambda: OWNER, NOW)["work_ready"])
        old_alert = self.key
        self.quiesce()
        case = self.lose_fence()
        before = self.store.read("transport")
        replies = (self.root / "replies.json").read_bytes()
        feed = d.load_fixture(self.feed_root, NOW)
        feed_bytes = (self.feed_root / "decisions.fixture.json").read_bytes()
        result = s.recover_missing_fence(self.store, PIN, expected_digest=case, evidence="fixture-loss-reviewed",
                                         now=NOW, quiesce_listener=self.quiesce)
        self.assertEqual(result["state"], "held")
        self.assertEqual((self.feed_root / "decisions.fixture.json").read_bytes(), feed_bytes)
        self.assertEqual(m.project_status(self.store, NOW, True)["state"], "held")
        retired = self.store.read("alerts")
        tampered = copy.deepcopy(retired)
        tampered[old_alert]["cancelled"] = False
        self.store.write("alerts", tampered)  # Corrupt fixture: qualification must positively revalidate retirement.
        calls = len(self.calls)
        with self.assertRaises(d.Invalid):
            self.transport.qualify(fixtures.ui_evidence())
        self.assertEqual(len(self.calls), calls)
        self.store.write("alerts", retired)
        self.owner = s.Session(self.store)
        self.owner.update("connected")
        self.addCleanup(self.owner.release)
        self.review_gap()
        self.assertEqual((self.root / "replies.json").read_bytes(), replies)
        self.assertFalse(m.finish(self.store, self.feed_root, key, self.transport, lambda: OWNER, NOW)["work_ready"])
        self.assertFalse(m.dispatch(self.store, self.feed_root, key, self.transport, None, lambda: OWNER, NOW)["work_ready"])
        for name in ("posts", "bridges", "events", "routes"):
            self.assertEqual(self.store.read("transport")[name], before[name])
        self.assertEqual(d.load_fixture(self.feed_root, NOW), feed)
        with self.assertRaises(d.Invalid):
            a.notice(self.store, old_alert, "clarification", "fixture", PIN, self.transport.exchange)
        self.feed = revision(feed)  # An explicitly new fixture question, never answer-copy/reissue recovery.
        d.save_fixture(self.feed_root, self.feed, d.digest(feed), NOW)
        self.key = self.enqueue(self.feed)
        self.assertNotEqual(self.key, old_alert)
        row = self.sending()
        self.assertNotEqual(row["parent"]["receipt"]["ts"], a.inspect(self.store, old_alert)["parent"]["receipt"]["ts"])
        self.callback(fixtures.payload("EvFresh", thread_ts=row["parent"]["receipt"]["ts"], ts="105.000001", event_ts="105.000001"))
        self.assertEqual(s.drain(self.store), 1)
        manager = dict(actor="manager-fixture", answer="New explicit answer", scope="New synthetic question only", alert=self.key,
                       context_digest=row["intent"]["context_digest"], audit=r.snapshot(self.store, self.key)["audit"],
                       owner=OWNER, interpretation="answer")
        scoped = m.ResolutionStore(self.transport)
        fresh = r.interpret(scoped, self.feed_root, "EvFresh", manager, OWNER, NOW)
        self.assertEqual(r.resume(scoped, self.feed_root, fresh, OWNER, NOW), "queued")
        process = self.start(fresh)
        outbound = self.line(process)
        self.write(process, accepted(outbound))
        self.write(process, assistant(outbound))
        self.assertTrue(self.line(process)["result"]["work_ready"])
        process.wait(5)
        self.assertEqual(process.returncode, 0)
        self.assertFalse(m.finish(self.store, self.feed_root, fresh, self.transport, lambda: OWNER, NOW)["work_ready"])
        self.assertEqual(self.store.read("transport")["bridges"][key], before["bridges"][key])

    def test_fence_recovery_refuses_historical_late_ack_and_allows_new_question(self):
        self.fence_recovery_flow(False)

    def test_fence_recovery_preserves_consumed_permit_and_allows_new_question(self):
        self.fence_recovery_flow(True)

    def test_local_fence_registration_has_no_implicit_runtime_credentials_or_sdk(self):
        self.quiesce()
        case = self.lose_fence()
        incoming, writer = os.pipe()
        reader, outgoing = os.pipe()
        data = dict(binding=PIN, case_digest=case, evidence="fixture-loss-reviewed", stopped=True)
        def forbidden():
            self.fail("local recovery accessed live credentials or native tools")
        try:
            with os.fdopen(incoming, "rb") as source, os.fdopen(outgoing, "wb") as sink:
                for operation, hook in (("inspect-fence-loss", None), ("recover-missing-fence", None),
                                        ("recover-missing-fence", self.quiesce)):
                    os.write(writer, d.canonical(data) + b"\n")
                    args = [operation, "--store", str(self.root)]
                    kwargs = dict(stdin=source, stdout=sink, credentials=forbidden, observe_owner=forbidden, now=lambda: NOW)
                    before = self.store.read("transport")
                    with patch.object(s.Transport, "web", side_effect=AssertionError("SDK accessed")):
                        if operation == "recover-missing-fence" and hook is None:
                            with self.assertRaises(d.Invalid):
                                m.main(args, **kwargs)
                            self.assertEqual(self.store.read("transport"), before)
                            continue
                        self.assertEqual(m.main(args, quiesce_listener=hook, **kwargs)["state"], "held")
                    self.assertEqual(json.loads(os.read(reader, 4096))["type"], "result")
        finally:
            os.close(writer)
            os.close(reader)

    def retained_ack(self):
        key = self.queue()
        process = self.start(key, "acceptance-crash")
        outbound = self.line(process)
        self.write(process, accepted(outbound))
        process.wait(5)
        self.assertEqual(process.returncode, 34)
        m.retain_evidence(self.store, key, assistant(outbound))
        return key

    def test_sigkill_connected_renewal_denies_cli_status_and_finish_before_restart(self):
        key = self.retained_ack()
        listener, _ = self.start_listener()
        self.review_gap()
        self.assertEqual(m.project_status(self.store, NOW, True)["state"], "last-verified")
        saved = self.store.read("transport")
        started = fixtures.time.monotonic()
        listener.kill()
        listener.join(3)
        self.assertEqual(listener.exitcode, -fixtures.signal.SIGKILL)
        with self.assertRaises(d.Invalid):
            self.transport.gate()
        self.assertEqual(m.project_status(a.Store(self.root), NOW, True)["state"], "held")
        output = io.StringIO()
        m.main(["status", "--store", str(self.root), "--live"], now=lambda: NOW, stdout=output)
        self.assertEqual(json.loads(output.getvalue())["state"], "held")
        process = self.start(key)
        stdout, _ = process.communicate(timeout=3)
        self.assertNotEqual(process.returncode, 0)
        self.assertEqual(stdout, b"")
        with self.assertRaises(d.Invalid):
            m.finish(a.Store(self.root), self.feed_root, key, self.transport, lambda: OWNER, NOW)
        with self.assertRaises(d.Invalid), m.ResolutionStore(self.transport).lock():
            self.fail("dead listener admitted canonical mutation")
        self.assertLess(fixtures.time.monotonic() - started, 5)
        journal = a.Store(self.root).read("transport")
        self.assertEqual(journal["bridges"], saved["bridges"])
        self.assertEqual(journal["posts"], saved["posts"])
        self.assertFalse(journal["bridges"][key]["unlocked"])
        replacement, _ = self.start_listener()
        self.assertTrue(replacement.is_alive())
        with self.assertRaises(d.Invalid):
            self.transport.gate()
        self.review_gap()
        self.assertTrue(m.finish(self.store, self.feed_root, key, self.transport, lambda: OWNER, NOW)["work_ready"])
        self.assertFalse(m.finish(self.store, self.feed_root, key, self.transport, lambda: OWNER, NOW)["work_ready"])

    def test_epoch_change_after_admission_prevents_durable_permit(self):
        key = self.retained_ack()
        gate = self.transport.gate
        def boundary():
            admitted = gate()
            self.owner.update("gap")
            self.owner.update("connected")
            self.review_gap()
            return admitted
        with patch.object(self.transport, "gate", side_effect=boundary), self.assertRaises(d.Invalid):
            m.finish(self.store, self.feed_root, key, self.transport, lambda: OWNER, NOW)
        self.assertFalse(a.Store(self.root).read("transport")["bridges"][key]["unlocked"])

    def test_actual_listener_killed_during_durable_permit_write_cannot_emit_work(self):
        key = self.retained_ack()
        listener, _ = self.start_listener()
        self.review_gap()
        write = self.store.write
        def boundary(name, value):
            write(name, value)
            if name == "transport" and value["bridges"][key]["unlocked"]:
                listener.kill()
                listener.join(3)
                self.assertEqual(listener.exitcode, -fixtures.signal.SIGKILL)
        with patch.object(self.store, "write", side_effect=boundary), self.assertRaises(d.Invalid):
            m.finish(self.store, self.feed_root, key, self.transport, lambda: OWNER, NOW)
        self.assertTrue(a.Store(self.root).read("transport")["bridges"][key]["unlocked"])
        self.start_listener()
        self.review_gap()
        self.assertFalse(m.finish(self.store, self.feed_root, key, self.transport, lambda: OWNER, NOW)["work_ready"])

    def test_changed_durable_epoch_after_permit_write_is_reloaded_without_reissue(self):
        key = self.retained_ack()
        write = self.store.write
        def boundary(name, value):
            write(name, value)
            if name == "transport" and value["bridges"][key]["unlocked"]:
                changed = copy.deepcopy(value)  # Storage-boundary fault, not an unlocked production writer.
                changed["lifecycle"]["epoch"] += 1
                write(name, changed)
        with patch.object(self.store, "write", side_effect=boundary), self.assertRaises(d.Invalid):
            m.finish(self.store, self.feed_root, key, self.transport, lambda: OWNER, NOW)
        self.assertTrue(a.Store(self.root).read("transport")["bridges"][key]["unlocked"])
        self.assertFalse(m.finish(self.store, self.feed_root, key, self.transport, lambda: OWNER, NOW)["work_ready"])

    def notice_interleaving(self, change):
        key = self.queue()
        def reply(method, result):
            if method == "chat.postMessage":
                if change == "delete":
                    self.callback(fixtures.payload("EvNotice", subtype="message_deleted", deleted_ts="101.000001", event_ts="102.000001"))
                elif change == "edit":
                    self.callback(fixtures.payload("EvNotice", subtype="message_changed", message=dict(
                        user=PIN["human"], ts="101.000001", thread_ts="100.000001", text="Reconsider", edited=dict(user=PIN["human"])),
                        event_ts="102.000001"))
                elif change == "outage":
                    s.hold(self.store, "outage-gap")
                elif change == "feed":
                    feed = d.load_fixture(self.feed_root, NOW)
                    d.save_fixture(self.feed_root, revision(feed), d.digest(feed), NOW)
            return result, 200, {}
        self.reply = reply
        env = dict(os.environ, PYTHONPATH=str(Path(__file__).parent) + os.pathsep + os.environ.get("PYTHONPATH", ""))
        command = [sys.executable, "-B", "-c", "import sys; from test_decision_manager import cli_child; cli_child(*sys.argv[1:])",
                   str(self.root), str(self.feed_root), self.endpoint]
        with subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env) as process:
            self.write(process, dict(binding=PIN, key=key))
            outbound = self.line(process)
            self.write(process, accepted(outbound))
            self.write(process, assistant(outbound))
            output, errors = process.communicate(timeout=10)
        results = [json.loads(line)["result"] for line in output.splitlines()]
        self.assertEqual(len(self.messages), 3)  # Actual SDK acknowledgment notice occurred.
        self.assertEqual(any(value["work_ready"] for value in results), change is None, errors)
        with s.locked(a.Store(self.root)) as journal:
            self.assertEqual(journal["bridges"][key]["unlocked"], change is None)

    def test_cli_notice_edit_precedes_permit(self):
        self.notice_interleaving("edit")

    def test_cli_notice_delete_precedes_permit(self):
        self.notice_interleaving("delete")

    def test_cli_notice_outage_precedes_permit(self):
        self.notice_interleaving("outage")

    def test_cli_notice_new_feed_precedes_permit(self):
        self.notice_interleaving("feed")

    def test_cli_notice_success_precedes_one_permit(self):
        self.notice_interleaving(None)

    def backlog_cannot_resolve(self, operation):
        feed = copy.deepcopy(self.feed)
        feed["revision"] += 1
        feed["decisions"].append(dict(copy.deepcopy(feed["decisions"][0]), id="choice-2"))
        d.save_fixture(self.feed_root, feed, d.digest(self.feed), NOW)
        self.sending()
        other = a.enqueue(self.store, feed, "choice-2", "https://dashboard.example.test", PIN, OWNER, NOW)
        row = a.send(self.store, other, PIN, self.transport.exchange)
        self.transport.bind_alert(other, row)
        self.callback()
        s.drain(self.store)
        manager = dict(actor="manager-fixture", answer="Five testers", scope="Synthetic pilot only", alert=self.key,
                       context_digest=a.inspect(self.store, self.key)["intent"]["context_digest"],
                       audit=r.snapshot(self.store, self.key)["audit"], owner=OWNER, interpretation="answer")
        key = r.interpret(self.store, self.feed_root, "Ev001", manager, OWNER, NOW) if operation == "resume" else None
        for index in range(10):
            self.callback(fixtures.payload("EvOther" + str(index), thread_ts=row["parent"]["receipt"]["ts"],
                          ts=f"110.{index:06d}", event_ts=f"110.{index:06d}"))
        self.callback(fixtures.payload("EvBacklogDelete", subtype="message_deleted", deleted_ts="101.000001", event_ts="120.000001"))
        incoming, writer = os.pipe()
        reader, outgoing = os.pipe()
        try:
            with os.fdopen(incoming, "rb") as source, os.fdopen(outgoing, "wb") as sink:
                os.write(writer, d.canonical(dict(binding=PIN, key=key, event_id="Ev001", manager=manager)) + b"\n")
                with self.assertRaises(d.Invalid):
                    m.main([operation, "--store", str(self.root), "--feed", str(self.feed_root), "--live"],
                           stdin=source, stdout=sink, observe_owner=lambda: OWNER, now=lambda: NOW)
            self.assertEqual(d.load_fixture(self.feed_root, NOW), feed)
            self.assertEqual(s.drain(a.Store(self.root)), 1)
            self.assertEqual(r.snapshot(self.store, self.key)["replies"]["Ev001"]["state"], "needs-clarification")
            self.assertEqual(len(self.store.read("replies")["intents"]), int(operation == "resume"))
            manager.update(audit=r.snapshot(self.store, self.key)["audit"], interpretation="clarified-answer")
            clarified = r.interpret(m.ResolutionStore(self.transport), self.feed_root, "Ev001", manager, OWNER, NOW)
            self.assertEqual(r.resume(m.ResolutionStore(self.transport), self.feed_root, clarified, OWNER, NOW), "queued")
            if key is not None:
                self.assertEqual(clarified, key)
            self.assertEqual(len(d.load_fixture(self.feed_root, NOW)["decisions"][0]["revisions"]), 2)
        finally:
            os.close(writer)
            os.close(reader)

    def test_resume_holds_target_delete_behind_ten_unrelated_events(self):
        self.backlog_cannot_resolve("resume")

    def test_interpret_holds_target_delete_behind_ten_unrelated_events(self):
        self.backlog_cannot_resolve("interpret")

    def test_delete_during_final_durable_unlock_write_cannot_release_work(self):
        key = self.queue()
        process = self.start(key, "acceptance-crash")
        outbound = self.line(process)
        self.write(process, accepted(outbound))
        process.wait(5)
        self.assertEqual(process.returncode, 34)
        m.retain_evidence(self.store, key, assistant(outbound))
        write = self.store.write
        def checkpoint(name, value):
            write(name, value)
            if name == "transport" and value["bridges"][key]["unlocked"]:
                contender = multiprocessing.get_context("spawn").Process(target=contended_delete, args=(str(self.root),))
                contender.start()
                contender.join(5)
                self.assertEqual(contender.exitcode, 0)
        with patch.object(self.store, "write", side_effect=checkpoint), self.assertRaises(d.Invalid):
            m.finish(self.store, self.feed_root, key, self.transport, lambda: OWNER, NOW)
        with self.assertRaises(d.Invalid):
            m.finish(a.Store(self.root), self.feed_root, key, self.transport, lambda: OWNER, NOW)
        with s.locked(self.store) as journal:
            self.assertTrue(journal["bridges"][key]["unlocked"])
            self.assertGreater(s.ingress_count(self.store), journal["ingress"])

    def test_production_clock_advances_and_cli_has_no_frozen_time_override(self):
        instants = [m.dt.datetime(2026, 9, 12, hour, tzinfo=m.dt.timezone.utc) for hour in (12, 13, 14)]
        incoming, writer = os.pipe()
        reader, outgoing = os.pipe()
        seen = []
        with os.fdopen(incoming, "rb") as source, os.fdopen(outgoing, "wb") as sink:
            os.write(writer, d.canonical(dict(binding=PIN)) + b"\n")
            def listen(manager, **kwargs):
                seen.extend([m.utc_now(), m.utc_now()])  # The same clock callable used by the real child.
                manager.process = SimpleNamespace(poll=lambda: 0, returncode=0)
            with patch.object(m.dt, "datetime") as clock, patch.object(s.Transport, "gate", return_value={}), \
                    patch.object(m.ManagedListener, "start", autospec=True, side_effect=listen), \
                    patch.object(m.ManagedListener, "stop"), \
                    patch.object(m, "project_status", return_value={}):
                clock.now.side_effect = instants
                m.main(["listen", "--store", str(self.root), "--live", "--ongoing"], stdin=source, stdout=sink)
        os.close(writer)
        os.close(reader)
        self.assertEqual(seen, ["2026-09-12T12:00:00Z", "2026-09-12T13:00:00Z"])
        with patch("sys.stderr", io.StringIO()), self.assertRaises(SystemExit) as rejected:
            m.main(["status", "--store", str(self.root), "--now", NOW])
        self.assertEqual(rejected.exception.code, 2)

    def lost_ingress_cannot_unlock(self, contend):
        key = self.queue()
        process = self.start(key, "acceptance-crash")
        outbound = self.line(process)
        self.write(process, accepted(outbound))
        process.wait(5)
        self.assertEqual(process.returncode, 34)
        edit = fixtures.payload("EvLost", subtype="message_deleted", deleted_ts="101.000001", event_ts="102.000001")
        if contend:
            ctx = multiprocessing.get_context("spawn")
            ready, release = ctx.Event(), ctx.Event()
            writer = ctx.Process(target=fixtures.hold_transport, args=(str(self.root), ready, release))
            writer.start()
            self.assertTrue(ready.wait(5))
            try:
                self.callback(edit)
            finally:
                release.set()
                writer.join(5)
            self.assertEqual(writer.exitcode, 0)
        else:
            with patch.object(a.os, "fsync", side_effect=OSError("fixture fsync failure")):
                self.callback(edit)
        m.retain_evidence(a.Store(self.root), key, assistant(outbound))
        with self.assertRaises(d.Invalid):
            m.finish(a.Store(self.root), self.feed_root, key, self.transport, lambda: OWNER, NOW)
        self.transport.active = True  # New authenticated callback session receives Slack's retained retry.
        self.callback(edit)
        self.assertEqual(s.drain(a.Store(self.root)), 1)
        with s.locked(self.store) as journal:
            review = dict(watermark=journal["watermark"], binding=d.digest(PIN), evidence="fixture-gap-reviewed",
                          ingress=s.ingress_count(self.store), session=journal["lifecycle"]["session"]["id"],
                          epoch=journal["lifecycle"]["epoch"])
            self.assertIsNotNone(journal["hold"])
        with self.assertRaises(d.Invalid):
            self.transport.qualify(fixtures.ui_evidence(), dict(review, ingress=review["ingress"] - 1))
        self.transport.qualify(fixtures.ui_evidence(), review)
        self.assertFalse(m.finish(a.Store(self.root), self.feed_root, key, self.transport, lambda: OWNER, NOW)["work_ready"])

    def test_lost_delete_transport_process_contention_durably_fences_restart(self):
        self.lost_ingress_cannot_unlock(True)

    def test_lost_delete_fsync_failure_durably_fences_restart(self):
        self.lost_ingress_cannot_unlock(False)

    def test_expired_binding_allows_registered_local_drain_not_send_or_work(self):
        self.sending()
        self.callback()
        s.hold(self.store, "outage-gap")
        incoming, writer = os.pipe()
        reader, outgoing = os.pipe()
        with os.fdopen(incoming, "rb") as source, os.fdopen(outgoing, "wb") as sink:
            os.write(writer, d.canonical(dict(binding=PIN)) + b"\n")
            value = m.main(["drain", "--store", str(self.root)], now=lambda: "2026-09-12T13:00:00Z",
                           credentials=lambda: self.fail("local drain touched credentials"), stdin=source, stdout=sink)
            self.assertEqual(value, {"drained": 1})
            self.assertEqual(json.loads(os.read(reader, 4096))["result"], value)
        os.close(writer)
        os.close(reader)
        self.transport.now = lambda: "2026-09-12T13:00:00Z"
        with self.assertRaises(d.Invalid):
            self.transport.gate()
        self.assertEqual(len(r.snapshot(a.Store(self.root), self.key)["replies"]), 1)

    def test_fresh_off_process_does_not_import_sdk_and_cli_registration_executes(self):
        script = "import sys,decision_manager as m; m.main(['status','--store','/absent']); assert 'slack_sdk' not in sys.modules"
        env = dict(os.environ, PYTHONPATH=str(Path(__file__).parent) + os.pathsep + os.environ.get("PYTHONPATH", ""))
        result = subprocess.run([sys.executable, "-B", "-c", script], capture_output=True, env=env, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)["state"], "off")
        incoming, writer = os.pipe()
        reader, outgoing = os.pipe()
        with os.fdopen(incoming, "rb") as source, os.fdopen(outgoing, "wb") as sink:
            os.write(writer, d.canonical(dict(binding=PIN, ui_evidence=fixtures.ui_evidence())) + b"\n")
            with patch("slack_sdk.WebClient", side_effect=lambda **kw: fixtures.WebClient(base_url=self.endpoint, **kw)):
                value = m.main(["qualify", "--store", str(self.root), "--live"], now=lambda: NOW,
                               credentials=lambda: ("fixture-bot", "fixture-app"), stdin=source, stdout=sink)
            self.assertEqual(value, PIN)
            self.assertEqual(json.loads(os.read(reader, 4096))["result"], PIN)
        os.close(writer)
        os.close(reader)

    def queue(self):
        self.sending()
        self.callback()
        self.assertEqual(s.drain(self.store), 1)
        manager = dict(actor="manager-fixture", answer="Five testers", scope="Synthetic pilot only", alert=self.key,
                       context_digest=a.inspect(self.store, self.key)["intent"]["context_digest"],
                       audit=r.snapshot(self.store, self.key)["audit"], owner=OWNER, interpretation="answer")
        key = r.interpret(self.store, self.feed_root, "Ev001", manager, OWNER, NOW)
        self.assertEqual(r.resume(self.store, self.feed_root, key, OWNER, NOW), "queued")
        return key

    def start(self, key, mode="normal"):
        command = [sys.executable, "-B", "-c", "import sys; from test_decision_manager import bridge_child; bridge_child(*sys.argv[1:])",
                   str(self.root), str(self.feed_root), key, mode]
        env = dict(os.environ, PYTHONPATH=str(Path(__file__).parent) + os.pathsep + os.environ.get("PYTHONPATH", ""))
        process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env)
        def cleanup():
            if process.poll() is None:
                process.terminate()
            process.communicate(timeout=10)
        self.addCleanup(cleanup)
        return process

    def line(self, process):
        self.assertTrue(select.select([process.stdout], [], [], 10)[0], "manager pipe timed out")
        line = process.stdout.readline()
        self.assertTrue(line, "manager pipe closed before its response")
        return json.loads(line)

    def write(self, process, value):
        process.stdin.write(d.canonical(value) + b"\n")
        process.stdin.flush()

    def test_real_stdio_exact_tool_and_assistant_ack_one_unlock_after_restart(self):
        key = self.queue()
        process = self.start(key)
        outbound = self.line(process)
        self.assertEqual(outbound["type"], "acknowledgment-only")
        self.assertIn("do not execute", outbound["instruction"])
        with s.locked(self.store) as journal:
            self.assertIsNone(journal["bridges"][key]["submission_id"])
        self.write(process, accepted(outbound))
        self.write(process, assistant(outbound))
        self.assertTrue(self.line(process)["result"]["work_ready"])
        process.wait(5)
        self.assertEqual(process.returncode, 0)
        with s.locked(self.store) as journal:
            bridge = journal["bridges"][key]
            self.assertEqual(bridge["submission_id"], "actual-fixture-submission-1")
            self.assertEqual(bridge["ack"], outbound["expected_ack"])
            self.assertNotEqual(bridge["receipt"]["receipt_id"], bridge["submission_id"])
        again = m.dispatch(a.Store(self.root), self.feed_root, key, self.transport, None, lambda: OWNER, NOW)
        self.assertFalse(again["work_ready"])
        self.assertEqual(r.snapshot(self.store, self.key)["intents"][key]["state"], "agent-acknowledged")
        self.assertEqual(len(self.messages), 2)  # Native ACK is not an implicit Slack/network post.

    def test_edit_during_ack_drains_and_fences_work_but_retains_actual_ack(self):
        key = self.queue()
        process = self.start(key)
        outbound = self.line(process)
        self.callback(fixtures.payload("Ev002", subtype="message_changed", event_ts="102.000001", message=dict(
            user=PIN["human"], ts="101.000001", thread_ts="100.000001", text="Ten testers", edited={"user": PIN["human"]})))
        self.write(process, accepted(outbound))
        self.write(process, assistant(outbound))
        result = self.line(process)["result"]
        self.assertFalse(result["work_ready"])
        self.assertEqual(result["watermark"], 2)
        process.wait(5)
        self.assertEqual(process.returncode, 0)
        self.assertEqual(len(r.snapshot(self.store, self.key)["replies"]), 2)
        with s.locked(self.store) as journal:
            self.assertTrue(journal["events"]["Ev002"]["drained"])
            self.assertIsNotNone(journal["bridges"][key]["ack"])

    def test_request_crash_has_no_resend_or_invented_acceptance(self):
        key = self.queue()
        process = self.start(key, "request-crash")
        output, _ = process.communicate(timeout=10)
        self.assertEqual((process.returncode, output), (33, b""))
        result = m.dispatch(a.Store(self.root), self.feed_root, key, self.transport, None, lambda: OWNER, NOW)
        self.assertFalse(result["work_ready"])
        intent = r.snapshot(self.store, self.key)["intents"][key]
        self.assertEqual(intent["delivery"], "uncertain")
        self.assertIsNone(intent["receipt"])

    def test_acceptance_crash_recovers_exact_late_assistant_without_resubmission(self):
        key = self.queue()
        process = self.start(key, "acceptance-crash")
        outbound = self.line(process)
        self.write(process, accepted(outbound))
        process.wait(5)
        self.assertEqual(process.returncode, 34)
        with s.locked(self.store) as journal:
            self.assertIsNone(journal["bridges"][key]["ack"])
        m.retain_evidence(a.Store(self.root), key, assistant(outbound))
        result = m.finish(a.Store(self.root), self.feed_root, key, self.transport, lambda: OWNER, NOW)
        self.assertTrue(result["work_ready"])
        m.retain_evidence(a.Store(self.root), key, assistant(outbound))
        self.assertFalse(m.finish(self.store, self.feed_root, key, self.transport, lambda: OWNER, NOW)["work_ready"])

    def test_stdio_eof_and_timeout_do_not_invent_ack(self):
        key = self.queue()
        process = self.start(key, "timeout")
        self.line(process)
        process.wait(5)
        self.assertNotEqual(process.returncode, 0)
        self.assertEqual(r.snapshot(self.store, self.key)["intents"][key]["delivery"], "uncertain")
        with s.locked(self.store) as journal:
            self.assertIsNone(journal["bridges"][key]["submission_id"])
        read, write = os.pipe()
        with os.fdopen(read, "rb") as source:
            os.close(write)
            channel = m.Stdio(source)
            with self.assertRaises(d.Invalid):
                channel.read()

    def test_fake_wrong_agent_wrong_submission_and_generic_text_are_not_ack(self):
        key = self.queue()
        process = self.start(key, "acceptance-crash")
        outbound = self.line(process)
        self.write(process, accepted(outbound))
        process.wait(5)
        self.assertEqual(process.returncode, 34)
        good = assistant(outbound)
        for wrong in (dict(good, agent="other-agent"), dict(good, submission_id="stale-submission"),
                      dict(good, text="done"), dict(good, text=json.dumps({"example": outbound["expected_ack"]})),
                      dict(good, text=json.dumps(dict(outbound["expected_ack"], allocation="different")))):
            with self.assertRaises((d.Invalid, ValueError)):
                m.retain_evidence(self.store, key, wrong)
        self.assertFalse(m.finish(self.store, self.feed_root, key, self.transport, lambda: OWNER, NOW)["work_ready"])
        self.assertEqual(r.snapshot(self.store, self.key)["intents"][key]["state"], "delivered")

    def test_current_allocation_not_display_status_and_preexisting_ingress_fence(self):
        key = self.queue()
        for owner in (dict(OWNER, running=False), dict(OWNER, agent="unrelated"), dict(OWNER, allocation="reallocated")):
            result = m.dispatch(self.store, self.feed_root, key, self.transport, None, lambda: owner, NOW)
            self.assertFalse(result["work_ready"])
        self.callback(fixtures.payload("Ev002", ts="102.000001", event_ts="102.000001", text="Reconsider"))
        result = m.dispatch(self.store, self.feed_root, key, self.transport, None, lambda: OWNER, NOW)
        self.assertFalse(result["work_ready"])
        with s.locked(self.store) as journal:
            self.assertEqual(journal["bridges"], {})
        self.assertEqual(r.snapshot(self.store, self.key)["intents"][key]["delivery"], "pending")

    def test_post_ack_newer_feed_stopped_cancelled_and_outage_hold_work(self):
        key = self.queue()
        process = self.start(key, "acceptance-crash")
        outbound = self.line(process)
        self.write(process, accepted(outbound))
        process.wait(5)
        m.retain_evidence(self.store, key, assistant(outbound))
        self.assertFalse(m.finish(self.store, self.feed_root, key, self.transport, lambda: dict(OWNER, running=False), NOW)["work_ready"])
        saved = d.load_fixture(self.feed_root, NOW)
        newer = revision(saved)
        worker = multiprocessing.get_context("spawn").Process(target=d.save_fixture, args=(str(self.feed_root), newer, d.digest(saved), NOW))
        worker.start()
        worker.join(5)
        self.assertEqual(worker.exitcode, 0)
        self.assertFalse(m.finish(a.Store(self.root), self.feed_root, key, self.transport, lambda: OWNER, NOW)["work_ready"])
        a.send(self.store, self.key, PIN, lambda _: self.fail("cancel sent"), cancelled=True)
        self.assertFalse(m.finish(self.store, self.feed_root, key, self.transport, lambda: OWNER, NOW)["work_ready"])
        s.hold(self.store, "outage-gap")
        with self.assertRaises(d.Invalid):
            m.finish(self.store, self.feed_root, key, self.transport, lambda: OWNER, NOW)

    def test_off_status_needs_no_files_sdk_credentials_or_tools_and_redacts(self):
        output = io.StringIO()
        with patch.object(s.Transport, "web", side_effect=AssertionError("SDK accessed")):
            value = m.main(["status", "--store", "/does-not-exist"], stdout=output,
                           credentials=lambda: self.fail("credentials"), observe_owner=lambda: self.fail("tools"))
        self.assertEqual(value["state"], "off")
        self.assertFalse(value["enabled"])
        key = self.queue()
        value = m.project_status(self.store, NOW, True)
        self.assertEqual(value["queued"], 1)
        self.assertEqual(m.validate_status(value), value)
        self.assertNotIn("Five testers", json.dumps(value))
        self.assertNotIn(PIN["human"], json.dumps(value))
        with self.assertRaises(d.Invalid):
            m.validate_status(dict(value, answer="Five testers"))
        self.assertEqual(m.project_status(self.store, "2026-09-12T13:00:00Z", True)["state"], "stale")
        (self.root / "transport.json").unlink()
        self.assertEqual(m.project_status(self.store, NOW, True)["state"], "held")

    def test_stdio_allocation_observation_and_frame_bound(self):
        incoming, writer = os.pipe()
        reader, outgoing = os.pipe()
        with os.fdopen(incoming, "rb") as source, os.fdopen(outgoing, "wb") as sink:
            channel = m.Stdio(source, sink)
            raw = d.canonical(dict(type="allocation", source="manager-native-allocation", owner=OWNER)) + b"\n"
            os.write(writer, raw)
            self.assertEqual(channel.owner(), OWNER)
            self.assertEqual(json.loads(os.read(reader, 4096))["type"], "observe-allocation")
            with self.assertRaises(d.Invalid):
                channel.emit(dict(text="x" * 16384))
        os.close(writer)
        os.close(reader)
