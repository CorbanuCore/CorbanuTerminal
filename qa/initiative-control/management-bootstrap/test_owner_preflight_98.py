"""Execute the documented preparation/migration on disposable state only."""
import copy
from pathlib import Path
import unittest
from unittest.mock import patch

import fable_launcher as f
import owner_tmux
import owner_promotion_94_preflight as gate
import test_owner_promotion_94 as previous


class Preflight98Tests(unittest.TestCase):
    setUp = previous.PromotionTests.setUp
    tearDown = previous.PromotionTests.tearDown
    installation = previous.PromotionTests.installation
    install = previous.PromotionTests.install
    arm = previous.PromotionTests.arm
    sql = previous.PromotionTests.sql
    prepare = previous.PromotionTests.prepare
    evaluate = previous.PromotionTests.evaluate

    def test_documented_adapter_emits_gate_record_and_preserves_frozen_inputs(self):
        self.prepare()
        source = Path(__file__).with_name("owner-preflight-98.md").read_text()
        block = source.split("```python\n# owner-preflight-98: worked preparation adapter\n", 1)[1]
        block = block.split("```", 1)[0]
        namespace = {}
        exec(compile(block, "owner-preflight-98.md", "exec"), namespace)
        before = self.c.snapshot()
        original = copy.deepcopy(before["actions"]["old"]["inputs"])
        record = self.manifest["preparation"]
        frozen, preparation = namespace["prepare_for_registration"](
            original, provider=record["provider"], policy=record["policy"],
            authority=record["authority"], adoption=record["adoption"])
        self.assertEqual(record, preparation)
        self.assertEqual(before["actions"]["new"]["inputs"], frozen)
        self.assertEqual(before["actions"]["old"]["inputs"], original)
        self.assertEqual(before, self.c.snapshot())
        self.manifest["preparation"] = preparation
        self.assertTrue(self.evaluate()["ok"])
        # A changed evidence file is not silently repinned by the adapter.
        Path(record["adoption"]["path"]).write_text("changed fixture evidence")
        with self.assertRaisesRegex(f.LaunchError, "evidence_digest_mismatch"):
            namespace["prepare_for_registration"](
                original, provider=record["provider"], policy=record["policy"],
                authority=record["authority"], adoption=record["adoption"])

    def test_missing_sidecar_can_be_added_after_real_fixture_replacement(self):
        self.prepare()
        before = self.c.snapshot()
        replacement = self.manifest.pop("replacements")
        result = self.evaluate()
        self.assertEqual("replacement_evidence_for_each_action_required",
                         result["results"][1]["reason"])
        self.manifest["replacements"] = replacement
        self.assertTrue(self.evaluate()["ok"])
        self.assertEqual(before, self.c.snapshot())
        old, new = before["actions"]["old"], before["actions"]["new"]
        self.assertEqual("cancelled", old["status"])
        self.assertTrue(old["owner_cancellation"])
        self.assertNotEqual(old["allocation_digest"], new["allocation_digest"])
        self.assertNotEqual(old["manager_run"], new["manager_run"])

    def test_sidecar_does_not_retrofit_a_flat_prepared_action(self):
        self.prepare()
        before = self.c.snapshot()
        # Model the old pre-migration state without writing to the fixture DB.
        state = copy.deepcopy(before)
        state["actions"]["new"]["inputs"].pop("worker")
        context = (None, None, None, {"worktrees": [str(self.root)]}, self.c)
        with patch.object(self.c, "snapshot", return_value=state):
            with self.assertRaisesRegex(f.LaunchError, "action_not_bound_by_authorized_bridge"):
                gate.allocations(self.manifest, context, ["new"])
        self.assertEqual(before, self.c.snapshot())

    def test_resolved_tmux_passes_without_following_or_opening_auth(self):
        self.prepare()
        link = Path(self.tmp.name) / "tmux-link"
        link.symlink_to(self.transport["tmux"])
        transport = dict(self.transport, tmux=str(link))
        with self.assertRaisesRegex(f.LaunchError, "symlink_path"):
            owner_tmux.validate(transport)
        transport["tmux"] = str(link.resolve(strict=True))
        # A nonexistent auth.json deliberately passes structural validation;
        # this establishes why it is not the symlink error OR isolation proof.
        auth = Path(self.tmp.name) / "never-created/auth.json"
        transport["auth_link"] = str(auth)
        self.assertFalse(auth.exists())
        self.assertEqual(transport, owner_tmux.validate(transport))
        self.assertFalse(auth.exists())


if __name__ == "__main__":
    unittest.main()
