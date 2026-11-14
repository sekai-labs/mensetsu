# Landing App Development Guide

Developing, extending, and verifying the `app/landing` Astro application.

---

## 1. Prerequisites

- **Bun 1.x** (the lockfile is `bun.lock`).
- No local Node runtime is required: the package scripts invoke Astro with `bun --bun`, and nginx serves the production static build.
- If you invoke `astro` directly instead of through the scripts, use Node `>=22.23.4` (the version required by Astro 7).

---

## 2. Local Execution

```bash
cd app/landing

bun install --frozen-lockfile
bun run dev

# Different port
bun run dev -- --port 3002
```

Astro's configured dev port is `3000` (`astro.config.mjs`). From the repository root, `make dev-landing` starts the
same command.

Useful checks:

```bash
bun run check   # Astro/TypeScript diagnostics
bun run test    # focused Vitest suite
bun run build   # static output in dist/
bun run preview # inspect the built site
```

---

## 3. Adding a Documentation Page

1. Create `src/pages/docs/<section>/index.astro`.
2. Import and wrap the content in `DocsLayout`:
   ```astro
   ---
   import DocsLayout from '../../../layouts/DocsLayout.astro';
   import DocsBreadcrumb from '../../../components/docs/DocsBreadcrumb.astro';
   ---

   <DocsLayout title="Topic | Mensetsu Documentation" description="...">
     <DocsBreadcrumb crumbs={[{ label: 'Topic' }]} />
     <h1>Topic</h1>
     <!-- content -->
   </DocsLayout>
   ```
3. Register the page in `DOCS_SECTIONS` in `src/data/docsNavigation.ts`:
   ```typescript
   {
     id: 'new-topic',
     title: 'New Topic Title',
     description: 'Summary of the new topic.',
     path: '/docs/new-topic',
     badge: 'Optional Badge',
     tags: ['tag1', 'tag2'],
   }
   ```
4. Add the page to `guides` in `src/pages/docs/index.astro` and to the footer links if it belongs in global navigation.
5. Keep claims verifiable. Check the Rust route/DTO, proto, migration, or Python module before documenting behavior.

There is no content-collection pipeline. Documentation is Astro-native `.astro` content so it can use the shared
`Callout`, `CodeBlock`, and table styles without a second rendering path.

---

## 4. Testing & Verification

Vitest runs in a Node environment from `test/`:

- `content-integrity.test.ts`: verifies landing identity, absence of hype, and required sections.
- `docs-navigation.test.ts`: verifies section order, unique paths, search, and adjacent links.
- `oauth-facts.test.ts`: verifies OAuth routes and invariants against the documented contract.
- `workspace-url.test.ts`: verifies workspace URL normalization and link generation.

```bash
bun run test
bun run check
bun run build
```

Do not run tests or builds concurrently with other active implementation workers in a shared checkout.
