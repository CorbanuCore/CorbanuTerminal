import importlib.util
import os
import tempfile
import unittest
from pathlib import Path


CHECKER_PATH = Path(__file__).resolve().parents[1] / "check.py"
SPEC = importlib.util.spec_from_file_location("sprint_checker", CHECKER_PATH)
assert SPEC and SPEC.loader
checker = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(checker)


def sprint_text(
    *,
    status="draft",
    feature="PF-01",
    plan_file="docs/plans/proposed/plan.md",
    worktree="UNALLOCATED",
    branch="UNALLOCATED",
    base_commit="UNALLOCATED",
    lane="",
    write_scope="",
):
    return f"""---
sprint_id: "PF-01-S01"
title: "One task"
status: {status}
plan_file: "{plan_file}"
plan_feature: "{feature}"
execution_order: 1
owner: "Owner"
lane: "{lane}"
write_scope: "{write_scope}"
worktree: "{worktree}"
branch: "{branch}"
base_commit: "{base_commit}"
depends_on: "none"
created: 2026-08-23
updated: 2026-08-23
---

# PF-01-S01 — One task

## Execution mandate

- Deliver: one task

## Plan linkage

- Feature: `PF-01`

## Code boundaries

- Existing: `code.rs`

## Preconditions

- [ ] Plan active.

## Done

- [x] Sprint contract drafted.

## Remaining

- [ ] Implement one task.

## Verification

- [ ] Focused test passes.

## Exit evidence

- [ ] Commit recorded.
"""


