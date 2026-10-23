"""Nonempty discovery target for the hand-computed PF-60-S01 fixtures."""

from copy import deepcopy
from fractions import Fraction
from itertools import permutations, product
import json
from pathlib import Path
import subprocess
import sys
import unittest

from reference import Replay, estimate, exact, normalize, select_price


HERE = Path(__file__).resolve().parent


class AccountingContractTests(unittest.TestCase):
    def setUp(self):
        self.data = json.loads((HERE / "fixtures.json").read_text())
        self.prices = self.data["prices"]
        self.events = self.data["observations"]

    def replay(self, events=None, attempts=None, threads=None):
        replay = Replay(attempts if attempts is not None else self.data["attempts"],
                        threads if threads is not None else self.data["threads"])
        replay.ingest(events if events is not None else self.events)
        return replay

    def test_every_attempt_matches_literal_hand_computed_golden(self):
        rows = self.replay().rows(self.prices)
        self.assertEqual(len(rows), 6)
        for key, row in rows.items():
            with self.subTest(attempt=key):
                expected = self.data["expected"][key]
                for field, value in expected.items():
                    if field in {"known_cost", "cost"} and value is not None:
                        value = exact(value)
                    self.assertEqual(row[field], value, field)
                self.assertEqual(bool(row["unknown_reasons"]), expected["cost"] is None)

    def test_parent_two_children_three_providers_and_three_requests(self):
        result = self.replay().aggregate("root", self.prices)
        expected = deepcopy(self.data["expected"]["root_aggregate"])
        for key in ("known_cost", "known_billed"):
            expected[key] = exact(expected[key])
        self.assertEqual(result, expected)
        providers = {a["provider"] for a in self.data["attempts"] if a["thread_id"] != "history"}
        self.assertEqual(providers, {"openai", "anthropic", "pfterminal-plan"})

    def test_retry_counts_both_consumed_attempts(self):
        result = self.replay().aggregate("child-a", self.prices)
        self.assertEqual((result["requests"], result["attempts"]), (1, 2))
        self.assertEqual(result["known_cost"], exact("0.000864"))
        self.assertIsNone(result["cost"])
        self.assertEqual(result["input"]["total"], 240)

    def test_cumulative_output_replaces_and_does_not_sum_stream_updates(self):
        row = self.replay().rows(self.prices)["child-a-2"]
        self.assertEqual(row["output"], 30)
        self.assertNotEqual(row["output"], 42)
        self.assertEqual(row["cost"], exact("0.000636"))

    def test_reasoning_and_cache_are_subsets_not_extra_tokens(self):
        row = self.replay().rows(self.prices)["root-1"]
        self.assertEqual(row["total"], 120)
        self.assertNotEqual(row["total"], 168)
        self.assertEqual(row["cost"], exact("0.000220"))
        self.assertNotEqual(row["cost"], exact("0.000252"))

    def test_interrupted_partial_input_retained_output_unknown(self):
        row = self.replay().rows(self.prices)["child-a-1"]
        self.assertEqual(row["input"], 80)
        self.assertIsNone(row["output"])
        self.assertIsNone(row["cost"])
        self.assertEqual(row["known_cost"], exact("0.000228"))
        self.assertIn("output:usage_unknown", row["unknown_reasons"])

    def test_missing_price_is_not_free_work(self):
        row = self.replay().rows(self.prices)["child-b-1"]
        self.assertIsNone(row["cost"])
        self.assertIsNone(row["price_id"])
        self.assertEqual(row["known_cost"], 0)
        self.assertIn("output:price_missing", row["unknown_reasons"])

    def test_native_corbanu_chat_does_not_invent_cache_write_measurement(self):
        row = self.replay().rows(self.prices)["child-b-1"]
        self.assertEqual(row["input"], 80)
        self.assertEqual(row["read"], 20)
        self.assertIsNone(row["write"])
        self.assertIn("input:usage_unknown", row["unknown_reasons"])

    def test_omitted_historical_cache_and_reasoning_are_not_zero(self):
        row = self.replay().rows(self.prices)["historical-unknown"]
        self.assertIsNone(row["read"])
        self.assertIsNone(row["write"])
        self.assertIsNone(row["reasoning"])
        self.assertIsNone(row["cost"])
        self.assertEqual(row["known_cost"], exact("0.000040"))

    def test_replay_twice_and_reordered_with_duplicates_is_identical(self):
        baseline = self.replay()
        replay = self.replay([])
        order = [self.events[i] for i in self.data["replay"]["duplicate_reordered_indices"]]
        replay.ingest(order)
        replay.ingest(order)
        self.assertEqual(replay.snapshot(), baseline.snapshot())
        self.assertEqual(replay.rows(self.prices), baseline.rows(self.prices))
        self.assertEqual(replay.aggregate("root", self.prices), baseline.aggregate("root", self.prices))

    def test_all_retry_revision_permutations_have_same_result(self):
        expected = self.replay().rows(self.prices)
        others = [e for e in self.events if e["attempt_id"] != "child-a-2"]
        retry = [e for e in self.events if e["attempt_id"] == "child-a-2"]
        for permutation in permutations(retry):
            with self.subTest(order=[e["revision"] for e in permutation]):
                self.assertEqual(self.replay(others + list(permutation)).rows(self.prices), expected)

    def test_conflicting_duplicate_rejected_without_committing_batch(self):
        replay = self.replay()
        before = replay.snapshot()
        conflict = deepcopy(self.events[0])
        conflict["usage"]["output_tokens"] = 21
        with self.assertRaisesRegex(ValueError, "conflicting replay"):
            replay.ingest([conflict])
        self.assertEqual(replay.snapshot(), before)

    def test_changed_attempt_identity_cannot_be_patched_by_observation(self):
        event = deepcopy(self.events[0])
        event["thread_id"] = "child-b"
        with self.assertRaisesRegex(ValueError, "immutable identity"):
            self.replay([event])

    def test_json_checkpoint_reopened_in_fresh_python_process(self):
        checkpoint = self.replay([self.events[i] for i in self.data["replay"]["checkpoint_indices"]])
        # stdin/stdout only. This tests the oracle checkpoint contract, not SQLite,
        # native rollouts, fsync, process kill recovery or a provider request.
        payload = {"attempts": self.data["attempts"], "threads": self.data["threads"],
                   "checkpoint": checkpoint.snapshot(), "events": self.events, "prices": self.prices}
        script = ("import json,sys; from reference import Replay; d=json.load(sys.stdin); "
                  "r=Replay(d['attempts'],d['threads']); r.ingest(d['checkpoint']); "
                  "r.ingest(d['events']); r.ingest(d['events']); "
                  "print(json.dumps(r.aggregate('root',d['prices']),sort_keys=True,default=str))")
        output = subprocess.run([sys.executable, "-B", "-c", script], cwd=HERE,
                                input=json.dumps(payload), text=True, capture_output=True, check=True)
        expected = json.dumps(self.replay().aggregate("root", self.prices), sort_keys=True, default=str)
        self.assertEqual(output.stdout.strip(), expected)

    def test_old_timestamp_uses_old_price_not_replay_date(self):
        row = self.replay().rows(self.prices)["historical-priced"]
        self.assertEqual(row["price_id"], "synthetic-openai-old")
        self.assertEqual(row["cost"], exact("0.000110"))
        self.assertNotEqual(row["cost"], exact("0.000220"))

    def test_effective_date_boundary_is_half_open(self):
        attempt = deepcopy(self.data["attempts"][0])
        attempt["dispatched_at"] = "2026-09-01T00:00:00Z"
        self.assertEqual(select_price(attempt, self.prices)["price_id"], "synthetic-openai-new")
        attempt["dispatched_at"] = "2026-08-31T23:59:59Z"
        self.assertEqual(select_price(attempt, self.prices)["price_id"], "synthetic-openai-old")

    def test_unknown_dispatch_time_and_price_gap_stay_unknown(self):
        attempt = deepcopy(self.data["attempts"][0])
        for timestamp in (None, "2025-12-31T00:00:00Z"):
            attempt["dispatched_at"] = timestamp
            self.assertIsNone(select_price(attempt, self.prices))

    def test_later_price_version_does_not_reprice_old_request(self):
        before = self.replay().rows(self.prices)
        prices = deepcopy(self.prices)
        prices[1]["effective_to"] = "2027-01-01T00:00:00Z"
        future = deepcopy(prices[1])
        future.update(price_id="synthetic-future", effective_from="2027-01-01T00:00:00Z", effective_to=None)
        future["rates"]["input"] = "999"
        self.assertEqual(self.replay().rows(prices + [future]), before)

    def test_price_overlap_duplicate_id_currency_and_unit_rejected(self):
        for problem in ("overlap", "duplicate", "currency", "unit"):
            prices = deepcopy(self.prices)
            if problem == "overlap":
                prices[0]["effective_to"] = None
            elif problem == "duplicate":
                prices.append(deepcopy(prices[0]))
            elif problem == "currency":
                prices[1]["currency"] = "EUR"
            else:
                prices[1]["unit"] = "per_token"
            with self.subTest(problem=problem), self.assertRaises(ValueError):
                self.replay().rows(prices)

    def test_negative_noninteger_or_boolean_tokens_rejected(self):
        for invalid in (-1, 1.5, True, "100"):
            events = deepcopy(self.events)
            events[0]["usage"]["input_tokens"] = invalid
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                self.replay(events).rows(self.prices)

    def test_cache_reasoning_and_total_inconsistencies_rejected(self):
        for field in ("cache", "reasoning", "total"):
            events = deepcopy(self.events)
            usage = events[0]["usage"]
            if field == "cache":
                usage["input_tokens_details"]["cached_tokens"] = 101
            elif field == "reasoning":
                usage["output_tokens_details"]["reasoning_tokens"] = 21
            else:
                usage["total_tokens"] = 999
            with self.subTest(field=field), self.assertRaises(ValueError):
                self.replay(events).rows(self.prices)

    def test_cache_read_exceeding_input_rejected_with_unknown_write(self):
        for wire, prefix, details in (("chat", "prompt", {}),
                                      ("responses", "input", {}),
                                      ("responses", "input", {"cache_write_tokens": None})):
            for inputs, reads in ((10, 20), (0, 1)):
                usage = {prefix + "_tokens": inputs,
                         prefix + "_tokens_details": {**details, "cached_tokens": reads}}
                with self.subTest(wire=wire, usage=usage), self.assertRaisesRegex(
                        ValueError, "cache exceeds inclusive input"):
                    normalize(wire, usage)

    def test_cache_write_exceeding_input_rejected_with_unknown_read(self):
        for details in ({}, {"cached_tokens": None}):
            for inputs, writes in ((10, 20), (0, 1)):
                usage = {"input_tokens": inputs,
                         "input_tokens_details": {**details, "cache_write_tokens": writes}}
                with self.subTest(usage=usage), self.assertRaisesRegex(
                        ValueError, "cache exceeds inclusive input"):
                    normalize("responses", usage)

    def test_valid_partial_cache_splits_preserve_unknowns(self):
        for wire, prefix, field, known, unknown in (
                ("chat", "prompt", "cached_tokens", "read", "write"),
                ("responses", "input", "cached_tokens", "read", "write"),
                ("responses", "input", "cache_write_tokens", "write", "read")):
            for inputs, cached in ((10, 0), (10, 4), (10, 10), (0, 0)):
                usage = {prefix + "_tokens": inputs, prefix + "_tokens_details": {field: cached}}
                with self.subTest(wire=wire, field=field, inputs=inputs, cached=cached):
                    row = normalize(wire, usage)
                    self.assertEqual(row["input"], inputs)
                    self.assertEqual(row[known], cached)
                    self.assertIsNone(row[unknown])

    def test_complete_cache_split_checks_sum_not_individual_components(self):
        for reads, writes in ((6, 5), (10, 1), (1, 10)):
            usage = {"input_tokens": 10,
                     "input_tokens_details": {"cached_tokens": reads, "cache_write_tokens": writes}}
            with self.subTest(reads=reads, writes=writes), self.assertRaisesRegex(
                    ValueError, "cache exceeds inclusive input"):
                normalize("responses", usage)
        row = normalize("responses", {"input_tokens": 10, "input_tokens_details": {
            "cached_tokens": 6, "cache_write_tokens": 4}})
        self.assertEqual((row["input"], row["read"], row["write"]), (10, 6, 4))

    def test_unknown_inclusive_input_does_not_bound_known_cache(self):
        for wire, prefix, details, expected in (
                ("chat", "prompt", {"cached_tokens": 20}, (20, None)),
                ("responses", "input", {"cached_tokens": 20}, (20, None)),
                ("responses", "input", {"cache_write_tokens": 20}, (None, 20)),
                ("responses", "input", {"cached_tokens": 20, "cache_write_tokens": 30}, (20, 30))):
            for inputs in ({}, {prefix + "_tokens": None}):
                usage = {**inputs, prefix + "_tokens_details": details}
                with self.subTest(wire=wire, usage=usage):
                    row = normalize(wire, usage)
                    self.assertIsNone(row["input"])
                    self.assertEqual((row["read"], row["write"]), expected)

    def test_anthropic_cache_is_added_to_noncached_input_only_when_complete(self):
        usage = {"input_tokens": 10, "cache_read_input_tokens": 20,
                 "cache_creation_input_tokens": 30}
        for missing, expected in ((None, (60, 20, 30)), ("input_tokens", (None, 20, 30)),
                                  ("cache_read_input_tokens", (None, None, 30)),
                                  ("cache_creation_input_tokens", (None, 20, None))):
            partial = {key: value for key, value in usage.items() if key != missing}
            with self.subTest(missing=missing):
                row = normalize("anthropic", partial)
                self.assertEqual((row["input"], row["read"], row["write"]), expected)

    def test_anthropic_missing_write_keeps_measured_noncached_cost(self):
        events = deepcopy(self.events)
        for event in events:
            if event["attempt_id"] == "child-a-1":
                event["usage"].pop("cache_creation_input_tokens", None)
        replay = self.replay(events)
        row = replay.rows(self.prices)["child-a-1"]
        self.assertEqual(row["known_cost"], Fraction(153, 1_000_000))
        self.assertEqual(row["read"], 10)
        for field in ("input", "write", "output", "total", "cost"):
            self.assertIsNone(row[field], field)
        self.assertEqual(set(row["unknown_reasons"]),
                         {"write:usage_unknown", "output:usage_unknown", "attempt_not_complete"})
        aggregate = replay.aggregate("root", self.prices)
        self.assertEqual(aggregate["known_cost"], Fraction(1009, 1_000_000))
        self.assertEqual(aggregate["input"], {"known": 340, "unknown": 1, "total": None})
        self.assertIsNone(aggregate["cost"])

    def test_anthropic_partial_bucket_matrix_prices_only_measured_parts(self):
        price = next(p for p in self.prices if p["provider"] == "anthropic")
        fields = ("input_tokens", "cache_read_input_tokens", "cache_creation_input_tokens")
        # 27 combinations, each with omitted and explicit-null unknowns.
        # Literal rates/expectations are independent of the oracle's price lookup.
        for parts in product((None, 0, 50), (None, 0, 10), (None, 0, 20)):
            for omit in (False, True):
                raw = {key: value for key, value in zip(fields, parts)
                       if value is not None or not omit}
                raw["output_tokens"] = 2
                with self.subTest(parts=parts, omit=omit):
                    usage = normalize("anthropic", raw)
                    result = estimate(usage, price, "completed")
                    known = Fraction(12, 1_000_000) + sum(
                        (value * rate / 1_000_000 for value, rate in
                         zip(parts, (Fraction(3), Fraction(3, 10), Fraction(15, 4)))
                         if value is not None), Fraction(0))
                    missing = {name + ":usage_unknown" for name, value in
                               zip(("input", "read", "write"), parts) if value is None}
                    self.assertEqual(result["known_cost"], known)
                    self.assertEqual(set(result["unknown_reasons"]), missing)
                    self.assertEqual(result["cost"], None if missing else known)
                    self.assertEqual(usage["input"], None if missing else sum(parts))
                    self.assertEqual(usage["total"], None if missing else sum(parts) + 2)
                    self.assertEqual((usage["read"], usage["write"]), parts[1:])
                    self.assertIsNone(usage["reasoning"])

    def test_anthropic_partial_missing_price_and_explicit_zero(self):
        price = deepcopy(next(p for p in self.prices if p["provider"] == "anthropic"))
        price["rates"]["input"] = None
        usage = normalize("anthropic", {"input_tokens": 50, "cache_read_input_tokens": 10,
                                        "output_tokens": 0})
        result = estimate(usage, price, "completed")
        self.assertEqual(result["known_cost"], Fraction(3, 1_000_000))
        self.assertEqual(set(result["unknown_reasons"]),
                         {"input:price_missing", "write:usage_unknown"})
        self.assertIsNone(result["cost"])
        for write in (None, 0):
            usage = normalize("anthropic", {"input_tokens": 0, "cache_read_input_tokens": 0,
                                            "cache_creation_input_tokens": write, "output_tokens": 0})
            result = estimate(usage, None, "completed")
            self.assertEqual(result["known_cost"], 0)
            self.assertEqual(result["unknown_reasons"], ["write:usage_unknown"] if write is None else [])
            self.assertEqual(result["cost"], None if write is None else 0)
            self.assertEqual(usage["input"], None if write is None else 0)

    def test_anthropic_partial_replay_preserves_replaces_and_completes_buckets(self):
        template = next(e for e in self.events if e["attempt_id"] == "child-a-1")
        patches = [{"input_tokens": 50},
                   {"input_tokens": None, "cache_read_input_tokens": 10},
                   {"input_tokens": 0, "cache_creation_input_tokens": None},
                   {"input_tokens": 60, "cache_creation_input_tokens": 20, "output_tokens": 2}]
        events = [{**deepcopy(template), "revision": i + 1,
                   "status": "completed" if i == 3 else "streaming", "usage": patch}
                  for i, patch in enumerate(patches)]
        replay = self.replay([])
        for i, expected in enumerate((150, 153, 3, 270)):
            replay.ingest([events[i], events[i]])
            # Reopen the partial JSON checkpoint before receiving the next patch.
            replay = self.replay(json.loads(json.dumps(replay.snapshot())))
            row = replay.rows(self.prices)["child-a-1"]
            self.assertEqual(row["known_cost"], Fraction(expected, 1_000_000))
            self.assertEqual(row["input"], 90 if i == 3 else None)
            self.assertEqual(row["total"], 92 if i == 3 else None)
            self.assertEqual(row["cost"], Fraction(270, 1_000_000) if i == 3 else None)
            self.assertNotIn("input:usage_unknown", row["unknown_reasons"])
        for order in permutations(events):
            with self.subTest(order=[e["revision"] for e in order]):
                self.assertEqual(self.replay(list(order) * 2).rows(self.prices), replay.rows(self.prices))

    def test_anthropic_provider_does_not_override_inclusive_wire_dialect(self):
        attempts = deepcopy(self.data["attempts"])
        attempt = next(a for a in attempts if a["attempt_id"] == "child-a-1")
        template = next(e for e in self.events if e["attempt_id"] == "child-a-1")
        for wire, prefix in (("responses", "input"), ("chat", "prompt")):
            attempt["wire"] = wire
            event = {**deepcopy(template), "status": "completed", "usage": {
                prefix + "_tokens": 50, prefix + "_tokens_details": {"cached_tokens": 10},
                "output_tokens" if wire == "responses" else "completion_tokens": 2}}
            with self.subTest(wire=wire):
                row = self.replay([event], attempts=attempts).rows(self.prices)["child-a-1"]
                self.assertEqual(row["input"], 50)
                self.assertIsNone(row["write"])
                self.assertEqual(row["known_cost"], Fraction(15, 1_000_000))
                self.assertEqual(set(row["unknown_reasons"]), {"input:usage_unknown", "write:usage_unknown"})
                self.assertIsNone(row["cost"])

    def test_anthropic_partial_noncached_measurement_is_validated(self):
        for value in (-1, True, 1.5, "50"):
            with self.subTest(value=value), self.assertRaisesRegex(ValueError, "invalid token count"):
                normalize("anthropic", {"input_tokens": value, "cache_read_input_tokens": 10})

    def test_anthropic_internal_pricing_count_does_not_extend_row_or_checkpoint(self):
        row_fields = {"input", "read", "write", "output", "reasoning", "total", "known_cost",
                      "cost", "unknown_reasons", "price_id", "billed"}
        for replay in (self.replay(), self.replay([])):
            before = replay.snapshot()
            for row in replay.rows(self.prices).values():
                self.assertEqual(set(row), row_fields)
            self.assertEqual(replay.snapshot(), before)

    def test_replay_rejects_invalid_partial_cache_before_later_valid_revision(self):
        for wire, prefix, field in (("chat", "prompt", "cached_tokens"),
                                    ("responses", "input", "cached_tokens"),
                                    ("responses", "input", "cache_write_tokens")):
            attempts = deepcopy(self.data["attempts"])
            attempts[0]["wire"] = wire
            first = deepcopy(self.events[0])
            first.update(revision=1, status="streaming", usage={prefix + "_tokens": 10})
            invalid = deepcopy(first)
            invalid.update(revision=2, usage={prefix + "_tokens_details": {field: 20}})
            repaired = deepcopy(first)
            repaired.update(revision=3, status="completed", usage={prefix + "_tokens": 30})
            with self.subTest(wire=wire, field=field), self.assertRaisesRegex(
                    ValueError, "cache exceeds inclusive input"):
                self.replay([repaired, invalid, first], attempts=attempts).rows(self.prices)

    def test_replay_valid_partial_cache_keeps_unmeasured_split_unknown(self):
        for wire, prefix, field, known, unknown in (
                ("chat", "prompt", "cached_tokens", "read", "write"),
                ("responses", "input", "cached_tokens", "read", "write"),
                ("responses", "input", "cache_write_tokens", "write", "read")):
            attempts = deepcopy(self.data["attempts"])
            attempts[0]["wire"] = wire
            first = deepcopy(self.events[0])
            first.update(revision=1, status="streaming", usage={prefix + "_tokens": 10})
            second = deepcopy(first)
            second.update(revision=2, status="completed", usage={prefix + "_tokens_details": {field: 4}})
            with self.subTest(wire=wire, field=field):
                row = self.replay([second, first], attempts=attempts).rows(self.prices)["root-1"]
                self.assertEqual((row["input"], row[known]), (10, 4))
                self.assertIsNone(row[unknown])
                self.assertIsNone(row["cost"])
                self.assertIn("input:usage_unknown", row["unknown_reasons"])

    def test_lineage_cycle_unknown_parent_and_unattributed_attempt_rejected(self):
        for problem in ("cycle", "parent", "owner"):
            threads = deepcopy(self.data["threads"])
            if problem == "cycle":
                threads["root"] = "child-a"
            elif problem == "parent":
                threads["root"] = "nonexistent"
            else:
                threads.pop("history")
            with self.subTest(problem=problem), self.assertRaises(ValueError):
                self.replay(threads=threads)

    def test_duplicate_attempt_and_bad_retry_or_request_owner_rejected(self):
        for problem in ("duplicate", "cycle", "missing", "request", "owner"):
            attempts = deepcopy(self.data["attempts"])
            if problem == "duplicate":
                attempts.append(deepcopy(attempts[0]))
            elif problem == "cycle":
                attempts[1]["retry_of"] = "child-a-2"
            elif problem == "missing":
                attempts[2]["retry_of"] = "nonexistent"
            elif problem == "request":
                attempts[2]["retry_of"] = "root-1"
            else:
                attempts[2]["thread_id"] = "root"
            with self.subTest(problem=problem), self.assertRaises(ValueError):
                self.replay(attempts=attempts)

    def test_reused_or_missing_provider_response_id_does_not_collapse_attempts(self):
        attempts = deepcopy(self.data["attempts"])
        attempts[1]["provider_response_id"] = attempts[2]["provider_response_id"]
        self.assertEqual(self.replay(attempts=attempts).aggregate("root", self.prices),
                         self.replay().aggregate("root", self.prices))

    def test_billed_and_estimated_cost_are_separate(self):
        row = self.replay().rows(self.prices)["root-1"]
        self.assertEqual(row["billed"], exact("0.000300"))
        self.assertEqual(row["cost"], exact("0.000220"))
        self.assertNotEqual(row["cost"], exact("0.000520"))

    def test_invoice_line_cannot_be_allocated_twice(self):
        attempts = deepcopy(self.data["attempts"])
        attempts[1]["billed"] = deepcopy(attempts[0]["billed"])
        with self.assertRaisesRegex(ValueError, "invoice line allocated twice"):
            self.replay(attempts=attempts).aggregate("root", self.prices)

    def test_allowance_and_api_balance_are_not_task_spend(self):
        before = self.replay().aggregate("root", self.prices)
        balance = self.data["corbanu_api_balance"]
        self.assertEqual(int(balance["balanceMicrousd"]) - int(balance["reservedMicrousd"]),
                         int(balance["availableMicrousd"]))
        for key in ("balance", "reserved", "available"):
            self.assertEqual(exact(balance[key + "Usd"]) * 1_000_000, int(balance[key + "Microusd"]))
        self.data["corbanu_api_balance"]["availableMicrousd"] = "999999999"
        self.data["provider_allowance"]["primary"]["used_percent"] = 100
        self.assertEqual(self.replay().aggregate("root", self.prices), before)
        self.assertIsNone(self.data["provider_allowance"]["credits"]["balance"])

    def test_no_observations_means_unknown_not_zero_usage(self):
        row = self.replay([]).rows(self.prices)["root-1"]
        self.assertIsNone(row["input"])
        self.assertIsNone(row["cost"])
        self.assertIn("attempt_not_complete", row["unknown_reasons"])

    def test_explicit_zero_is_distinct_from_omission(self):
        events = [deepcopy(self.events[0])]
        events[0]["usage"] = {"input_tokens": 0, "input_tokens_details": {"cached_tokens": 0, "cache_write_tokens": 0},
                              "output_tokens": 0, "output_tokens_details": {"reasoning_tokens": 0}, "total_tokens": 0}
        row = self.replay(events).rows([])["root-1"]
        self.assertEqual(row["cost"], 0)
        self.assertEqual(row["total"], 0)

    def test_partial_cache_price_does_not_hide_missing_output_price(self):
        prices = deepcopy(self.prices)
        prices[1]["rates"]["output"] = None
        row = self.replay().rows(prices)["root-1"]
        self.assertEqual(row["known_cost"], exact("0.000140"))
        self.assertIsNone(row["cost"])
        self.assertIn("output:price_missing", row["unknown_reasons"])

    def test_invalid_money_and_exact_submicro_precision(self):
        for value in ("-1", "NaN", "Infinity", "not-a-price", 0.1):
            with self.subTest(value=value), self.assertRaises(ValueError):
                exact(value)
        self.assertEqual(exact("0.1") + exact("0.2"), exact("0.3"))
        self.assertEqual(exact("0.0000001") * 3, Fraction(3, 10_000_000))

    def test_unknown_wire_is_not_assumed_to_match_anthropic(self):
        with self.assertRaisesRegex(ValueError, "unsupported wire"):
            normalize("anthropic-compatible-unknown", {"input_tokens": 100})

    def test_unknown_event_attempt_invalid_revision_and_missing_provenance_rejected(self):
        for field, value in (("attempt_id", "missing"), ("revision", 0), ("revision", True), ("provenance", "")):
            event = deepcopy(self.events[0])
            event[field] = value
            with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                self.replay([event])


if __name__ == "__main__":
    unittest.main()
