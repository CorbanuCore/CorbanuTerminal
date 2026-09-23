#!/usr/bin/env bash
# Build the pinned per-harness contestant images.
#
# Usage: build_image.sh <corbanu-version> <hermes-tag> <kilo-version> <staging-dir>
# Example: build_image.sh 0.1.44 v2026.9.21 7.7.9 /data/scratch/bench-image
#
# The staging directory must not be inside this repository: the image must
# never see task packets or verifiers. Each tag encodes all three
# contestant versions; the image map for the config is printed on success.
set -euo pipefail

corbanu_version="$1"
hermes_tag="$2"
kilo_version="$3"
staging="$4"
here="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$here/../../.." && pwd)"

mkdir -p "$staging"
staging="$(cd "$staging" && pwd)"
case "$staging/" in
  "$repo_root"/*) echo "staging dir must be outside $repo_root" >&2; exit 2 ;;
esac

package="corbanu-terminal-package-x86_64-unknown-linux-gnu.tar.gz"
if [ ! -d "$staging/corbanu" ]; then
  gh release download "rust-v${corbanu_version}" --repo CorbanuCore/CorbanuTerminal \
    -p "$package" -p corbanu-terminal-package_SHA256SUMS -D "$staging" --clobber
  (cd "$staging" && grep " ${package}\$" corbanu-terminal-package_SHA256SUMS | sha256sum -c -)
  mkdir "$staging/corbanu"
  tar -xzf "$staging/$package" -C "$staging/corbanu"
fi
if [ ! -d "$staging/hermes" ]; then
  git clone --quiet --depth 1 --branch "$hermes_tag" \
    https://github.com/NousResearch/hermes-agent.git "$staging/hermes"
fi
cp "$here/Dockerfile" "$staging/Dockerfile"

versions="c${corbanu_version}-h${hermes_tag#v}-k${kilo_version}"
for target in corbanu hermes kilo; do
  docker build --quiet --target "$target" \
    --build-arg "BENCH_UID=$(id -u)" --build-arg "BENCH_GID=$(id -g)" \
    --build-arg "KILO_VERSION=${kilo_version}" \
    -t "corbanu-bench-${target}:${versions}" "$staging" >/dev/null
done
docker run --rm --network none "corbanu-bench-corbanu:${versions}" corbanu --version >&2
docker run --rm --network none "corbanu-bench-hermes:${versions}" sh -c 'hermes --version 2>/dev/null | sed -n 1p' >&2
docker run --rm --network none "corbanu-bench-kilo:${versions}" kilo --version >&2
printf '{"corbanu": "corbanu-bench-corbanu:%s", "hermes": "corbanu-bench-hermes:%s", "kilo": "corbanu-bench-kilo:%s"}\n' \
  "$versions" "$versions" "$versions"
