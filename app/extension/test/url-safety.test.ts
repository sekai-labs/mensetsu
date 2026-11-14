import { describe, it } from 'node:test';
import assert from 'node:assert/strict';

import { isUnsafeHost, safeHttpsUrl } from '../src/shared/url-safety.js';

describe('Saved job destination safety', () => {
  it('keeps public HTTPS job destinations intact', () => {
    assert.strictEqual(
      safeHttpsUrl('https://boards.greenhouse.io/gitlab/jobs/8629019002'),
      'https://boards.greenhouse.io/gitlab/jobs/8629019002'
    );
    assert.strictEqual(
      safeHttpsUrl('https://www.topcv.vn/brand/example/tuyen-dung/backend-developer/123.html'),
      'https://www.topcv.vn/brand/example/tuyen-dung/backend-developer/123.html'
    );
  });

  it('rejects non-HTTPS, credential-bearing, and port-bearing URLs', () => {
    assert.strictEqual(safeHttpsUrl('http://boards.greenhouse.io/jobs/1'), '');
    assert.strictEqual(safeHttpsUrl('https://user:secret@boards.greenhouse.io/jobs/1'), '');
    assert.strictEqual(safeHttpsUrl('https://boards.greenhouse.io:8443/jobs/1'), '');
  });

  it('rejects IP-literal hosts including normalized IPv4 forms', () => {
    assert.strictEqual(safeHttpsUrl('https://127.0.0.1/apply'), '');
    assert.strictEqual(safeHttpsUrl('https://192.168.1.1/apply'), '');
    assert.strictEqual(safeHttpsUrl('https://10.0.0.5/apply'), '');
    assert.strictEqual(safeHttpsUrl('https://[::1]/apply'), '');
    assert.strictEqual(safeHttpsUrl('https://2130706433/apply'), '');
    assert.strictEqual(safeHttpsUrl('https://0x7f000001/apply'), '');
  });

  it('rejects localhost, private suffixes, and single-label hosts', () => {
    assert.strictEqual(safeHttpsUrl('https://localhost/apply'), '');
    assert.strictEqual(safeHttpsUrl('https://localhost./apply'), '');
    assert.strictEqual(safeHttpsUrl('https://review.localhost/apply'), '');
    assert.strictEqual(safeHttpsUrl('https://intranet/apply'), '');
    assert.strictEqual(safeHttpsUrl('https://careers.local/apply'), '');
    assert.strictEqual(safeHttpsUrl('https://apply.internal/apply'), '');
    assert.strictEqual(safeHttpsUrl('https://example.com:443/apply'), 'https://example.com/apply');
  });

  it('flags unsafe hostnames directly', () => {
    assert.strictEqual(isUnsafeHost('localhost'), true);
    assert.strictEqual(isUnsafeHost('foo.localhost'), true);
    assert.strictEqual(isUnsafeHost('jobs.local'), true);
    assert.strictEqual(isUnsafeHost('jobs.internal'), true);
    assert.strictEqual(isUnsafeHost('jobs'), true);
    assert.strictEqual(isUnsafeHost('172.16.0.1'), true);
    assert.strictEqual(isUnsafeHost('boards.greenhouse.io'), false);
  });

  it('rejects malformed and empty values', () => {
    assert.strictEqual(safeHttpsUrl(''), '');
    assert.strictEqual(safeHttpsUrl(undefined), '');
    assert.strictEqual(safeHttpsUrl('not a url'), '');
  });
});
