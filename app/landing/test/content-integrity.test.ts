import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

describe('Landing Page Product Identity & Content Verification', () => {
  const heroPath = fileURLToPath(new URL('../src/components/landing/HeroSection.astro', import.meta.url));
  const heroContent = readFileSync(heroPath, 'utf-8');

  it('maintains calm, evidence-backed product identity without hype', () => {
    const raw = heroContent.toLowerCase();

    // Verify absence of hype, fake testimonials, crypto, or fake pricing
    expect(raw).not.toContain('testimonials');
    expect(raw).not.toContain('customers love');
    expect(raw).not.toContain('pricing tiers');
    expect(raw).not.toContain('crypto');
    expect(raw).not.toContain('guaranteed offers');
  });

  it('contains verified evidence requirements and provenance notice', () => {
    expect(heroContent).toContain('Citation Provenance First');
    expect(heroContent).toContain('Self-Hosted & Contained');
    expect(heroContent).toContain('SurrealDB Multi-Model Graph');
  });
});
