import copy
import io
import json
import multiprocessing
import os
from pathlib import Path
import select
import subprocess
import sys
from unittest.mock import patch

import decisions as d
import decision_alerts as a
import decision_replies as r
import decision_manager as m
import slack_transport as s
import test_slack_transport as fixtures
from test_decisions import NOW, revision
from test_decision_alerts import PIN, OWNER


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
            def listen(transport, **kwargs):
                seen.extend([transport.now(), transport.now()])
            with patch.object(m.dt, "datetime") as clock, patch.object(s.Transport, "gate", return_value={}), \
                    patch.object(s.Transport, "listen", autospec=True, side_effect=listen), \
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
