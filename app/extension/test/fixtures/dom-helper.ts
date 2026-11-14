export interface MockEvent {
  type: string;
  bubbles?: boolean;
  target?: unknown;
}

export type MockListener = (event: MockEvent) => void;

export class MockElement {
  tagName: string;
  attributes: Record<string, string>;
  children: Array<MockElement | string>;
  parentNode: MockElement | null;
  ownerDocument: MockElement | null;
  files: Array<{ name: string; size: number; type: string }>;
  clicked: boolean;
  listeners: Map<string, MockListener[]>;
  private _value: string;
  private _checked: boolean;
  private _text: string;

  constructor(tagName: string, attrs: Record<string, string> = {}, ownerDocument: MockElement | null = null) {
    this.tagName = tagName.toUpperCase();
    this.attributes = { ...attrs };
    this.children = [];
    this.parentNode = null;
    this.ownerDocument = ownerDocument;
    this._value = attrs.value || '';
    this._checked = 'checked' in attrs;
    this.files = [];
    this.clicked = false;
    this.listeners = new Map();
    this._text = '';
  }

  get id(): string {
    return this.getAttribute('id') || '';
  }

  set id(value: string) {
    this.setAttribute('id', value);
  }

  get name(): string {
    return this.getAttribute('name') || '';
  }

  get type(): string {
    return (this.getAttribute('type') || (this.tagName === 'TEXTAREA' ? 'textarea' : this.tagName === 'SELECT' ? 'select' : 'text')).toLowerCase();
  }

  get disabled(): boolean {
    return this.getAttribute('disabled') !== null && this.getAttribute('disabled') !== undefined;
  }

  set disabled(value: boolean) {
    if (value) this.setAttribute('disabled', 'disabled');
    else this.removeAttribute('disabled');
  }

  get required(): boolean {
    return this.getAttribute('required') !== null || this.getAttribute('aria-required') === 'true';
  }

  get hidden(): boolean {
    return this.getAttribute('hidden') !== null || this.getAttribute('aria-hidden') === 'true';
  }

  get value(): string {
    return this._value;
  }

  set value(value: string) {
    this._value = String(value ?? '');
  }

  get checked(): boolean {
    return this._checked;
  }

  set checked(value: boolean) {
    this._checked = Boolean(value);
  }

  get textContent(): string {
    if (this.children.length === 0) return this._text || '';
    return this.children.map((child) => (typeof child === 'string' ? child : child.textContent)).join(' ');
  }

  set textContent(value: string) {
    this.children = [];
    this._text = String(value ?? '');
  }

  getAttribute(name: string): string | null {
    const lower = name.toLowerCase();
    return this.attributes[lower] !== undefined ? this.attributes[lower] : null;
  }

  setAttribute(name: string, value: string): void {
    this.attributes[name.toLowerCase()] = String(value);
  }

  removeAttribute(name: string): void {
    delete this.attributes[name.toLowerCase()];
  }

  appendChild(child: MockElement | string): void {
    if (typeof child === 'string') {
      this.children.push(child);
      return;
    }
    if (child) {
      child.parentNode = this;
      child.ownerDocument = this.ownerDocument;
      this.children.push(child);
    }
  }

  addEventListener(type: string, listener: MockListener): void {
    if (!this.listeners.has(type)) this.listeners.set(type, []);
    this.listeners.get(type)?.push(listener);
  }

  dispatchEvent(event: MockEvent): boolean {
    const list = this.listeners.get(event.type) || [];
    for (const listener of list) listener(event);
    return true;
  }

  click(): void {
    this.clicked = true;
    this.dispatchEvent({ type: 'click', bubbles: true, target: this });
  }

