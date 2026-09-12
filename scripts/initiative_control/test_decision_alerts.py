import copy
import json
import multiprocessing
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import decisions as d
import decision_alerts as a
from test_decisions import fixture, revision, NOW

PIN = dict(team="TTEST", channel="CTEST", app="ATEST", bot="BTEST", generation="generation-1", human="UTEST")
OWNER = dict(agent="agent-fixture", allocation="allocation-1", running=True)
REMOTE = "https://dashboard.example.test"


def receipt(request):
    return dict(attempt=request["attempt"], identity=request["identity"], payload_digest=request["payload_digest"],
                thread_ts=request["thread_ts"], ts="100.000002" if request["thread_ts"] else "100.000001")


def send_process(root, key, barrier, crash):
    if barrier is not None:
        barrier.wait(timeout=10)

    def exchange(request):
        fd = os.open(Path(root) / "exchanges", os.O_CREAT | os.O_APPEND | os.O_WRONLY, 0o600)
        with os.fdopen(fd, "a") as stream:
            stream.write(request["attempt"] + "\n")
            stream.flush()
            os.fsync(stream.fileno())
        if crash == "parent" or (crash == "details" and request["thread_ts"]):
            os._exit(23)
        return receipt(request)
    a.send(a.Store(root), key, PIN, exchange)


