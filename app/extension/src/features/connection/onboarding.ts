import { normalizeApiOrigin } from '../../shared/api-origin.js';
import { checkApiConnection, CONNECTION_ERROR } from './connection.js';
import { REASON } from '../../shared/constants.js';
import type {
  ConnectionResult,
  ConnectionStatus,
  ConnectApiOriginInput,
  ConnectApiOriginResult,
  Settings,
  ViewStateInput
} from '../../shared/types.js';

export const VIEW = Object.freeze({
  SETUP: 'setup',
  CONNECTION_ERROR: 'connection-error',
  LOGIN: 'login',
  WORKSPACE: 'workspace'
} as const);

export type ViewName = (typeof VIEW)[keyof typeof VIEW];

function errorConnection(code: string, message: string): ConnectionStatus {
  return { state: 'error', code, message };
}

export function deriveViewState({ apiOrigin, connection, authenticated }: ViewStateInput = {}): ViewName {
  if (!apiOrigin) return VIEW.SETUP;
  if (connection?.state === 'healthy') {
    return authenticated ? VIEW.WORKSPACE : VIEW.LOGIN;
  }
  return VIEW.CONNECTION_ERROR;
}

export interface EvaluateConnectionDeps {
  checkConnection?: (origin: string) => Promise<ConnectionResult>;
}

export function isLinkedInPageUrl(rawUrl: unknown): boolean {
  try {
    const host = new URL(String(rawUrl)).hostname.toLowerCase();
    return host === 'linkedin.com' || host.endsWith('.linkedin.com');
  } catch {
    return false;
  }
}

export function assertLinkedInDetectionAllowed(
  rawUrl: unknown,
  settings: Partial<Settings> | null | undefined
): void {
  if (settings?.linkedinOptIn === true) return;
  if (!isLinkedInPageUrl(rawUrl)) return;
  throw new Error(
    `LinkedIn page detection is disabled. Enable LinkedIn detection in Mensetsu settings before scanning LinkedIn. (${REASON.LINKEDIN_OPT_IN_REQUIRED})`
  );
}

export async function evaluateStoredConnection(
  settings: Partial<Settings> | null | undefined,
  deps: EvaluateConnectionDeps = {}
): Promise<ConnectionStatus> {
  const checkConnection = deps.checkConnection || checkApiConnection;
  if (!settings?.apiOrigin) return { state: 'missing' };

  let origin: string;
  try {
    origin = normalizeApiOrigin(settings.apiOrigin, {
      allowInsecureLoopback: settings.allowInsecureLoopback === true
    });
  } catch (error) {
    const failure = error as { code?: string; message?: string };
    return errorConnection(failure?.code || CONNECTION_ERROR.INVALID, String(failure?.message || 'The stored API origin is invalid.'));
  }

  let result: ConnectionResult;
  try {
    result = await checkConnection(origin);
  } catch (error) {
    return errorConnection(CONNECTION_ERROR.UNREACHABLE, String((error as Error)?.message || 'Connection check failed.'));
  }

  if (!result?.ok) {
    return errorConnection(
      result?.code || CONNECTION_ERROR.UNREACHABLE,
      result?.message || `Could not verify ${origin}.`
    );
  }
  return { state: 'healthy', origin };
}

export async function connectApiOrigin({
  rawOrigin,
  allowInsecureLoopback = false,
  currentSettings,
  store,
  checkConnection = checkApiConnection,
  permissionGranted
}: ConnectApiOriginInput = {}): Promise<ConnectApiOriginResult> {
  let origin: string;
  try {
    origin = normalizeApiOrigin(rawOrigin, { allowInsecureLoopback: allowInsecureLoopback === true });
  } catch (error) {
    const failure = error as { code?: string; message?: string };
    return {
      connected: false,
      origin: '',
      connection: errorConnection(
        failure?.code || CONNECTION_ERROR.INVALID,
        String(failure?.message || 'Enter a valid API origin.')
      )
    };
  }

  if (typeof permissionGranted === 'function') {
    let granted: boolean;
    try {
      granted = await permissionGranted(origin);
    } catch (error) {
      return {
        connected: false,
        origin,
        connection: errorConnection(
          'api_permission_missing',
          String((error as Error)?.message || `Permission to contact ${origin} was not granted.`)
        )
      };
    }
    if (granted !== true) {
      return {
        connected: false,
        origin,
        connection: errorConnection(
          'api_permission_missing',
          `Mensetsu needs permission to contact ${origin}. Allow access and try again.`
        )
      };
    }
  }

  let result: ConnectionResult;
  try {
    result = await checkConnection(origin);
  } catch (error) {
    return {
      connected: false,
      origin,
      connection: errorConnection(
        CONNECTION_ERROR.UNREACHABLE,
        String((error as Error)?.message || `Could not reach ${origin}.`)
      )
    };
  }

  if (!result?.ok) {
    return {
      connected: false,
      origin,
      connection: errorConnection(
        result?.code || CONNECTION_ERROR.UNREACHABLE,
        result?.message || `Could not verify ${origin}.`
      )
    };
  }

  let settings: Partial<Settings> = currentSettings || {};
  if (!currentSettings && store) settings = await store.getSettings();

  const originChanged = String(settings.apiOrigin || '') !== origin;

  if (store) {
    if (originChanged) {
      if (typeof store.clearTokens === 'function') await store.clearTokens();
      if (typeof store.clearTempToken === 'function') await store.clearTempToken();
    }
    await store.setSettings({
      ...settings,
      apiOrigin: origin,
      allowInsecureLoopback: allowInsecureLoopback === true
    });
  }

  return {
    connected: true,
    origin,
    connection: { state: 'healthy', origin },
    tokensCleared: originChanged
  };
}
