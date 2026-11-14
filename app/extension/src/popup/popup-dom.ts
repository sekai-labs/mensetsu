export interface PopupElement {
  value: string;
  checked: boolean;
  disabled: boolean;
  hidden: boolean;
  className: string;
  textContent: string | null;
  type: string;
  href: string;
  target: string;
  rel: string;
  classList: {
    add(...tokens: string[]): void;
    remove(...tokens: string[]): void;
    toggle(token: string, force?: boolean): void;
    contains(token: string): boolean;
  };
  appendChild(child: PopupElement): void;
  replaceChildren(...nodes: PopupElement[]): void;
  setAttribute(name: string, value: string): void;
  addEventListener(type: string, listener: (event: Event) => void): void;
}

export function appendText(parent: PopupElement, tagName: string, className: string, value: unknown): PopupElement {
  const element = document.createElement(tagName) as unknown as PopupElement;
  if (className) element.className = className;
  element.textContent = String(value ?? '');
  parent.appendChild(element);
  return element;
}
