#!/usr/bin/env bash

# Source-only helpers; also used by the setup recovery regression tests.
install_remote_env_packages() {
  local container_name="$1"
  local attempt delay
  local attempts=6

  for (( attempt = 1; attempt <= attempts; attempt++ )); do
    # APT otherwise reports some failed index downloads as success. Keep both
    # metadata and package failures fatal, and bound even a stalled DNS request.
    if docker exec "${container_name}" timeout --kill-after=10s 180s sh -c \
      'apt-get -o APT::Update::Error-Mode=any -o Acquire::Retries=1 -o Acquire::http::Timeout=30 update &&
       DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::Retries=1 -o Acquire::http::Timeout=30 install -y python3 zsh bubblewrap'; then
      return 0
    fi

    echo "apt-get failed in ${container_name} (attempt ${attempt}/${attempts})" >&2
    if (( attempt == 1 )); then
      # Record both resolvers before changing mirrors; an outage may affect the
      # runner too. Do not silently change the remote executor's DNS policy.
      echo "Runner resolver:" >&2
      cat /etc/resolv.conf >&2 || true
      echo "Container resolver and Ubuntu lookups:" >&2
      docker exec "${container_name}" timeout --kill-after=5s 15s sh -c \
        'cat /etc/resolv.conf; getent ahostsv4 archive.ubuntu.com; getent ahostsv4 security.ubuntu.com; getent ahostsv4 azure.archive.ubuntu.com' >&2 || true
      # Ubuntu 24.04 uses deb822 sources. Only replace the two official mirror
      # URLs; suites, components, signing keys and signature verification stay.
      # Azure is the mirror used by GitHub's Ubuntu runners.
      if ! docker exec "${container_name}" sh -c \
        'sed -i -e "s|http://archive.ubuntu.com/ubuntu|http://azure.archive.ubuntu.com/ubuntu|g" -e "s|http://security.ubuntu.com/ubuntu|http://azure.archive.ubuntu.com/ubuntu|g" /etc/apt/sources.list.d/ubuntu.sources'; then
        echo "Could not configure the Ubuntu mirror fallback" >&2
        return 1
      fi
      echo "Retrying with azure.archive.ubuntu.com" >&2
    fi
    if (( attempt < attempts )); then
      # 30 + 60 + 120 + 120 + 120 = 450 seconds between attempts, plus bounded
      # APT work (at most 180s + 10s kill grace per attempt).
      delay=$((30 << (attempt - 1)))
      (( delay <= 120 )) || delay=120
      echo "Retrying package setup in ${delay}s" >&2
      sleep "${delay}"
    fi
  done
  echo "apt-get failed in ${container_name} after ${attempts} attempts; remote tests cannot start" >&2
  return 1
}
