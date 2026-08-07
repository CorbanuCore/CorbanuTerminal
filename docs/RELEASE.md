# PfTerminal Release Process

This document is the release runbook. Commands assume the canonical repository
is the current directory and the operator has authenticated `git` and `gh`.
A release is never merged or published from an unreviewed local tree.

## Version and release identity

PfTerminal uses semantic versions, including optional `-alpha[.N]` or
`-beta[.N]` prerelease suffixes.

| Item | Value for version `X.Y.Z` |
| --- | --- |
| Workspace version | `version = "X.Y.Z"` in `codex-rs/Cargo.toml` |
| Git tag | `rust-vX.Y.Z` |
| GitHub release name | `X.Y.Z` |
| Prerelease flag | Set automatically when the version contains `-` |

The release workflow rejects an input that does not match the Cargo workspace
version. Always pass `release_version` explicitly; do not rely on the
workflow's checked-in default.

## Source branch

Cut releases from current `main`. A short-lived `release/X.Y.Z` branch may
hold the version bump and QA evidence, but it must start at `origin/main` and
return through a reviewed PR. Actual release tags point to the reviewed commit
on `main`.

Version 0.1.27 was an exceptional reconstruction: work converged through
`release/0.1.27-reconstruction`. That branch and `main` both point to
`dc15684c4` today. The reconstruction is historical precedent for recovery,
not the normal release path.

## Build and integrity model

`.github/workflows/pfterminal-release.yml` builds release-profile binaries and
packages them with `scripts/build_codex_package.py`.

- macOS: Apple Silicon and Intel tar archives, plus DMGs built by
  `scripts/install/build_macos_dmg.sh`.
- Linux: x86_64 GNU and aarch64 musl tar archives.
- Windows: x86_64 MSVC ZIP archive.
- Packaged entry points include `pfterminal`, `pfterminal-debug`,
  `pfterminal-walletd`, and the platform-specific helpers selected by the
  workflow.

Every archive is unpacked and smoke-tested. The assembled bundle must contain
five package archives and two DMGs. It also includes both installers and
SHA-256 manifests for packages and DMGs.

**Current signing status:** the release workflow does not code-sign or notarize
the macOS, Linux, or Windows deliverables, and it does not create a signed Git
tag. The repository contains a reusable Azure Trusted Signing action for
Windows, but the PfTerminal release workflow does not invoke it. SHA-256
manifests provide integrity checking, not publisher authentication. Do not
describe current assets as signed. Adding signing requires a separately
reviewed workflow change and keyless/secret handling review.

## Feature-manifest gate

The feature manifest inventories a Git tree rather than a mutable worktree. It
records entry points, slash commands and dispatch bindings, configuration,
migrations, model catalogue, app-server methods, platform artifacts, protected
integrations, and source paths.

For each release:

1. Build a baseline manifest from the previous release tag.
2. Build the candidate manifest from the exact proposed release commit.
3. Compare them.
4. Resolve every difference, or explicitly allow an intentional difference
   with at least one acceptance-test path that exists in the candidate.
5. Require zero unresolved differences and zero invalid allowlist entries.

The comparator exits nonzero if either condition is violated. Never copy a
prior allowlist forward without re-evaluating every entry. The 0.1.27 evidence
under `qa/release/0.1.27/` demonstrates convergence: early reconstruction and
recovery manifests exposed product drift; the final comparison has three
accepted slash-command changes and zero unresolved differences.

Example for a patch release:

```bash
PREVIOUS=0.1.27
VERSION=0.1.28
QA_DIR="qa/release/$VERSION"
mkdir -p "$QA_DIR"

python3 scripts/release/build_pf_feature_manifest.py \
  --ref "rust-v$PREVIOUS" \
  --output "$QA_DIR/pf-feature-manifest-$PREVIOUS.json"

python3 scripts/release/build_pf_feature_manifest.py \
  --ref HEAD \
  --output "$QA_DIR/pf-feature-manifest-candidate.json"

# Create a release-specific allowlist. Start with:
printf '{"differences": []}\n' > "$QA_DIR/pf-feature-allowlist.json"

python3 scripts/release/compare_pf_feature_manifests.py \
  --baseline "$QA_DIR/pf-feature-manifest-$PREVIOUS.json" \
  --candidate "$QA_DIR/pf-feature-manifest-candidate.json" \
  --allowlist "$QA_DIR/pf-feature-allowlist.json" \
  --output "$QA_DIR/pf-feature-comparison.json"

python3 - <<'PY'
import json
from pathlib import Path
report = json.loads(
    Path("qa/release/0.1.28/pf-feature-comparison.json").read_text()
)
assert report["unresolved_difference_count"] == 0
assert report["invalid_allowlist_entry_count"] == 0
PY
```

If the comparison fails, inspect the report. Restore unintended drift in code;
for intentional drift, add focused regression coverage and a narrowly scoped
allowlist entry naming those test files, then rerun the comparison.

## QA acceptance

