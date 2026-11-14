import { API_ROUTES } from '../shared/api-routes.js';
import type { ApiRoute } from '../shared/api-routes.js';
import { isSameOriginUrl, normalizeApiOrigin, resolveApiUrl } from '../shared/api-origin.js';
import { MAX_RESUME_BYTES } from '../shared/constants.js';
import { isPlainObject, safeJsonParse } from '../shared/util.js';
import type {
  ApiClient,
  AuthStatus,
  CandidateProfileDto,
  ExtensionStore,
  FetchLike,
  LoginResult,
  PrepareApplicationBody,
  ResumeDownload,
  ResumeOption,
  SavedJobsPage,
  StoredTokens
} from '../shared/types.js';
import type {
  ApproveApplicationRequestDto,
  BlockApplicationRequestDto,
  CreateSavedJobRequestDto,
  JobApplicationResponseDto,
  Login2faRequest,
  LoginRequest,
  LoginResponse,
  PrepareSavedJobRequestDto,
  ReconcileApplicationRequestDto,
  ResultApplicationRequestDto,
  ResumeUploadItemDto,
  SavedJobResponseDto,
  UserResponseDto
} from '../shared/contracts.js';

export class ApiError extends Error {
  readonly code: string;
  readonly status: number | undefined;
  readonly body: unknown;

  constructor(code: string, message: string, details: { status?: number; body?: unknown } = {}) {
    super(message);
    this.name = 'ApiError';
    this.code = code;
    this.status = details.status;
    this.body = details.body;
  }
}

interface RawRequestOptions {
  method?: string;
  token?: string;
  body?: unknown;
  timeoutMs?: number;
  responseType?: 'json' | 'bytes';
}

export interface CreateApiOptions {
  store?: ExtensionStore | null;
  fetchImpl?: FetchLike;
  timeoutMs?: number;
}

function jsonHeaders(token: string | undefined): Record<string, string> {
  const headers: Record<string, string> = { Accept: 'application/json' };
  if (token) headers.Authorization = `Bearer ${token}`;
  return headers;
}

function validTokenResponse(value: unknown): value is StoredTokens {
  return isPlainObject(value)
    && typeof value.access_token === 'string'
    && value.access_token.length > 0
    && typeof value.refresh_token === 'string'
    && value.refresh_token.length > 0;
}

function isUserResponse(value: unknown): value is UserResponseDto {
  return isPlainObject(value)
    && typeof value.id === 'string'
    && typeof value.email === 'string'
    && typeof value.name === 'string'
    && (value.role === 'admin' || value.role === 'member')
    && (value.status === 'active' || value.status === 'suspended')
    && typeof value.totp_enabled === 'boolean'
    && typeof value.created_at === 'string'
    && typeof value.updated_at === 'string';
}

function normalizeErrorBody(text: string): unknown {
  return safeJsonParse(text, null);
}

function itemsOf(result: unknown): unknown[] {
  if (Array.isArray(result)) return result;
  if (isPlainObject(result) && Array.isArray(result.items)) return result.items;
  return [];
}

