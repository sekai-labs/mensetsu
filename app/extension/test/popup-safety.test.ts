import { describe, it } from 'node:test';
import assert from 'node:assert/strict';

import { appendText, type PopupElement } from '../src/popup/popup-dom.js';

describe('Popup rendering safety', () => {
  it('renders untrusted text literally without creating HTML content', () => {
    let content = '';
    let html = '';
    const child = {
      className: '',
      get textContent() {
        return content;
      },
      set textContent(value: string) {
        content = value;
      },
      set innerHTML(value: string) {
        html = value;
      }
    } as unknown as PopupElement;
    const children: PopupElement[] = [];
    const parent = {
      appendChild(node: PopupElement) {
        children.push(node);
      }
    } as unknown as PopupElement;
    const previousDocument = Object.getOwnPropertyDescriptor(globalThis, 'document');
    Object.defineProperty(globalThis, 'document', {
      configurable: true,
      value: { createElement: () => child }
    });
    try {
      const unsafeText = '<img src=x onerror=alert(1)>';
      const rendered = appendText(parent, 'span', 'job-title', unsafeText);
      assert.equal(rendered.textContent, unsafeText);
      assert.equal(rendered.className, 'job-title');
      assert.deepEqual(children, [child]);
      assert.equal(html, '');
    } finally {
      if (previousDocument) Object.defineProperty(globalThis, 'document', previousDocument);
      else Reflect.deleteProperty(globalThis, 'document');
    }
  });
});
