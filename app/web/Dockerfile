FROM oven/bun:1-slim AS builder

WORKDIR /build

COPY app/web/package.json app/web/bun.lock ./app/web/
WORKDIR /build/app/web
RUN bun install --frozen-lockfile

WORKDIR /build
COPY contracts/rest/src ./contracts/rest/src
COPY app/web ./app/web

WORKDIR /build/app/web
RUN bun run build

FROM node:22-slim AS runner

WORKDIR /app

ENV NODE_ENV=production
ENV PORT=3000
ENV HOST=0.0.0.0

USER node

COPY --chown=node:node --from=builder /build/app/web/.output ./

EXPOSE 3000

CMD ["node", "server/index.mjs"]
