export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE';

export interface ApiRoute {
  readonly method: HttpMethod;
  readonly path: string;
}

export interface ApiRoutes {
  readonly login: ApiRoute;
  readonly login2fa: ApiRoute;
  readonly refresh: ApiRoute;
  readonly logout: ApiRoute;
  readonly me: ApiRoute;
  readonly candidateProfile: ApiRoute;
  readonly profile: ApiRoute;
  readonly resumes: ApiRoute;
  readonly resumeDownload: (uploadId: string) => ApiRoute;
  readonly listSavedJobs: (cursor?: string, limit?: number) => ApiRoute;
  readonly saveJob: ApiRoute;
  readonly approveSavedJob: (savedJobId: string) => ApiRoute;
  readonly dismissSavedJob: (savedJobId: string) => ApiRoute;
  readonly prepareSavedJob: (savedJobId: string) => ApiRoute;
  readonly prepareApplication: ApiRoute;
  readonly reprepareApplication: (applicationId: string) => ApiRoute;
  readonly listApplications: ApiRoute;
  readonly getApplication: (applicationId: string) => ApiRoute;
  readonly approveApplication: (applicationId: string) => ApiRoute;
  readonly startApplication: (applicationId: string) => ApiRoute;
  readonly blockApplication: (applicationId: string) => ApiRoute;
  readonly resultApplication: (applicationId: string) => ApiRoute;
  readonly reconcileApplication: (applicationId: string) => ApiRoute;
}

export const API_ROUTES = Object.freeze({
  login: { method: 'POST', path: '/api/v1/auth/login' },
  login2fa: { method: 'POST', path: '/api/v1/auth/login-2fa' },
  refresh: { method: 'POST', path: '/api/v1/auth/refresh' },
  logout: { method: 'POST', path: '/api/v1/auth/logout' },
  me: { method: 'GET', path: '/api/v1/auth/me' },
  candidateProfile: { method: 'GET', path: '/api/v1/candidate-profile' },
  profile: { method: 'GET', path: '/api/v1/settings/profile' },
  resumes: { method: 'GET', path: '/api/v1/resume-uploads' },
  resumeDownload: (uploadId: string): ApiRoute => ({
    method: 'GET',
    path: `/api/v1/resume-uploads/${encodeURIComponent(uploadId)}/download`
  }),
  listSavedJobs: (cursor?: string, limit?: number): ApiRoute => {
    const params = new URLSearchParams();
    if (cursor) params.set('cursor', cursor);
    if (limit !== undefined) params.set('limit', String(limit));
    const query = params.toString();
    return { method: 'GET', path: `/api/v1/saved-jobs${query ? `?${query}` : ''}` };
  },
  saveJob: { method: 'POST', path: '/api/v1/saved-jobs' },
  approveSavedJob: (savedJobId: string): ApiRoute => ({
    method: 'POST',
    path: `/api/v1/saved-jobs/${encodeURIComponent(savedJobId)}/approve`
  }),
  dismissSavedJob: (savedJobId: string): ApiRoute => ({
    method: 'POST',
    path: `/api/v1/saved-jobs/${encodeURIComponent(savedJobId)}/dismiss`
  }),
  prepareSavedJob: (savedJobId: string): ApiRoute => ({
    method: 'POST',
    path: `/api/v1/saved-jobs/${encodeURIComponent(savedJobId)}/prepare`
  }),
  prepareApplication: { method: 'POST', path: '/api/v1/applications' },
  reprepareApplication: (applicationId: string): ApiRoute => ({
    method: 'PUT',
    path: `/api/v1/applications/${encodeURIComponent(applicationId)}/prepare`
  }),
  listApplications: { method: 'GET', path: '/api/v1/applications' },
  getApplication: (applicationId: string): ApiRoute => ({
    method: 'GET',
    path: `/api/v1/applications/${encodeURIComponent(applicationId)}`
  }),
  approveApplication: (applicationId: string): ApiRoute => ({
    method: 'POST',
    path: `/api/v1/applications/${encodeURIComponent(applicationId)}/approve`
  }),
  startApplication: (applicationId: string): ApiRoute => ({
    method: 'POST',
    path: `/api/v1/applications/${encodeURIComponent(applicationId)}/start`
  }),
  blockApplication: (applicationId: string): ApiRoute => ({
    method: 'POST',
    path: `/api/v1/applications/${encodeURIComponent(applicationId)}/block`
  }),
  resultApplication: (applicationId: string): ApiRoute => ({
    method: 'POST',
    path: `/api/v1/applications/${encodeURIComponent(applicationId)}/result`
  }),
  reconcileApplication: (applicationId: string): ApiRoute => ({
    method: 'POST',
    path: `/api/v1/applications/${encodeURIComponent(applicationId)}/reconcile`
  })
} as const satisfies ApiRoutes);
