#!/bin/sh

set -eu

RELEASE="${CORBANU_RELEASE:-${PFTERMINAL_RELEASE:-${CODEX_RELEASE:-latest}}}"
NON_INTERACTIVE="${CORBANU_NON_INTERACTIVE:-${PFTERMINAL_NON_INTERACTIVE:-${CODEX_NON_INTERACTIVE:-false}}}"
DEFAULT_PREFER_RELEASES_OPENAI_COM="false"
PREFER_RELEASES_OPENAI_COM="${CORBANU_INSTALLER_USE_RELEASES_OPENAI_COM:-${PFTERMINAL_INSTALLER_USE_RELEASES_OPENAI_COM:-${CODEX_INSTALLER_USE_RELEASES_OPENAI_COM:-$DEFAULT_PREFER_RELEASES_OPENAI_COM}}}"
RELEASES_BASE_URL="https://releases.openai.com/codex"
GITHUB_REPOSITORY="CorbanuCore/CorbanuTerminal"
RELEASES_CONNECT_TIMEOUT=10
RELEASES_METADATA_TIMEOUT=30
RELEASES_ASSET_TIMEOUT=300
KEEP_RELEASES="${CORBANU_KEEP_RELEASES:-2}"
BUNDLED_PACKAGE_ARCHIVE="${CORBANU_PACKAGE_ARCHIVE:-}"
BUNDLED_CHECKSUM_MANIFEST="${CORBANU_CHECKSUM_MANIFEST:-}"
release_source="github"

BIN_DIR="${CORBANU_INSTALL_DIR:-${PFTERMINAL_INSTALL_DIR:-${CODEX_INSTALL_DIR:-$HOME/.local/bin}}}"
BIN_PATH="$BIN_DIR/corbanu"
LEGACY_BIN_PATH="$BIN_DIR/pfterminal"
DEBUG_BIN_PATH="$BIN_DIR/corbanu-debug"
LEGACY_DEBUG_BIN_PATH="$BIN_DIR/pfterminal-debug"
CODE_MODE_HOST_BIN_PATH="$BIN_DIR/codex-code-mode-host"
if [ -n "${CORBANU_HOME:-}" ]; then
  CODEX_HOME_DIR="$CORBANU_HOME"
elif [ -n "${PFTERMINAL_HOME:-}" ]; then
  CODEX_HOME_DIR="$PFTERMINAL_HOME"
elif [ -n "${CODEX_HOME:-}" ]; then
  CODEX_HOME_DIR="$CODEX_HOME"
elif [ -d "$HOME/.corbanu" ] && [ -d "$HOME/.pfterminal" ]; then
  CODEX_HOME_DIR="$HOME/.corbanu"
  printf 'WARNING: Canonical and legacy state directories both exist; using %s without modifying either directory.\n' \
    "$HOME/.corbanu" >&2
elif [ -d "$HOME/.pfterminal" ]; then
  CODEX_HOME_DIR="$HOME/.pfterminal"
else
  CODEX_HOME_DIR="$HOME/.corbanu"
fi
DEBUG_CODEX_HOME_DIR="${CODEX_HOME_DIR}-debug"
STANDALONE_ROOT="$CODEX_HOME_DIR/packages/standalone"
RELEASES_DIR="$STANDALONE_ROOT/releases"
CURRENT_LINK="$STANDALONE_ROOT/current"
LOCK_FILE="$STANDALONE_ROOT/install.lock"
LOCK_DIR="$STANDALONE_ROOT/install.lock.d"
LOCK_STALE_AFTER_SECS=600

path_action="already"
path_profile=""
conflict_manager=""
conflict_path=""
lock_kind=""
tmp_dir=""
debug_launchers_installed="false"

step() {
  printf '==> %s\n' "$1"
}

warn() {
  printf 'WARNING: %s\n' "$1" >&2
}

normalize_version() {
  case "$1" in
    "" | latest)
      printf 'latest\n'
      ;;
    rust-v*)
      printf '%s\n' "${1#rust-v}"
      ;;
    v*)
      printf '%s\n' "${1#v}"
      ;;
    *)
      printf '%s\n' "$1"
      ;;
  esac
}

validate_version() {
  version="$1"

  if [ "$version" = "latest" ]; then
    return
  fi

  if ! printf '%s\n' "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+(-alpha(\.[0-9]+){0,2}|-beta(\.[0-9]+)?)?$'; then
    echo "Invalid Corbanu Terminal release version: $version. Expected latest or x.y.z[-alpha[.N[.M]]|-beta[.N]]." >&2
    return 1
  fi
}

parse_args() {
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --release)
        if [ "$#" -lt 2 ]; then
          echo "--release requires a value." >&2
          exit 1
        fi
        RELEASE="$2"
        shift
        ;;
      --help | -h)
        cat <<EOF
Usage: install.sh [--release VERSION]

Environment:
  CORBANU_RELEASE             Version to install; overridden by --release.
  CORBANU_NON_INTERACTIVE     Set to 1, true, or yes to skip prompts.
  CORBANU_INSTALL_DIR         Directory for the corbanu launchers.
  CORBANU_HOME                Corbanu state directory; fresh installs default to ~/.corbanu.
  CORBANU_KEEP_RELEASES       Number of prior standalone releases to retain (default: 2).

  Legacy installer variables from earlier product names are still honored as fallbacks.
EOF
        exit 0
        ;;
      *)
        echo "Unknown argument: $1" >&2
        exit 1
        ;;
    esac
    shift
  done
}

