#!/usr/bin/env python3
"""Sync-branch CI mode (upstreamSyncPlan.md section 4): apply after the skeleton commit, revert in S4.

  ci_mode.py apply  <worktree>   heavy blocking-ci jobs skip PRs whose base is sync/**, other PR workflows ignore
                                 sync/** bases, and sync-package-ci.yml runs package-scoped clippy/tests on pushes
                                 to sync-ci/**
  ci_mode.py revert <worktree>   remove exactly what apply added (every added line ends with the TAG comment)
  ci_mode.py check  <worktree>   exit 1 if any CI-mode line or file is present (S5 entry check)
  ci_mode.py codeowners apply|revert|check|drift <worktree> [@owner]
                                 sync-branch CODEOWNERS block for the named security paths, the
                                 dependency-policy files and main's D318 paths (reverted after main
                                 merge #3, as the last sync-branch commit before the final PR); drift
                                 exits 1 if origin/main's D318 block names a pattern the block lacks,
                                 the block isn't last, or (given @owner) another owner appears

Edits are textual so they apply on top of whatever Tier 0 resolution the skeleton produced.
"""

import os
import re
import shutil
import subprocess
import sys

TAG = "# sync-ci-mode"
RESTORE = "# sync-ci-mode-restore {}"
HEAVY = ("bazel", "rust-ci", "sdk")
SKIP_IF = "${{ !startsWith(github.base_ref, 'sync/') }}"
OK_ENV = (
    "${{ startsWith(github.base_ref, 'sync/') && '" + ",".join(HEAVY) + "' || '' }}"
)
# Whole-workspace PR workflows that can't pass while the tree has markers. The security lanes
# (credential-boundary-canary, security-ingress-contract, security-platform-contract, windows-security-probes)
# and v8-canary keep running on sync PRs and can also be dispatched on the sync branch.
PR_WORKFLOWS = ("orchestrate-tui-matrix.yml", "tmux-smoke.yml")
HERE = os.path.dirname(os.path.abspath(__file__))
NEW_FILES = {
    ".github/workflows/sync-package-ci.yml": os.path.join(
        HERE, "ci_mode", "sync-package-ci.yml"
    )
}
CHECK_SCRIPT = ".github/scripts/check_ci_results.py"
CHECK_OLD = '        if dependency["result"] != "success"\n'
CHECK_NEW = (
    '        if dependency["result"] != "success"\n'
    '        and not (dependency["result"] == "skipped" and name in os.environ.get("SYNC_SKIPPED_OK", "")'
    '.split(","))  ' + TAG + "\n"
)


def edit(path, fn):
    text = open(path).read()
    new = fn(text)
    if new == text:
        sys.exit(
            f"error: no change applied to {path}; update ci_mode.py for the resolved file"
        )
    open(path, "w").write(new)


def blocking(text):
    for job in HEAVY:
        text, n = re.subn(
            rf"(?m)^(  {re.escape(job)}:\n    name: [^\n]*\n)",
            lambda m: m.group(1) + f"    if: {SKIP_IF}  {TAG}\n",
            text,
        )
        if n != 1:
            sys.exit(f"error: blocking-ci job {job} not found")
    text, n = re.subn(
        r"(?m)^(        env:\n          NEEDS: \$\{\{ toJSON\(needs\) \}\}\n)",
        lambda m: m.group(1) + f"          SYNC_SKIPPED_OK: {OK_ENV}  {TAG}\n",
        text,
    )
    if n != 1:
        sys.exit("error: blocking-ci required-job env not found")
    return text


def pr_ignore(text):
    m = re.search(r"(?m)^  pull_request:(?P<rest>[^\n]*)\n", text)
    if not m:
        sys.exit("error: no pull_request trigger found")
    block = []
    for line in text[m.end() :].splitlines():
        if line.strip() and not line.startswith("    "):
            break
        block.append(line)
    if any(line.startswith("    branches:") for line in block):
        return text  # already restricted to listed bases
    line = f'    branches-ignore: ["sync/**"]  {TAG}\n'
    if m.group("rest").strip() == "{}":
        return (
            text[: m.start()] + f"  pull_request:  {RESTORE}\n" + line + text[m.end() :]
        )
    return text[: m.end()] + line + text[m.end() :]


def apply(wt):
    edit(os.path.join(wt, ".github/workflows/blocking-ci.yml"), blocking)
    edit(os.path.join(wt, CHECK_SCRIPT), lambda t: t.replace(CHECK_OLD, CHECK_NEW, 1))
    for wf in PR_WORKFLOWS:
        p = os.path.join(wt, ".github/workflows", wf)
        if os.path.exists(p):
            edit(p, pr_ignore)
    for dst, src in NEW_FILES.items():
        shutil.copyfile(src, os.path.join(wt, dst))
    print("CI mode applied")


