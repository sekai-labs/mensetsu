import { describe, it } from 'node:test';
import assert from 'node:assert/strict';

import { loadResumePayload } from '../src/background/background.js';
import type { ResumeDownload } from '../src/shared/types.js';

function download(overrides: Partial<ResumeDownload> = {}): ResumeDownload {
  return {
    bytes: new Uint8Array([0x25, 0x50, 0x44, 0x46, 0x2d, 0x31, 0x2e, 0x37]),
    filename: 'candidate_resume.pdf',
    contentType: 'application/pdf',
    ...overrides
  };
}

describe('Selected resume handoff', () => {
  it('returns the downloaded bytes for the chosen resume', async () => {
    const payload = await loadResumePayload({ downloadResume: async () => download() }, 'upload_1');

    assert.deepStrictEqual(payload.bytes, [0x25, 0x50, 0x44, 0x46, 0x2d, 0x31, 0x2e, 0x37]);
    assert.strictEqual(payload.filename, 'candidate_resume.pdf');
    assert.strictEqual(payload.mime, 'application/pdf');
  });

  it('propagates resume download failures instead of filling without the selected resume', async () => {
    await assert.rejects(
      () => loadResumePayload({
        downloadResume: async () => {
          throw new Error('resume download failed');
        }
      }, 'upload_1'),
      /resume download failed/
    );
  });

  it('rejects an empty resume download', async () => {
    await assert.rejects(
      () => loadResumePayload({ downloadResume: async () => download({ bytes: new Uint8Array() }) }, 'upload_1'),
      /no bytes/
    );
  });
});
