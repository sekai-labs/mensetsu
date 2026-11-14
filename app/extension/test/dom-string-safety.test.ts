import { describe, it } from 'node:test';
import assert from 'node:assert/strict';

import { parseHtml } from './fixtures/dom-helper.js';
import {
  createDiscoveredJob,
  fillPreparedFields,
  labelFor,
  visibleText
} from '../src/content/adapters/dom.js';
import type { DomElement } from '../src/content/adapters/dom-types.js';

describe('DOM string safety', () => {
  it('looks up fields without interpolating IDs into CSS selectors', () => {
    const doc = parseHtml('<form><input id=":r0:" name="candidate" type="text"></form>');
    const form = doc.querySelector('form');
    assert.ok(form);

    const originalQuerySelector = form.querySelector.bind(form);
    form.querySelector = (selector) => {
      if (selector.startsWith('#') || selector.startsWith('[name=')) {
        throw new Error(`Dynamic selector was used: ${selector}`);
      }
      return originalQuerySelector(selector);
    };

    const report = fillPreparedFields(form, { ':r0:': 'Ada Lovelace' });
    assert.equal(form.querySelectorAll('input')[0]?.value, 'Ada Lovelace');
    assert.deepEqual(report.filled.map((field) => field.key), [':r0:']);
  });

  it('resolves labels by exact IDs and all aria-labelledby tokens', () => {
    const ariaDoc = parseHtml(
      '<span id="first-label">First</span><span id="second:label">Last</span>' +
      '<input id=":r0:" aria-labelledby="first-label second:label">'
    );
    const ariaInput = ariaDoc.querySelector('input');
    assert.ok(ariaInput);
    assert.equal(labelFor(ariaInput), 'First Last');

    const labelDoc = parseHtml('<label for=":r0:">Candidate</label><input id=":r0:">');
    const labelledInput = labelDoc.querySelector('input');
    assert.ok(labelledInput);
    assert.equal(labelFor(labelledInput), 'Candidate');
  });

  it('keeps direct text nodes that precede inline elements', () => {
    const text = (textContent: string) => ({ nodeType: 3, textContent }) as DomElement;
    const element = (...childNodes: DomElement[]) => ({
      nodeType: 1,
      childNodes,
      getAttribute: () => null
    }) as DomElement;
    const inlineLink = element(text('World'));
    const container = element(text('Hello '), inlineLink);

    assert.equal(visibleText(container), 'Hello World');
  });

  it('rejects unsafe canonical URLs and falls back to the validated source URL', () => {
    const job = createDiscoveredJob({
      sourceUrl: 'https://example.com/jobs/1',
      canonicalUrl: 'javascript:alert(1)'
    });

    assert.equal(job.canonicalUrl, job.sourceUrl);
  });
});