download_file() {
  url="$1"
  output="$2"

  if command -v curl >/dev/null 2>&1; then
    case "$url" in
      "$RELEASES_BASE_URL"/*)
        curl -fsSL --connect-timeout "$RELEASES_CONNECT_TIMEOUT" --max-time "$RELEASES_ASSET_TIMEOUT" "$url" -o "$output"
        ;;
      *)
        curl -fsSL "$url" -o "$output"
        ;;
    esac
    return
  fi

  if command -v wget >/dev/null 2>&1; then
    case "$url" in
      "$RELEASES_BASE_URL"/*)
        wget -q -t 1 -T "$RELEASES_ASSET_TIMEOUT" -O "$output" "$url"
        ;;
      *)
        wget -q -O "$output" "$url"
        ;;
    esac
    return
  fi

    echo "curl or wget is required to install Corbanu Terminal." >&2
  exit 1
}

download_text() {
  url="$1"

  if command -v curl >/dev/null 2>&1; then
    case "$url" in
      "$RELEASES_BASE_URL"/*)
        curl -fsSL --connect-timeout "$RELEASES_CONNECT_TIMEOUT" --max-time "$RELEASES_METADATA_TIMEOUT" "$url"
        ;;
      *)
        curl -fsSL "$url"
        ;;
    esac
    return
  fi

  if command -v wget >/dev/null 2>&1; then
    case "$url" in
      "$RELEASES_BASE_URL"/*)
        wget -q -t 1 -T "$RELEASES_METADATA_TIMEOUT" -O - "$url"
        ;;
      *)
        wget -q -O - "$url"
        ;;
    esac
    return
  fi

    echo "curl or wget is required to install Corbanu Terminal." >&2
  exit 1
}

download_file_with_fallback() {
  primary_url="$1"
  fallback_url="$2"
  output="$3"
  expected_digest="$4"
  fallback_asset="$5"
  required_manifest_asset="${6:-}"

  if download_file "$primary_url" "$output" &&
    verify_archive_digest "$output" "$expected_digest" &&
    { [ -z "$required_manifest_asset" ] || package_archive_digest "$required_manifest_asset" "$output" >/dev/null; }; then
    return
  fi

  if [ -z "$fallback_url" ]; then
    return 1
  fi

  warn "Could not download or verify $primary_url; retrying from GitHub Releases."
  download_file "$fallback_url" "$output"
  if verify_archive_digest "$output" "$expected_digest" &&
    { [ -z "$required_manifest_asset" ] || package_archive_digest "$required_manifest_asset" "$output" >/dev/null; }; then
    return
  fi

  resolve_release_from_github "$resolved_version"
  fallback_digest="$(release_asset_digest "$fallback_asset")"
  verify_archive_digest "$output" "$fallback_digest"
  if [ -n "$required_manifest_asset" ]; then
    package_archive_digest "$required_manifest_asset" "$output" >/dev/null
  fi
}

parse_release_metadata() {
  # Bound awk's record size so compact, single-line JSON stays fast on every
  # supported awk implementation. JSON strings cannot contain literal newlines,
  # so the record boundaries inserted by fold do not change the document.
  LC_ALL=C fold -b -w 4096 | LC_ALL=C awk '
    function finish_string(value) {
      if (object_depth == 1 && key == "tag_name") {
        print "tag_name\t" value
      } else if (object_depth == asset_object_depth) {
        if (key == "name") {
          asset_name = value
        } else if (key == "digest") {
          asset_digest = value
        }
      }

      expecting_value = 0
      key = ""
    }

    {
      for (i = 1; i <= length($0); i++) {
        char = substr($0, i, 1)

        if (in_string) {
          if (escaped) {
            token = token "\\" char
            escaped = 0
          } else if (char == "\\") {
            escaped = 1
          } else if (char == "\"") {
            in_string = 0
            if (string_is_value) {
              finish_string(token)
            } else {
              pending_key = token
            }
          } else {
            token = token char
          }
          continue
        }

        if (char == "\"") {
          in_string = 1
          token = ""
          escaped = 0
          string_is_value = expecting_value
        } else if (char == ":" && pending_key != "") {
          key = pending_key
          pending_key = ""
          expecting_value = 1
        } else if (char == "{") {
          object_depth++
          if (assets_array_depth != 0 &&
              array_depth == assets_array_depth &&
              asset_object_depth == 0) {
            asset_object_depth = object_depth
            asset_name = ""
            asset_digest = ""
          }
          expecting_value = 0
          key = ""
        } else if (char == "}") {
          if (object_depth == asset_object_depth) {
            if (asset_name != "" && asset_digest != "") {
              print "asset\t" asset_name "\t" asset_digest
            }
            asset_object_depth = 0
            asset_name = ""
            asset_digest = ""
          }
          object_depth--
          expecting_value = 0
          key = ""
          pending_key = ""
        } else if (char == "[") {
          array_depth++
          if (expecting_value && key == "assets" && object_depth == 1) {
            assets_array_depth = array_depth
          }
          expecting_value = 0
          key = ""
        } else if (char == "]") {
          if (array_depth == assets_array_depth) {
            assets_array_depth = 0
          }
          array_depth--
          expecting_value = 0
          key = ""
          pending_key = ""
        } else if (char == ",") {
          expecting_value = 0
          key = ""
          pending_key = ""
        }
      }
    }

    END {
      if (in_string || object_depth != 0 || array_depth != 0) {
        exit 1
      }
    }
  '
}

release_url_for_asset() {
  asset="$1"
  resolved_version="$2"

  printf 'https://github.com/%s/releases/download/rust-v%s/%s\n' "$GITHUB_REPOSITORY" "$resolved_version" "$asset"
}

releases_url_for_asset() {
  asset="$1"
  resolved_version="$2"

  printf '%s/releases/%s/%s\n' "$RELEASES_BASE_URL" "$resolved_version" "$asset"
}

release_metadata_url() {
  resolved_version="$1"

  printf 'https://api.github.com/repos/%s/releases/tags/rust-v%s\n' "$GITHUB_REPOSITORY" "$resolved_version"
}

parse_downloaded_release_metadata() {
  requested_release="$1"
  source_name="$2"
  if ! release_metadata="$(printf '%s\n' "$release_json" | parse_release_metadata)"; then
    echo "Could not parse $source_name release metadata for Corbanu Terminal $requested_release." >&2
    return 1
  fi
}

resolve_metadata_version() {
  release_tag="$(printf '%s\n' "$release_metadata" | awk -F '\t' '$1 == "tag_name" { print $2; exit }')"
  case "$release_tag" in
    rust-v*) metadata_version="${release_tag#rust-v}" ;;
    *) metadata_version="" ;;
  esac
  if [ -z "$metadata_version" ]; then
    echo "Failed to resolve the latest Corbanu Terminal release version." >&2
    return 1
  fi
  validate_version "$metadata_version"
}

resolve_release_from_github() {
  normalized_version="$1"
  if [ "$normalized_version" = "latest" ]; then
    requested_release="latest"
    metadata_url="https://api.github.com/repos/$GITHUB_REPOSITORY/releases/latest"
  else
    resolved_version="$normalized_version"
    requested_release="$resolved_version"
    metadata_url="$(release_metadata_url "$resolved_version")"
  fi

  if ! release_json="$(download_text "$metadata_url")"; then
    echo "Could not fetch GitHub release metadata for Corbanu Terminal $requested_release. GitHub API may be unavailable or rate limited." >&2
    exit 1
  fi

  parse_downloaded_release_metadata "$requested_release" "GitHub"

  if [ "$normalized_version" = "latest" ]; then
    resolve_metadata_version
    resolved_version="$metadata_version"
  fi

  release_source="github"
}

resolve_release_from_releases() {
  normalized_version="$1"

  if [ "$normalized_version" = "latest" ]; then
    requested_release="latest"
    metadata_url="$RELEASES_BASE_URL/channels/latest"
  else
    requested_release="$normalized_version"
    metadata_url="$RELEASES_BASE_URL/releases/$normalized_version/release.json"
  fi

  if ! release_json="$(download_text "$metadata_url")"; then
    return 1
  fi

  if ! parse_downloaded_release_metadata "$requested_release" "releases.openai.com"; then
    return 1
  fi
  if ! resolve_metadata_version; then
    return 1
  fi
  if [ "$normalized_version" != "latest" ] && [ "$metadata_version" != "$normalized_version" ]; then
    echo "Release metadata version did not match requested Corbanu Terminal version $normalized_version." >&2
    return 1
  fi
  resolved_version="$metadata_version"
  release_source="releases.openai.com"
}

resolve_release() {
  normalized_version="$(normalize_version "$RELEASE")"
  validate_version "$normalized_version"

  case "$PREFER_RELEASES_OPENAI_COM" in
    1 | [Tt][Rr][Uu][Ee] | [Yy][Ee][Ss])
      if resolve_release_from_releases "$normalized_version" &&
        select_release_assets; then
        return
      fi
      warn "releases.openai.com is unavailable; falling back to GitHub Releases."
      ;;
  esac

  resolve_release_from_github "$normalized_version"
  select_release_assets
}

release_asset_digest_or_empty() {
  asset="$1"

  digest="$(printf '%s\n' "$release_metadata" | awk -F '\t' -v asset="$asset" '
    $1 == "asset" && $2 == asset {
      print $3
      exit
    }
  ')"

  case "$digest" in
    sha256:????????????????????????????????????????????????????????????????)
      digest="${digest#sha256:}"
      case "$digest" in
        *[!0-9a-fA-F]*) return 1 ;;
      esac
      printf '%s\n' "$digest"
      ;;
    *)
      return 1
      ;;
  esac
}

release_asset_exists() {
  asset="$1"

  release_asset_digest_or_empty "$asset" >/dev/null 2>&1
}

release_asset_digest() {
  asset="$1"

  digest="$(release_asset_digest_or_empty "$asset" || true)"
  if [ -z "$digest" ]; then
    echo "Could not find SHA-256 digest for release asset $asset." >&2
    exit 1
  fi

  printf '%s\n' "$digest"
}

select_release_assets() {
  package_asset="corbanu-terminal-package-$vendor_target.tar.gz"
  checksum_asset="corbanu-terminal-package_SHA256SUMS"
  download_fallback_url=""
  checksum_fallback_url=""

  if release_asset_exists "$package_asset" &&
    release_asset_exists "$checksum_asset"; then
    install_layout="package"
    asset="$package_asset"
  elif release_asset_exists "codex-package-$vendor_target.tar.gz" &&
    release_asset_exists "codex-package_SHA256SUMS"; then
    install_layout="package"
    package_asset="codex-package-$vendor_target.tar.gz"
    checksum_asset="codex-package_SHA256SUMS"
    asset="$package_asset"
  elif release_asset_exists "codex-npm-$npm_tag-$resolved_version.tgz"; then
    install_layout="legacy-platform-npm"
    asset="codex-npm-$npm_tag-$resolved_version.tgz"
  else
    echo "Could not find Corbanu Terminal package or compatible legacy release assets for $resolved_version." >&2
    return 1
  fi

  if [ "$release_source" = "releases.openai.com" ]; then
    download_url="$(releases_url_for_asset "$asset" "$resolved_version")"
    download_fallback_url="$(release_url_for_asset "$asset" "$resolved_version")"
    if [ "$install_layout" = "package" ]; then
      checksum_url="$(releases_url_for_asset "$checksum_asset" "$resolved_version")"
      checksum_fallback_url="$(release_url_for_asset "$checksum_asset" "$resolved_version")"
    fi
  else
    download_url="$(release_url_for_asset "$asset" "$resolved_version")"
    if [ "$install_layout" = "package" ]; then
      checksum_url="$(release_url_for_asset "$checksum_asset" "$resolved_version")"
    fi
  fi
}

package_archive_digest() {
  asset="$1"
  manifest_path="$2"

  digest="$(awk -v asset="$asset" '
    $2 == asset && length($1) == 64 && $1 !~ /[^0-9a-fA-F]/ {
      print tolower($1)
      found = 1
      exit
    }
    END {
      if (!found) {
        exit 1
      }
    }
  ' "$manifest_path" 2>/dev/null || true)"

  if [ -z "$digest" ]; then
    echo "Could not find SHA-256 digest for $asset in $checksum_asset." >&2
    return 1
  fi

  printf '%s\n' "$digest"
}

file_sha256() {
  path="$1"

  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$path" | awk '{print $1}'
    return
  fi

  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$path" | awk '{print $1}'
    return
  fi

  if command -v openssl >/dev/null 2>&1; then
    openssl dgst -sha256 "$path" | sed 's/^.*= //'
    return
  fi

  echo "sha256sum, shasum, or openssl is required to verify the Corbanu Terminal download." >&2
  exit 1
}

verify_archive_digest() {
  archive_path="$1"
  expected_digest="$2"
  actual_digest="$(file_sha256 "$archive_path")"

  if [ "$actual_digest" != "$expected_digest" ]; then
    echo "Downloaded Corbanu Terminal archive checksum did not match expected digest." >&2
    echo "expected: $expected_digest" >&2
    echo "actual:   $actual_digest" >&2
    return 1
  fi
}

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "$1 is required to install Corbanu Terminal." >&2
    exit 1
  fi
}

pick_profile() {
  # Use the same shell-specific split Homebrew documents because there is no
  # universal startup file across macOS/Linux login and interactive shells.
  case "$os:${SHELL:-}" in
    darwin:*/zsh)
      printf '%s\n' "$HOME/.zprofile"
      ;;
    darwin:*/bash)
      printf '%s\n' "$HOME/.bash_profile"
      ;;
    linux:*/zsh)
      printf '%s\n' "$HOME/.zshrc"
      ;;
    linux:*/bash)
      printf '%s\n' "$HOME/.bashrc"
      ;;
    *)
      printf '%s\n' "$HOME/.profile"
      ;;
  esac
}

