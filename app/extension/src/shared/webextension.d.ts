export interface ExtensionLastError {
  message?: string;
}

export interface ExtensionMessageSender {
  id?: string;
  url?: string;
  tab?: ExtensionTab;
  frameId?: number;
}

export interface ExtensionMessageListener {
  (message: unknown, sender: ExtensionMessageSender, sendResponse: (response?: unknown) => void): boolean | void | Promise<unknown>;
}

export interface ExtensionRuntime {
  id?: string;
  lastError?: ExtensionLastError;
  sendMessage(message: unknown): Promise<unknown>;
  getURL(path: string): string;
  openOptionsPage?(): Promise<void>;
  onMessage: {
    addListener(listener: ExtensionMessageListener): void;
  };
  onInstalled?: {
    addListener(listener: () => void): void;
  };
  onStartup?: {
    addListener(listener: () => void): void;
  };
}

export interface ExtensionTab {
  id?: number;
  url?: string;
  title?: string;
}

export interface ExtensionQueryInfo {
  active?: boolean;
  currentWindow?: boolean;
}

export interface ExtensionTabs {
  query(info: ExtensionQueryInfo): Promise<ExtensionTab[]>;
  sendMessage(tabId: number, message: unknown): Promise<unknown>;
}

export interface ExtensionScriptTarget {
  tabId: number;
  allFrames?: boolean;
  frameIds?: number[];
}

export interface ExtensionScriptDetails {
  target: ExtensionScriptTarget;
  files?: string[];
  func?: (...args: never[]) => unknown;
  args?: unknown[];
  world?: string;
  injectImmediately?: boolean;
}

export interface ExtensionScripting {
  executeScript(details: ExtensionScriptDetails): Promise<unknown[]>;
}

export interface ExtensionPermissionRequest {
  origins?: string[];
  permissions?: string[];
}

export interface ExtensionPermissions {
  request(permissions: ExtensionPermissionRequest): Promise<boolean>;
  contains(permissions: ExtensionPermissionRequest): Promise<boolean>;
  remove?(permissions: ExtensionPermissionRequest): Promise<boolean>;
}

export interface ExtensionStorageChange {
  oldValue?: unknown;
  newValue?: unknown;
}

export interface ExtensionStorageArea {
  get(keys?: string | string[] | Record<string, unknown> | null): Promise<Record<string, unknown>>;
  set(items: Record<string, unknown>): Promise<void>;
  remove(keys: string | string[]): Promise<void>;
  clear(): Promise<void>;
  setAccessLevel?(details: { accessLevel: string }): Promise<void>;
}

export interface ExtensionStorage {
  local: ExtensionStorageArea;
  session?: ExtensionStorageArea;
  sync?: ExtensionStorageArea;
  onChanged?: {
    addListener(listener: (changes: Record<string, ExtensionStorageChange>, areaName: string) => void): void;
  };
}

export interface ExtensionApi {
  runtime: ExtensionRuntime;
  tabs?: ExtensionTabs;
  scripting?: ExtensionScripting;
  permissions?: ExtensionPermissions;
  storage: ExtensionStorage;
}

declare global {
  var chrome: ExtensionApi | undefined;
  var browser: ExtensionApi | undefined;
}
