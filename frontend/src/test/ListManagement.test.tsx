import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ListManagement } from '../pages/ListManagement';

const FOLDERS = [
  { id: 'default', name: 'Default Lists', isDefault: true, locked: false },
  { id: 'fld-family1', name: 'Family facts', isDefault: false, locked: true },
];

const DEFAULT_LISTS = {
  lists: [
    {
      id: 'default_world_history',
      title: 'World History & Inventions',
      description: 'Key human milestones',
      orderDirection: 'low_to_high',
      columns: [{ id: 'year', name: 'Year' }],
      itemCount: 11,
      isDefault: true,
    },
  ],
};

const SECRET_LISTS = {
  lists: [
    {
      id: 'lst-recipes',
      title: "Grandma's recipes",
      description: '',
      orderDirection: 'low_to_high',
      columns: [{ id: 'year', name: 'Year' }],
      itemCount: 12,
      isDefault: false,
    },
  ],
};

function jsonResponse(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  });
}

function stubApi(overrides: { unlockStatus?: number } = {}) {
  const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
    const url = String(input);
    const method = init?.method ?? 'GET';
    const headers = (init?.headers ?? {}) as Record<string, string>;

    if (url === '/api/folders' && method === 'GET') return jsonResponse(FOLDERS);
    if (url === '/api/default-lists') return jsonResponse(DEFAULT_LISTS);
    if (url === '/api/folders/default/lists') return jsonResponse(DEFAULT_LISTS);
    if (url === '/api/folders/fld-family1/lists') {
      if (headers['X-Quiz-Password'] === 'hunter2') return jsonResponse(SECRET_LISTS);
      return jsonResponse(
        { error: { code: 'unauthorized', message: 'Incorrect password!' } },
        400,
      );
    }
    if (url.endsWith('/unlock')) {
      if ((overrides.unlockStatus ?? 200) === 200) return jsonResponse({ name: 'Family facts' });
      return jsonResponse(
        { error: { code: 'unauthorized', message: 'Incorrect password!' } },
        400,
      );
    }
    if (url === '/api/folders' && method === 'POST') {
      return jsonResponse({ id: 'fld-new1234', name: 'Weekend', isDefault: false }, 201);
    }
    if (url.endsWith('/delete')) return new Response(null, { status: 204 });
    if (url.endsWith('/rename')) {
      return jsonResponse({ id: 'fld-family1', name: 'Family & friends', isDefault: false });
    }
    return jsonResponse({ error: { code: 'not_found', message: 'unexpected' } }, 404);
  });
  vi.stubGlobal('fetch', fetchMock);
  return fetchMock;
}

describe('folder bar', () => {
  beforeEach(() => {
    stubApi();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('opens on the default folder and shows the seeded lists', async () => {
    render(<ListManagement onBackToHome={() => {}} />);

    expect(await screen.findByText('World History & Inventions')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /default lists \(default\)/i })).toHaveAttribute(
      'aria-current',
      'true',
    );
    // Read-only: no write affordances on a default-folder card (FR-050).
    expect(screen.getByText('Read-only')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /duplicate list/i })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /^delete list$/i })).not.toBeInTheDocument();
  });

  it('never shows a locked folder list name before the password', async () => {
    const fetchMock = stubApi();
    render(<ListManagement onBackToHome={() => {}} />);
    await screen.findByText('World History & Inventions');

    const lockedTab = screen.getByRole('button', { name: /family facts/i });
    expect(lockedTab.textContent).toContain('\u{1F512}');
    // Asking for a locked folder's lists is not even attempted.
    expect(
      fetchMock.mock.calls.filter(([url]) => String(url).includes('fld-family1/lists')),
    ).toHaveLength(0);
  });

  it('unlocks a folder and then shows its lists', async () => {
    const user = userEvent.setup();
    render(<ListManagement onBackToHome={() => {}} />);
    await screen.findByText('World History & Inventions');

    await user.click(screen.getByRole('button', { name: /family facts/i }));
    expect(await screen.findByRole('dialog', { name: /unlock folder/i })).toBeInTheDocument();
    await user.type(screen.getByLabelText(/folder password/i), 'hunter2');
    await user.click(screen.getByRole('button', { name: /^unlock$/i }));

    expect(await screen.findByText("Grandma's recipes")).toBeInTheDocument();
    // The password now travels in the header for this folder's reads.
    const call = (fetch as unknown as ReturnType<typeof vi.fn>).mock.calls.find(([url]) =>
      String(url).includes('fld-family1/lists'),
    );
    expect((call?.[1] as { headers: Record<string, string> }).headers['X-Quiz-Password']).toBe(
      'hunter2',
    );
  });

  it('reports a wrong folder password with the draft copy', async () => {
    stubApi({ unlockStatus: 400 });
    const user = userEvent.setup();
    render(<ListManagement onBackToHome={() => {}} />);
    await screen.findByText('World History & Inventions');

    await user.click(screen.getByRole('button', { name: /family facts/i }));
    await user.type(await screen.findByLabelText(/folder password/i), 'nope');
    await user.click(screen.getByRole('button', { name: /^unlock$/i }));

    expect(await screen.findByRole('alert')).toHaveTextContent('Incorrect password!');
    expect(screen.queryByText("Grandma's recipes")).not.toBeInTheDocument();
  });

  it('names the list count before a delete and re-entered password confirms it', async () => {
    const user = userEvent.setup();
    render(<ListManagement onBackToHome={() => {}} />);
    await screen.findByText('World History & Inventions');
    await user.click(screen.getByRole('button', { name: /family facts/i }));
    await user.type(await screen.findByLabelText(/folder password/i), 'hunter2');
    await user.click(screen.getByRole('button', { name: /^unlock$/i }));
    await screen.findByText("Grandma's recipes");

    await user.click(screen.getByRole('button', { name: /^delete$/i }));
    expect(await screen.findByRole('dialog', { name: /delete family facts/i })).toHaveTextContent(
      '1 list',
    );

    await user.type(screen.getByLabelText(/re-enter the folder password/i), 'hunter2');
    await user.click(screen.getByRole('button', { name: /delete folder/i }));

    const call = (fetch as unknown as ReturnType<typeof vi.fn>).mock.calls.find(([url]) =>
      String(url).endsWith('/delete'),
    );
    const init = call?.[1] as { body: string; headers: Record<string, string> };
    expect(JSON.parse(init.body)).toEqual({ confirmPassword: 'hunter2' });
    expect(init.headers['X-Quiz-Password']).toBe('hunter2');
  });

  it('creates a folder with a mandatory password', async () => {
    const user = userEvent.setup();
    render(<ListManagement onBackToHome={() => {}} />);
    await screen.findByText('World History & Inventions');

    await user.click(screen.getByRole('button', { name: /new folder/i }));
    const submit = screen.getByRole('button', { name: /create folder/i });
    expect(submit).toBeDisabled();

    await user.type(screen.getByLabelText(/folder name/i), 'Weekend');
    expect(submit).toBeDisabled();
    await user.type(screen.getByLabelText(/folder password/i), 'letmein');
    expect(submit).toBeEnabled();
    await user.click(submit);

    await waitFor(() => {
      const call = (fetch as unknown as ReturnType<typeof vi.fn>).mock.calls.find(
        ([url, init]) => String(url) === '/api/folders' && (init as RequestInit)?.method === 'POST',
      );
      expect(JSON.parse(String((call?.[1] as { body: string }).body))).toEqual({
        name: 'Weekend',
        password: 'letmein',
      });
    });
  });
});