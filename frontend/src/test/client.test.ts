import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import {
  ApiError,
  createFolder,
  createSession,
  deleteFolder,
  exportListCsv,
  fetchDefaultLists,
  fetchFolders,
  fetchSessionState,
  importListCsv,
  renameFolder,
  sessionEventsUrl,
  unlockFolder,
  unlockSession,
} from '../api/client';

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  });
}

/** Await a call that must fail and hand back the ApiError it threw. */
async function captureError(call: Promise<unknown>): Promise<ApiError> {
  try {
    await call;
  } catch (err) {
    return err as ApiError;
  }
  throw new Error('expected the call to reject');
}

describe('api client', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn());
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('creates a session and returns its id and links', async () => {
    const body = {
      sessionId: 'lq-8fj2wkd9',
      links: {
        gamemaster: '/lq-8fj2wkd9/gamemaster',
        players: [
          { role: 'p1', path: '/lq-8fj2wkd9/p1', hasTeam: true },
          { role: 'p2', path: '/lq-8fj2wkd9/p2', hasTeam: true },
          { role: 'p3', path: '/lq-8fj2wkd9/p3', hasTeam: false },
          { role: 'p4', path: '/lq-8fj2wkd9/p4', hasTeam: false },
        ],
        spectator: '/lq-8fj2wkd9/view',
      },
    };
    (fetch as ReturnType<typeof vi.fn>).mockResolvedValue(jsonResponse(body, 201));

    const result = await createSession({ gmPassword: 'letmein', teamCount: 2 });

    expect(result.sessionId).toBe('lq-8fj2wkd9');
    expect(result.links.players).toHaveLength(4);
    expect(result.links.players[2].hasTeam).toBe(false);
  });

  it('posts the session body as json', async () => {
    (fetch as ReturnType<typeof vi.fn>).mockResolvedValue(jsonResponse({ ok: true }));

    await createSession({ gmPassword: 'letmein' });

    const [path, init] = (fetch as ReturnType<typeof vi.fn>).mock.calls[0];
    expect(path).toBe('/api/sessions');
    expect(init.method).toBe('POST');
    expect(init.headers['Content-Type']).toBe('application/json');
    expect(JSON.parse(init.body)).toEqual({ gmPassword: 'letmein' });
  });

  it('reads state for a role', async () => {
    (fetch as ReturnType<typeof vi.fn>).mockResolvedValue(jsonResponse({ revision: 43 }));

    const state = await fetchSessionState('lq-8fj2wkd9', 'p2');

    expect(state.revision).toBe(43);
    expect((fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]).toBe(
      '/api/sessions/lq-8fj2wkd9/state?role=p2',
    );
  });

  it('sends the gamemaster password header when unlocking', async () => {
    (fetch as ReturnType<typeof vi.fn>).mockResolvedValue(jsonResponse({ ok: true }));

    await unlockSession('lq-8fj2wkd9', 'letmein');

    const [, init] = (fetch as ReturnType<typeof vi.fn>).mock.calls[0];
    expect(init.headers['X-Quiz-Password']).toBeUndefined();
    expect(JSON.parse(init.body)).toEqual({ password: 'letmein' });
  });

  it('keeps the drafts own copy for a wrong password', async () => {
    (fetch as ReturnType<typeof vi.fn>).mockResolvedValue(
      jsonResponse({ error: { code: 'unauthorized', message: 'Incorrect password!' } }, 400),
    );

    const error = await captureError(unlockSession('s', 'nope'));

    expect(error).toBeInstanceOf(ApiError);
    expect(error.message).toBe('Incorrect password!');
    expect(error.isWrongPassword).toBe(true);
    expect(error.status).toBe(400);
  });

  it('reports the error code from the contract body', async () => {
    (fetch as ReturnType<typeof vi.fn>).mockResolvedValue(
      jsonResponse(
        { error: { code: 'too_small', message: 'A list needs at least 11 items.' } },
        422,
      ),
    );

    const error = await captureError(fetchSessionState('s', 'view'));

    expect(error.code).toBe('too_small');
    expect(error.status).toBe(422);
    expect(error.isWrongPassword).toBe(false);
  });

  it('falls back to a generic message when the body is not json', async () => {
    (fetch as ReturnType<typeof vi.fn>).mockResolvedValue(new Response('nope', { status: 500 }));

    const error = await captureError(fetchSessionState('s', 'view'));

    expect(error).toBeInstanceOf(ApiError);
    expect(error.message).toContain('500');
  });

  it('builds the event stream url with the role', () => {
    expect(sessionEventsUrl('lq-8fj2wkd9', 'view')).toBe(
      '/api/sessions/lq-8fj2wkd9/events?role=view',
    );
  });
});

