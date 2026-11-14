import { describe, it, expect, afterEach } from 'vitest';
import { getWorkspaceUrl, getWorkspaceLinks } from '../src/data/workspace';

describe('Workspace URL Resolution (Astro)', () => {
  const originalEnv = process.env.PUBLIC_WORKSPACE_URL;

  afterEach(() => {
    process.env.PUBLIC_WORKSPACE_URL = originalEnv;
  });

  it('defaults to http://localhost:3001 when unset', () => {
    delete process.env.PUBLIC_WORKSPACE_URL;
    expect(getWorkspaceUrl()).toBe('http://localhost:3001');
  });

  it('strips trailing slashes from configured URL', () => {
    process.env.PUBLIC_WORKSPACE_URL = 'https://app.mensetsu.example///';
    expect(getWorkspaceUrl()).toBe('https://app.mensetsu.example');
  });

  it('builds login, register, and dashboard URLs correctly', () => {
    process.env.PUBLIC_WORKSPACE_URL = 'https://workspace.mensetsu.org';
    const links = getWorkspaceLinks();
    expect(links.loginUrl).toBe('https://workspace.mensetsu.org/login');
    expect(links.registerUrl).toBe('https://workspace.mensetsu.org/register');
    expect(links.dashboardUrl).toBe('https://workspace.mensetsu.org/dashboard');
    expect(links.newInterviewUrl).toBe('https://workspace.mensetsu.org/interviews/new');
  });

  it('resolves arbitrary paths safely', () => {
    process.env.PUBLIC_WORKSPACE_URL = 'https://workspace.mensetsu.org';
    const links = getWorkspaceLinks();
    expect(links.resolvePath('/settings')).toBe('https://workspace.mensetsu.org/settings');
    expect(links.resolvePath('cvs')).toBe('https://workspace.mensetsu.org/cvs');
  });
});