Create a release-specific acceptance ledger based on
`qa/release/0.1.27/HANDS-ON-ACCEPTANCE-20260801.md`. At minimum, record:

- exact candidate branch, commit, tree, version, and package hashes;
- clean-tree and feature-manifest results;
- installer/package contract tests and archive smoke tests;
- focused and full Rust test results, including skipped-test counts;
- fresh install and upgrade behavior on Linux, macOS, and Windows;
- packaged binary identity, wallet daemon startup, bundled resources, and the
  absence of a forbidden stock `codex` entry point;
- core product surfaces: provider routing, vault/wallet, Telegram, Task Node,
  GPU, native agents/spawn/orchestration/panes, permissions, persistence,
  restart, cancellation, and failure recovery;
- spend, credentials, defects, residual open cells, and explicit operator
  authorization.

Automated evidence supports but does not replace hands-on acceptance. A release
may proceed with an open cell only when the ledger names it and the operator
explicitly accepts the residual risk.

## Patch-release command sequence

### 1. Prepare the release PR

```bash
set -euo pipefail
PREVIOUS=0.1.27
VERSION=0.1.28
BRANCH="release/$VERSION"

git fetch origin --prune --tags
git switch main
git pull --ff-only origin main
git status --short
test "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)"
git switch -c "$BRANCH"

bash scripts/dev/check_worktree.sh
```

Update `codex-rs/Cargo.toml` from `PREVIOUS` to `VERSION`, then refresh the
workspace lockfile without changing dependency selections:

```bash
cargo check --manifest-path codex-rs/Cargo.toml -p codex-cli
git diff -- codex-rs/Cargo.toml codex-rs/Cargo.lock
```

Run the feature-manifest commands above and create/update the acceptance ledger.
Then run the release-facing local tests:

```bash
bash scripts/dev/test_check_worktree.sh
python3 -m unittest scripts/release/test_pf_feature_manifest.py
python3 scripts/install/test_pfterminal_release_contract.py
python3 -m unittest discover -s scripts/codex_package -p 'test_*.py'
python3 -m unittest discover -s scripts/install -p 'test_*.py'
```

Run the Rust suites and hands-on matrix required by the acceptance ledger.
Record exact commands, results, artifact hashes, failures, and accepted residual
risk. Then scan the complete diff for secret material, commit, push, and open a
PR:

```bash
git add codex-rs/Cargo.toml codex-rs/Cargo.lock "qa/release/$VERSION"
git commit -m "release: prepare PfTerminal $VERSION"
git push -u origin "$BRANCH"
gh pr create --base main --head "$BRANCH" \
  --title "release: prepare PfTerminal $VERSION" \
  --body-file "qa/release/$VERSION/PR.md"
```

Do not merge the PR yourself unless you are the designated operator. Require
review, green CI, a passing feature comparison, and recorded release
authorization.

### 2. Qualify the merged commit without publishing

After the operator merges the PR:

```bash
git switch main
git pull --ff-only origin main
bash scripts/dev/check_worktree.sh
test "$(grep -m1 '^version' codex-rs/Cargo.toml | sed -E 's/.*"([^"]+)".*/\1/')" = "$VERSION"

gh workflow run pfterminal-release.yml --ref main \
  -f release_version="$VERSION" \
  -f publish_release=false \
  -f make_latest=false

RUN_ID="$(gh run list --workflow pfterminal-release.yml --branch main \
  --limit 1 --json databaseId --jq '.[0].databaseId')"
gh run watch "$RUN_ID" --exit-status
```

Inspect the run's `pfterminal-release-assets` artifact and reconcile its hashes
with the acceptance ledger before tagging.

### 3. Tag and publish

```bash
RELEASE_COMMIT="$(git rev-parse HEAD)"
git tag -a "rust-v$VERSION" "$RELEASE_COMMIT" \
  -m "PfTerminal $VERSION"
test "$(git rev-list -n1 "rust-v$VERSION")" = "$RELEASE_COMMIT"

# Irreversible publication boundary: obtain operator authorization first.
git push origin "refs/tags/rust-v$VERSION"

gh workflow run pfterminal-release.yml --ref main \
  -f release_version="$VERSION" \
  -f publish_release=true \
  -f make_latest=true

RUN_ID="$(gh run list --workflow pfterminal-release.yml --branch main \
  --limit 1 --json databaseId --jq '.[0].databaseId')"
gh run watch "$RUN_ID" --exit-status
```

Finally verify the tag, release name, target commit, seven platform artifacts,
two checksum manifests, and two installers:

```bash
test "$(git ls-remote origin "refs/tags/rust-v$VERSION^{}" | awk '{print $1}')" \
  = "$RELEASE_COMMIT"
gh release view "$VERSION" --json name,tagName,targetCommitish,isDraft,isPrerelease,url
gh release download "$VERSION" --dir "/tmp/pfterminal-$VERSION-release" --skip-existing
(cd "/tmp/pfterminal-$VERSION-release" &&
  sha256sum --check pfterminal-package_SHA256SUMS)
```

Document any discrepancy; do not move or replace the release tag.
