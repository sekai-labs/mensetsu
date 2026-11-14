# Mensetsu Landing & Documentation (`app/landing`)

Independent **Astro** application serving the public landing page and full documentation for **Mensetsu** (面接).

The landing app is deliberately separate from the authenticated workspace. It has no database, no session cookie,
and no API credentials. Sign-in and workspace actions are plain links to `PUBLIC_WORKSPACE_URL`.

---

## 1. Stack

| Concern | Choice |
| --- | --- |
| Framework | Astro 7, static `output: 'static'` (scripts run Astro under Bun) |
| Styling | Tailwind CSS v4 via `@tailwindcss/vite`, typography plugin |
| Icons | `@lucide/astro` |
| Tests | Vitest (unit checks for navigation, workspace URLs, content invariants, OAuth facts) |
| Runtime | nginx serving the prerendered `dist/` directory |
| Port | Container `80`, host `3002` in `docker-compose.yml` |

`trailingSlash: 'always'` keeps every route canonical (`/docs/setup/`). nginx resolves directory indexes and falls
back to `404.html`.

---

## 2. Directory Tree

```
app/landing/
├── astro.config.mjs             Static Astro, Tailwind v4, site URL, port 3000
├── Dockerfile                   Bun build stage + nginx static runtime
├── nginx.conf                   Clean-URL, 404, security headers, asset caching
├── package.json                 Astro, Tailwind, Lucide, Vitest
├── bun.lock                     Bun dependency lockfile
├── tsconfig.json                Strict TypeScript configuration
├── vitest.config.ts             Node-environment unit tests
├── .dockerignore                Build-context exclusions
├── README.md                    This file
├── docs/
│   ├── architecture.md          Landing runtime and integration architecture
│   ├── deployment.md            Container, build args, nginx, and verification
│   └── development.md           Local development, routes, and tests
├── public/
│   ├── favicon.svg
│   └── logo.svg
├── src/
│   ├── components/
│   │   ├── landing/             Hero, pipeline, feature grid, architecture, CTA
│   │   ├── docs/                Sidebar, breadcrumb, pager
│   │   ├── navigation/          Header and footer
│   │   └── ui/                  Badge, button, callout, code block
│   ├── data/
│   │   ├── docsNavigation.ts    Documentation tree, search, adjacent links
│   │   └── workspace.ts         PUBLIC_WORKSPACE_URL resolver
│   ├── layouts/
│   │   ├── DocsLayout.astro
│   │   └── Layout.astro
│   ├── pages/
│   │   ├── index.astro
│   │   ├── 404.astro
│   │   └── docs/
│   │       ├── index.astro                 System overview
│   │       ├── setup/index.astro           Local setup and deployment
│   │       ├── workflows/index.astro       Product workflows and pipeline
│   │       ├── architecture/index.astro    API and service architecture
│   │       ├── modules/index.astro         Module tree and ownership
│   │       ├── auth/index.astro            Authentication and OAuth setup
│   │       ├── metrics/index.astro         Metrics and usage
│   │       ├── security/index.astro        Security and privacy boundaries
│   │       ├── operations/index.astro      Operations and maintenance
│   │       └── contributing/index.astro    Contributing guide
│   └── styles/
│       └── global.css           Tailwind import, theme tokens, base styles
└── test/
    ├── content-integrity.test.ts
    ├── docs-navigation.test.ts
    ├── oauth-facts.test.ts
    └── workspace-url.test.ts
```

---

## 3. Configuration

| Variable | When | Purpose |
| --- | --- | --- |
| `PUBLIC_WORKSPACE_URL` | **Build time** | Workspace origin used by Sign In, Create Account, and Launch Workspace links. Default `http://localhost:3001`. |
| `PUBLIC_SITE_URL` | **Build time** | Canonical site origin written into Astro metadata. Default `http://localhost:3002`. |

Astro inlines `PUBLIC_*` values during `astro build`. Changing either value requires a rebuild; runtime environment
variables do not affect an already-built static image.

The Docker build accepts the same values as build arguments:

```bash
docker compose build landing \
  --build-arg PUBLIC_WORKSPACE_URL=https://app.example.com \
  --build-arg PUBLIC_SITE_URL=https://docs.example.com
```

`docker-compose.yml` forwards `PUBLIC_WORKSPACE_URL` and `PUBLIC_SITE_URL` from the shell or `.env`.

---

## 4. Commands

```bash
bun install --frozen-lockfile   # install dependencies
bun run dev                     # Astro dev server on :3000
bun run check                   # Astro/TypeScript diagnostics
bun run test                    # focused Vitest suite
bun run build                   # static output in dist/
bun run preview                 # local preview of dist/
```

Production image:

```bash
docker compose build landing
docker compose up -d landing
curl -fsS http://localhost:3002/ >/dev/null
```

---

## 5. Routing Verification

```bash
for path in / /docs/ /docs/setup/ /docs/workflows/ /docs/architecture/ \
            /docs/modules/ /docs/auth/ /docs/metrics/ /docs/security/ \
            /docs/operations/ /docs/contributing/; do
  code=$(curl -s -o /dev/null -w '%{http_code}' "http://localhost:3002${path}")
  printf '%-28s %s\n' "$path" "$code"
done
```

All routes should return `200`. Check that the rendered HTML contains the configured workspace origin:

```bash
curl -fsS http://localhost:3002/ | grep -o 'http://localhost:3001' | head -1
```

---

## 6. Separation From the Workspace

- The landing app never imports `app/web` code and never receives API tokens.
- All authentication and workspace state live in `app/web` and the Rust API.
- Workspace root routing is owned by `app/web/pages/index.vue`: authenticated sessions go to `/dashboard`,
  unauthenticated sessions go to `/login`.
- Documentation must describe only implemented behavior. Do not add framework, workflow, or provider claims without
  checking the current Rust API, Python worker, contracts, or migrations.

---

## 7. Verification Before Commit

```bash
bun run check
bun run test
bun run build
```

Do not run these commands concurrently with other active implementation workers in a shared checkout.
