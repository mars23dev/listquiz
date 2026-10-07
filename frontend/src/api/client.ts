import type {
  ApiErrorBody,
  ApiErrorCode,
  CreateSessionResponse,
  ImportReport,
  ListDetail,
  ListSummary,
  SessionState,
} from '../types/session';
import type { UserRole } from '../types/quiz';

/**
 * The one place the frontend talks to the backend. Every call goes through
 * `request`, so error handling and the `X-Quiz-Password` header are consistent
 * (contracts/rest-api.md).
 */
export class ApiError extends Error {
  readonly code: ApiErrorCode;
  readonly status: number;

  constructor(code: ApiErrorBody['error']['code'], message: string, status: number) {
    super(message);
    this.name = 'ApiError';
    this.code = code;
    this.status = status;
  }

  /** True for the draft's own copy, used to show "Incorrect password!". */
  get isWrongPassword(): boolean {
    return this.code === 'unauthorized';
  }
}

const PASSWORD_HEADER = 'X-Quiz-Password';

interface RequestOptions {
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE';
  body?: unknown;
  password?: string;
}

async function send(path: string, options: RequestOptions): Promise<Response> {
  const { method = 'GET', body, password } = options;
  const headers: Record<string, string> = {};
  if (body !== undefined) headers['Content-Type'] = 'application/json';
  if (password) headers[PASSWORD_HEADER] = password;

  return fetch(path, {
    method,
    headers,
    body: body === undefined ? undefined : JSON.stringify(body),
  });
}

async function toApiError(response: Response): Promise<ApiError> {
  let code: ApiErrorCode = 'bad_request';
  let message = `Request failed with status ${response.status}.`;
  try {
    const parsed = (await response.json()) as ApiErrorBody;
    code = parsed.error.code;
    message = parsed.error.message;
  } catch {
    // A non-JSON error body keeps the generic message above.
  }
  return new ApiError(code, message, response.status);
}

async function request<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const response = await send(path, options);
  if (!response.ok) throw await toApiError(response);
  if (response.status === 204) return undefined as T;
  return (await response.json()) as T;
}

/** `export.csv` answers with CSV rather than JSON (contracts/rest-api.md). */
async function requestText(path: string, options: RequestOptions = {}): Promise<string> {
  const response = await send(path, options);
  if (!response.ok) throw await toApiError(response);
  return response.text();
}

// --- Sessions -------------------------------------------------------------

export interface CreateSessionRequest {
  gmPassword: string;
  teamCount?: number;
  teams?: { name?: string; primaryHex?: string }[];
  roundCount?: number;
  rounds?: {
    roundNumber: number;
    quizType: 'ORDER' | 'TRIPMINE';
    listId: string;
    columnId: string;
    tripmineSide?: 'highest' | 'lowest';
  }[];
}

export function createSession(body: CreateSessionRequest): Promise<CreateSessionResponse> {
  return request<CreateSessionResponse>('/api/sessions', { method: 'POST', body });
}

/** Start a one-team quick game of the chosen type (feature 005, US5). */
export function createQuickSession(
  gameType: 'ORDER' | 'TRIPMINE',
): Promise<CreateSessionResponse> {
  return request<CreateSessionResponse>('/api/sessions/quick', {
    method: 'POST',
    body: { gameType },
  });
}

export function unlockSession(sessionId: string, password: string): Promise<{ ok: boolean }> {
  return request<{ ok: boolean }>(`/api/sessions/${sessionId}/unlock`, {
    method: 'POST',
    body: { password },
  });
}

/** Delete a finished session (feature 005, US4); refused while the game runs. */
export function deleteSession(sessionId: string): Promise<void> {
  return request<void>(`/api/sessions/${sessionId}`, { method: 'DELETE' });
}

export function fetchSessionState(
  sessionId: string,
  role: UserRole,
): Promise<SessionState> {
  return request<SessionState>(
    `/api/sessions/${sessionId}/state?role=${encodeURIComponent(role)}`,
  );
}

