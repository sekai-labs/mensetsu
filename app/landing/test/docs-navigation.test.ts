import { describe, it, expect } from 'vitest';
import {
  DOCS_SECTIONS,
  ALL_DOC_ITEMS,
  getItemByPath,
  getAdjacentItems,
  searchDocs,
} from '../src/data/docsNavigation';

describe('Documentation Navigation & Sitemap (Astro)', () => {
  it('defines four semantic documentation sections', () => {
    expect(DOCS_SECTIONS).toHaveLength(4);
    expect(DOCS_SECTIONS.map((s) => s.title)).toEqual([
      'Getting Started',
      'Core Engine & Pipelines',
      'Identity & Governance',
      'Community',
    ]);
  });

  it('contains all ten mandatory documentation routes', () => {
    expect(ALL_DOC_ITEMS).toHaveLength(10);
    expect(ALL_DOC_ITEMS.map((item) => item.path)).toEqual([
      '/docs',
      '/docs/setup',
      '/docs/workflows',
      '/docs/architecture',
      '/docs/modules',
      '/docs/auth',
      '/docs/metrics',
      '/docs/security',
      '/docs/operations',
      '/docs/contributing',
    ]);
  });

  it('does not duplicate routes or ids', () => {
    const paths = ALL_DOC_ITEMS.map((item) => item.path);
    const ids = ALL_DOC_ITEMS.map((item) => item.id);
    expect(new Set(paths).size).toBe(paths.length);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it('resolves items by exact path or trailing slash', () => {
    expect(getItemByPath('/docs/auth/')?.id).toBe('auth');
    expect(getItemByPath('/docs/modules/')?.title).toBe('Module Tree & Ownership');
  });

  it('resolves adjacent items accurately', () => {
    expect(getAdjacentItems('/docs').prev).toBeUndefined();
    expect(getAdjacentItems('/docs').next?.id).toBe('setup');

    const authAdj = getAdjacentItems('/docs/auth');
    expect(authAdj.prev?.id).toBe('modules');
    expect(authAdj.next?.id).toBe('metrics');

    const lastAdj = getAdjacentItems('/docs/contributing');
    expect(lastAdj.prev?.id).toBe('operations');
    expect(lastAdj.next).toBeUndefined();
  });

  it('searches documentation across titles, descriptions, and tags', () => {
    expect(searchDocs('oauth')[0].id).toBe('auth');
    expect(searchDocs('ssrf').some((item) => item.id === 'security')).toBe(true);
    expect(searchDocs('prometheus')[0].id).toBe('metrics');
    expect(searchDocs('backup')[0].id).toBe('operations');
    expect(searchDocs('module tree')[0].id).toBe('modules');
    expect(searchDocs('nonexistent_random_token_123')).toHaveLength(0);
  });
});
