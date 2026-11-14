# Landing App Deployment Guide

Instructions for building, containerizing, and deploying `app/landing`.

---

## 1. Containerization

The image is a two-stage build:

1. **Builder (`oven/bun:1-slim`)**
   - Installs from `package.json` and `bun.lock` with `bun install --frozen-lockfile`.
   - Runs `bun run build` with the build-time `PUBLIC_WORKSPACE_URL` and `PUBLIC_SITE_URL`.
   - Produces a static `dist/` directory.
2. **Runner (`nginx:1.27-alpine`)**
   - Serves `dist/` with `nginx.conf`.
   - Listens on port `80`.
   - Uses directory indexes, a `404.html` fallback, security headers, gzip, and immutable caching for `/_astro/`.

There is no Node server, adapter, or SSR entrypoint. Astro inlines `PUBLIC_*` values at build time, so changing the
workspace origin requires rebuilding the image.

---

## 2. Docker Compose Integration

```yaml
landing:
  build:
    context: .
    dockerfile: app/landing/Dockerfile
    args:
      PUBLIC_WORKSPACE_URL: ${PUBLIC_WORKSPACE_URL:-http://localhost:3001}
      PUBLIC_SITE_URL: ${PUBLIC_SITE_URL:-http://localhost:3002}
  container_name: mensetsu-landing
  ports:
    - "3002:80"
  healthcheck:
    test: ["CMD", "wget", "-qO-", "http://127.0.0.1/"]
    interval: 10s
    timeout: 3s
    start_period: 5s
    retries: 3
  restart: unless-stopped
  mem_limit: 512m
  cpus: 1.0
```

---

## 3. Deployment Commands

```bash
# Build with a production workspace origin.
PUBLIC_WORKSPACE_URL=https://app.example.com \
PUBLIC_SITE_URL=https://docs.example.com \
  docker compose build landing

docker compose up -d landing
docker compose ps landing
docker compose logs -f landing
```

Verify the build-time handoff:

```bash
curl -fsS http://localhost:3002/ | grep -o 'https://app.example.com' | head -1
```

---

## 4. Reverse Proxy

The container already serves HTTP on port 80. A reverse proxy only needs to forward traffic and terminate TLS:

```nginx
server {
    server_name docs.example.com;
    listen 443 ssl http2;

    location / {
        proxy_pass http://127.0.0.1:3002;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

`nginx.conf` sets `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, and a strict referrer policy. Add a
Content-Security-Policy at the proxy only after checking the inline scripts used for documentation search and code
copying.

---

## 5. Verification

```bash
for path in / /docs/ /docs/setup/ /docs/architecture/ /docs/modules/ \
            /docs/metrics/ /docs/operations/; do
  printf '%-28s %s\n' "$path" \
    "$(curl -s -o /dev/null -w '%{http_code}' "http://localhost:3002${path}")"
done
```

All routes should return `200`. The container healthcheck should report `healthy` in `docker compose ps`.