def revert(wt):
    for root in (".github/workflows", ".github/scripts"):
        for name in os.listdir(os.path.join(wt, root)):
            p = os.path.join(wt, root, name)
            if not os.path.isfile(p):
                continue
            text = open(p).read()
            if TAG not in text or os.path.join(root, name) in NEW_FILES:
                continue
            out = [
                line
                for line in text.splitlines(keepends=True)
                if not line.rstrip("\n").endswith(TAG)
            ]
            open(p, "w").write(
                "".join(out).replace(
                    f"  pull_request:  {RESTORE}\n", "  pull_request: {}\n"
                )
            )
    for dst in NEW_FILES:
        p = os.path.join(wt, dst)
        if os.path.exists(p):
            os.remove(p)
    print("CI mode reverted")


def check(wt):
    hits = []
    for root in (".github/workflows", ".github/scripts"):
        for name in os.listdir(os.path.join(wt, root)):
            p = os.path.join(wt, root, name)
            if os.path.isfile(p) and TAG in open(p).read():
                hits.append(os.path.join(root, name))
    if hits:
        sys.exit("CI mode still present: " + ", ".join(hits))
    print("no CI-mode lines")


def codeowners_pattern(spec):
    """Git pathspec -> CODEOWNERS pattern. In CODEOWNERS `*` stops at `/`, so a leading wildcard in the last
    segment (e.g. tui/src/*approval_overlay*) must become `**/`."""
    head, _, last = spec.rpartition("/")
    if last.startswith("*"):
        return f"/{head}/**/{last}"
    return f"/{spec}"


CO_BEGIN = (
    "# BEGIN sync-branch security owners (upstream sync; revert before the final PR)"
)
CO_END = "# END sync-branch security owners"
# Dependency policy: a slice PR must not change these without the security owner (SG1). A new third-party crate
# always changes a lockfile; per-crate Cargo.toml files are not listed, so the S5 SG1 re-run remains the backstop.
DEPENDENCY_POLICY = (
    "codex-rs/Cargo.toml",
    "codex-rs/Cargo.lock",
    "codex-rs/deny.toml",
    "codex-rs/.cargo/audit.toml",
    "MODULE.bazel",
    "MODULE.bazel.lock",
    "pnpm-lock.yaml",
    ".github/workflows/cargo-deny.yml",
)
# Paths `main` assigns to @corbanu-security (D318, #1914) that the named list above doesn't already cover, as
# CODEOWNERS patterns. Keeps the sync branch's coverage a superset of `main`'s until main's own block arrives
# with a main merge. Update when main's block changes.
MAIN_D318 = (
    "/.github/CODEOWNERS",
    "/codex-rs/bwrap/",
    "/codex-rs/cli/src/debug_sandbox/",
    "/codex-rs/cli/src/debug_sandbox.rs",
    "/codex-rs/core/src/sandboxing/",
    "/codex-rs/core/src/exec_env.rs",
    "/codex-rs/core/src/windows_sandbox*",
    "/codex-rs/exec-server/src/fs_sandbox.rs",
    "/codex-rs/exec-server/src/process_sandbox.rs",
    "/codex-rs/exec-server/src/sandboxed_file_system.rs",
    "/codex-rs/protocol/src/secretless_launch.rs",
    "/codex-rs/shell-command/src/command_safety/",
    "/codex-rs/core/src/tools/approvals.rs",
    "/codex-rs/core/src/tools/handlers/request_permissions.rs",
    "/codex-rs/core/src/state/granted_permissions.rs",
    "/codex-rs/core/src/tools/handlers/mod.rs",
    "/codex-rs/hooks/src/events/permission_request.rs",
    "/codex-rs/telegram/src/approvals.rs",
    "/codex-rs/telegram/src/auth.rs",
    "/codex-rs/app-server-transport/src/transport/auth.rs",
    "/codex-rs/protocol/src/permissions.rs",
    "/codex-rs/protocol/src/permissions/",
    "/codex-rs/core/src/config/resolved_permission_profile.rs",
    "/codex-rs/core/src/config/permission_profile_catalog.rs",
    "/codex-rs/config/src/permissions_toml.rs",
    "/codex-rs/config/src/requirements_layers/",
    "/codex-rs/config/src/security_state.rs",
    "/codex-rs/tui/src/security/",
    "/codex-rs/cli/src/nested_security.rs",
    "/codex-rs/vault/",
    "/codex-rs/keyring-store/",
    "/codex-rs/wallet/",
    "/codex-rs/wallet-daemon/",
    "/codex-rs/login/src/auth/storage*",
    "/codex-rs/login/src/auth/provider_key_vault.rs",
    "/codex-rs/core/src/config/auth_keyring.rs",
    "/codex-rs/core/src/config/network_proxy_credential*",
    "/codex-rs/core/src/model_broker_auth*",
    "/codex-rs/model-provider/src/model_key_broker.rs",
)
# Upstream moved code that main owns to paths main's block doesn't name yet. Carry these into main's
# D318 block in the final sync PR.
SYNC_SUCCESSORS = (
    # main: /codex-rs/app-server-transport/src/transport/auth.rs (upstream #47447 moved it here)
    "/codex-rs/websocket-auth/",
    # main: /codex-rs/core/src/guardian/policy*.md (upstream #41477 moved the guardian prompts here)
    "/codex-rs/prompts/templates/guardian/",
    # main: /codex-rs/core/src/guardian/ (upstream #44227, #44252, #44544 moved the review engine here)
    "/codex-rs/ext/guardian-reviewer/",
    # main: /codex-rs/core/src/guardian/prompt.rs (upstream #46026 moved prompt assembly here)
    "/codex-rs/prompts/src/guardian_instructions.rs",
    "/codex-rs/prompts/src/model_messages/guardian.rs",
    # New upstream crate (#41392: guardian authorization, enforcement, trusted tools); not a move, owned
    # here pending the security owner's decision for main.
    "/codex-rs/guardian-context/",
)
MAIN_BLOCK_BEGIN = "# BEGIN Corbanu security owners"
MAIN_BLOCK_END = "# END Corbanu security owners"


