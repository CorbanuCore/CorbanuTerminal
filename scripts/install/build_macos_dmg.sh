#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat >&2 <<'EOF'
Usage: build_macos_dmg.sh --archive PATH --target TARGET --version VERSION --output PATH

Builds a macOS DMG containing:
  - install.command
  - install.sh
  - corbanu-terminal-package-<target>.tar.gz
  - corbanu-terminal-package_SHA256SUMS

The DMG installer uses the bundled package archive and does not need to fetch
release assets from GitHub.
EOF
}

archive_path=""
target=""
version=""
output_path=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --archive)
      archive_path="${2:-}"
      shift 2
      ;;
    --target)
      target="${2:-}"
      shift 2
      ;;
    --version)
      version="${2:-}"
      shift 2
      ;;
    --output)
      output_path="${2:-}"
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

if [[ -z "$archive_path" || -z "$target" || -z "$version" || -z "$output_path" ]]; then
  usage
  exit 2
fi

if [[ ! -f "$archive_path" ]]; then
  echo "Package archive does not exist: $archive_path" >&2
  exit 1
fi

if [[ "$target" != *apple-darwin ]]; then
  echo "DMG target must be a macOS target, got: $target" >&2
  exit 2
fi

if ! command -v hdiutil >/dev/null 2>&1; then
  echo "hdiutil is required to build a macOS DMG." >&2
  exit 1
fi

if ! command -v shasum >/dev/null 2>&1; then
  echo "shasum is required to build a macOS DMG." >&2
  exit 1
fi

case "$target" in
  aarch64-apple-darwin)
    platform_label="macOS Apple Silicon"
    ;;
  x86_64-apple-darwin)
    platform_label="macOS Intel"
    ;;
  *)
    platform_label="macOS"
    ;;
esac

repo_root="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
archive_name="$(basename "$archive_path")"
expected_archive_name="corbanu-terminal-package-${target}.tar.gz"
if [[ "$archive_name" != "$expected_archive_name" ]]; then
  echo "Archive name must be $expected_archive_name, got: $archive_name" >&2
  exit 2
fi

mkdir -p "$(dirname "$output_path")"
rm -f "$output_path"

work_dir="$(mktemp -d)"
trap 'rm -rf "$work_dir"' EXIT

staging_dir="$work_dir/Corbanu Terminal Installer"
mkdir -p "$staging_dir"

cp "$archive_path" "$staging_dir/$archive_name"
cp "$repo_root/scripts/install/install.sh" "$staging_dir/install.sh"
chmod 0755 "$staging_dir/install.sh"

archive_sha256="$(shasum -a 256 "$archive_path" | awk '{ print $1 }')"
printf '%s  %s\n' "$archive_sha256" "$archive_name" > "$staging_dir/corbanu-terminal-package_SHA256SUMS"

cat > "$staging_dir/install.command" <<EOF
#!/bin/sh
set -eu

SCRIPT_DIR=\$(CDPATH= cd -- "\$(dirname -- "\$0")" && pwd)

export CORBANU_RELEASE="${version}"
export CORBANU_PACKAGE_ARCHIVE="\$SCRIPT_DIR/${archive_name}"
export CORBANU_CHECKSUM_MANIFEST="\$SCRIPT_DIR/corbanu-terminal-package_SHA256SUMS"

exec /bin/sh "\$SCRIPT_DIR/install.sh" "\$@"
EOF
chmod 0755 "$staging_dir/install.command"

cat > "$staging_dir/README.txt" <<EOF
Corbanu Terminal ${version} for ${platform_label}

Double-click install.command to install Corbanu Terminal.

Default install locations:
  Primary launcher: \$HOME/.local/bin/corbanu
  Fresh-install state: \$HOME/.corbanu

The installer leaves any existing stock codex command alone. It installs the
bundled package archive from this DMG and verifies it against
corbanu-terminal-package_SHA256SUMS before installation.

Advanced terminal install:
  sh /Volumes/CorbanuTerminal-${version}-${target}/install.command
EOF

volume_name="CorbanuTerminal-${version}-${target}"
archive_size_bytes="$(stat -f '%z' "$archive_path")"
image_size_mib="$(( (archive_size_bytes + 1048575) / 1048576 + 256 ))"

# GitHub-hosted macOS runners occasionally leave diskimages-helper busy for a
# few seconds after large SDK/cache cleanup. hdiutil reports that transient as
# "create failed - Resource busy" even though the source tree and destination
# are valid. Retrying the hdiutil boundary is safe because every attempt uses
# -ov and removes any partial destination first.
max_create_attempts=5
create_attempt=1
retry_delay_seconds=5
while true; do
  rm -f "$output_path"
  if hdiutil create \
    -volname "$volume_name" \
    -srcfolder "$staging_dir" \
    -size "${image_size_mib}m" \
    -ov \
    -format UDZO \
    "$output_path"; then
    break
  fi
  if (( create_attempt >= max_create_attempts )); then
    echo "hdiutil create failed after ${max_create_attempts} attempts." >&2
    exit 1
  fi
  echo \
    "hdiutil create attempt ${create_attempt} failed; retrying in " \
    "${retry_delay_seconds}s." >&2
  sleep "$retry_delay_seconds"
  create_attempt="$((create_attempt + 1))"
  retry_delay_seconds="$((retry_delay_seconds * 2))"
done

echo "Built $output_path"