  matchesSelector(selector: string): boolean {
    const sel = selector.trim();
    if (sel.includes(',')) {
      return sel.split(',').some((part) => this.matchesSelector(part.trim()));
    }

    const parts = sel.match(/^([a-zA-Z0-9_-]+)?(#([a-zA-Z0-9_-]+))?(\.([a-zA-Z0-9_.-]+))?(\[([a-zA-Z0-9_-]+)([*^$]?=)?("([^"]*)"|'([^']*)'|([^\]]*))?\])?$/);
    if (parts) {
      const tag = parts[1];
      const id = parts[3];
      const cls = parts[5];
      const attrName = parts[7];
      const attrOp = parts[8];
      const attrVal1 = parts[10];
      const attrVal2 = parts[11];
      const attrVal3 = parts[12];
      if (tag && tag.toUpperCase() !== this.tagName) return false;
      if (id && this.getAttribute('id') !== id) return false;
      if (cls) {
        const classes = cls.split('.');
        const elClasses = (this.getAttribute('class') || '').split(/\s+/);
        if (!classes.every((name) => elClasses.includes(name))) return false;
      }
      if (attrName) {
        const attrVal = attrVal1 ?? attrVal2 ?? attrVal3 ?? '';
        const currentAttr = this.getAttribute(attrName);
        if (currentAttr === null) return false;
        if (attrOp === '=' && currentAttr !== attrVal) return false;
        if (attrOp === '*=' && !currentAttr.includes(attrVal)) return false;
        if (attrOp === '^=' && !currentAttr.startsWith(attrVal)) return false;
        if (attrOp === '$=' && !currentAttr.endsWith(attrVal)) return false;
      }
      return true;
    }

    if (sel.startsWith('.')) {
      const clsName = sel.slice(1);
      const elClasses = (this.getAttribute('class') || '').split(/\s+/);
      return elClasses.includes(clsName);
    }
    if (sel.startsWith('#')) {
      return this.getAttribute('id') === sel.slice(1);
    }
    if (/^[a-zA-Z0-9_-]+$/.test(sel)) {
      return this.tagName === sel.toUpperCase();
    }
    if (sel.startsWith('[') && sel.endsWith(']')) {
      const inside = sel.slice(1, -1);
      const [key, value] = inside.split('=');
      const expected = value ? value.replace(/["']/g, '') : null;
      const current = this.getAttribute(key.trim());
      return expected !== null ? current === expected : current !== null;
    }
    return false;
  }

  querySelectorAll(selector: string): MockElement[] {
    const results: MockElement[] = [];
    const sel = selector.trim();

    if (sel.includes(',')) {
      const subSelectors = sel.split(',').map((part) => part.trim());
      const seen = new Set<MockElement>();
      for (const sub of subSelectors) {
        for (const el of this.querySelectorAll(sub)) {
          if (!seen.has(el)) {
            seen.add(el);
            results.push(el);
          }
        }
      }
      return results;
    }

    const spaceIndex = sel.indexOf(' ');
    if (spaceIndex > 0) {
      const ancestor = sel.slice(0, spaceIndex).trim();
      const descendant = sel.slice(spaceIndex + 1).trim();
      const seen = new Set<MockElement>();
      for (const parent of this.querySelectorAll(ancestor)) {
        for (const el of parent.querySelectorAll(descendant)) {
          if (!seen.has(el)) {
            seen.add(el);
            results.push(el);
          }
        }
      }
      return results;
    }

    const traverse = (node: MockElement): void => {
      for (const child of node.children) {
        if (typeof child !== 'string') {
          if (child.matchesSelector(sel)) results.push(child);
          traverse(child);
        }
      }
    };
    traverse(this);
    return results;
  }

  querySelector(selector: string): MockElement | null {
    const list = this.querySelectorAll(selector);
    return list.length > 0 ? list[0] : null;
  }
}

export class MockDocument extends MockElement {
  title: string;
  body: MockElement;
  defaultView: { getComputedStyle: () => { display: string; visibility: string; opacity: string } };

  constructor(title = '') {
    super('DOCUMENT');
    this.title = title;
    this.ownerDocument = this;
    this.body = new MockElement('BODY', {}, this);
    this.appendChild(this.body);
    this.defaultView = {
      getComputedStyle: () => ({
        display: 'block',
        visibility: 'visible',
        opacity: '1'
      })
    };
  }
}

export function parseHtml(htmlString: string, title = ''): MockDocument {
  const doc = new MockDocument(title);
  let currentParent: MockElement = doc.body;

  const tagRegex = /<!--[\s\S]*?-->|<(\/)?([a-zA-Z0-9_-]+)((?:\s+[a-zA-Z0-9_:-]+(?:\s*=\s*(?:"[^"]*"|'[^']*'|[^>\s]+))?)*)\s*(\/)?>|([^<]+)/g;
  const selfClosing = new Set(['input', 'img', 'br', 'hr', 'meta', 'link']);

  let match: RegExpExecArray | null;
  while ((match = tagRegex.exec(htmlString)) !== null) {
    const [full, isClosing, tagName, rawAttrs, isSelfClose, text] = match;

    if (full.startsWith('<!--')) continue;

    if (text) {
      const clean = text.replace(/\s+/g, ' ');
      if (clean.trim()) currentParent.appendChild(clean);
      continue;
    }

    if (isClosing) {
      if (currentParent.parentNode) currentParent = currentParent.parentNode;
      continue;
    }

    const attrs: Record<string, string> = {};
    if (rawAttrs) {
      const attrRegex = /([a-zA-Z0-9_:-]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^>\s]+)))?/g;
      let attrMatch: RegExpExecArray | null;
      while ((attrMatch = attrRegex.exec(rawAttrs)) !== null) {
        attrs[attrMatch[1].toLowerCase()] = attrMatch[2] ?? attrMatch[3] ?? attrMatch[4] ?? '';
      }
    }

    const element = new MockElement(tagName, attrs, doc);
    currentParent.appendChild(element);

    if (!isSelfClose && !selfClosing.has(tagName.toLowerCase())) {
      currentParent = element;
    }
  }

  return doc;
}
