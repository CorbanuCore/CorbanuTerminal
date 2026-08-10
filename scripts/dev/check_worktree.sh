#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
Usage: check_worktree.sh [--branch BRANCH]

Verify that a release is running from a clean Git worktree and from main or a
release/* branch. In GitHub Actions, the branch defaults to GITHUB_HEAD_REF or
GITHUB_REF_NAME. In a local checkout, it defaults to the symbolic branch.
EOF
}

branch_override=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --branch)
      branch_override="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

repo_root="$(git rev-parse --show-toplevel 2>/dev/null)" || {
  echo "Release worktree check must run inside a Git checkout." >&2
  exit 1
}
cd "$repo_root"

if ! git diff --quiet --ignore-submodules -- ||
   ! git diff --cached --quiet --ignore-submodules -- ||
   [[ -n "$(git ls-files --others --exclude-standard)" ]]; then
  echo "Release worktree is dirty; commit, stash, or quarantine all changes first." >&2
  exit 1
fi

branch="$branch_override"
if [[ -z "$branch" ]]; then
  branch="${PF_RELEASE_BRANCH:-}"
fi
if [[ -z "$branch" && -n "${GITHUB_HEAD_REF:-}" ]]; then
  branch="$GITHUB_HEAD_REF"
fi
if [[ -z "$branch" && -n "${GITHUB_REF_NAME:-}" ]]; then
  branch="$GITHUB_REF_NAME"
fi
if [[ -z "$branch" ]]; then
  branch="$(git symbolic-ref --quiet --short HEAD 2>/dev/null || true)"
fi
branch="${branch#refs/heads/}"

case "$branch" in
  main|release/*)
    ;;
  "")
    echo "Release worktree is detached and no source branch was supplied." >&2
    exit 1
    ;;
  *)
    echo "Release builds are allowed only from main or release/*, not '$branch'." >&2
    exit 1
    ;;
esac

echo "Release worktree check passed for $branch."