class SprintCheckerTests(unittest.TestCase):
    def parallel_repo(self, temporary, count=2, limit=3):
        repo, root, first = self.make_repo(temporary, plan_status="active")
        plan = repo / "docs/plans/proposed/plan.md"
        front = [
            "---",
            "status: active",
            f"max_active_sprints: {limit}",
            "integration_owner: Owner",
            "implementation_worktrees:",
        ]
        links = []
        paths = []
        for number in range(1, count + 1):
            sprint_id = f"PF-01-S{number:02}"
            path = first.with_name(f"{sprint_id.lower()}-one-task.md")
            worktree = f"/tmp/lane-{number}"
            branch = f"codex/lane-{number}"
            front.extend(
                [
                    f'  - path: "{worktree}"',
                    f'    branch: "{branch}"',
                    f'    base_commit: "{"a" * 40}"',
                ]
            )
            text = sprint_text(
                status="in_progress",
                worktree=worktree,
                branch=branch,
                base_commit="a" * 40,
                lane=f"lane-{number}",
                write_scope=f"src/lane-{number}",
            )
            text = text.replace("PF-01-S01", sprint_id).replace(
                "execution_order: 1", f"execution_order: {number}"
            )
            path.write_text(text, encoding="utf-8")
            links.append(Path(os.path.relpath(path, plan.parent)).as_posix())
            paths.append(path)
        plan.write_text("\n".join(front + ["---", "PF-01"] + links), encoding="utf-8")
        return repo, root, plan, paths

    def make_repo(
        self,
        temporary,
        *,
        backlink=True,
        plan_status="draft",
        plan_worktree=None,
    ):
        repo = Path(temporary)
        sprint_root = repo / "docs" / "sprints"
        sprint_path = sprint_root / "current" / "plan" / "pf-01-s01-one-task.md"
        sprint_path.parent.mkdir(parents=True)
        (sprint_root / "archive").mkdir(parents=True)
        plan_path = repo / "docs" / "plans" / "proposed" / "plan.md"
        plan_path.parent.mkdir(parents=True)
        link = Path(os.path.relpath(sprint_path, plan_path.parent)).as_posix()
        worktree_front = ""
        if plan_worktree is not None:
            path, branch, base_commit = plan_worktree
            worktree_front = (
                "implementation_worktrees:\n"
                f'  - path: "{path}"\n'
                f'    branch: "{branch}"\n'
                f'    base_commit: "{base_commit}"\n'
            )
        plan_path.write_text(
            f"---\nstatus: {plan_status}\n{worktree_front}---\n\n# Plan\n\nPF-01\n"
            + (f"\n[{link}]({link})\n" if backlink else ""),
            encoding="utf-8",
        )
        return repo, sprint_root, sprint_path

    def test_valid_draft_sprint_passes(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo, root, sprint = self.make_repo(temporary)
            sprint.write_text(sprint_text(), encoding="utf-8")
            result = checker.check_sprints(root, repo)
            self.assertTrue(result["ok"], result["errors"])
            self.assertEqual(result["current_count"], 1)

    def test_sprint_must_link_exactly_one_feature(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo, root, sprint = self.make_repo(temporary)
            sprint.write_text(sprint_text(feature="PF-01,PF-02"), encoding="utf-8")
            result = checker.check_sprints(root, repo)
            self.assertFalse(result["ok"])
            self.assertTrue(
                any("exactly one PF-NN" in error for error in result["errors"])
            )

    def test_plan_backlink_is_required(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo, root, sprint = self.make_repo(temporary, backlink=False)
            sprint.write_text(sprint_text(), encoding="utf-8")
            result = checker.check_sprints(root, repo)
            self.assertFalse(result["ok"])
            self.assertTrue(
                any("missing sprint backlink" in error for error in result["errors"])
            )

    def test_ready_sprint_requires_active_plan_and_worktree(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo, root, sprint = self.make_repo(temporary)
            sprint.write_text(sprint_text(status="ready"), encoding="utf-8")
            result = checker.check_sprints(root, repo)
            self.assertFalse(result["ok"])
            self.assertTrue(
                any("requires an active plan" in error for error in result["errors"])
            )
            self.assertTrue(
                any(
                    "requires an exact absolute worktree" in error
                    for error in result["errors"]
                )
            )

    def test_ready_sprint_coordinates_must_match_active_plan(self):
        with tempfile.TemporaryDirectory() as temporary:
            expected = ("/tmp/plan-worktree", "feat/security", "a" * 40)
            repo, root, sprint = self.make_repo(
                temporary,
                plan_status="active",
                plan_worktree=expected,
            )
            sprint.write_text(
                sprint_text(
                    status="ready",
                    worktree="/tmp/other-worktree",
                    branch=expected[1],
                    base_commit=expected[2],
                ),
                encoding="utf-8",
            )
            result = checker.check_sprints(root, repo)
            self.assertFalse(result["ok"])
            self.assertTrue(any("do not match" in error for error in result["errors"]))

    def test_ready_sprint_with_matching_active_plan_passes(self):
        with tempfile.TemporaryDirectory() as temporary:
            coordinates = ("/tmp/plan-worktree", "feat/security", "a" * 40)
            repo, root, sprint = self.make_repo(
                temporary,
                plan_status="active",
                plan_worktree=coordinates,
            )
            sprint.write_text(
                sprint_text(
                    status="ready",
                    worktree=coordinates[0],
                    branch=coordinates[1],
                    base_commit=coordinates[2],
                ),
                encoding="utf-8",
            )
            result = checker.check_sprints(root, repo)
            self.assertTrue(result["ok"], result["errors"])

    def test_three_independent_active_sprints_pass(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo, root, _, _ = self.parallel_repo(temporary, count=3)
            result = checker.check_sprints(root, repo)
            self.assertTrue(result["ok"], result["errors"])

    def test_default_limit_still_one(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo, root, plan, _ = self.parallel_repo(temporary)
            plan.write_text(plan.read_text().replace("max_active_sprints: 3\n", ""))
            result = checker.check_sprints(root, repo)
            self.assertTrue(
                any("exceed max_active_sprints=1" in e for e in result["errors"])
            )

    def test_blocked_sprint_keeps_slot(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo, root, _, paths = self.parallel_repo(temporary, count=4)
            paths[-1].write_text(
                paths[-1].read_text().replace("status: in_progress", "status: blocked")
            )
            result = checker.check_sprints(root, repo)
            self.assertTrue(
                any("4 active sprints exceed" in e for e in result["errors"])
            )

    def test_ready_sprint_does_not_reserve_slot(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo, root, _, paths = self.parallel_repo(temporary, count=4)
            paths[-1].write_text(
                paths[-1].read_text().replace("status: in_progress", "status: ready")
            )
            result = checker.check_sprints(root, repo)
            self.assertTrue(result["ok"], result["errors"])

    def test_invalid_limit_and_missing_integration_owner(self):
        for limit in ("0", "4", "many"):
            with self.subTest(limit=limit), tempfile.TemporaryDirectory() as temporary:
                repo, root, _, _ = self.parallel_repo(temporary, count=1, limit=limit)
                result = checker.check_sprints(root, repo)
                self.assertTrue(
                    any("must be 1, 2, or 3" in e for e in result["errors"])
                )
        with tempfile.TemporaryDirectory() as temporary:
            repo, root, plan, _ = self.parallel_repo(temporary)
            plan.write_text(plan.read_text().replace("integration_owner: Owner\n", ""))
            result = checker.check_sprints(root, repo)
            self.assertTrue(
                any("requires integration_owner" in e for e in result["errors"])
            )

    def test_lane_worktree_branch_and_scope_collisions(self):
        for key, before, after, expected in (
            ("lane", "lane-2", "lane-1", "lane collision"),
            ("worktree", "/tmp/lane-2", "/tmp/lane-1", "worktree collision"),
            ("branch", "codex/lane-2", "codex/lane-1", "branch collision"),
            (
                "write_scope",
                "src/lane-2",
                "src/lane-1/child.rs",
                "write_scope overlaps",
            ),
        ):
            with self.subTest(key=key), tempfile.TemporaryDirectory() as temporary:
                repo, root, _, paths = self.parallel_repo(temporary)
                paths[1].write_text(
                    paths[1]
                    .read_text()
                    .replace(f'{key}: "{before}"', f'{key}: "{after}"')
                )
                result = checker.check_sprints(root, repo)
                self.assertTrue(
                    any(expected in e for e in result["errors"]), result["errors"]
                )

    def test_literal_scope_validation_and_prefix_semantics(self):
        for scope in (
            ".",
            "../src",
            "/src",
            "src/../lib",
            "src/*.rs",
            "src\\lib",
            "C:/src",
            "UNALLOCATED",
        ):
            with self.subTest(scope=scope):
                self.assertFalse(checker.valid_scope(scope))
        self.assertTrue(checker.scopes_overlap("src/core", "SRC/Core/file.rs"))
        self.assertTrue(checker.scopes_overlap("src/core/file.rs", "src/core"))
        self.assertFalse(checker.scopes_overlap("src/core", "src/core-extra"))

    def test_missing_concurrent_metadata_is_rejected(self):
        for line in ('lane: "lane-1"\n', 'write_scope: "src/lane-1"\n'):
            with self.subTest(line=line), tempfile.TemporaryDirectory() as temporary:
                repo, root, _, paths = self.parallel_repo(temporary, count=1)
                paths[0].write_text(paths[0].read_text().replace(line, ""))
                result = checker.check_sprints(root, repo)
                self.assertFalse(result["ok"])
                self.assertTrue(
                    any(
                        "executable concurrent sprint requires" in e
                        for e in result["errors"]
                    )
                )

    def test_cycles_detected_in_drafts(self):
        for self_cycle in (False, True):
            with (
                self.subTest(self_cycle=self_cycle),
                tempfile.TemporaryDirectory() as temporary,
            ):
                repo, root, _, paths = self.parallel_repo(temporary)
                for index, path in enumerate(paths):
                    dependency = (
                        "PF-01-S01" if self_cycle or index == 1 else "PF-01-S02"
                    )
                    path.write_text(
                        path.read_text()
                        .replace("status: in_progress", "status: draft")
                        .replace('depends_on: "none"', f'depends_on: "{dependency}"')
                    )
                result = checker.check_sprints(root, repo)
                self.assertTrue(any("dependency cycle" in e for e in result["errors"]))

    def test_parallelism_does_not_waive_dependency_completion(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo, root, _, paths = self.parallel_repo(temporary)
            paths[1].write_text(
                paths[1]
                .read_text()
                .replace('depends_on: "none"', 'depends_on: "PF-01-S01"')
            )
            result = checker.check_sprints(root, repo)
            self.assertTrue(
                any("not completed and archived" in e for e in result["errors"])
            )

    def test_completed_dependency_with_later_display_order_is_valid(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo, root, _, paths = self.parallel_repo(temporary)
            paths[0].write_text(
                paths[0]
                .read_text()
                .replace('depends_on: "none"', 'depends_on: "PF-01-S02"')
            )
            archived = root / "archive" / paths[1].name
            paths[1].rename(archived)
            archived.write_text(
                archived.read_text()
                .replace("status: in_progress", "status: completed")
                .replace("- [ ]", "- [x]")
            )
            result = checker.check_sprints(root, repo)
            self.assertTrue(result["ok"], result["errors"])

    def test_cancelled_dependency_is_not_completion(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo, root, _, paths = self.parallel_repo(temporary)
            paths[0].write_text(
                paths[0]
                .read_text()
                .replace('depends_on: "none"', 'depends_on: "PF-01-S02"')
            )
            archived = root / "archive" / paths[1].name
            paths[1].rename(archived)
            archived.write_text(
                archived.read_text().replace("status: in_progress", "status: cancelled")
            )
            result = checker.check_sprints(root, repo)
            self.assertTrue(
                any("not completed and archived" in e for e in result["errors"])
            )

    def test_cross_plan_collisions_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo, root, plan, paths = self.parallel_repo(temporary)
            second_plan = plan.with_name("second.md")
            second_plan.write_text(
                plan.read_text().replace(
                    "max_active_sprints: 3", "max_active_sprints: 1"
                )
            )
            paths[1].write_text(
                paths[1]
                .read_text()
                .replace("docs/plans/proposed/plan.md", "docs/plans/proposed/second.md")
                .replace('write_scope: "src/lane-2"', 'write_scope: "src/lane-1"')
            )
            result = checker.check_sprints(root, repo)
            self.assertTrue(any("write_scope overlaps" in e for e in result["errors"]))


if __name__ == "__main__":
    unittest.main()