add_to_path() {
  path_action="already"
  path_profile=""

  case ":$PATH:" in
    *":$BIN_DIR:"*)
      resolved_visible="$(command -v corbanu 2>/dev/null || true)"
      if [ -z "$conflict_manager" ] || [ "$resolved_visible" = "$BIN_PATH" ]; then
        return
      fi
      ;;
  esac

  profile="$(pick_profile)"
  path_profile="$profile"
  begin_marker="# >>> PFTerminal installer >>>"
  end_marker="# <<< PFTerminal installer <<<"
  path_line="export PATH=\"$BIN_DIR:\$PATH\""

  if [ -f "$profile" ] && grep -F "$begin_marker" "$profile" >/dev/null 2>&1; then
    if grep -F "$path_line" "$profile" >/dev/null 2>&1; then
      path_action="configured"
      return
    fi

    if grep -F "$end_marker" "$profile" >/dev/null 2>&1; then
      rewrite_path_block "$profile" "$begin_marker" "$end_marker" "$path_line"
      path_action="updated"
      return
    fi
  fi

  append_path_block "$profile" "$begin_marker" "$end_marker" "$path_line"
  path_action="added"
}

append_path_block() {
  profile="$1"
  begin_marker="$2"
  end_marker="$3"
  path_line="$4"

  {
    printf '\n%s\n' "$begin_marker"
    printf '%s\n' "$path_line"
    printf '%s\n' "$end_marker"
  } >>"$profile"
}

