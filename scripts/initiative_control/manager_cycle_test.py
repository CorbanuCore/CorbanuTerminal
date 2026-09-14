"""Synthetic regression coverage for terminal-history briefing compaction."""

import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from coordinator import Coordinator, Rejected, TERMINAL, digest, encoded
import fable_launcher as f
import manager_cycle as m
from test_coordinator import seed


class BriefingSizeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.context = self.root / "context.json"
        f.write_json(self.context, {"observed_at": "2026-09-14T00:00:00Z",
                                  "context": {"scope": "synthetic offline test"}})
        self.c = Coordinator(self.root / "state")
        self.c.initialize(*seed())
        self.c.set_enabled(True, {"source": "fixture"})
        self.c.event({"id": "tick", "text": "pending work"})

    def ref(self, body):
        with self.c.connection() as db:
            return self.c._reference(db, body)

    def action(self, key="prior", status="accepted", size=1000):
        action = dict(id=key, kind="repair", workstream="delivery", sprint="PF-80-S01",
                      status=status, sequence=[0, 0], rationale="fixture",
                      inputs={"allocation": "historical", "task": "x" * size},
                      allocation_digest="a" * 64)
        for field in ("dispatch_receipt", "ack_receipt", "result", "verification"):
            action[field] = self.ref({"kind": field, "text": key * size})
        return action

    def packet(self, actions, recent=None):
        packet = self.c.begin_manager()
        packet["actions"] = {a["id"]: a for a in actions}
        packet["last_three_actions"]["delivery"] = actions[-3:] if recent is None else recent
        return packet

    def brief(self, packet):
        return json.loads(m.briefing(self.c, packet, self.context))

    def test_terminal_statuses_keep_identity_and_digest_omissions(self):
        actions = [self.action(status, status) for status in sorted(TERMINAL)]
        packet = self.packet(actions)
        before = copy.deepcopy(packet)
        brief = self.brief(packet)
        for action in actions:
            compact = brief["actions"][action["id"]]
            for field in ("id", "kind", "workstream", "sprint", "status",
                          "rationale", "allocation_digest"):
                self.assertEqual(action[field], compact[field])
            self.assertEqual("historical", compact["allocation"])
            self.assertNotIn("inputs", compact)
            omission = next(o for o in brief["evidence_omissions"] if o["id"] == action["id"])
            self.assertEqual(digest(action["inputs"]), omission["inputs_digest"])
            for field in ("dispatch_receipt", "ack_receipt", "result", "verification"):
                key = action[field]["evidence_digest"]
                self.assertEqual(key, compact[field]["evidence_digest"])
                if field in {"result", "verification"}:
                    self.assertEqual(action[field], compact[field])
                else:
                    self.assertNotIn("preview", compact[field])
                self.assertNotIn(key, brief["original_evidence"])
                self.assertIn(key, omission["evidence_digests"])
        self.assertEqual(before, packet)
        self.assertEqual(m.briefing(self.c, packet, self.context),
                         m.briefing(self.c, packet, self.context))

    def test_recent_transition_keeps_inputs_and_recursive_originals(self):
        for status, event in (
            ("accepted", {"id": "verified:prior", "accepted": True}),
            ("failed", {"id": "verified:prior", "accepted": False}),
            ("failed", {"id": "dispatch-reconciled:prior:23"}),
            ("accepted", {"id": "owner_complete:23", "action": "prior"}),
            ("accepted", {"id": "owner_successor:23", "action": "prior"}),
            ("cancelled", {"id": "owner_allocation:23", "allocation": "historical"}),
            ("accepted", {"id": "status-update", "action": "prior", "status": "accepted"}),
        ):
            with self.subTest(status=status, event=event):
                action = self.action(status=status)
                nested = self.ref({"full": "z" * 2000})
                action["result"] = self.ref({"nested": nested})
                packet = {"events": [{"id": event["id"], **self.ref(event)}],
                          "workstreams": dict.fromkeys(("delivery", "accounting", "privacy"), {}),
                          "actions": {"prior": action}, "allocations": {},
                          "last_three_actions": {"delivery": [action]}}
                brief = self.brief(packet)
                self.assertEqual(action["inputs"], brief["actions"]["prior"]["inputs"])
                for field in ("dispatch_receipt", "ack_receipt", "result", "verification"):
                    key = action[field]["evidence_digest"]
                    self.assertEqual(self.c.read_evidence(key), brief["original_evidence"][key])
                self.assertEqual({"full": "z" * 2000},
                                 brief["original_evidence"][nested["evidence_digest"]])
                self.assertEqual([], brief["evidence_omissions"])

    def test_outcome_previews_survive_after_transition_batch_is_consumed(self):
        fields = ("result", "verification", "owner_failure", "owner_cancellation")
        for status in sorted(TERMINAL):
            with self.subTest(status=status):
                action = self.action(status=status)
                for field in fields:
                    action[field] = self.ref({"reason": field + ":" + "理由" * 1000})
                event = {"id": "transition", "action": "prior", "status": status}
                packet = {"events": [{"id": event["id"], **self.ref(event)}],
                          "workstreams": dict.fromkeys(("delivery", "accounting", "privacy"), {}),
                          "actions": {"prior": action}, "allocations": {},
                          "last_three_actions": {"delivery": [action]}}
                transition = self.brief(packet)
                for field in fields:
                    self.assertNotIn("preview", transition["actions"]["prior"][field])
                    key = action[field]["evidence_digest"]
                    self.assertEqual(self.c.read_evidence(key), transition["original_evidence"][key])

                # A later claim contains only an unrelated event; the transition is consumed.
                tick = {"id": "next-tick", "text": "pending work"}
                packet["events"] = [{"id": tick["id"], **self.ref(tick)}]
                before = copy.deepcopy(packet)
                forbidden = {action[field]["evidence_digest"] for field in fields}
                original_read = self.c.read_evidence

                def read(key):
                    self.assertNotIn(key, forbidden)
                    return original_read(key)

                with patch.object(self.c, "read_evidence", read):
                    brief = self.brief(packet)
                for field in fields:
                    self.assertEqual(action[field], brief["actions"]["prior"][field])
                    self.assertEqual(400, len(brief["actions"]["prior"][field]["preview"]))
                    key = action[field]["evidence_digest"]
                    self.assertNotIn(key, brief["original_evidence"])
                    self.assertIn(key, brief["evidence_omissions"][0]["evidence_digests"])
                self.assertEqual(before, packet)

    def test_compact_preview_byte_growth_is_bounded(self):
        packet = self.large_packet()
        fields = ("result", "verification", "owner_failure", "owner_cancellation")
        for action in packet["actions"].values():
            for field in fields:
                action[field] = self.ref({"reason": field + ":" + "理由" * 1000})
        raw = m.briefing(self.c, packet, self.context)
        brief = json.loads(raw)
        without_previews = copy.deepcopy(brief)
        count = 0
        for action in without_previews["actions"].values():
            for field in fields:
                preview = action[field].pop("preview")
                self.assertEqual(400, len(preview))
                count += 1
        growth = len(raw) - len(encoded(without_previews).encode())
        self.assertGreater(growth, 0)
        # Six bytes per character covers JSON escapes, plus the field syntax.
        self.assertLessEqual(growth, count * (400 * 6 + 32))
        self.assertLess(len(raw), f.BRIEF_LIMIT)
        for action in packet["actions"].values():
            for field in fields:
                self.assertNotIn(action[field]["evidence_digest"], brief["original_evidence"])

    def test_changed_action_outside_last_three_stays_compact_but_event_is_full(self):
        action = self.action()
        event = {"id": "verified:prior", "accepted": True, "evidence": action["verification"]}
        self.c.event(event)
        packet = self.packet([action], recent=[])
        brief = self.brief(packet)
        self.assertNotIn("inputs", brief["actions"]["prior"])
        self.assertEqual(event, brief["original_evidence"][digest(event)])
        key = action["verification"]["evidence_digest"]
        self.assertEqual(self.c.read_evidence(key), brief["original_evidence"][key])
        omission = brief["evidence_omissions"][0]
        self.assertNotIn(key, omission["evidence_digests"])
        self.assertNotIn(action["dispatch_receipt"]["evidence_digest"], brief["original_evidence"])

    def test_unrelated_nested_and_prefix_events_do_not_establish_transition(self):
        action = self.action(status="failed")
        self.c.event({"id": "observation", "action": "prior",
                      "nested": {"id": "verified:prior", "action": "prior", "status": "failed"}})
        self.c.event({"id": "dispatch-reconciled:prior:other:23"})
        brief = self.brief(self.packet([action]))
        self.assertNotIn("inputs", brief["actions"]["prior"])
        self.assertNotIn(action["result"]["evidence_digest"], brief["original_evidence"])

    def test_nonterminal_inputs_and_owner_proofs_are_preserved(self):
        action = self.action(status="running")
        nested = self.ref({"full": "body"})
        action["inputs"]["nested"] = nested
        terminal = self.action("closed")
        terminal["owner_failure"] = self.ref({"proof": nested})
        brief = self.brief(self.packet([action, terminal]))
        self.assertEqual(action["inputs"], brief["actions"]["prior"]["inputs"])
        for field in ("dispatch_receipt", "ack_receipt", "result", "verification"):
            key = action[field]["evidence_digest"]
            self.assertEqual(self.c.read_evidence(key), brief["original_evidence"][key])
        self.assertEqual(self.c.read_evidence(nested["evidence_digest"]),
                         brief["original_evidence"][nested["evidence_digest"]])
        self.assertEqual(terminal["owner_failure"], brief["actions"]["closed"]["owner_failure"])
        key = terminal["owner_failure"]["evidence_digest"]
        self.assertNotIn(key, brief["original_evidence"])
        self.assertIn(key, brief["evidence_omissions"][0]["evidence_digests"])

    def test_consumed_allocations_stay_compact_without_losing_active_inputs(self):
        action = self.action(status="running")
        packet = self.packet([action])
        allocation = packet["allocations"]["bootstrap"]
        allocation["inputs"] = {"consumed": True, "payload": "x" * 4000,
                                "reference": self.ref({"full": "allocation evidence"})}
        action["inputs"] = {"allocation": "bootstrap", **allocation["inputs"]}
        action["allocation_digest"] = digest(allocation)
        before = copy.deepcopy(packet)
        brief = self.brief(packet)
        self.assertEqual({"consumed": True}, brief["allocations"]["bootstrap"]["inputs"])
        self.assertEqual(action["inputs"], brief["actions"]["prior"]["inputs"])
        self.assertNotIn("inputs_from_allocation", brief["actions"]["prior"])
        omission = brief["evidence_omissions"][0]
        self.assertEqual(("allocations", "bootstrap"),
                         (omission["source"], omission["id"]))
        self.assertEqual(digest(allocation["inputs"]), omission["inputs_digest"])
        self.assertNotIn("evidence_digests", omission)  # Still needed by the running action.
        self.assertEqual(before, packet)

    def test_consumed_requires_boolean_true_and_omits_only_unneeded_references(self):
        packet = self.packet([])
        allocation = packet["allocations"]["bootstrap"]
        ref = self.ref({"full": "omitted allocation evidence"})
        for value in (False, 1, "true", True):
            with self.subTest(value=value):
                allocation["inputs"] = {"consumed": value, "ref": ref}
                brief = self.brief(packet)
                if value is True:
                    self.assertEqual({"consumed": True}, brief["allocations"]["bootstrap"]["inputs"])
                    self.assertEqual([ref["evidence_digest"]],
                                     brief["evidence_omissions"][0]["evidence_digests"])
                else:
                    self.assertEqual(allocation, brief["allocations"]["bootstrap"])
                    self.assertEqual([], brief["evidence_omissions"])

    def test_unexpanded_roots_are_recorded_without_loading_originals(self):
        action = self.action()
        missing = {"evidence_digest": "b" * 64, "bytes": 1000}
        action["result"] = missing
        action["inputs"]["nested"] = {"evidence_digest": "c" * 64}
        packet = self.packet([action])
        original_read = self.c.read_evidence

        def read(key):
            self.assertNotIn(key, {"b" * 64, "c" * 64})
            return original_read(key)

        with patch.object(self.c, "read_evidence", read):
            brief = self.brief(packet)
        for key in ("b" * 64, "c" * 64):
            self.assertIn(key, brief["evidence_omissions"][0]["evidence_digests"])

    def test_retained_missing_corrupt_and_wrong_size_evidence_still_holds(self):
        action = self.action(status="running")
        packet = self.packet([action])
        with patch.object(self.c, "read_evidence", return_value={"corrupt": True}):
            with self.assertRaisesRegex(f.LaunchError, "evidence_digest_mismatch"):
                self.brief(packet)
        action["result"] = {"evidence_digest": "d" * 64}
        with self.assertRaisesRegex(Rejected, "unknown evidence"):
            self.brief(packet)
        action["result"] = {**action["ack_receipt"], "bytes": 1}
        with self.assertRaisesRegex(f.LaunchError, "evidence_size_mismatch"):
            self.brief(packet)

    def test_oversized_event_still_holds_without_truncation(self):
        event = {"id": "large", "text": "z" * 70000}
        self.c.event(event)
        with self.assertRaisesRegex(Rejected, "record exceeds JSON byte limit"):
            self.brief(self.packet([]))

    def test_fifo_restriction_recomputes_recent_transition_from_selected_events(self):
        action = self.action(size=18000)
        self.c.event({"id": "verified:prior", "accepted": True})
        packet = self.packet([action])
        selected, raw = m.fit_briefing(self.c, packet, self.context)
        brief = json.loads(raw)
        self.assertEqual({"selected": 1, "deferred": 1}, brief["event_batch"])
        self.assertEqual(["tick"], [event["id"] for event in selected["events"]])
        self.assertNotIn("inputs", brief["actions"]["prior"])
        self.assertNotIn(action["result"]["evidence_digest"], brief["original_evidence"])
        with self.c.connection() as db:
            self.assertEqual(2, db.execute("SELECT COUNT(*) FROM events WHERE consumed IS NULL").fetchone()[0])

    def large_packet(self):
        actions = [self.action("closed-" + str(i), size=2000) for i in range(3)]
        packet = self.packet(actions)
        packet["allocations"]["bootstrap"]["inputs"] = {
            "consumed": True, "task": "x" * 20000,
            "proof": self.ref({"full": "y" * 20000})}
        return packet

    def test_one_event_with_large_terminal_history_fits_unchanged_limit(self):
        packet = self.large_packet()
        raw = m.briefing(self.c, packet, self.context)
        brief = json.loads(raw)
        self.assertEqual(65536, f.BRIEF_LIMIT)
        self.assertLess(len(raw), f.BRIEF_LIMIT)
        self.assertEqual(1, len(brief["events"]))
        self.assertEqual(4, len(brief["evidence_omissions"]))


if __name__ == "__main__":
    unittest.main()
