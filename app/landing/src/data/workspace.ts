export function getWorkspaceUrl(): string {
  const envUrl = (typeof import.meta !== 'undefined' && import.meta.env?.PUBLIC_WORKSPACE_URL)
    || (typeof process !== 'undefined' && process.env?.PUBLIC_WORKSPACE_URL)
    || 'http://localhost:3001';
  return envUrl.replace(/\/+$/, '');
}

export function getWorkspaceLinks() {
  const base = getWorkspaceUrl();
  return {
    baseUrl: base,
    loginUrl: `${base}/login`,
    registerUrl: `${base}/register`,
    dashboardUrl: `${base}/dashboard`,
    newInterviewUrl: `${base}/interviews/new`,
    resolvePath: (path: string) => `${base}${path.startsWith('/') ? path : `/${path}`}`,
  };
}