rewrite_path_block() {
  profile="$1"
  begin_marker="$2"
  end_marker="$3"
  path_line="$4"
  tmp_profile="$tmp_dir/profile.$$.tmp"

  awk -v begin="$begin_marker" -v end="$end_marker" -v line="$path_line" '
    BEGIN {
      in_block = 0
      replaced = 0
    }
    $0 == begin {
      if (!replaced) {
        print begin
        print line
        print end
        replaced = 1
      }
      in_block = 1
      next
    }
    in_block {
      if ($0 == end) {
        in_block = 0
      }
      next
    }
    {
      print
    }
    END {
      if (in_block != 0) {
        exit 1
      }
    }
  ' "$profile" >"$tmp_profile"
  mv "$tmp_profile" "$profile"
}

mkdir_lock_is_stale() {
  [ -d "$LOCK_DIR" ] || return 1

  pid="$(cat "$LOCK_DIR/pid" 2>/dev/null || true)"
  started_at="$(cat "$LOCK_DIR/started_at" 2>/dev/null || true)"
  now="$(date +%s 2>/dev/null || printf '0')"

  case "$started_at" in
    ''|*[!0-9]*)
      started_at=0
      ;;
  esac

  if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
    return 1
  fi

  if [ "$started_at" -eq 0 ] || [ "$now" -eq 0 ]; then
    return 0
  fi

  [ $((now - started_at)) -ge "$LOCK_STALE_AFTER_SECS" ]
}

acquire_install_lock() {
  mkdir -p "$STANDALONE_ROOT"

  if [ "$os" = "darwin" ] && command -v lockf >/dev/null 2>&1; then
    : >>"$LOCK_FILE"
    exec 9<>"$LOCK_FILE"
    lockf 9
    lock_kind="lockf"
    return
  fi

  if command -v flock >/dev/null 2>&1; then
    exec 9>"$LOCK_FILE"
    flock 9
    lock_kind="flock"
    return
  fi

  while ! mkdir "$LOCK_DIR" 2>/dev/null; do
    if mkdir_lock_is_stale; then
      warn "Removing stale installer lock at $LOCK_DIR"
      rm -rf "$LOCK_DIR"
      continue
    fi
    sleep 1
  done

  printf '%s\n' "$$" >"$LOCK_DIR/pid"
  date +%s >"$LOCK_DIR/started_at" 2>/dev/null || true
  lock_kind="mkdir"
}

release_install_lock() {
  if [ "$lock_kind" = "mkdir" ]; then
    rm -rf "$LOCK_DIR" 2>/dev/null || true
  elif [ "$lock_kind" = "flock" ] || [ "$lock_kind" = "lockf" ]; then
    exec 9>&- 2>/dev/null || true
  fi
  lock_kind=""
}

