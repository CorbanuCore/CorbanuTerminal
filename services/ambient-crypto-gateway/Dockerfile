FROM node:22-bookworm-slim AS build

ENV PNPM_HOME=/pnpm
ENV PATH=$PNPM_HOME:$PATH

RUN corepack enable
WORKDIR /source

COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY services/ambient-crypto-gateway/package.json services/ambient-crypto-gateway/package.json
RUN pnpm install --frozen-lockfile --filter @agticorp/ambient-crypto-gateway...

COPY services/ambient-crypto-gateway/tsconfig.json services/ambient-crypto-gateway/tsconfig.build.json services/ambient-crypto-gateway/
COPY services/ambient-crypto-gateway/src services/ambient-crypto-gateway/src
RUN pnpm --filter @agticorp/ambient-crypto-gateway build
RUN pnpm --filter @agticorp/ambient-crypto-gateway deploy --prod --legacy /gateway

FROM node:22-bookworm-slim AS runtime

ENV NODE_ENV=production
WORKDIR /gateway

COPY --from=build /gateway ./

USER node
CMD ["node", "dist/index.js"]