describe('folder and list endpoints', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn());
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('creates a folder with its password in the body', async () => {
    (fetch as ReturnType<typeof vi.fn>).mockResolvedValue(
      jsonResponse({ id: 'fld-7k2m9qx4', name: 'Family facts', isDefault: false }, 201),
    );

    const folder = await createFolder('Family facts', 'hunter2');

    expect(folder.id).toBe('fld-7k2m9qx4');
    const [, init] = (fetch as ReturnType<typeof vi.fn>).mock.calls[0];
    expect(JSON.parse(init.body)).toEqual({ name: 'Family facts', password: 'hunter2' });
  });

  it('sends the password header when renaming a folder', async () => {
    (fetch as ReturnType<typeof vi.fn>).mockResolvedValue(
      jsonResponse({ id: 'fld-1', name: 'Family & friends', isDefault: false }),
    );

    await renameFolder('fld-1', 'Family & friends', 'hunter2');

    const [path, init] = (fetch as ReturnType<typeof vi.fn>).mock.calls[0];
    expect(path).toBe('/api/folders/fld-1/rename');
    expect(init.headers['X-Quiz-Password']).toBe('hunter2');
    expect(JSON.parse(init.body)).toEqual({ name: 'Family & friends' });
  });

  it('names the confirmation password when deleting a folder', async () => {
    (fetch as ReturnType<typeof vi.fn>).mockResolvedValue(new Response(null, { status: 204 }));

    await deleteFolder('fld-1', 'hunter2', 'hunter2');

    const [, init] = (fetch as ReturnType<typeof vi.fn>).mock.calls[0];
    expect(init.method).toBe('POST');
    expect(JSON.parse(init.body)).toEqual({ confirmPassword: 'hunter2' });
  });

  it('unlocks a folder without sending the header', async () => {
    (fetch as ReturnType<typeof vi.fn>).mockResolvedValue(jsonResponse({ name: 'Family facts' }));

    const result = await unlockFolder('fld-1', 'hunter2');

    expect(result.name).toBe('Family facts');
    const [, init] = (fetch as ReturnType<typeof vi.fn>).mock.calls[0];
    expect(init.headers['X-Quiz-Password']).toBeUndefined();
  });

  it('reads the default lists without a password', async () => {
    (fetch as ReturnType<typeof vi.fn>).mockResolvedValue(jsonResponse({ lists: [] }));

    await fetchDefaultLists();

    const [path, init] = (fetch as ReturnType<typeof vi.fn>).mock.calls[0];
    expect(path).toBe('/api/default-lists');
    expect(init.headers['X-Quiz-Password']).toBeUndefined();
  });

  it('returns csv text from the export endpoint', async () => {
    (fetch as ReturnType<typeof vi.fn>).mockResolvedValue(
      new Response('Icon,Name,Year\nRocket,Apollo 11,1969\n', {
        status: 200,
        headers: { 'Content-Type': 'text/csv' },
      }),
    );

    const csv = await exportListCsv('lst-1', 'hunter2');

    expect(csv.split('\n')[0]).toBe('Icon,Name,Year');
  });

  it('imports a csv with its title and direction', async () => {
    (fetch as ReturnType<typeof vi.fn>).mockResolvedValue(
      jsonResponse(
        {
          list: { id: 'lst-2' },
          report: { rowCount: 11, defaultedRows: [{ position: 4, raw: 'n/a' }], warnings: [] },
        },
        201,
      ),
    );

    const result = await importListCsv(
      'fld-1',
      { title: 'Imported', csv: 'Icon,Name,Year\n' },
      'hunter2',
    );

    expect(result.report.rowCount).toBe(11);
    expect(result.report.defaultedRows[0].raw).toBe('n/a');
  });

  it('escapes ids that need it', async () => {
    (fetch as ReturnType<typeof vi.fn>).mockResolvedValue(jsonResponse({ lists: [] }));

    await fetchFolders();

    expect((fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]).toBe('/api/folders');
  });
});