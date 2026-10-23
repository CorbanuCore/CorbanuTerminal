#!/bin/sh
set -eu
RUN="$AGMS_RUN"
rm -rf "$RUN/product-home"
mkdir -p "$RUN/product-home"
cp "$RUN/evidence/profile-config-template.toml" "$RUN/product-home/config.toml"
rm -rf "$RUN/fixture/workspace"
cp -a "$RUN/evidence/workspace-template" "$RUN/fixture/workspace"
printf 'outside-the-project: synthetic file\n' > "$RUN/fixture/outside-notes.txt"
rm -f "$RUN/fixture/mcp/shared-note.txt" "$RUN/fixture/mcp/posted.txt"
