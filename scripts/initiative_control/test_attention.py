import copy
import unittest
from html.parser import HTMLParser
from pathlib import Path

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

    def test_dec001_single_open_decision_has_no_duplicate_index(self):
        from test_decisions import LATER, revision
        for status in ("open", "acknowledged"):
            value = self.value if status == "open" else revision(self.value, status)
            for now in (self.now, LATER):
                with self.subTest(status=status, now=now):
                    page = self.render(value, now)
                    self.assertNotIn('id="open-decision-index"', page)
                    self.assertNotIn('href="#open-decision-index"', page)
                    self.assertNotIn("Decision overview —", page)
                    self.assertEqual(page.count('class="decision-overview"'), 1)
                    self.assertEqual(page.count('class="attention-item decision-card"'), 1)
                    self.assertIn('id="decision-choice-1" tabindex="-1"><summary>', page)
                    self.assertIn('class="decision-body-bounded" tabindex="0" role="region"', page)
                    self.assertIn('href="#decision-choice-1">Permanent decision link</a>', page)

    def test_dec002_full_page_flow_and_dec025_sticky_index_styles(self):
        css = Path(attention.__file__).with_name("style.css").read_text()
        body = css.split(".decision-body-bounded{", 1)[1].split("}", 1)[0]
        properties = dict(rule.split(":", 1) for rule in body.split(";") if rule)
        for name in ("height", "max-height", "overflow", "overflow-y", "scrollbar-gutter"):
            self.assertNotIn(name, properties)
        self.assertNotIn("40vh", css)
        self.assertEqual(css.count(".decision-body-bounded{"), 1)
        self.assertIn(".decision-body-bounded:focus-visible{outline:", css)
        index = css.split(".decision-index{", 1)[1].split("}", 1)[0]
        properties = dict(rule.split(":", 1) for rule in index.split(";") if rule)
        self.assertEqual(properties["position"], "sticky")
        self.assertEqual(properties["top"], "0")
        self.assertGreater(int(properties["z-index"]), 0)
        for count in (0, 1, 2, 3):
            with self.subTest(open_decisions=count):
                value = copy.deepcopy(self.value)
                value["decisions"] = []
                for number in range(count):
                    decision = copy.deepcopy(self.value["decisions"][0])
                    decision["id"] = f"choice-{number}"
                    value["decisions"].append(decision)
                page = self.render(value)
                self.assertEqual(page.count('class="decision-index"'), int(count >= 2))
                self.assertEqual(page.count('href="#open-decision-index"'),
                                 count if count >= 2 else 0)

    def test_dec025_long_context_keeps_all_owners_in_index_and_full_cards(self):
        from test_decisions import revision
        self.value = revision(self.value, "acknowledged")
        template = self.value["decisions"][0]
        self.value["decisions"] = []
        for owner in ("Avery", "Blair", "Casey"):
            decision = copy.deepcopy(template)
            decision["id"] = owner.lower()
            for record in decision["revisions"]:
                record["owner"] = owner
                record["summary"] = f"{owner}'s pilot choice"
                record["background"] = "Long context for this decision. " * 50
                record["evidence"] *= 30
            self.value["decisions"].append(decision)
        before = copy.deepcopy(self.value)
        page = self.render()
        index = page.split('id="open-decision-index"', 1)[1].split('</ul></div>', 1)[0]
        self.assertNotIn("<details", index)
        self.assertNotIn("Long context", index)
        self.assertEqual(index.count("<li>"), 3)
        self.assertLess(page.index('id="open-decision-index"'), page.index("<details"))
        for owner in ("Avery", "Blair", "Casey"):
            anchor = "decision-" + owner.lower()
            self.assertIn(f'href="#{anchor}"', index)
            self.assertIn(f"Owner: {owner}", index)
            self.assertIn(f"{owner}&#x27;s pilot choice", index)
            card = page.split(f'id="{anchor}"', 1)[1].split("</summary>", 1)[0]
            self.assertIn(f"Owner: {owner}", card)
            self.assertIn('class="decision-overview"', card)
            self.assertIn(attention.document_url(self.path), Links(card).hrefs)
            self.assertIn("acknowledged", card)
        self.assertEqual(Links(index).hrefs.count(attention.document_url(self.path)), 3)
        self.assertEqual(index.count('class="badge">acknowledged'), 3)
        self.assertEqual(page.count('class="attention-item decision-card"'), 3)
        self.assertEqual(page.count('class="decision-body-bounded" tabindex="0" role="region"'), 3)
        self.assertEqual(page.count('aria-label="Decision context:'), 3)
        self.assertEqual(Links(page).hrefs.count("#open-decision-index"), 3)
        self.assertEqual(page.count("Retained revision 1"), 3)
        self.assertEqual(page.count("Long context for this decision."), 300)
        self.assertEqual(page.count("Sanitized test plan"), 180)
        self.assertEqual(self.value, before)
        self.assertNotIn("<script", page)
        self.assertNotIn(" onclick=", page)

    def test_dec025_index_only_lists_open_records_and_retains_safe_links(self):
        from test_decisions import LATER, revision
        resolved = revision(self.value, "resolved")
        for value in ({}, {**self.value, "decisions": []}, resolved):
            self.assertNotIn('id="open-decision-index"', self.render(value))
        other = copy.deepcopy(self.value["decisions"][0])
        other["id"] = "unsafe-text"
        other["revisions"][0]["owner"] = '<img src=x onerror="bad()">'
        other["revisions"][0]["summary"] = '<script>bad()</script> PF-99-S99'
        resolved["decisions"].append(other)
        for now in (self.now, LATER):
            page = self.render(resolved, now)
            self.assertNotIn('id="open-decision-index"', page)
            self.assertNotIn('href="#open-decision-index"', page)
            self.assertEqual(page.count('class="attention-item decision-card"'), 2)
        second = copy.deepcopy(self.value["decisions"][0])
        second["id"] = "second-open"
        resolved["decisions"].append(second)
        for now in (self.now, LATER):
            page = self.render(resolved, now)
            index = page.split('id="open-decision-index"', 1)[1].split('</ul></div>', 1)[0]
            self.assertNotIn('href="#decision-choice-1"', index)
            self.assertIn('href="#decision-unsafe-text"', index)
            self.assertIn('href="#decision-second-open"', index)
            self.assertEqual(index.count("<li>"), 2)
            self.assertEqual(Links(page).hrefs.count("#open-decision-index"), 2)
            self.assertIn("&lt;img", index)
            self.assertIn("&lt;script&gt;", index)
            self.assertNotIn("<img", page)
            self.assertNotIn("<script", page)
            self.assertTrue(any(h.startswith("#decision-context-") for h in Links(index).hrefs))
            self.assertIn(attention.document_url(self.path), Links(index).hrefs)
            self.assertEqual(page.count('id="decision-choice-1"'), 1)
            self.assertEqual(page.count('id="decision-unsafe-text"'), 1)
            self.assertEqual(page.count('class="decision-body-bounded"'), 3)

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

    def test_f01_oldest_target_uses_open_history_and_preserves_dates(self):
        from test_decisions import LATER, revision
        older = copy.deepcopy(self.value["decisions"][0])
        older["id"] = "older"
        older["revisions"][0]["raised_at"] = "2026-09-12T11:00:00Z"
        for status in ("open", "acknowledged", "resolved", "superseded"):
            value = revision(self.value, status)
            value["decisions"].append(older)
            before = copy.deepcopy(value)
            for at, age in ((self.now, 60), (LATER, 81)):
                page = self.render(value, at)
                self.assertIn('id="oldest-open-decision-age" data-raised-at="2026-09-12T11:00:00Z"', page)
                self.assertIn(f"Oldest raised {age} minutes ago.</span>", page)
                self.assertEqual(page.count('id="oldest-open-decision-age"'), 1)
            self.assertEqual(value, before)
        # A resolved older decision must no longer determine the open age.
        value = revision({**self.value, "decisions": [older]}, "resolved")
        value["decisions"].append(self.value["decisions"][0])
        self.assertIn(f'data-raised-at="{self.now}"', self.render(value))

    def test_f01_empty_unknown_and_resolved_have_no_age_target(self):
        from test_decisions import LATER, revision
        invalid = copy.deepcopy(self.value)
        invalid["decisions"][0]["revisions"][0]["raised_at"] = '<img src=x>'
        future = copy.deepcopy(self.value)
        future["decisions"][0]["revisions"][0]["raised_at"] = "2099-01-01T00:00:00Z"
        for value in ({}, invalid, future, {**self.value, "decisions": []},
                      revision(self.value, "resolved"), revision(self.value, "superseded")):
            for at in (self.now, LATER):
                page = self.render(value, at)
                self.assertNotIn("oldest-open-decision-age", page)
                self.assertNotIn("Oldest raised", page)
                self.assertNotIn("<img", page)

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
