import { describe, it } from 'node:test';
import assert from 'node:assert/strict';

import { matchDiscoveredJob } from '../src/features/jobs/matching.js';

function job(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    sourceUrl: 'https://www.linkedin.com/jobs/view/1234567890',
    canonicalUrl: 'https://linkedin.com/jobs/view/1234567890',
    roleTitle: 'Senior Backend Engineer',
    companyName: 'Acme',
    jobDescription: 'Build distributed systems in Rust and PostgreSQL.',
    location: 'Remote - Canada',
    applicationUrl: null,
    contactEmail: null,
    observedAt: '2026-11-11T12:00:00.000Z',
    completeness: 0.8,
    evidence: { source: 'linkedin' },
    ...overrides
  };
}

function profile(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    target_role: 'Backend Engineer',
    required_keywords: ['Rust', 'PostgreSQL'],
    excluded_keywords: ['COBOL'],
    target_locations: ['Remote'],
    ...overrides
  };
}

describe('matchDiscoveredJob', () => {
  it('marks a job eligible when every profile criterion matches', () => {
    const result = matchDiscoveredJob(job(), profile());

    assert.strictEqual(result.eligible, true);
    assert.deepStrictEqual(result.missingCriteria, []);
    assert.deepStrictEqual(result.excludedMatches, []);
    assert.deepStrictEqual(result.matchedCriteria, [
      'role: Backend Engineer',
      'keyword: Rust',
      'keyword: PostgreSQL',
      'location: Remote'
    ]);
  });

  it('reports missing required keywords and stays ineligible', () => {
    const result = matchDiscoveredJob(
      job({ jobDescription: 'Build services in Rust.' }),
      profile()
    );

    assert.strictEqual(result.eligible, false);
    assert.deepStrictEqual(result.missingCriteria, ['keyword: PostgreSQL']);
    assert.ok(result.matchedCriteria.includes('keyword: Rust'));
  });

  it('reports excluded keywords without treating them as matched criteria', () => {
    const result = matchDiscoveredJob(
      job({ jobDescription: 'Maintain COBOL and Rust services.' }),
      profile()
    );

    assert.strictEqual(result.eligible, false);
    assert.deepStrictEqual(result.excludedMatches, ['COBOL']);
    assert.ok(!result.matchedCriteria.some((entry) => entry.includes('COBOL')));
  });

  it('matches roles in either direction', () => {
    const narrower = matchDiscoveredJob(job({ roleTitle: 'Backend Engineer' }), profile());
    assert.ok(narrower.matchedCriteria.includes('role: Backend Engineer'));

    const broader = matchDiscoveredJob(job({ roleTitle: 'Engineer' }), profile({ target_role: 'Staff Engineer' }));
    assert.ok(broader.matchedCriteria.includes('role: Staff Engineer'));
  });

  it('reports a mismatched target role', () => {
    const result = matchDiscoveredJob(
      job({ roleTitle: 'Product Designer' }),
      profile({ target_role: 'Backend Engineer' })
    );

    assert.strictEqual(result.eligible, false);
    assert.ok(result.missingCriteria.includes('role: Backend Engineer'));
  });

  it('requires a location match when target locations are configured', () => {
    const result = matchDiscoveredJob(
      job({ location: null }),
      profile()
    );

    assert.strictEqual(result.eligible, false);
    assert.deepStrictEqual(result.missingCriteria, ['location: Remote']);
  });

  it('matches locations in either direction and lists every matching target', () => {
    const result = matchDiscoveredJob(
      job({ location: 'Remote' }),
      profile({ target_locations: ['Remote', 'Montreal, QC'] })
    );

    assert.strictEqual(result.eligible, true);
    assert.ok(result.matchedCriteria.includes('location: Remote'));
    assert.ok(!result.matchedCriteria.includes('location: Montreal, QC'));
  });

  it('ignores case, accents, and surrounding whitespace', () => {
    const result = matchDiscoveredJob(
      job({ roleTitle: 'Ingénieur Backend', jobDescription: 'RUST et postgresql' }),
      profile({ target_role: 'ingenieur backend', required_keywords: [' rust ', 'PostgreSQL'] })
    );

    assert.strictEqual(result.eligible, true);
    assert.deepStrictEqual(result.missingCriteria, []);
  });

  it('folds Vietnamese đ/Đ to d when matching profile terms', () => {
    const jobWithStrokedD = matchDiscoveredJob(
      job({ jobDescription: 'Bạn sẽ được đào tạo bài bản.' }),
      { required_keywords: ['duoc'] }
    );
    assert.deepStrictEqual(jobWithStrokedD.missingCriteria, []);
    assert.deepStrictEqual(jobWithStrokedD.matchedCriteria, ['keyword: duoc']);

    const profileWithStrokedD = matchDiscoveredJob(
      job({ jobDescription: 'Duoc huong nhieu quyen loi.' }),
      { required_keywords: ['ĐƯỢC'] }
    );
    assert.deepStrictEqual(profileWithStrokedD.missingCriteria, []);
    assert.deepStrictEqual(profileWithStrokedD.matchedCriteria, ['keyword: ĐƯỢC']);
  });

  it('does not match keywords inside longer words', () => {
    const result = matchDiscoveredJob(
      job({ jobDescription: 'We value trusting processes and trustworthy vendors.' }),
      profile({ required_keywords: ['Rust'], excluded_keywords: [] })
    );

    assert.strictEqual(result.eligible, false);
    assert.deepStrictEqual(result.missingCriteria, ['keyword: Rust']);
  });

  it('accepts camelCase profiles and snake_case job aliases', () => {
    const result = matchDiscoveredJob(
      {
        source_url: 'https://topcv.vn/viec-lam-123',
        role_title: 'Backend Engineer',
        company_name: 'Acme',
        job_description: 'Rust and PostgreSQL',
        location: 'Remote'
      },
      {
        targetRole: 'Backend Engineer',
        requiredKeywords: ['Rust'],
        excludedKeywords: [],
        targetLocations: ['Remote']
      }
    );

    assert.strictEqual(result.eligible, true);
    assert.deepStrictEqual(result.missingCriteria, []);
  });

  it('does not claim a positive fit when no targeting preferences are configured', () => {
    const result = matchDiscoveredJob(job(), {});

    assert.deepStrictEqual(result, {
      eligible: false,
      matchedCriteria: [],
      missingCriteria: ['No targeting preferences configured'],
      excludedMatches: []
    });
  });

  it('screens exclusions without claiming a positive fit for excluded-only profiles', () => {
    const result = matchDiscoveredJob(
      job({ jobDescription: 'Maintain COBOL services.' }),
      { excluded_keywords: ['COBOL'] }
    );

    assert.strictEqual(result.eligible, false);
    assert.deepStrictEqual(result.missingCriteria, ['No targeting preferences configured']);
    assert.deepStrictEqual(result.excludedMatches, ['COBOL']);
  });

  it('rejects a missing job instead of silently matching it', () => {
    assert.throws(() => matchDiscoveredJob(null, profile()), /requires a discovered job/);
    assert.throws(() => matchDiscoveredJob(undefined, profile()), /requires a discovered job/);
    assert.throws(() => matchDiscoveredJob([], profile()), /requires a discovered job/);
  });
});