/** The URL of the live update stream for a session (contracts/realtime.md). */
export function sessionEventsUrl(sessionId: string, role: UserRole): string {
  return `/api/sessions/${sessionId}/events?role=${encodeURIComponent(role)}`;
}

/**
 * The action channel every board control posts to (contracts/rest-api.md).
 * The request body is sent as-is, so field names must be camelCase.
 */
export function postSessionAction(
  sessionId: string,
  action: Record<string, unknown>,
  password?: string,
): Promise<{ ok: boolean }> {
  return request<{ ok: boolean }>(`/api/sessions/${sessionId}/actions`, {
    method: 'POST',
    body: action,
    password,
  });
}

// --- Folders --------------------------------------------------------------

export interface FolderSummary {
  id: string;
  name: string;
  isDefault: boolean;
  locked: boolean;
}

export function fetchFolders(): Promise<FolderSummary[]> {
  return request<FolderSummary[]>('/api/folders');
}

export function createFolder(name: string, password: string): Promise<FolderSummary> {
  return request<FolderSummary>('/api/folders', { method: 'POST', body: { name, password } });
}

export function unlockFolder(folderId: string, password: string): Promise<{ name: string }> {
  return request<{ name: string }>(`/api/folders/${encodeURIComponent(folderId)}/unlock`, {
    method: 'POST',
    body: { password },
  });
}

export function renameFolder(
  folderId: string,
  name: string,
  password: string,
): Promise<FolderSummary> {
  return request<FolderSummary>(`/api/folders/${encodeURIComponent(folderId)}/rename`, {
    method: 'POST',
    body: { name },
    password,
  });
}

export function deleteFolder(
  folderId: string,
  confirmPassword: string,
  password: string,
): Promise<void> {
  return request<void>(`/api/folders/${encodeURIComponent(folderId)}/delete`, {
    method: 'POST',
    body: { confirmPassword },
    password,
  });
}

// --- Lists ----------------------------------------------------------------

export function fetchDefaultLists(): Promise<{ lists: ListSummary[] }> {
  return request<{ lists: ListSummary[] }>('/api/default-lists');
}

export function fetchFolderLists(
  folderId: string,
  password?: string,
): Promise<{ lists: ListSummary[] }> {
  return request<{ lists: ListSummary[] }>(
    `/api/folders/${encodeURIComponent(folderId)}/lists`,
    { password },
  );
}

export function fetchList(listId: string, password?: string): Promise<ListDetail> {
  return request<ListDetail>(`/api/lists/${encodeURIComponent(listId)}`, { password });
}

export function createList(
  folderId: string,
  list: Partial<ListDetail> & { title: string },
  password?: string,
): Promise<ListDetail> {
  return request<ListDetail>(`/api/folders/${encodeURIComponent(folderId)}/lists`, {
    method: 'POST',
    body: list,
    password,
  });
}

export function updateList(
  listId: string,
  list: Partial<ListDetail> & { title: string },
  password?: string,
): Promise<ListDetail> {
  return request<ListDetail>(`/api/lists/${encodeURIComponent(listId)}`, {
    method: 'PUT',
    body: list,
    password,
  });
}

export function deleteList(listId: string, password?: string): Promise<void> {
  return request<void>(`/api/lists/${encodeURIComponent(listId)}`, {
    method: 'DELETE',
    password,
  });
}

export function exportListCsv(listId: string, password?: string): Promise<string> {
  return requestText(`/api/lists/${encodeURIComponent(listId)}/export.csv`, { password });
}

export function importListCsv(
  folderId: string,
  payload: { title: string; orderDirection: string; csv: string },
  password?: string,
): Promise<{ list: ListDetail; report: ImportReport }> {
  return request<{ list: ListDetail; report: ImportReport }>(
    `/api/folders/${encodeURIComponent(folderId)}/lists/import`,
    { method: 'POST', body: payload, password },
  );
}