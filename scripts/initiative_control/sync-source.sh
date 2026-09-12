#!/bin/bash
# The caller owns the source-sync lock; never reconcile or mutate a worktree here.
set -euo pipefail
task_root=$1
task_repo=$2
task_branch=$3
task_remote=pfrpc@178.156.143.199
task_server_root=/home/pfrpc/corbanu-control
task_failure() {
  "$task_root/ssh-server" "$task_remote" "/usr/bin/python3 $task_server_root/source/scripts/initiative_control/sync_check.py failed --root $task_server_root" || true
}
trap task_failure ERR
task_bundle=$(mktemp -d "$task_root/export.XXXXXX")
task_upload="upload-$(/usr/bin/uuidgen | tr -d '-' | tr '[:upper:]' '[:lower:]')"
"$task_root/venv/bin/python" "$task_repo/scripts/initiative_control/export.py" --repo "$task_repo" --state "$task_root/state" --destination "$task_bundle" --expected-branch "$task_branch"
# Validate/render the complete bundle locally before replacing the live source.
"$task_root/venv/bin/python" "$task_bundle/source/scripts/initiative_control/control.py" publish --repo "$task_bundle/source" --state "$task_bundle/state" --output "$task_bundle/preview"
"$task_root/ssh-server" "$task_remote" "umask 077; mkdir -p $task_server_root/incoming/$task_upload"
/usr/bin/rsync -az -e "$task_root/ssh-server" "$task_bundle/source" "$task_bundle/state" "$task_remote:$task_server_root/incoming/$task_upload/"
"$task_root/venv/bin/python" "$task_repo/scripts/initiative_control/export.py" --repo "$task_repo" --state "$task_root/state" --verify "$task_bundle/state/source.json" --expected-branch "$task_branch"
"$task_root/ssh-server" "$task_remote" "/usr/bin/python3 $task_server_root/incoming/$task_upload/source/scripts/initiative_control/activate.py --root $task_server_root --incoming $task_server_root/incoming/$task_upload --units /home/pfrpc/.config/systemd/user"
task_commit=$("$task_root/venv/bin/python" -c 'import json,sys; print(json.load(open(sys.argv[1]))["commit"])' "$task_bundle/state/source.json")
task_digest=$("$task_root/venv/bin/python" -c 'import json,sys; print(json.load(open(sys.argv[1]))["tree_digest"])' "$task_bundle/state/source.json")
"$task_root/ssh-server" "$task_remote" "/usr/bin/python3 $task_server_root/source/scripts/initiative_control/sync_check.py check --root $task_server_root --expected-commit $task_commit --expected-digest $task_digest"
echo "Source sync completed; local export retained at $task_bundle"
