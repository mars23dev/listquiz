import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SetupQuiz } from '../pages/SetupQuiz';

const LISTS = {
  lists: [
    {
      id: 'default_world_history',
      title: 'World History & Inventions',
      description: '',
      orderDirection: 'low_to_high',
      itemCount: 12,
      isDefault: true,
      columns: [{ id: 'year', name: 'Year' }],
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
      vi.fn(async (input: RequestInfo | URL) => {
        const url = String(input);
        if (url === '/api/default-lists') return jsonResponse(LISTS);
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
});