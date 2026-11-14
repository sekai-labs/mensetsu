export interface DocItem {
  id: string;
  title: string;
  description: string;
  path: string;
  badge?: string;
  tags?: string[];
}

export interface DocSection {
  title: string;
  items: DocItem[];
}

export const DOCS_SECTIONS: DocSection[] = [
  {
    title: 'Getting Started',
    items: [
      {
        id: 'overview',
        title: 'System Overview',
        description: 'Introduction to Mensetsu, evidence-backed philosophy, and core capabilities.',
        path: '/docs',
        badge: 'Start here',
        tags: ['overview', 'philosophy', 'capabilities', 'evidence', 'product'],
      },
      {
        id: 'setup',
        title: 'Local Setup & Deployment',
        description: 'Prerequisites, Docker Compose configuration, environment secrets, and initial setup.',
        path: '/docs/setup',
        badge: 'DevOps',
        tags: ['docker', 'compose', 'env', 'secrets', 'installation', 'minio', 'surrealdb', 'deployment'],
      },
    ],
  },
  {
    title: 'Core Engine & Pipelines',
    items: [
      {
        id: 'workflows',
        title: 'Product Workflows',
        description: 'Asynchronous durable pipeline from resume parsing to interview question assembly.',
        path: '/docs/workflows',
        badge: 'Pipeline',
        tags: ['workflows', 'resume', 'job description', 'redis-streams', 'pipeline', 'research', 'sse'],
      },
      {
        id: 'architecture',
        title: 'API & Service Architecture',
        description: 'Rust Axum core, Python intelligence worker, bi-directional gRPC, and SurrealDB schema.',
        path: '/docs/architecture',
        badge: 'Topology',
        tags: ['rust', 'axum', 'python', 'grpc', 'redis', 'surrealdb', 'searxng', 'architecture'],
      },
      {
        id: 'modules',
        title: 'Module Tree & Ownership',
        description: 'Repository layout, Rust bounded contexts, Python packages, clients, contracts, and infrastructure.',
        path: '/docs/modules',
        badge: 'Code map',
        tags: ['modules', 'tree', 'repository', 'rust', 'python', 'web', 'landing', 'contracts', 'infra'],
      },
    ],
  },
  {
    title: 'Identity & Governance',
    items: [
      {
        id: 'auth',
        title: 'Authentication & OAuth Setup',
        description: 'Password and TOTP sign-in, web BFF sessions, Google/GitHub/Facebook/custom OIDC setup, and SSRF allowlisting.',
        path: '/docs/auth',
        badge: 'OAuth / OIDC',
        tags: ['auth', 'oauth', 'oidc', 'google', 'github', 'facebook', 'sso', 'jwt', 'session', 'security'],
      },
      {
        id: 'metrics',
        title: 'Metrics & Usage',
        description: 'Role-scoped token usage, daily analytics, dashboard summary, and Prometheus scraping.',
        path: '/docs/metrics',
        badge: 'Observability',
        tags: ['metrics', 'prometheus', 'token usage', 'analytics', 'usage', 'admin', 'observability'],
      },
      {
        id: 'security',
        title: 'Security & Privacy Boundaries',
        description: 'SSRF guardrails, indirect prompt injection defense, S3 signed URLs, and local retention.',
        path: '/docs/security',
        badge: 'Verified',
        tags: ['security', 'ssrf', 'prompt injection', 'privacy', 'pii', 'retention', 'secrets'],
      },
      {
        id: 'operations',
        title: 'Operations & Maintenance',
        description: 'Migrations, backups, health probes, logs, resource limits, upgrades, and troubleshooting.',
        path: '/docs/operations',
        badge: 'Runbook',
        tags: ['operations', 'migrations', 'backup', 'restore', 'upgrade', 'health', 'logs', 'troubleshooting'],
      },
    ],
  },
  {
    title: 'Community',
    items: [
      {
        id: 'contributing',
        title: 'Contributing Guide',
        description: 'Development standards, contract generation, testing expectations, and contribution flow.',
        path: '/docs/contributing',
        badge: 'Community',
        tags: ['contributing', 'standards', 'contracts', 'openapi', 'testing', 'review'],
      },
    ],
  },
];

export const ALL_DOC_ITEMS: DocItem[] = DOCS_SECTIONS.flatMap((s) => s.items);

export function getItemByPath(path: string): DocItem | undefined {
  const cleanPath = path.replace(/\/+$/, '') || '/docs';
  return ALL_DOC_ITEMS.find((item) => item.path.replace(/\/+$/, '') === cleanPath);
}

export function getAdjacentItems(path: string): { prev?: DocItem; next?: DocItem } {
  const cleanPath = path.replace(/\/+$/, '') || '/docs';
  const index = ALL_DOC_ITEMS.findIndex((item) => item.path.replace(/\/+$/, '') === cleanPath);
  if (index === -1) {
    return {};
  }
  return {
    prev: index > 0 ? ALL_DOC_ITEMS[index - 1] : undefined,
    next: index < ALL_DOC_ITEMS.length - 1 ? ALL_DOC_ITEMS[index + 1] : undefined,
  };
}

export function searchDocs(query: string): DocItem[] {
  const q = query.trim().toLowerCase();
  if (!q) return [];
  return ALL_DOC_ITEMS.filter((item) => {
    const matchTitle = item.title.toLowerCase().includes(q);
    const matchDesc = item.description.toLowerCase().includes(q);
    const matchTags = item.tags?.some((t) => t.toLowerCase().includes(q));
    return matchTitle || matchDesc || matchTags;
  });
}