def codeowners(action, wt, owner=None):
    if action not in ("apply", "revert", "check", "drift"):
        sys.exit(f"error: unknown codeowners action {action!r}")
    sys.path.insert(0, HERE)
    import footprint

    p = os.path.join(wt, ".github", "CODEOWNERS")
    text = open(p).read() if os.path.exists(p) else ""
    start, end = text.find(CO_BEGIN), text.find(CO_END)
    if (start == -1) != (end == -1) or end < start:
        sys.exit("error: CODEOWNERS block markers are incomplete; fix by hand")
    if action == "drift":
        # Every pattern in main's D318 block must be in the sync block; run after fetching origin/main.
        main = subprocess.run(
            ["git", "-C", wt, "show", "origin/main:.github/CODEOWNERS"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
        m_start, m_end = main.find(MAIN_BLOCK_BEGIN), main.find(MAIN_BLOCK_END)
        if m_start == -1 or m_end < m_start or start == -1:
            sys.exit("error: main's D318 block or the sync block is missing")
        if text[end + len(CO_END) :].strip():
            sys.exit("error: the sync block is not last in CODEOWNERS")
        owners = {
            line.split()[1]
            for line in text[start:end].splitlines()
            if line.startswith("/")
        }
        if owner and owners != {owner}:
            sys.exit(f"error: sync block owners {sorted(owners)} != {owner}")
        owned = {
            line.split()[0]
            for line in text[start:end].splitlines()
            if line.startswith("/")
        }
        missing = [
            line.split()[0]
            for line in main[m_start:m_end].splitlines()
            if line.startswith("/") and line.split()[0] not in owned
        ]
        if missing:
            sys.exit("sync block misses main's D318 patterns: " + ", ".join(missing))
        print("sync block covers main's D318 block")
        return
    if action == "check":
        if start != -1:
            sys.exit("CODEOWNERS block still present")
        print("no CODEOWNERS block")
        return
    if start != -1:
        text = text[:start].rstrip("\n") + "\n" + text[end + len(CO_END) :].lstrip("\n")
    if action == "apply":
        if not owner or not re.fullmatch(
            r"@[A-Za-z0-9][A-Za-z0-9-]*(/[A-Za-z0-9._-]+)?", owner
        ):
            sys.exit("error: codeowners apply needs an @owner")
        # Last matching pattern wins, so this block goes at the end.
        patterns = [
            codeowners_pattern(spec) for spec in [*footprint.NAMED, *DEPENDENCY_POLICY]
        ]
        patterns += [p for p in MAIN_D318 + SYNC_SUCCESSORS if p not in patterns]
        block = [CO_BEGIN] + [f"{p} {owner}" for p in patterns] + [CO_END]
        text = text.rstrip("\n") + "\n\n" + "\n".join(block) + "\n"
    open(p, "w").write(text)
    print(f"CODEOWNERS block {'added' if action == 'apply' else 'removed'}")


if __name__ == "__main__":
    if sys.argv[1] == "codeowners":
        codeowners(sys.argv[2], sys.argv[3], sys.argv[4] if len(sys.argv) > 4 else None)
    else:
        actions = {"apply": apply, "revert": revert, "check": check}
        if sys.argv[1] not in actions:
            sys.exit(f"error: unknown action {sys.argv[1]!r}")
        actions[sys.argv[1]](sys.argv[2])
