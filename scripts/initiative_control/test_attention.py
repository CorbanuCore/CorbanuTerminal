import copy
import unittest
from html.parser import HTMLParser

import attention


class Links(HTMLParser):
    def __init__(self, page):
        super().__init__()
        self.hrefs = []
        self.feed(page)

    def handle_starttag(self, tag, attrs):
        if tag == "a":
            self.hrefs.append(dict(attrs)["href"])


class AttentionTests(unittest.TestCase):
    def setUp(self):
        self.sprints = [dict(sprint_id="PF-80-S01", path="docs/sprints/current/delivery.md"),
                        dict(sprint_id="PF-76-S01", path="docs/sprints/current/provider.md")]
        self.documents = {s["path"]: "published" for s in self.sprints}
        self.documents[attention.HISTORY] = "history"

    def render(self, items):
        return attention.render(items, self.sprints, self.documents)

    def test_legacy_links_history_not_provider_and_explains_impact(self):
        page = self.render([attention.issue(attention.LEGACY, 10)])
        self.assertIn(attention.document_url(attention.HISTORY), Links(page).hrefs)
        self.assertNotIn(attention.document_url(self.sprints[1]["path"]), Links(page).hrefs)
        self.assertIn(attention.document_url(self.sprints[0]["path"]), Links(page).hrefs)
        for phrase in ("Manager bookkeeping", "No decision needed from you", "Corbanu manager", "Inventory historical", "does not block current", "10 source warnings"):
            self.assertIn(phrase, page)
        self.assertNotIn("Question for you", page)
        self.assertNotIn("Manager attention required", page)

    def test_missing_legacy_document_never_links_modern_provider(self):
        del self.documents[attention.HISTORY]
        page = self.render([attention.issue(attention.LEGACY)])
        self.assertIn('href="#attention-context-PF-76-S01"', page)
        self.assertNotIn(attention.document_url(self.sprints[1]["path"]), page)

    def test_known_unknown_and_ambiguous_references_link_honestly(self):
        item = attention.issue("PF-80-S01 requires PF-99-S99")
        page = self.render([item])
        self.assertIn(attention.document_url(self.sprints[0]["path"]), page)
        self.assertIn('id="attention-context-PF-99-S99"', page)
        self.assertIn("No unique published context", page)
        self.sprints.append(dict(sprint_id="PF-80-S01", path="docs/duplicate.md"))
        self.documents["docs/duplicate.md"] = "duplicate"
        self.assertIn('href="#attention-context-PF-80-S01"', self.render([item]))

    def test_modern_pf76_reference_is_not_implicitly_historical(self):
        page = self.render([attention.issue("PF-76-S01 validation failed")])
        self.assertIn(attention.document_url(self.sprints[1]["path"]), page)
        self.assertNotIn(attention.document_url(attention.HISTORY), page)

    def test_explicit_question_is_read_only_and_all_its_refs_are_linked(self):
        item = attention.issue("PF-80-S01 test decision")
        item.update(owner="Travis", background="Synthetic decision fixture, not a live request.",
                    impact="Waiting delays the synthetic pilot, not unrelated work.",
                    next_step="Manager records the answer before allocating approved work.",
                    question="For PF-80-S01, should the synthetic pilot have five testers or ten? Five is recommended to bound triage time.")
        page = self.render([item])
        self.assertIn("Needs your decision", page)
        self.assertIn("Question for you", page)
        self.assertIn("five testers or ten?", page)
        self.assertIn("Slack reply routing is not connected yet", page)
        self.assertEqual(Links(page).hrefs.count(attention.document_url(self.sprints[0]["path"])), 2)
        self.assertNotIn("<form", page)
        self.assertNotIn("<input", page)

    def test_all_fields_escape_untrusted_markup(self):
        item = attention.issue('<script>bad</script> PF-99-S99')
        for field in ("owner", "kind", "background", "impact", "next_step", "question"):
            item[field] = '<img src=x onerror="bad()"> PF-99-S99'
        page = self.render([item])
        self.assertNotIn("<img", page)
        self.assertNotIn("<script>", page)
        self.assertIn("&lt;img", page)
        self.assertTrue(all(h.startswith("#attention") for h in Links(page).hrefs))

    def test_grouping_is_stable_and_never_mutates_inputs_or_suppresses_other_notices(self):
        data = dict(problems=[attention.LEGACY] * 10 + ["Validation PF-80-S01"],
                    sprints={"sprints": self.sprints}, documents=self.documents)
        original = copy.deepcopy(data)
        page = attention.notices(data)
        self.assertEqual(page.count('<details class="attention-item"'), 2)
        self.assertNotIn('<details class="attention-item" open', page)
        self.assertEqual(data, original)
        self.assertEqual(page, attention.notices(data))
        self.assertEqual(attention.render([], [], {}), "")

    def test_rejected_and_unreadable_reports_have_specific_explanations(self):
        page = self.render([attention.issue("A worker report was rejected; manager inspection required."),
                            attention.issue("Writeback status is unreadable; manager inspection required.")])
        self.assertIn("not proof that the worker stopped", page)
        self.assertIn("Delivery status is unknown", page)
        self.assertIn("do not retry an uncertain external post", page)


