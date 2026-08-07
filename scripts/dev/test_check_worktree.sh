#!/usr/bin/env bash
set -euo pipefail

script_dir="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
guard="$script_dir/check_worktree.sh"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

repo="$tmp/repo"
git init -q -b main "$repo"
git -C "$repo" config user.name "Release Guard Test"
git -C "$repo" config user.email "release-guard@example.invalid"
printf 'clean\n' > "$repo/tracked.txt"
git -C "$repo" add tracked.txt
git -C "$repo" commit -qm "initial"

(
  cd "$repo"
  "$guard" >/dev/null
  "$guard" --branch refs/heads/main >/dev/null
)

git -C "$repo" switch -qc release/9.9.9
(
  cd "$repo"
  "$guard" >/dev/null
)

git -C "$repo" switch -qc feature/not-a-release
if (cd "$repo" && "$guard" >/dev/null 2>&1); then
  echo "guard accepted a feature branch" >&2
  exit 1
fi

git -C "$repo" switch -q main
printf 'dirty\n' >> "$repo/tracked.txt"
if (cd "$repo" && "$guard" >/dev/null 2>&1); then
  echo "guard accepted a modified tracked file" >&2
  exit 1
fi
git -C "$repo" restore tracked.txt

printf 'untracked\n' > "$repo/untracked.txt"
if (cd "$repo" && "$guard" >/dev/null 2>&1); then
  echo "guard accepted an untracked file" >&2
  exit 1
fi
git -C "$repo" clean -qf

git -C "$repo" switch -q --detach
if (cd "$repo" && env -u PF_RELEASE_BRANCH -u GITHUB_HEAD_REF -u GITHUB_REF_NAME "$guard" >/dev/null 2>&1); then
  echo "guard accepted a detached checkout without source-branch context" >&2
  exit 1
fi
(
  cd "$repo"
  PF_RELEASE_BRANCH=main "$guard" >/dev/null
)

echo "release worktree guard tests passed"