export function createApi({
  store,
  fetchImpl = globalThis.fetch as unknown as FetchLike,
  timeoutMs = 15_000
}: CreateApiOptions = {}): ApiClient {
  if (!store) throw new Error('createApi requires a store');

  const activeStore = store;
  let refreshInFlight: Promise<StoredTokens> | null = null;

  async function origin(): Promise<string> {
    const settings = await activeStore.getSettings();
    return normalizeApiOrigin(settings.apiOrigin, {
      allowInsecureLoopback: settings.allowInsecureLoopback === true
    });
  }

  async function rawRequest(path: string, options: RawRequestOptions = {}): Promise<unknown> {
    const base = await origin();
    const url = resolveApiUrl(base, path);
    const method = options.method || 'GET';
    const headers = jsonHeaders(options.token);
    let body: string | undefined;
    if (options.body !== undefined) {
      headers['Content-Type'] = 'application/json';
      body = JSON.stringify(options.body);
    }

    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), options.timeoutMs || timeoutMs);
    let response;
    try {
      response = await fetchImpl(url.toString(), {
        method,
        headers,
        body,
        credentials: 'omit',
        redirect: 'error',
        cache: 'no-store',
        referrerPolicy: 'no-referrer',
        signal: controller.signal
      });
    } catch (error) {
      throw new ApiError('network_error', `Request to ${base} failed.`, { body: String((error as Error)?.message || error) });
    } finally {
      clearTimeout(timer);
    }

    if (response.url && !isSameOriginUrl(response.url, base)) {
      throw new ApiError('api_origin_invalid', 'API response left the configured origin.');
    }

    if (options.responseType === 'bytes' && response.ok) {
      const contentType = response.headers?.get?.('content-type') || '';
      if (!contentType.toLowerCase().startsWith('application/pdf')) {
        throw new ApiError('resume_invalid_type', 'The resume download was not a PDF.');
      }
      const length = Number(response.headers?.get?.('content-length') || 0);
      if (length > MAX_RESUME_BYTES) {
        throw new ApiError('resume_too_large', 'The resume is larger than the 15 MB limit.');
      }
      const buffer = await response.arrayBuffer!();
      if (buffer.byteLength > MAX_RESUME_BYTES) {
        throw new ApiError('resume_too_large', 'The resume is larger than the 15 MB limit.');
      }
      const disposition = response.headers?.get?.('content-disposition') || '';
      const filenameMatch = /filename="?([^";]+)"?/i.exec(disposition);
      const filename = filenameMatch ? filenameMatch[1].replace(/[\\/]/g, '_').slice(0, 180) : 'resume.pdf';
      return { bytes: new Uint8Array(buffer), filename, contentType };
    }

    const text = await response.text!();
    const parsed = normalizeErrorBody(text);
    if (!response.ok) {
      throw new ApiError(
        response.status === 401 ? 'unauthorized' : 'api_error',
        `API request failed with status ${response.status}.`,
        { status: response.status, body: parsed ?? text }
      );
    }
    return parsed;
  }

  async function withAuth<T = unknown>(pathOptions: ApiRoute, options: RawRequestOptions = {}): Promise<T> {
    const tokens = await activeStore.getTokens();
    if (!tokens?.access_token) throw new ApiError('unauthorized', 'Sign in to Mensetsu first.');
    try {
      return await rawRequest(pathOptions.path, {
        ...options,
        method: pathOptions.method,
        token: tokens.access_token
      }) as T;
    } catch (error) {
      if (!(error instanceof ApiError) || error.status !== 401) throw error;
      const refreshed = await refreshTokens();
      return rawRequest(pathOptions.path, {
        ...options,
        method: pathOptions.method,
        token: refreshed.access_token
      }) as Promise<T>;
    }
  }

  async function refreshTokens(): Promise<StoredTokens> {
    if (refreshInFlight) return refreshInFlight;
    refreshInFlight = (async () => {
      const tokens = await activeStore.getTokens();
      if (!tokens?.refresh_token) throw new ApiError('unauthorized', 'Refresh token is missing.');
      const result = await rawRequest(API_ROUTES.refresh.path, {
        method: API_ROUTES.refresh.method,
        body: { refresh_token: tokens.refresh_token }
      });
      if (!validTokenResponse(result)) {
        await activeStore.clearTokens();
        throw new ApiError('unauthorized', 'Token refresh returned an invalid response.');
      }
      await activeStore.setTokens(result);
      return result;
    })();
    try {
      return await refreshInFlight;
    } finally {
      refreshInFlight = null;
    }
  }

  return {
    async login(email: string, password: string): Promise<LoginResult> {
      const body: LoginRequest = { email, password };
      const result = await rawRequest(API_ROUTES.login.path, {
        method: API_ROUTES.login.method,
        body
      });
      const login = result as LoginResponse;
      if (login.requires_2fa) {
        if (login.temp_token) await activeStore.setTempToken(login.temp_token);
        await activeStore.clearTokens();
        return { requires2fa: true };
      }
      const accessToken = login.access_token;
      const refreshToken = login.refresh_token;
      if (!accessToken || !refreshToken) {
        throw new ApiError('api_error', 'Login response did not include tokens.');
      }
      await activeStore.setTokens({ access_token: accessToken, refresh_token: refreshToken });
      await activeStore.clearTempToken();
      return { requires2fa: false, user: (login.user as UserResponseDto | null | undefined) ?? null };
    },

    async login2fa(code: string): Promise<LoginResult> {
      const tempToken = await activeStore.getTempToken();
      if (!tempToken) throw new ApiError('unauthorized', 'The 2FA challenge expired. Sign in again.');
      const body: Login2faRequest = { code, temp_token: tempToken };
      const result = await rawRequest(API_ROUTES.login2fa.path, {
        method: API_ROUTES.login2fa.method,
        body
      });
      if (!validTokenResponse(result)) {
        throw new ApiError('api_error', '2FA response did not include tokens.');
      }
      await activeStore.setTokens({ access_token: result.access_token, refresh_token: result.refresh_token });
      await activeStore.clearTempToken();
      return { requires2fa: false, user: isUserResponse(result.user) ? result.user : null };
    },

    async logout(): Promise<void> {
      const tokens = await activeStore.getTokens();
      try {
        if (tokens?.access_token) {
          await rawRequest(API_ROUTES.logout.path, {
            method: API_ROUTES.logout.method,
            token: tokens.access_token,
            body: {}
          });
        }
      } catch {
        return;
      } finally {
        await activeStore.clearTokens();
        await activeStore.clearTempToken();
      }
    },

    async authStatus(): Promise<AuthStatus> {
      const tokens = await activeStore.getTokens();
      if (!tokens?.access_token) return { authenticated: false };
      try {
        const user = await withAuth<unknown>(API_ROUTES.me);
        if (!isUserResponse(user)) throw new ApiError('api_error', 'Authentication response did not contain a valid user.');
        return { authenticated: true, user };
      } catch (error) {
        if (error instanceof ApiError && error.status === 401) return { authenticated: false };
        throw error;
      }
    },

    async listResumes(): Promise<ResumeOption[]> {
      const result = await withAuth<unknown>(API_ROUTES.resumes);
      return itemsOf(result)
        .filter(isPlainObject)
        .map((item) => item as unknown as ResumeUploadItemDto)
        .map((item) => ({
          uploadId: String(item.upload_id ?? ''),
          filename: String(item.filename || 'resume.pdf'),
          createdAt: String(item.created_at || '')
        }))
        .filter((item) => item.uploadId);
    },

    async downloadResume(uploadId: string): Promise<ResumeDownload> {
      if (!uploadId) throw new ApiError('resume_missing', 'Choose a completed resume from the library.');
      return withAuth<ResumeDownload>(API_ROUTES.resumeDownload(uploadId), { responseType: 'bytes' });
    },

    async candidateProfile(): Promise<CandidateProfileDto | null> {
      const result = await withAuth<unknown>(API_ROUTES.candidateProfile);
      return isPlainObject(result) ? result as unknown as CandidateProfileDto : null;
    },

    async listSavedJobs(cursor?: string, limit?: number): Promise<SavedJobsPage> {
      const result = await withAuth<unknown>(API_ROUTES.listSavedJobs(cursor, limit));
      const nextCursor = isPlainObject(result) && typeof result.next_cursor === 'string' && result.next_cursor.length > 0
        ? result.next_cursor
        : null;
      return {
        items: itemsOf(result).filter(isPlainObject) as SavedJobResponseDto[],
        next_cursor: nextCursor
      };
    },

    async saveJob(payload: CreateSavedJobRequestDto): Promise<SavedJobResponseDto> {
      const body: CreateSavedJobRequestDto = {
        source_url: payload.source_url,
        role_title: payload.role_title ?? null,
        company_name: payload.company_name ?? null,
        job_description: payload.job_description ?? null,
        location: payload.location ?? null,
        application_url: payload.application_url ?? null,
        contact_email: payload.contact_email ?? null
      };
      return withAuth<SavedJobResponseDto>(API_ROUTES.saveJob, { body });
    },

    async approveSavedJob(savedJobId: string): Promise<SavedJobResponseDto> {
      return withAuth<SavedJobResponseDto>(API_ROUTES.approveSavedJob(savedJobId));
    },

    async dismissSavedJob(savedJobId: string): Promise<SavedJobResponseDto> {
      return withAuth<SavedJobResponseDto>(API_ROUTES.dismissSavedJob(savedJobId));
    },

    async prepareSavedJob(savedJobId: string, payload: PrepareSavedJobRequestDto): Promise<JobApplicationResponseDto> {
      const body: PrepareSavedJobRequestDto = {
        resume_upload_id: payload.resume_upload_id,
        application_method: payload.application_method,
        contact_email: payload.contact_email ?? null
      };
      return withAuth<JobApplicationResponseDto>(API_ROUTES.prepareSavedJob(savedJobId), { body });
    },

    async prepareApplication(payload: PrepareApplicationBody): Promise<JobApplicationResponseDto> {
      const body = {
        site: payload.site,
        source_url: payload.source_url,
        company_name: payload.company_name,
        role_title: payload.role_title,
        job_description: payload.job_description,
        application_method: payload.application_method ?? 'web',
        contact_email: payload.contact_email ?? null,
        resume_upload_id: payload.resume_upload_id
      };
      return withAuth<JobApplicationResponseDto>(API_ROUTES.prepareApplication, { body });
    },

    async approveApplication(applicationId: string, payload: ApproveApplicationRequestDto): Promise<JobApplicationResponseDto> {
      const body: ApproveApplicationRequestDto = { revision: payload.revision };
      return withAuth<JobApplicationResponseDto>(API_ROUTES.approveApplication(applicationId), { body });
    },

    async startApplication(applicationId: string): Promise<JobApplicationResponseDto> {
      return withAuth<JobApplicationResponseDto>(API_ROUTES.startApplication(applicationId));
    },

    async blockApplication(applicationId: string, payload: BlockApplicationRequestDto): Promise<JobApplicationResponseDto> {
      const body: BlockApplicationRequestDto = { reason_code: payload.reason_code };
      return withAuth<JobApplicationResponseDto>(API_ROUTES.blockApplication(applicationId), { body });
    },

    async resultApplication(applicationId: string, payload: ResultApplicationRequestDto): Promise<JobApplicationResponseDto> {
      const body: ResultApplicationRequestDto = {
        outcome: payload.outcome,
        confirmation_url: payload.confirmation_url ?? null
      };
      return withAuth<JobApplicationResponseDto>(API_ROUTES.resultApplication(applicationId), { body });
    },

    async reconcileApplication(applicationId: string, payload: ReconcileApplicationRequestDto): Promise<JobApplicationResponseDto> {
      const body: ReconcileApplicationRequestDto = { submitted: Boolean(payload.submitted) };
      return withAuth<JobApplicationResponseDto>(API_ROUTES.reconcileApplication(applicationId), { body });
    },

    async getApplication(applicationId: string): Promise<JobApplicationResponseDto> {
      return withAuth<JobApplicationResponseDto>(API_ROUTES.getApplication(applicationId));
    },

    async listApplications(): Promise<JobApplicationResponseDto[]> {
      const result = await withAuth<unknown>(API_ROUTES.listApplications);
      return itemsOf(result).filter(isPlainObject) as JobApplicationResponseDto[];
    },

    refreshTokens
  };
}
