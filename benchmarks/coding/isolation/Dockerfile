# Contestant images for leak-isolated coding benchmarks.
#
# One target per harness, so each contestant can see only its own CLI.
# Task packets, verifiers, pristine tests, prior runs, host homes, and provider
# credentials are never copied in; the runner mounts one fresh candidate
# workspace and one empty agent home per run. Build-time version checks use a
# throwaway HOME so /home/bench stays empty and owned by the contestant user.
FROM node:22-bookworm-slim AS node

FROM python:3.12-slim-bookworm AS base
ARG BENCH_UID=1000
ARG BENCH_GID=1000
RUN apt-get update \
    && apt-get install -y --no-install-recommends git ripgrep curl ca-certificates procps less \
    && rm -rf /var/lib/apt/lists/* \
    && pip install --no-cache-dir pytest \
    && groupadd -g "${BENCH_GID}" bench \
    && useradd -u "${BENCH_UID}" -g "${BENCH_GID}" -m -d /home/bench bench \
    && mkdir /workspace && chown bench:bench /workspace
ENV HOME=/home/bench PYTHONNOUSERSITE=1 PYTHONDONTWRITEBYTECODE=1 PYTHONUNBUFFERED=1
WORKDIR /workspace

FROM base AS corbanu
COPY corbanu /opt/corbanu
RUN ln -s /opt/corbanu/bin/corbanu /usr/local/bin/corbanu \
    && HOME=/tmp/build-home corbanu --version && rm -rf /tmp/build-home
USER bench

FROM base AS hermes
COPY hermes /opt/hermes
RUN pip install --no-cache-dir -e /opt/hermes \
    && HOME=/tmp/build-home hermes --version >/dev/null && rm -rf /tmp/build-home
USER bench

FROM base AS kilo
ARG KILO_VERSION
COPY --from=node /usr/local/bin/node /usr/local/bin/node
COPY --from=node /usr/local/lib/node_modules /usr/local/lib/node_modules
RUN ln -s ../lib/node_modules/npm/bin/npm-cli.js /usr/local/bin/npm \
    && ln -s ../lib/node_modules/npm/bin/npx-cli.js /usr/local/bin/npx \
    && export HOME=/tmp/build-home \
    && npm install -g --no-fund --no-audit "@kilocode/cli@${KILO_VERSION}" \
    && kilo --version && rm -rf /tmp/build-home
USER bench
