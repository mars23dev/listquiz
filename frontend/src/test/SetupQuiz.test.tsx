import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SetupQuiz } from '../pages/SetupQuiz';

const LISTS = {
  lists: [
    {
      id: 'default_world_history',
      title: 'World History & Inventions',
      description: '',
      itemCount: 12,
      isDefault: true,
      columns: [{ id: 'year', name: 'Year' }],
    },
  ],
};

const FOLDERS = [
  { id: 'default', name: 'Default Lists', isDefault: true, locked: false },
  { id: 'fld-family1', name: 'Family facts', isDefault: false, locked: true },
  { id: 'fld-work1', name: 'Work trivia', isDefault: false, locked: true },
];

const FAMILY_LISTS = {
  lists: [
    {
      id: 'lst-test',
      title: 'Test',
      description: '',
      itemCount: 11,
      isDefault: false,
      columns: [
        { id: 'value', name: 'Value' },
        { id: 'year', name: 'Year' },
      ],
    },
  ],
};

const WORK_LISTS = {
  lists: [
    {
      id: 'lst-work',
      title: 'Test',
      description: '',
      itemCount: 14,
      isDefault: false,
      columns: [{ id: 'value', name: 'Value' }],
    },
  ],
};

function jsonResponse(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  });
}

describe('setup wizard', () => {
  beforeEach(() => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
        const url = String(input);
        const method = init?.method ?? 'GET';
        const headers = (init?.headers ?? {}) as Record<string, string>;
        const body = init?.body
          ? (JSON.parse(String(init.body)) as Record<string, unknown>)
          : undefined;

        if (url === '/api/default-lists') return jsonResponse(LISTS);
        if (url === '/api/folders' && method === 'GET') return jsonResponse(FOLDERS);
        if (url === '/api/folders/fld-family1/unlock' && method === 'POST') {
          return body?.password === 'hunter2'
            ? jsonResponse({ name: 'Family facts' })
            : jsonResponse(
                { error: { code: 'unauthorized', message: 'Incorrect password!' } },
                400,
              );
        }
        if (url === '/api/folders/fld-work1/unlock' && method === 'POST') {
          return body?.password === 'workpw'
            ? jsonResponse({ name: 'Work trivia' })
            : jsonResponse(
                { error: { code: 'unauthorized', message: 'Incorrect password!' } },
                400,
              );
        }
        if (url === '/api/folders/fld-family1/lists') {
          return headers['X-Quiz-Password'] === 'hunter2'
            ? jsonResponse(FAMILY_LISTS)
            : jsonResponse(
                { error: { code: 'unauthorized', message: 'Incorrect password!' } },
                400,
              );
        }
        if (url === '/api/folders/fld-work1/lists') {
          return headers['X-Quiz-Password'] === 'workpw'
            ? jsonResponse(WORK_LISTS)
            : jsonResponse(
                { error: { code: 'unauthorized', message: 'Incorrect password!' } },
                400,
              );
        }
        if (url === '/api/sessions') {
          return jsonResponse(
            {
              sessionId: 'lq-abcd1234',
              links: {
                gamemaster: '/lq-abcd1234/gamemaster',
                players: [
                  { role: 'p1', path: '/lq-abcd1234/p1', hasTeam: true },
                  { role: 'p2', path: '/lq-abcd1234/p2', hasTeam: true },
                  { role: 'p3', path: '/lq-abcd1234/p3', hasTeam: false },
                  { role: 'p4', path: '/lq-abcd1234/p4', hasTeam: false },
                ],
                spectator: '/lq-abcd1234/view',
              },
            },
            201,
          );
        }
        return jsonResponse({ error: { code: 'bad_request', message: 'unexpected' } }, 400);
      }),
    );
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('refuses to continue without a gamemaster password', async () => {
    const user = userEvent.setup();
    render(<SetupQuiz onBackToHome={() => {}} onLaunchRole={() => {}} />);

    await screen.findByRole('heading', { name: /step 1/i });
    await user.click(screen.getByRole('button', { name: /continue/i }));

    expect(await screen.findByRole('alert')).toHaveTextContent(/password is required/i);
    expect(screen.queryByRole('heading', { name: /step 2/i })).not.toBeInTheDocument();
  });

  it('offers one to four teams', async () => {
    const user = userEvent.setup();
    render(<SetupQuiz onBackToHome={() => {}} onLaunchRole={() => {}} />);
    await screen.findByRole('heading', { name: /step 1/i });

    await user.click(screen.getByRole('button', { name: /^1 Team$/i }));

    expect(screen.getAllByRole('textbox')).toHaveLength(1);
    expect(screen.getByRole('button', { name: /^4 Teams$/i })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /^5 Teams$/i })).not.toBeInTheDocument();
  });

  it('adds and removes rounds, renumbering what remains', async () => {
    const user = userEvent.setup();
    render(<SetupQuiz onBackToHome={() => {}} onLaunchRole={() => {}} />);

    await user.type(await screen.findByLabelText(/moderator password/i), 'letmein');
    await user.click(screen.getByRole('button', { name: /continue/i }));
    await screen.findByRole('heading', { name: /step 2/i });

    await user.click(screen.getByRole('button', { name: /add round/i }));
    await user.click(screen.getByRole('button', { name: /add round/i }));
    expect(screen.getByText('Round 3')).toBeInTheDocument();

    const removeButtons = screen.getAllByRole('button', { name: /remove/i });
    await user.click(removeButtons[1]);

    expect(screen.queryByText('Round 3')).not.toBeInTheDocument();
    expect(screen.getByText('Round 2')).toBeInTheDocument();
  });

  it('sends the wizard state to the API and shows every role link', async () => {
    const user = userEvent.setup();
    const launched: string[] = [];
    render(
      <SetupQuiz onBackToHome={() => {}} onLaunchRole={(path) => launched.push(path)} />,
    );

    await user.type(await screen.findByLabelText(/moderator password/i), 'letmein');
    await user.click(screen.getByRole('button', { name: /continue/i }));
    await screen.findByRole('heading', { name: /step 2/i });
    await user.click(screen.getByRole('button', { name: /create session/i }));

    expect(await screen.findByText(/session ready:/i)).toBeInTheDocument();

    const call = (fetch as unknown as ReturnType<typeof vi.fn>).mock.calls.find(([url]) =>
      String(url) === '/api/sessions',
    );
    const body = JSON.parse(String((call?.[1] as { body: string }).body));
    expect(body.gmPassword).toBe('letmein');
    expect(body.teamCount).toBe(2);
    expect(body.roundCount).toBe(1);
    expect(body.rounds[0]).toMatchObject({
      quizType: 'ORDER',
      listId: 'default_world_history',
      columnId: 'year',
    });
    expect(body.rounds[0]).not.toHaveProperty('tripmineSide');
    expect(body.folderPasswords).toBeUndefined();

    expect(screen.getByText('/lq-abcd1234/gamemaster')).toBeInTheDocument();
    expect(screen.getByText('HOST')).toBeInTheDocument();
    expect(screen.getByText('PUBLIC')).toBeInTheDocument();
    for (const role of ['p1', 'p2', 'p3', 'p4']) {
      expect(screen.getByText(`/lq-abcd1234/${role}`)).toBeInTheDocument();
    }

    await user.click(screen.getByRole('button', { name: /start game as gamemaster/i }));
    await waitFor(() => expect(launched).toEqual(['/lq-abcd1234/gamemaster']));
  });

  it('surfaces a backend refusal instead of pretending to be ready', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL) => {
        const url = String(input);
        if (url === '/api/default-lists') return jsonResponse(LISTS);
        return jsonResponse(
          { error: { code: 'bad_request', message: 'Round count must be between 1 and 10.' } },
          400,
        );
      }),
    );
    const user = userEvent.setup();
    render(<SetupQuiz onBackToHome={() => {}} onLaunchRole={() => {}} />);

    await user.type(await screen.findByLabelText(/moderator password/i), 'letmein');
    await user.click(screen.getByRole('button', { name: /continue/i }));
    await screen.findByRole('heading', { name: /step 2/i });
    await user.click(screen.getByRole('button', { name: /create session/i }));

    expect(await screen.findByRole('alert')).toHaveTextContent(/between 1 and 10/i);
    expect(screen.queryByText(/session ready:/i)).not.toBeInTheDocument();
  });

  it('advances the wizard with the keyboard alone', async () => {
    const user = userEvent.setup();
    render(<SetupQuiz onBackToHome={() => {}} onLaunchRole={() => {}} />);
    await screen.findByRole('heading', { name: /step 1/i });

    const password = screen.getByLabelText(/moderator password/i);
    password.focus();
    await user.keyboard('letmein');

    const continueButton = screen.getByRole('button', { name: /continue/i });
    continueButton.focus();
    await user.keyboard('{Enter}');

    expect(await screen.findByRole('heading', { name: /step 2/i })).toBeInTheDocument();
  });

  it('keeps a team-name field focused while a full name is typed', async () => {
    const user = userEvent.setup();
    render(<SetupQuiz onBackToHome={() => {}} onLaunchRole={() => {}} />);
    await screen.findByRole('heading', { name: /step 1/i });

    const field = screen.getAllByRole('textbox')[0];
    field.focus();
    await user.keyboard('Team Cyan Pulsar');

    expect(field).toHaveValue('Team Cyan Pulsar');
    expect(document.activeElement).toBe(field);
  });

  it('shows the tripmine side control only for Tripmine rounds', async () => {
    const user = userEvent.setup();
    render(<SetupQuiz onBackToHome={() => {}} onLaunchRole={() => {}} />);
    await user.type(await screen.findByLabelText(/moderator password/i), 'letmein');
    await user.click(screen.getByRole('button', { name: /continue/i }));
    await screen.findByRole('heading', { name: /step 2/i });

    // The default round is Order: no tripmine side control.
    expect(screen.queryByLabelText(/tripmine side/i)).not.toBeInTheDocument();

    const quizType = screen.getAllByRole('combobox')[0];
    await user.selectOptions(quizType, 'TRIPMINE');
    expect(screen.getByLabelText(/tripmine side/i)).toBeInTheDocument();

    await user.selectOptions(quizType, 'ORDER');
    expect(screen.queryByLabelText(/tripmine side/i)).not.toBeInTheDocument();
  });

  it('offers Random (Hard) as a Tripmine side', async () => {
    const user = userEvent.setup();
    render(<SetupQuiz onBackToHome={() => {}} onLaunchRole={() => {}} />);
    await user.type(await screen.findByLabelText(/moderator password/i), 'letmein');
    await user.click(screen.getByRole('button', { name: /continue/i }));
    await screen.findByRole('heading', { name: /step 2/i });

    const quizType = screen.getAllByRole('combobox')[0];
    await user.selectOptions(quizType, 'TRIPMINE');

    const side = screen.getByLabelText(/tripmine side/i);
    expect(within(side).getByRole('option', { name: /random \(hard\)/i })).toBeInTheDocument();
    expect(within(side).getByRole('option', { name: /^highest$/i })).toBeInTheDocument();
    expect(within(side).getByRole('option', { name: /^lowest$/i })).toBeInTheDocument();
  });

  it('unlocks a protected folder and selects its custom list', async () => {
    const user = userEvent.setup();
    const fetchMock = fetch as unknown as ReturnType<typeof vi.fn>;
    render(<SetupQuiz onBackToHome={() => {}} onLaunchRole={() => {}} />);

    await user.type(await screen.findByLabelText(/moderator password/i), 'letmein');
    await user.click(screen.getByRole('button', { name: /continue/i }));
    await screen.findByRole('heading', { name: /step 2/i });

    await user.click(screen.getByRole('button', { name: /family facts/i }));
    await user.type(
      screen.getByLabelText(/password for family facts/i),
      'hunter2',
    );
    await user.click(screen.getByRole('button', { name: /^unlock$/i }));

    expect(
      await screen.findByRole('option', {
        name: /family facts \/ test \(11 items\)/i,
      }),
    ).toBeInTheDocument();

    const listSelect = screen.getAllByLabelText(/^list$/i)[0];
    await user.selectOptions(listSelect, 'lst-test');
    const valueColumn = screen.getByLabelText(/value column/i);
    await user.selectOptions(valueColumn, 'year');
    await user.click(screen.getByRole('button', { name: /create session/i }));

    expect(await screen.findByText(/session ready:/i)).toBeInTheDocument();
    const call = fetchMock.mock.calls.find(([url]) => String(url) === '/api/sessions');
    const body = JSON.parse(String((call?.[1] as { body: string }).body));
    expect(body.rounds[0]).toMatchObject({
      listId: 'lst-test',
      columnId: 'year',
    });
    expect(body.folderPasswords).toEqual({ 'fld-family1': 'hunter2' });
  });

  it('keeps a locked folder list hidden and rejects a wrong password', async () => {
    const user = userEvent.setup();
    const fetchMock = fetch as unknown as ReturnType<typeof vi.fn>;
    render(<SetupQuiz onBackToHome={() => {}} onLaunchRole={() => {}} />);

    await user.type(await screen.findByLabelText(/moderator password/i), 'letmein');
    await user.click(screen.getByRole('button', { name: /continue/i }));
    await screen.findByRole('heading', { name: /step 2/i });

    expect(
      screen.queryByRole('option', { name: /family facts \/ test/i }),
    ).not.toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: /family facts/i }));
    await user.type(
      screen.getByLabelText(/password for family facts/i),
      'wrong',
    );
    await user.click(screen.getByRole('button', { name: /^unlock$/i }));

    expect(await screen.findByRole('alert')).toHaveTextContent(/incorrect password/i);
    expect(
      screen.queryByRole('option', { name: /family facts \/ test/i }),
    ).not.toBeInTheDocument();
    expect(
      fetchMock.mock.calls.filter(([url]) => String(url).includes('fld-family1/lists')),
    ).toHaveLength(0);
  });

  it('unlocks two folders and distinguishes duplicate custom list titles', async () => {
    const user = userEvent.setup();
    const fetchMock = fetch as unknown as ReturnType<typeof vi.fn>;
    render(<SetupQuiz onBackToHome={() => {}} onLaunchRole={() => {}} />);

    await user.type(await screen.findByLabelText(/moderator password/i), 'letmein');
    await user.click(screen.getByRole('button', { name: /continue/i }));
    await screen.findByRole('heading', { name: /step 2/i });

    await user.click(screen.getByRole('button', { name: /family facts/i }));
    await user.type(
      screen.getByLabelText(/password for family facts/i),
      'hunter2',
    );
    await user.click(screen.getByRole('button', { name: /^unlock$/i }));
    await screen.findByRole('option', {
      name: /family facts \/ test \(11 items\)/i,
    });

    await user.click(screen.getByRole('button', { name: /work trivia/i }));
    await user.type(screen.getByLabelText(/password for work trivia/i), 'workpw');
    await user.click(screen.getByRole('button', { name: /^unlock$/i }));

    expect(
      await screen.findByRole('option', {
        name: /family facts \/ test \(11 items\)/i,
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('option', { name: /work trivia \/ test \(14 items\)/i }),
    ).toBeInTheDocument();

    const firstLists = screen.getAllByLabelText(/^list$/i);
    await user.selectOptions(firstLists[0], 'lst-test');
    const valueColumn = screen.getByLabelText(/value column/i);
    await user.selectOptions(valueColumn, 'year');

    await user.click(screen.getByRole('button', { name: /add round/i }));
    const secondLists = screen.getAllByLabelText(/^list$/i);
    await user.selectOptions(secondLists[1], 'lst-work');
    expect(screen.getAllByLabelText(/value column/i)).toHaveLength(1);

    await user.click(screen.getByRole('button', { name: /create session/i }));
    expect(await screen.findByText(/session ready:/i)).toBeInTheDocument();

    const call = fetchMock.mock.calls.find(([url]) => String(url) === '/api/sessions');
    const body = JSON.parse(String((call?.[1] as { body: string }).body));
    expect(body.rounds).toMatchObject([
      { listId: 'lst-test', columnId: 'year' },
      { listId: 'lst-work', columnId: 'value' },
    ]);
    expect(body.folderPasswords).toEqual({
      'fld-family1': 'hunter2',
      'fld-work1': 'workpw',
    });
  });
});