cleanup_stale_install_artifacts() {
  mkdir -p "$RELEASES_DIR" "$STANDALONE_ROOT"

  find "$RELEASES_DIR" -mindepth 1 -maxdepth 1 -name '.staging.*' -exec rm -rf {} +
  find "$STANDALONE_ROOT" -mindepth 1 -maxdepth 1 -name '.current.*' -exec rm -f {} +

  if [ -d "$BIN_DIR" ]; then
    find "$BIN_DIR" -mindepth 1 -maxdepth 1 -name '.pfterminal.*' -exec rm -f {} +
  fi
}

replace_path_with_symlink() {
  link_path="$1"
  link_target="$2"
  tmp_link="$3"

  rm -f "$tmp_link"
  ln -s "$link_target" "$tmp_link"

  if mv -Tf "$tmp_link" "$link_path" 2>/dev/null; then
    return
  fi

  if mv -hf "$tmp_link" "$link_path" 2>/dev/null; then
    return
  fi

  rm -f "$link_path"
  mv -f "$tmp_link" "$link_path"
}

version_from_binary() {
  binary_path="$1"

  if [ ! -x "$binary_path" ]; then
    return 1
  fi

  "$binary_path" --version 2>/dev/null | sed -n 's/.* \([0-9][0-9A-Za-z.+-]*\)$/\1/p' | head -n 1
}

current_installed_version() {
  version="$(version_from_binary "$CURRENT_LINK/bin/corbanu" || true)"
  if [ -n "$version" ]; then
    printf '%s\n' "$version"
    return 0
  fi

  version="$(version_from_binary "$CURRENT_LINK/bin/pfterminal" || true)"
  if [ -n "$version" ]; then
    printf '%s\n' "$version"
    return 0
  fi

  version="$(version_from_binary "$CURRENT_LINK/pfterminal" || true)"
  if [ -n "$version" ]; then
    printf '%s\n' "$version"
    return 0
  fi

  return 0
}

resolve_existing_pfterminal() {
  command -v pfterminal 2>/dev/null || true
}

classify_existing_pfterminal() {
  existing_path="$1"

  if [ -z "$existing_path" ] || [ "$existing_path" = "$LEGACY_BIN_PATH" ]; then
    return 1
  fi

  if [ -f "$existing_path" ] && grep -F "#!/usr/bin/env node" "$existing_path" >/dev/null 2>&1; then
    case "$existing_path" in
      *".bun"*)
        printf 'bun\n'
        ;;
      *)
        printf 'npm\n'
        ;;
    esac
    return 0
  fi

  printf 'manual\n'
  return 0
}

prompt_yes_no() {
  prompt="$1"

  case "$NON_INTERACTIVE" in
    1 | [Tt][Rr][Uu][Ee] | [Yy][Ee][Ss])
      return 1
      ;;
  esac

  if ( : </dev/tty ) 2>/dev/null; then
    printf '%s [y/N] ' "$prompt" >/dev/tty
    if ! IFS= read -r answer </dev/tty; then
      return 1
    fi
  elif [ -t 0 ]; then
    printf '%s [y/N] ' "$prompt"
    if ! IFS= read -r answer; then
      return 1
    fi
  else
    return 1
  fi

  case "$answer" in
    y | Y | yes | YES)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

print_launch_instructions() {
  case "$path_action" in
    added)
      step "Current terminal: export PATH=\"$BIN_DIR:\$PATH\" && corbanu"
      step "Future terminals: open a new terminal and run: corbanu"
      step "PATH was added to $path_profile"
      ;;
    updated)
      step "Current terminal: export PATH=\"$BIN_DIR:\$PATH\" && corbanu"
      step "Future terminals: open a new terminal and run: corbanu"
      step "PATH was updated in $path_profile"
      ;;
    configured)
      step "Current terminal: export PATH=\"$BIN_DIR:\$PATH\" && corbanu"
      step "Future terminals: open a new terminal and run: corbanu"
      step "PATH is already configured in $path_profile"
      ;;
    *)
      step "Current terminal: corbanu"
      step "Future terminals: open a new terminal and run: corbanu"
      ;;
  esac
}

maybe_launch_pfterminal_now() {
  if prompt_yes_no "Start Corbanu Terminal now?"; then
    step "Launching Corbanu Terminal"
    release_install_lock 2>/dev/null || true
    if [ -e /dev/tty ]; then
      exec "$BIN_PATH" </dev/tty
    else
      exec "$BIN_PATH"
    fi
  fi
}

detect_conflicting_install() {
  existing_path="$(resolve_existing_pfterminal)"
  manager="$(classify_existing_pfterminal "$existing_path" || true)"

  if [ -z "$manager" ]; then
    return
  fi

  conflict_manager="$manager"
  conflict_path="$existing_path"
  step "Detected an existing legacy terminal install at $existing_path"
  warn "Multiple managed terminal installs are ambiguous because PATH order decides which command runs."
}

handle_conflicting_install() {
  if [ -z "$conflict_manager" ]; then
    return
  fi

  case "$conflict_manager" in
    manual)
      warn "Leaving the existing legacy terminal at $conflict_path in place. Put $BIN_DIR earlier on PATH to use Corbanu Terminal."
      return
      ;;
    bun)
      uninstall_cmd="bun remove -g @corbanucore/terminal @agticorp/pfterminal"
      ;;
    *)
      uninstall_cmd="npm uninstall -g @corbanucore/terminal @agticorp/pfterminal"
      ;;
  esac

  if prompt_yes_no "Uninstall the existing $conflict_manager-managed legacy terminal now?"; then
    step "Running: $uninstall_cmd"
    if ! sh -c "$uninstall_cmd"; then
      warn "Failed to uninstall the existing $conflict_manager-managed legacy terminal. Continuing with the standalone install."
    fi
  else
    warn "Leaving the existing $conflict_manager-managed legacy terminal installed. PATH order will determine which terminal command runs."
  fi
}

