#!/usr/bin/env bash
# Reproducible, user-owned prerequisite for the RTX's installed Node MCPs.
set -euo pipefail
version=v22.23.2
sha256=d60acfe00a2932254bb0ad20e01b0d74397a0875595de719654b214f4b03f307
tools=/home/travis/security-round5/tools
runtime="$tools/node-$version-linux-x64"
if [[ ! -x "$runtime/bin/node" ]]; then
    download=$(mktemp -d "$tools/node-download.XXXXXX")
    archive="$download/node.tar.xz"
    curl --fail --location --proto '=https' --tlsv1.2 \
        "https://nodejs.org/dist/$version/node-$version-linux-x64.tar.xz" -o "$archive"
    printf '%s  %s\n' "$sha256" "$archive" | sha256sum --check --status
    tar -xJf "$archive" -C "$tools"
fi
if [[ ! -e "$tools/node" && ! -L "$tools/node" ]]; then
    ln -s "$runtime/bin/node" "$tools/node"
fi
"$tools/node" --version