class SlackFixture(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(dir=Path(tempfile.gettempdir()).resolve())
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / "slack"
        self.feed_root = Path(self.temp.name) / "feed"
        self.root.mkdir(mode=0o700)
        self.feed_root.mkdir(mode=0o700)
        self.store = a.Store(self.root)
        self.store.initialize()
        self.feed = fixture()
        d.save_fixture(self.feed_root, self.feed, None, NOW)
        self.key = self.enqueue(self.feed)

    def enqueue(self, feed, **kwargs):
        args = dict(remote=REMOTE, pinned_identity=PIN, allocation=OWNER, now=NOW)
        args.update(kwargs)
        return a.enqueue(self.store, feed, "choice-1", **args)

    def send(self, exchange=receipt):
        return a.send(self.store, self.key, PIN, exchange)


class AlertTests(SlackFixture):
    def test_feed_updates_reuse_sent_notification_across_restart(self):
        self._feed_update_restart(False)

    def test_feed_updates_cannot_bypass_uncertain_notification(self):
        self._feed_update_restart(True)

    def _feed_update_restart(self, uncertain):
        calls = []
        def exchange(request):
            calls.append(request)
            if uncertain:
                raise TimeoutError()
            return receipt(request)
        original = self.send(exchange)["intent"]
        feed = copy.deepcopy(self.feed)
        for mutation in ("assessment", "other-decision"):
            feed["revision"] += 1
            feed["assessed_at"] = "2026-09-12T12:01:00Z"
            if mutation == "other-decision":
                other = copy.deepcopy(feed["decisions"][0])
                other["id"] = "choice-2"
                feed["decisions"].append(other)
            self.store = a.Store(self.root)
            key = self.enqueue(feed, now="2026-09-12T12:01:00Z")
            self.assertEqual(key, self.key)
            state = a.send(self.store, key, PIN, exchange)
            self.assertEqual(state["intent"], original)
            self.assertEqual(state["parent"]["state"], "uncertain" if uncertain else "sent")
        self.assertEqual(len(calls), 1 if uncertain else 2)
        self.assertEqual(state["parent"]["request"], calls[0])
        for changes in (dict(pinned_identity=dict(PIN, generation="rotated")), dict(allocation=dict(OWNER, allocation="new")), dict(remote="https://changed.example.test")):
            with self.assertRaises(d.Invalid):
                self.enqueue(feed, now="2026-09-12T12:01:00Z", **changes)
        feed["decisions"][0]["revisions"][0]["question"] = "Rewritten same revision?"
        with self.assertRaises(d.Invalid):
            self.enqueue(feed, now="2026-09-12T12:01:00Z")

    def test_previous_fixture_schema_cannot_requeue_old_attempts(self):
        for name in ("alerts", "replies"):
            path = self.root / (name + ".json")
            saved = path.read_bytes()
            previous = json.loads(saved)
            previous["schema"] = 1
            path.write_bytes(d.canonical(previous))
            with self.assertRaises(d.Invalid):
                self.enqueue(self.feed)
            path.write_bytes(saved)

    def test_new_revised_resolved_and_exact_context_payload(self):
        for feed, kind in ((self.feed, "new"), (revision(self.feed), "revised"), (revision(revision(self.feed), "resolved"), "resolved")):
            key = self.enqueue(feed)
            sent = a.send(self.store, key, PIN, receipt)
            self.assertEqual([sent[p]["state"] for p in ("parent", "details")], ["sent", "sent"])
            intent = sent["intent"]
            self.assertEqual(intent["kind"], kind)
            self.assertEqual(intent["feed_digest"], d.digest(feed))
            self.assertEqual(intent["record"], feed["decisions"][0]["revisions"][-1])
            self.assertTrue(intent["payloads"]["parent"]["text"].startswith(kind.upper() + ": "))
            details = sent["details"]["request"]
            self.assertEqual(details["thread_ts"], "100.000001")
            self.assertEqual(details["payload"]["thread_ts"], "100.000001")
            self.assertIn("https://dashboard.example.test/doc-6330009c9cfdad2604f8.html?feed=manager-fixture&decision=choice-1&revision=", details["payload"]["text"])
            self.assertIn("#decision-choice-1", intent["payloads"]["parent"]["text"])
            self.assertIn("raised 2026-09-12T12:00:00Z", details["payload"]["text"])
            self.assertIn("Five or ten testers?", details["payload"]["text"])
            self.assertFalse(details["payload"]["mrkdwn"])
            if kind == "resolved":
                self.assertIn("answer: Five testers", details["payload"]["text"])
        self.assertEqual(len({self.enqueue(f) for f in (self.feed, revision(self.feed), revision(revision(self.feed), "resolved"))}), 3)

    def test_duplicate_enqueue_and_restart_never_send_again(self):
        requests = []
        self.send(lambda request: (requests.append(request), receipt(request))[1])
        self.store = a.Store(self.root)
        self.assertEqual(self.enqueue(self.feed), self.key)
        self.send(lambda _: self.fail("duplicate exchange"))
        self.assertEqual(len(requests), 2)
        self.assertNotEqual(requests[0]["attempt"], requests[1]["attempt"])
        self.assertEqual(requests[0]["attempt"], d.digest([self.key, "parent"]))
        self.assertEqual(requests[1]["attempt"], d.digest([self.key, "details"]))

    def test_real_competing_processes_have_one_exchange_per_phase(self):
        ctx = multiprocessing.get_context("spawn")
        barrier = ctx.Barrier(2)
        workers = [ctx.Process(target=send_process, args=(str(self.root), self.key, barrier, None)) for _ in range(2)]
        for worker in workers:
            worker.start()
        for worker in workers:
            worker.join(15)
            self.assertFalse(worker.is_alive())
            self.assertEqual(worker.exitcode, 0)
        self.assertEqual(len((self.root / "exchanges").read_text().splitlines()), 2)
        self.assertEqual(a.inspect(self.store, self.key)["details"]["state"], "sent")

    def test_actual_process_death_after_parent_intent_holds_restart(self):
        self._crash("parent", "uncertain", "pending", 1)

    def test_actual_process_death_after_parent_receipt_holds_details(self):
        self._crash("details", "sent", "uncertain", 2)

    def _crash(self, phase, parent, details, count):
        ctx = multiprocessing.get_context("spawn")
        worker = ctx.Process(target=send_process, args=(str(self.root), self.key, None, phase))
        worker.start()
        worker.join(15)
        self.assertEqual(worker.exitcode, 23)
        state = a.send(a.Store(self.root), self.key, PIN, lambda _: self.fail("blind retry"))
        self.assertEqual([state[p]["state"] for p in ("parent", "details")], [parent, details])
        self.assertEqual(len((self.root / "exchanges").read_text().splitlines()), count)
        evidence = receipt(state[phase]["request"])
        a.reconcile(self.store, self.key, phase, evidence)
        final = self.send()
        self.assertEqual([final[p]["state"] for p in ("parent", "details")], ["sent", "sent"])

    def test_timeout_partial_thread_and_exact_retained_evidence(self):
        calls = []
        def exchange(request):
            calls.append(request)
            if request["thread_ts"]:
                raise TimeoutError("xoxb-fixture-canary")
            return receipt(request)
        sent = self.send(exchange)
        self.assertEqual([sent[p]["state"] for p in ("parent", "details")], ["sent", "uncertain"])
        self.send(lambda _: self.fail("blind retry"))
        good = receipt(calls[-1])
        for field, value in (("attempt", "other"), ("thread_ts", "101.000001"), ("payload_digest", "wrong"), ("ts", "bad"), ("identity", dict(PIN, generation="rotated"))):
            with self.subTest(field=field), self.assertRaises(d.Invalid):
                a.reconcile(self.store, self.key, "details", dict(good, **{field: value}))
        a.reconcile(self.store, self.key, "details", good)
        a.reconcile(self.store, self.key, "details", good)
        self.assertEqual(a.inspect(self.store, self.key)["details"]["state"], "sent")
        self.assertNotIn("canary", (self.root / "alerts.json").read_text())

    def test_identity_rotation_cancel_and_definitive_rejection(self):
        held = a.send(self.store, self.key, dict(PIN, generation="rotated"), lambda _: self.fail("rotated"))
        self.assertEqual((held["parent"]["state"], held["reason"]), ("pending", "identity-changed"))
        cancelled = a.send(self.store, self.key, PIN, lambda _: self.fail("cancelled"), cancelled=True)
        self.assertEqual((cancelled["parent"]["state"], cancelled["reason"]), ("pending", "cancelled"))
        self.send(lambda _: self.fail("cancel persists"))
        key = self.enqueue(revision(self.feed))
        def reject(_):
            raise a.Rejected("private response not retained")
        rejected = a.send(self.store, key, PIN, reject)
        self.assertEqual(rejected["parent"]["state"], "failed")
        a.send(self.store, key, PIN, lambda _: self.fail("rejection retried"))

    def test_wrong_transport_receipt_never_fabricates_success(self):
        result = self.send(lambda request: dict(receipt(request), identity=dict(PIN, team="TOTHER")))
        self.assertEqual(result["parent"]["state"], "uncertain")
        self.assertIsNone(result["parent"]["receipt"])
        self.assertEqual(result["details"]["state"], "pending")

    def test_markup_secrets_unsafe_remote_and_missing_context(self):
        feed = revision(self.feed)
        feed["decisions"][0]["revisions"][-1]["question"] = "<@UTEST> <!channel> @here & *run*?"
        key = self.enqueue(feed)
        text = a.inspect(self.store, key)["intent"]["payloads"]["details"]["text"]
        self.assertIn("&lt;＠UTEST&gt; &lt;!channel&gt; ＠here &amp; *run*?", text)
        self.assertNotIn("<@", text)
        for url in ("http://dashboard.example.test", "https://127.0.0.1", "https://localhost", "https://host.local", "https://u:p@host.example", "https://host.example/path", "https://host.example?token=bad", "https://host.example#bad", "https://host.example:444"):
            with self.subTest(url=url), self.assertRaises(d.Invalid):
                self.enqueue(feed, remote=url)
        for value in ("xoxb-fixture-canary", "secret=fixture", "raw_private_log", "bad\u202e", "x" * 2001):
            feed["decisions"][0]["revisions"][0]["question"] = value
            with self.assertRaisesRegex(d.Invalid, "invalid fixture"):
                self.enqueue(feed)
        feed = copy.deepcopy(self.feed)
        feed["decisions"][0]["revisions"][0]["sprints"][0]["path"] = None
        with self.assertRaises(d.Invalid):
            self.enqueue(feed)

    def test_missing_corrupt_oversized_or_public_state_is_held(self):
        path = self.root / "alerts.json"
        original = path.read_bytes()
        for raw in (b"{}", b'{"schema":1,"schema":1}', b"[" * 2000, b"x" * (d.MAX_BYTES + 1)):
            path.write_bytes(raw)
            with self.assertRaises(d.Invalid):
                self.send(lambda _: self.fail("invalid storage sent"))
        path.write_bytes(original)
        path.chmod(0o644)
        with self.assertRaises(d.Invalid):
            self.send()
        path.chmod(0o600)
        path.unlink()
        with self.assertRaises(d.Invalid):
            self.enqueue(self.feed)
        with self.assertRaises(d.Invalid):
            self.store.initialize()

    def test_symlinks_hardlinks_and_unsafe_directory_reject(self):
        for name in ("alerts.json", "replies.json", ".slack.lock"):
            path = self.root / name
            backup = self.root / (name + ".backup")
            path.rename(backup)
            path.symlink_to(backup)
            with self.assertRaises(d.Invalid):
                self.send()
            path.unlink()
            os.link(backup, path)
            with self.assertRaises(d.Invalid):
                self.send()
            path.unlink()
            backup.rename(path)
        self.root.chmod(0o755)
        with self.assertRaises(d.Invalid):
            self.send()
        self.root.chmod(0o700)
        link = Path(self.temp.name) / "alias"
        link.symlink_to(self.root)
        with self.assertRaises(d.Invalid):
            a.Store(link)

    def test_pre_replace_failure_sends_nothing_and_post_replace_holds(self):
        before = (self.root / "alerts.json").read_bytes()
        with patch.object(a.os, "replace", side_effect=OSError("private-canary")), self.assertRaises(d.Invalid):
            self.send(lambda _: self.fail("not durable"))
        self.assertEqual((self.root / "alerts.json").read_bytes(), before)
        real = a.os.replace
        def replaced_then_error(source, destination):
            real(source, destination)
            raise OSError("private-canary")
        with patch.object(a.os, "replace", side_effect=replaced_then_error), self.assertRaises(d.Invalid):
            self.send(lambda _: self.fail("uncertain write"))
        state = self.send(lambda _: self.fail("restart retried"))
        self.assertEqual(state["parent"]["state"], "uncertain")
        self.assertEqual(list(self.root.glob(".slack-pending-*")), [])
        for entry in self.root.iterdir():
            self.assertEqual(entry.stat().st_mode & 0o777, 0o600)

    def test_crash_after_parent_receipt_before_details_intent(self):
        real = self.store.write
        def crash(name, body):
            real(name, body)
            if name == "alerts" and body[self.key]["parent"]["state"] == "sent":
                raise SystemExit(27)
        with patch.object(self.store, "write", side_effect=crash), self.assertRaises(SystemExit):
            self.send()
        state = a.inspect(a.Store(self.root), self.key)
        self.assertEqual([state[p]["state"] for p in ("parent", "details")], ["sent", "pending"])
        requests = []
        a.send(a.Store(self.root), self.key, PIN, lambda req: (requests.append(req), receipt(req))[1])
        self.assertEqual(len(requests), 1)
        self.assertEqual(requests[0]["thread_ts"], "100.000001")

    def test_payload_limits_identity_validation_and_whole_feed_rejection(self):
        for changed in (dict(PIN, bot="UTEST"), dict(PIN, human="BTEST"), dict(PIN, generation="secret=fixture")):
            with self.assertRaises(d.Invalid):
                self.enqueue(self.feed, pinned_identity=changed)
        oversized = copy.deepcopy(self.feed)
        oversized["decisions"][0]["revisions"][0]["options"] = [str(i) + "x" * 1900 for i in range(10)]
        with self.assertRaises(d.Invalid):
            self.enqueue(oversized)
        other = copy.deepcopy(self.feed["decisions"][0])
        other.update(id="other")
        other["revisions"][0]["summary"] = "secret=fixture"
        invalid = copy.deepcopy(self.feed)
        invalid["decisions"].append(other)
        with self.assertRaises(d.Invalid):
            self.enqueue(invalid)