install_package_release() {
  release_dir="$1"
  archive_path="$2"
  stage_release="$RELEASES_DIR/.staging.$(basename "$release_dir").$$"

  mkdir -p "$RELEASES_DIR"
  rm -rf "$stage_release"
  mkdir -p "$stage_release"
  tar -xzf "$archive_path" -C "$stage_release"
  chmod 0755 \
    "$stage_release/bin/corbanu" \
    "$stage_release/bin/corbanu-walletd" \
    "$stage_release/bin/codex-code-mode-host" \
    "$stage_release/codex-path/rg"
  for optional_binary in \
    corbanu-debug \
    corbanu-acp; do
    if [ -f "$stage_release/bin/$optional_binary" ]; then
      chmod 0755 "$stage_release/bin/$optional_binary"
    fi
  done
  if [ -f "$stage_release/codex-resources/bwrap" ]; then
    chmod 0755 "$stage_release/codex-resources/bwrap"
  fi
  ln -sf "bin/corbanu" "$stage_release/corbanu"

  if [ -e "$release_dir" ] || [ -L "$release_dir" ]; then
    rm -rf "$release_dir"
  fi
  mv "$stage_release" "$release_dir"
}

install_legacy_platform_npm_release() {
  release_dir="$1"
  archive_path="$2"
  target="$3"
  stage_release="$RELEASES_DIR/.staging.$(basename "$release_dir").$$"
  extract_dir="$tmp_dir/extract"
  vendor_root="$extract_dir/package/vendor/$target"

  mkdir -p "$RELEASES_DIR"
  rm -rf "$stage_release" "$extract_dir"
  mkdir -p "$stage_release/codex-resources" "$extract_dir"
  tar -xzf "$archive_path" -C "$extract_dir"

  cp "$vendor_root/codex/codex" "$stage_release/pfterminal"
  cp "$vendor_root/path/rg" "$stage_release/codex-resources/rg"
  chmod 0755 "$stage_release/pfterminal" "$stage_release/codex-resources/rg"
  if [ -f "$vendor_root/codex-resources/bwrap" ]; then
    cp "$vendor_root/codex-resources/bwrap" "$stage_release/codex-resources/bwrap"
    chmod 0755 "$stage_release/codex-resources/bwrap"
  fi

  if [ -e "$release_dir" ] || [ -L "$release_dir" ]; then
    rm -rf "$release_dir"
  fi
  mv "$stage_release" "$release_dir"
}

release_dir_is_complete() {
  release_dir="$1"
  expected_version="$2"
  expected_target="$3"
  layout="$4"

  [ -d "$release_dir" ] &&
    [ "$(basename "$release_dir")" = "$expected_version-$expected_target" ] ||
    return 1

  case "$layout" in
    package)
      [ -f "$release_dir/codex-package.json" ] &&
        [ -x "$release_dir/bin/corbanu" ] &&
        [ -x "$release_dir/bin/corbanu-walletd" ] &&
        [ -x "$release_dir/bin/codex-code-mode-host" ] &&
        [ -x "$release_dir/corbanu" ] &&
        [ -x "$release_dir/codex-path/rg" ] ||
        return 1
      ;;
    legacy-platform-npm)
      [ -x "$release_dir/pfterminal" ] &&
        [ -x "$release_dir/codex-resources/rg" ] ||
        return 1
      ;;
    *)
      return 1
      ;;
  esac

  case "$layout:$expected_target" in
    package:*linux* | legacy-platform-npm:*linux*)
      [ -x "$release_dir/codex-resources/bwrap" ] || return 1
      ;;
  esac

  installed_version="$(version_from_binary "$release_dir/bin/corbanu" || version_from_binary "$release_dir/corbanu" || version_from_binary "$release_dir/pfterminal" || true)"
  [ "$installed_version" = "$expected_version" ]
}

update_current_link() {
  release_dir="$1"
  tmp_link="$STANDALONE_ROOT/.current.$$"

  replace_path_with_symlink "$CURRENT_LINK" "$release_dir" "$tmp_link"
}

release_terminal_relative_path() {
  release_dir="$1"

  if [ -x "$release_dir/bin/corbanu" ]; then
    printf 'bin/corbanu\n'
  elif [ -x "$release_dir/corbanu" ]; then
    printf 'corbanu\n'
  elif [ -x "$release_dir/bin/pfterminal" ]; then
    printf 'bin/pfterminal\n'
  elif [ -x "$release_dir/pfterminal" ]; then
    printf 'pfterminal\n'
  else
    echo "Installed release does not contain a Corbanu Terminal binary: $release_dir" >&2
    return 1
  fi
}

release_debug_terminal_relative_path() {
  release_dir="$1"

  if [ -x "$release_dir/bin/corbanu-debug" ]; then
    printf 'bin/corbanu-debug\n'
  elif [ -x "$release_dir/bin/pfterminal-debug" ]; then
    printf 'bin/pfterminal-debug\n'
  else
    return 1
  fi
}

