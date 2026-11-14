FROM oven/bun:1-slim AS builder

WORKDIR /build

COPY app/landing/package.json app/landing/bun.lock ./app/landing/
WORKDIR /build/app/landing
RUN bun install --frozen-lockfile

WORKDIR /build
COPY app/landing ./app/landing

WORKDIR /build/app/landing
ARG PUBLIC_WORKSPACE_URL=http://localhost:3001
ARG PUBLIC_SITE_URL=http://localhost:3002
ENV PUBLIC_WORKSPACE_URL=$PUBLIC_WORKSPACE_URL
ENV PUBLIC_SITE_URL=$PUBLIC_SITE_URL
RUN bun run build

FROM nginx:1.27-alpine AS runner

COPY app/landing/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /build/app/landing/dist /usr/share/nginx/html

EXPOSE 80

HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
  CMD wget -qO- http://127.0.0.1/ >/dev/null || exit 1

CMD ["nginx", "-g", "daemon off;"]