class DecisionRenderingTests(unittest.TestCase):
    def setUp(self):
        from test_decisions import NOW, SPRINT_PATH, fixture
        self.now, self.path, self.value = NOW, SPRINT_PATH, fixture()
        self.sprints = [dict(sprint_id="PF-80-S01", path=self.path)]
        self.documents = {self.path: "approved sanitized document"}

    def render(self, value=None, now=None):
        return attention.render_decisions(self.value if value is None else value,
                                          now or self.now, self.sprints, self.documents)

    def test_dec001_005_full_context_exact_summary_links_and_purity(self):
        record = self.value["decisions"][0]["revisions"][0]
        second = "docs/sprints/current/second.md"
        record["summary"] += " and PF-60-S02"
        record["sprints"].append(dict(sprint_id="PF-60-S02", path=second, historical=False))
        self.sprints += [dict(sprint_id="PF-60-S02", path=second), dict(sprint_id="PF-60-S01", path="docs/sprints/current/similar.md")]
        self.documents[second] = "approved"
        before = copy.deepcopy(self.value)
        page = self.render()
        summary = page.split('<summary>', 1)[1].split('</summary>', 1)[0]
        for path in (self.path, second):
            self.assertIn(attention.document_url(path), Links(summary).hrefs)
        for phrase in ("Needs your decision", "Open decisions: 1", "Oldest raised 0 minutes", "Travis", "Synthetic pilot planning", "Ten testers increase triage", "Pilot invitations", "Offline implementation", "Five testers", "Ten testers", "Five testers bounds triage", "Five or ten testers?", "Sanitized test plan"):
            self.assertIn(phrase, page)
        self.assertEqual(self.value, before)
        self.assertEqual(self.render(), page)

    def test_dec006_007_019_notices_separate_acknowledged_unresolved(self):
        from test_decisions import revision
        notice = attention.render([attention.issue(attention.LEGACY), attention.issue("Acknowledgment requested")], [], {})
        for value in (self.value, {**self.value, "decisions": []}, revision(self.value, "acknowledged")):
            page = self.render(value) + notice
            self.assertIn("Operations notices", page)
            self.assertIn("No decision needed from you", page)
            self.assertNotIn("Question for you", notice)
        acknowledged = self.render(revision(self.value, "acknowledged"))
        self.assertIn("Open decisions: 1", acknowledged)
        self.assertIn("Acknowledged; unresolved", acknowledged)

    def test_dec008_014_freshness_unknown_and_mixed_age(self):
        from test_decisions import LATER
        empty = {**self.value, "decisions": []}
        self.assertIn("No open decisions found in this fresh assessment", self.render(empty))
        missing = attention.render_decisions(None, self.now, [], {})
        self.assertIn("open count unknown", missing)
        self.assertNotIn("No open decisions", missing)
        for value in (empty, self.value):
            stale = self.render(value, LATER)
            self.assertIn("Stale: current decisions unknown", stale)
            self.assertNotIn("No open decisions found", stale)
            self.assertIn(self.now, stale)
            self.assertNotIn(LATER, stale)
        fresh = copy.deepcopy(self.value)
        fresh.update(revision=2, assessed_at=LATER)
        record = fresh["decisions"][0]["revisions"][0]
        record["owner"] = record["impact"] = None
        page = self.render(fresh, LATER)
        self.assertIn("Fresh manager assessment", page)
        self.assertIn("Stale context", page)
        self.assertIn("Stale evidence", page)
        self.assertIn("Unknown owner", page)
        self.assertIn("Unknown: not assessed", page)

    def test_dec015_018_resolution_anchors_history_and_no_actions(self):
        from test_decisions import revision
        for status in ("resolved", "superseded"):
            value = revision(self.value, status)
            other = copy.deepcopy(self.value["decisions"][0])
            other["id"] = "choice-2"
            value["decisions"].append(other)
            page = self.render(value)
            self.assertIn('id="decision-choice-1"', page)
            self.assertIn('href="#decision-choice-1"', page)
            self.assertIn("Open decisions: 1", page)
            self.assertIn("Retained revision 1", page)
            self.assertIn("against question revision 1", page)
            self.assertIn("Synthetic pilot only", page)
            self.assertIn("Slack not connected", page)
            self.assertLess(page.index('id="decision-choice-2"'), page.index("Decision history"))
            self.assertGreater(page.index('id="decision-choice-1"'), page.index("Decision history"))
            for tag in ("<form", "<input", "<button", "<script"):
                self.assertNotIn(tag, page)

    def test_dec020_historical_identity_and_missing_context(self):
        record = self.value["decisions"][0]["revisions"][0]
        record["summary"] = "PF-76-S01 historical pilot"
        record["sprints"] = [dict(sprint_id="PF-76-S01", path=attention.HISTORY, historical=True)]
        self.sprints.append(dict(sprint_id="PF-76-S01", path="docs/sprints/current/provider.md"))
        self.documents[attention.HISTORY] = "approved historical explanation"
        self.assertIn(attention.document_url(attention.HISTORY), Links(self.render()).hrefs)
        del self.documents[attention.HISTORY]
        self.assertIn("unavailable context", self.render())
        self.assertNotIn(attention.document_url("docs/sprints/current/provider.md"), self.render())

    def test_dec021_escape_and_reject_canaries_even_in_retained_history(self):
        from test_decisions import revision
        for field in ("owner", "summary", "background", "impact", "stopped", "continuing", "recommendation", "question"):
            value = copy.deepcopy(self.value)
            value["decisions"][0]["revisions"][0][field] = '<img src=x onerror="bad()">'
            page = self.render(value)
            self.assertNotIn("<img", page)
            self.assertIn("&lt;img", page)
            value["decisions"][0]["revisions"][0][field] = "SYNTHETIC_SECRET canary"
            page = self.render(revision(value))
            self.assertIn("Unknown", page)
            self.assertNotIn("canary", page)
        for field in ("label", "path"):
            value = copy.deepcopy(self.value)
            value["decisions"][0]["revisions"][0]["evidence"][0][field] = "RAW_PRIVATE_LOG canary"
            self.assertNotIn("canary", self.render(value))


if __name__ == "__main__":
    unittest.main()
