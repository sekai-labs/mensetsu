import { describe, it } from 'node:test';
import assert from 'node:assert/strict';

import { clampText } from '../src/shared/util.js';

const EMOJI = '😀';

describe('clampText', () => {
  it('returns text within the UTF-16 unit budget unchanged', () => {
    assert.strictEqual(clampText('hello', 10), 'hello');
    assert.strictEqual(clampText('hello', 5), 'hello');
    assert.strictEqual(clampText('hello', 3), 'hel');
  });

  it('never truncates inside a surrogate pair at the boundary', () => {
    const text = `ab${EMOJI}`;
    assert.strictEqual(text.length, 4);

    assert.strictEqual(clampText(text, 4), text);
    assert.strictEqual(clampText(text, 3), 'ab');
    assert.strictEqual(clampText(EMOJI, 1), '');
    assert.strictEqual(clampText(`${EMOJI}x`, 2), EMOJI);
  });

  it('treats non-string input as empty text', () => {
    assert.strictEqual(clampText(null, 3), '');
    assert.strictEqual(clampText(undefined, 3), '');
  });
});
