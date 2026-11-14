import { greenhouseAdapter } from './greenhouse.js';
import { topcvAdapter } from './topcv.js';
import { linkedinAdapter } from './linkedin.js';
import type { SiteAdapter } from '../../shared/types.js';

export const adapters: readonly SiteAdapter[] = [
  greenhouseAdapter,
  topcvAdapter,
  linkedinAdapter
];

export function getAdapterForUrl(rawUrl: string): SiteAdapter | null {
  for (const adapter of adapters) {
    if (adapter.matches(rawUrl)) {
      return adapter;
    }
  }
  return null;
}

export function getAdapterForDiscoveryUrl(rawUrl: string): SiteAdapter | null {
  for (const adapter of adapters) {
    if (typeof adapter.matchesDiscovery === 'function' && adapter.matchesDiscovery(rawUrl)) {
      return adapter;
    }
  }
  return null;
}

export { greenhouseAdapter, topcvAdapter, linkedinAdapter };