remove_managed_debug_wrapper() {
  path="$1"

  if [ -L "$path" ]; then
    case "$(readlink "$path" 2>/dev/null || true)" in
      "$CURRENT_LINK"/*) rm -f "$path" ;;
    esac
  elif [ -f "$path" ] && grep -F "$CURRENT_LINK/" "$path" >/dev/null 2>&1; then
    rm -f "$path"
  fi
}

shell_quote() {
  printf "'%s'" "$(printf '%s' "$1" | sed "s/'/'\\\\''/g")"
}

write_visible_command_wrapper() {
  destination="$1"
  target="$2"
  codex_home="$3"
  tmp_script="$4"

  rm -f "$tmp_script"
  {
    printf '#!/bin/sh\n'
    printf 'export CODEX_HOME=%s\n' "$(shell_quote "$codex_home")"
    printf 'exec %s "$@"\n' "$(shell_quote "$target")"
  } >"$tmp_script"
  chmod 0755 "$tmp_script"

  if mv -f "$tmp_script" "$destination"; then
    return
  fi

  rm -f "$destination"
  mv -f "$tmp_script" "$destination"
}

update_visible_command() {
  release_dir="$1"
  mkdir -p "$BIN_DIR"
  tmp_script="$BIN_DIR/.corbanu.$$"
  debug_tmp_script="$BIN_DIR/.corbanu-debug.$$"
  tmp_link="$BIN_DIR/.codex-code-mode-host.$$"
  if ! terminal_relative_path="$(release_terminal_relative_path "$release_dir")"; then
    exit 1
  fi

  write_visible_command_wrapper "$BIN_PATH" "$CURRENT_LINK/$terminal_relative_path" "$CODEX_HOME_DIR" "$tmp_script"
  remove_managed_debug_wrapper "$LEGACY_BIN_PATH"
  remove_managed_debug_wrapper "$LEGACY_DEBUG_BIN_PATH"

  if [ "$install_layout" = "package" ]; then
    if debug_terminal_relative_path="$(release_debug_terminal_relative_path "$release_dir")"; then
      write_visible_command_wrapper \
        "$DEBUG_BIN_PATH" \
        "$CURRENT_LINK/$debug_terminal_relative_path" \
        "$DEBUG_CODEX_HOME_DIR" \
        "$debug_tmp_script"
      debug_launchers_installed="true"
    else
      warn "release has no debug binary; skipping corbanu-debug launcher"
      remove_managed_debug_wrapper "$DEBUG_BIN_PATH"
    fi
  fi

  if [ "$os" = "darwin" ] && [ -x "$release_dir/bin/codex-code-mode-host" ]; then
    replace_path_with_symlink \
      "$CODE_MODE_HOST_BIN_PATH" \
      "$CURRENT_LINK/bin/codex-code-mode-host" \
      "$tmp_link"
  elif [ "$(readlink "$CODE_MODE_HOST_BIN_PATH" 2>/dev/null || true)" = \
    "$CURRENT_LINK/bin/codex-code-mode-host" ]; then
    rm -f "$CODE_MODE_HOST_BIN_PATH"
  fi
}

verify_visible_command() {
  "$BIN_PATH" --version >/dev/null
  if [ "$debug_launchers_installed" = "true" ]; then
    "$DEBUG_BIN_PATH" --version >/dev/null
  fi
  if [ "$os" = "darwin" ] && [ "$install_layout" = "package" ]; then
    [ -x "$CODE_MODE_HOST_BIN_PATH" ]
  fi
}

release_dir_mtime() {
  path="$1"
  case "$os" in
    darwin) stat -f '%m' "$path" ;;
    *) stat -c '%Y' "$path" ;;
  esac
}

prune_old_releases() {
  case "$KEEP_RELEASES" in
    "" | *[!0-9]*)
      warn "CORBANU_KEEP_RELEASES must be a non-negative integer; skipping release pruning"
      return
      ;;
  esac
  if [ "${#KEEP_RELEASES}" -gt 9 ]; then
    warn "CORBANU_KEEP_RELEASES is too large; skipping release pruning"
    return
  fi

  if [ ! -d "$RELEASES_DIR" ]; then
    return
  fi
  canonical_releases_dir="$(CDPATH= cd -- "$RELEASES_DIR" 2>/dev/null && pwd -P)" || return
  if ! current_target="$(CDPATH= cd -- "$CURRENT_LINK" 2>/dev/null && pwd -P)"; then
    warn "current release link is missing or dangling; skipping release pruning"
    return
  fi
  case "$current_target" in
    "$canonical_releases_dir"/*) ;;
    *)
      warn "current release target is outside the managed releases directory; skipping release pruning"
      return
      ;;
  esac

  prune_list="$STANDALONE_ROOT/.release-prune.$$"
  sorted_prune_list="$prune_list.sorted"
  : >"$prune_list"
  for candidate in "$RELEASES_DIR"/*; do
    [ -d "$candidate" ] || continue
    [ ! -L "$candidate" ] || continue
    candidate_target="$(CDPATH= cd -- "$candidate" 2>/dev/null && pwd -P)" || continue
    [ "$candidate_target" != "$current_target" ] || continue
    mtime="$(release_dir_mtime "$candidate" 2>/dev/null || printf '0')"
    printf '%s\t%s\n' "$mtime" "$candidate" >>"$prune_list"
  done
  sort -rn "$prune_list" >"$sorted_prune_list"

  retained=0
  tab="$(printf '\t')"
  while IFS="$tab" read -r _mtime candidate; do
    [ -n "$candidate" ] || continue
    if [ "$retained" -lt "$KEEP_RELEASES" ]; then
      retained=$((retained + 1))
      continue
    fi
    [ -d "$candidate" ] || continue
    [ ! -L "$candidate" ] || continue
    candidate_target="$(CDPATH= cd -- "$candidate" 2>/dev/null && pwd -P)" || continue
    [ "$candidate_target" != "$current_target" ] || continue
    case "$candidate_target" in
      "$canonical_releases_dir"/*) ;;
      *) continue ;;
    esac
    if rm -rf "$candidate"; then
      step "Pruned old standalone release: $(basename "$candidate")"
    else
      warn "could not prune old standalone release: $candidate"
    fi
  done <"$sorted_prune_list"
  rm -f "$prune_list" "$sorted_prune_list"
}

parse_args "$@"

require_command mktemp
require_command tar

case "$(uname -s)" in
  Darwin)
    os="darwin"
    ;;
  Linux)
    os="linux"
    ;;
  *)
    echo "install.sh supports macOS and Linux. Use install.ps1 on Windows." >&2
    exit 1
    ;;
esac

case "$(uname -m)" in
  x86_64 | amd64)
    arch="x86_64"
    ;;
  arm64 | aarch64)
    arch="aarch64"
    ;;
  *)
    echo "Unsupported architecture: $(uname -m)" >&2
    exit 1
    ;;
esac

if [ "$os" = "darwin" ] && [ "$arch" = "x86_64" ]; then
  if [ "$(sysctl -n sysctl.proc_translated 2>/dev/null || true)" = "1" ]; then
    arch="aarch64"
  fi
fi

if [ "$os" = "darwin" ]; then
  if [ "$arch" = "aarch64" ]; then
    npm_tag="darwin-arm64"
    vendor_target="aarch64-apple-darwin"
    platform_label="macOS (Apple Silicon)"
  else
    npm_tag="darwin-x64"
    vendor_target="x86_64-apple-darwin"
    platform_label="macOS (Intel)"
  fi
else
  if [ "$arch" = "aarch64" ]; then
    npm_tag="linux-arm64"
    vendor_target="aarch64-unknown-linux-musl"
    platform_label="Linux (ARM64)"
  else
    npm_tag="linux-x64"
    vendor_target="x86_64-unknown-linux-gnu"
    platform_label="Linux (x64)"
  fi
fi

if [ -n "$BUNDLED_PACKAGE_ARCHIVE" ] || [ -n "$BUNDLED_CHECKSUM_MANIFEST" ]; then
  if [ -z "$BUNDLED_PACKAGE_ARCHIVE" ] || [ -z "$BUNDLED_CHECKSUM_MANIFEST" ]; then
    echo "CORBANU_PACKAGE_ARCHIVE and CORBANU_CHECKSUM_MANIFEST must be set together." >&2
    exit 1
  fi
  [ -f "$BUNDLED_PACKAGE_ARCHIVE" ] || {
    echo "Bundled package archive does not exist: $BUNDLED_PACKAGE_ARCHIVE" >&2
    exit 1
  }
  [ -f "$BUNDLED_CHECKSUM_MANIFEST" ] || {
    echo "Bundled checksum manifest does not exist: $BUNDLED_CHECKSUM_MANIFEST" >&2
    exit 1
  }
  resolved_version="$(normalize_version "$RELEASE")"
  validate_version "$resolved_version"
  if [ "$resolved_version" = "latest" ]; then
    echo "A bundled Corbanu Terminal package requires an explicit CORBANU_RELEASE." >&2
    exit 1
  fi
  install_layout="package"
  release_source="bundled"
  asset="corbanu-terminal-package-$vendor_target.tar.gz"
  checksum_asset="corbanu-terminal-package_SHA256SUMS"
  if [ "$(basename "$BUNDLED_PACKAGE_ARCHIVE")" != "$asset" ]; then
    echo "Bundled package archive must be named $asset." >&2
    exit 1
  fi
  if [ "$(basename "$BUNDLED_CHECKSUM_MANIFEST")" != "$checksum_asset" ]; then
    echo "Bundled checksum manifest must be named $checksum_asset." >&2
    exit 1
  fi
else
  resolve_release
fi
release_name="$resolved_version-$vendor_target"
release_dir="$RELEASES_DIR/$release_name"
current_version="$(current_installed_version)"

if [ -n "$current_version" ] && [ "$current_version" != "$resolved_version" ]; then
  step "Updating Corbanu Terminal from $current_version to $resolved_version"
elif [ -n "$current_version" ]; then
  step "Updating Corbanu Terminal"
else
  step "Installing Corbanu Terminal"
fi
step "Detected platform: $platform_label"
step "Resolved version: $resolved_version"

detect_conflicting_install

tmp_dir="$(mktemp -d)"
cleanup() {
  release_install_lock
  if [ -n "$tmp_dir" ]; then
    rm -rf "$tmp_dir"
  fi
}
trap cleanup EXIT INT TERM

acquire_install_lock
cleanup_stale_install_artifacts

if ! release_dir_is_complete "$release_dir" "$resolved_version" "$vendor_target" "$install_layout"; then
  if [ -e "$release_dir" ] || [ -L "$release_dir" ]; then
    warn "Found incomplete existing release at $release_dir; reinstalling."
  fi

  archive_path="$tmp_dir/$asset"
  checksum_path="$tmp_dir/$checksum_asset"

  if [ "$release_source" = "bundled" ]; then
    step "Using bundled Corbanu Terminal package"
    cp "$BUNDLED_PACKAGE_ARCHIVE" "$archive_path"
    cp "$BUNDLED_CHECKSUM_MANIFEST" "$checksum_path"
    expected_digest="$(package_archive_digest "$asset" "$checksum_path")"
    verify_archive_digest "$archive_path" "$expected_digest"
  else
    step "Downloading Corbanu Terminal"
    if [ "$install_layout" = "package" ]; then
      checksum_digest="$(release_asset_digest "$checksum_asset")"
      download_file_with_fallback "$checksum_url" "$checksum_fallback_url" "$checksum_path" "$checksum_digest" "$checksum_asset" "$asset"
      expected_digest="$(package_archive_digest "$asset" "$checksum_path")"
    else
      expected_digest="$(release_asset_digest "$asset")"
    fi
    download_file_with_fallback "$download_url" "$download_fallback_url" "$archive_path" "$expected_digest" "$asset"
  fi

  step "Installing standalone package to $release_dir"
  if [ "$install_layout" = "package" ]; then
    install_package_release "$release_dir" "$archive_path"
  else
    install_legacy_platform_npm_release "$release_dir" "$archive_path" "$vendor_target"
  fi
fi
if ! release_dir_is_complete "$release_dir" "$resolved_version" "$vendor_target" "$install_layout"; then
  echo "Installed Corbanu Terminal command did not report expected version $resolved_version." >&2
  exit 1
fi
update_current_link "$release_dir"
update_visible_command "$release_dir"
add_to_path
verify_visible_command
prune_old_releases
release_install_lock
handle_conflicting_install

case "$path_action" in
  added)
    print_launch_instructions
    ;;
  updated)
    print_launch_instructions
    ;;
  configured)
    print_launch_instructions
    ;;
  *)
    step "$BIN_DIR is already on PATH"
    print_launch_instructions
    ;;
esac

printf 'Corbanu Terminal %s installed successfully.\n' "$resolved_version"
maybe_launch_pfterminal_now
