export interface DomFileLike {
  name?: string;
  size?: number;
  type?: string;
}

export interface DomStyleLike {
  display?: string;
  visibility?: string;
  opacity?: string;
}

export interface DomViewLike {
  getComputedStyle?(el: DomElement): DomStyleLike;
}

export interface DomRectLike {
  width?: number;
  height?: number;
  top?: number;
  left?: number;
}

export interface DomElement {
  tagName?: string;
  tag?: string;
  id?: string;
  name?: string;
  type?: string;
  nodeType?: number;
  textContent?: string | null;
  value?: string;
  checked?: boolean;
  disabled?: boolean;
  required?: boolean;
  hidden?: boolean;
  clicked?: boolean;
  files?: ArrayLike<DomFileLike>;
  children?: ArrayLike<DomElement | string>;
  childNodes?: ArrayLike<DomElement | string>;
  parentNode?: DomElement | null;
  ownerDocument?: DomElement | null;
  defaultView?: DomViewLike | null;
  getAttribute?(name: string): string | null | undefined;
  setAttribute?(name: string, value: string): void;
  removeAttribute?(name: string): void;
  querySelector?(selector: string): DomElement | null;
  querySelectorAll?(selector: string): ArrayLike<DomElement>;
  dispatchEvent?(event: unknown): boolean;
  addEventListener?(type: string, listener: (event: unknown) => void): void;
  getBoundingClientRect?(): DomRectLike;
}

export interface DomDocument extends DomElement {
  title?: string;
  body?: DomElement | null;
  documentElement?: DomElement | null;
  location?: { href?: string };
}

export type DomRoot = DomDocument | DomElement;
