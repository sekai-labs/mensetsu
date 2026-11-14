# Landing & Documentation Architecture

Design, routing, build-time configuration, and isolation properties of `app/landing`.

---

## 1. Architectural Role & Boundary

`app/landing` is an independent **Astro** static application, decoupled from `app/web`:

- `app/web` is the authenticated Nuxt workspace: BFF sessions, SSE streams, workspace pages, and Vue Flow graph views.
- `app/landing` is the public, SEO-oriented landing page and documentation site.
- `app/landing` never imports workspace code and never receives API tokens or session cookies.
- Workspace-root routing is owned by `app/web/pages/index.vue`: authenticated sessions go to `/dashboard`,
  unauthenticated sessions go to `/login`.

The separation keeps the public surface static and cheap while the workspace remains a stateful application.

---

## 2. Build-Time Workspace Handoff

`app/landing/src/data/workspace.ts` resolves the workspace origin from `import.meta.env.PUBLIC_WORKSPACE_URL` and
defaults to `http://localhost:3001`. Astro/Vite inlines public variables during `astro build`.

```typescript
const rawWorkspaceUrl = import.meta.env.PUBLIC_WORKSPACE_URL ?? 'http://localhost:3001';
export const WORKSPACE_URL = rawWorkspaceUrl.replace(/\/+$/, '');
```

Consequences:

- A runtime container environment variable does **not** change the built links.
- Change `PUBLIC_WORKSPACE_URL` and rebuild the image (or rebuild the static site) to repoint sign-in and dashboard links.
- `PUBLIC_SITE_URL` is the canonical origin written into Astro metadata.

---

## 3. Rendering and Routing

- `output: 'static'`; every route is prerendered at build time.
- `trailingSlash: 'always'` makes `/docs/setup/` canonical.
- Landing: `/` (`src/pages/index.astro`), composed from the `components/landing/*` sections.
- Documentation: `/docs/` and `/docs/<section>/` under `src/pages/docs/**`, rendered inside `DocsLayout.astro`.
- Navigation data lives in `src/data/docsNavigation.ts`; it drives the sidebar, breadcrumbs, search index, and pager.
- Interactive behavior is limited to progressive enhancement: sidebar search filtering, code-copy buttons, and anchor navigation.

---

## 4. Documentation Information Architecture

| Route | Content |
| --- | --- |
| `/docs` | System overview, evidence model, capability map, documentation sitemap |
| `/docs/setup` | Prerequisites, secrets, Compose, ports, initial admin |
| `/docs/workflows` | Product workflow and ordered workspace progress phases |
| `/docs/architecture` | Rust API, Python intelligence, gRPC, Redis, SurrealDB, SearXNG |
| `/docs/modules` | Repository tree, context ownership, contracts, clients, infrastructure |
| `/docs/auth` | Password/TOTP auth, web BFF sessions, OAuth provider setup, SSRF allowlist |
| `/docs/metrics` | Token usage routes, daily analytics, dashboard summary, Prometheus |
| `/docs/security` | SSRF, prompt injection, S3 isolation, secrets, PII |
| `/docs/operations` | Migrations, backups, health, logs, limits, upgrades, troubleshooting |
| `/docs/contributing` | Toolchain, contracts, tests, review flow |

Claims are grounded in the current Rust API, Python worker, protobuf contracts, and migrations. The landing app does
not assert an unimplemented workflow engine or provider feature.

---

## 5. Styling and Design Tokens

- Tailwind CSS v4 through `@tailwindcss/vite`; the theme is declared in `src/styles/global.css` with `@theme` and `@custom-variant dark`.
- `Inter` for prose and `JetBrains Mono` for code, identifiers, and stage names.
- Calm slate surfaces, indigo primary accent (`#4f46e5`), restrained borders, and high information density.
- No glassmorphism, neon gradients, glowing orbs, or decorative animation.
- Dark-mode utilities are available under the `.dark` class; the static build defaults to light and does not ship a theme toggle.
