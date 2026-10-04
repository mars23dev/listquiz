import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Session } from '../pages/Session';

function jsonResponse(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  });
}

const SESSION_STATE = {
  revision: 4,
  sessionId: 'lq-abcd1234',
  status: 'running',
  teamCount: 2,
  roundCount: 3,
  currentRound: 1,
  muted: false,
  revealValues: false,
  lastMessage: null,
  round: null,
  teams: [
    {
      id: 'team_1',
      name: 'Team Cyan Pulsar',
      color: 'cyan',
      primaryHex: '#06b6d4',
      accentGlow: 'rgba(6, 182, 212, 0.45)',
      score: 10,
      hearts: 3,
      onTurn: true,
    },
  ],
  order: null,
  tripmine: null,
  links: { gamemaster: '/lq-abcd1234/gamemaster', players: [], spectator: '/lq-abcd1234/view' },
};

/** An EventSource stand-in: the page opens a stream, which jsdom cannot do. */
class FakeEventSource {
  static instances: FakeEventSource[] = [];
  onmessage: ((event: MessageEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;

  constructor(readonly url: string) {
    FakeEventSource.instances.push(this);
  }

  close() {}
}

describe('gamemaster password gate', () => {
  beforeEach(() => {
    FakeEventSource.instances = [];
    vi.stubGlobal('EventSource', FakeEventSource as unknown as typeof EventSource);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  function stubFetch(unlockStatus: number) {
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL) => {
        const url = String(input);
        if (url.includes('/unlock')) {
          if (unlockStatus === 200) return jsonResponse({ ok: true });
          return jsonResponse(
            { error: { code: 'unauthorized', message: 'Incorrect password!' } },
            400,
          );
        }
        if (url.includes('/state')) return jsonResponse(SESSION_STATE);
        return jsonResponse({ error: { code: 'bad_request', message: 'unexpected' } }, 400);
      }),
    );
  }

  it('keeps the draft copy and refuses a wrong password with "Incorrect password!"', async () => {
    stubFetch(400);
    const user = userEvent.setup();
    render(<Session sessionId="lq-abcd1234" role="gamemaster" onNavigate={() => {}} />);

    expect(screen.getByRole('heading', { name: /gamemaster access/i })).toBeInTheDocument();
    expect(
      screen.getByText(/enter the moderator password to unlock host controls/i),
    ).toBeInTheDocument();
    expect(screen.getByPlaceholderText('Enter password...')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /switch to view/i })).toBeInTheDocument();

    await user.type(screen.getByPlaceholderText('Enter password...'), 'wrong');
    await user.click(screen.getByRole('button', { name: /unlock/i }));

    expect(await screen.findByRole('alert')).toHaveTextContent('Incorrect password!');
    expect(screen.getByRole('heading', { name: /gamemaster access/i })).toBeInTheDocument();
  });

  it('unlocks with the right password and then shows the session', async () => {
    stubFetch(200);
    const user = userEvent.setup();
    render(<Session sessionId="lq-abcd1234" role="gamemaster" onNavigate={() => {}} />);

    await user.type(screen.getByPlaceholderText('Enter password...'), 'letmein');
    await user.click(screen.getByRole('button', { name: /unlock/i }));

    expect(await screen.findByText('Team Cyan Pulsar')).toBeInTheDocument();
    expect(screen.getByText(/revision 4/)).toHaveTextContent('round 1 of 3');
  });

  it('sends the password in the body, not a header', async () => {
    stubFetch(200);
    const user = userEvent.setup();
    render(<Session sessionId="lq-abcd1234" role="gamemaster" onNavigate={() => {}} />);

    await user.type(screen.getByPlaceholderText('Enter password...'), 'letmein');
    await user.click(screen.getByRole('button', { name: /unlock/i }));

    const call = (fetch as unknown as ReturnType<typeof vi.fn>).mock.calls.find(([url]) =>
      String(url).includes('/unlock'),
    );
    const init = call?.[1] as { method: string; body: string; headers: Record<string, string> };
    expect(init.method).toBe('POST');
    expect(JSON.parse(init.body)).toEqual({ password: 'letmein' });
    expect(init.headers['X-Quiz-Password']).toBeUndefined();
  });

  it('skips the gate for a player link and opens its stream', async () => {
    stubFetch(200);
    render(<Session sessionId="lq-abcd1234" role="p1" onNavigate={() => {}} />);

    expect(screen.queryByRole('heading', { name: /gamemaster access/i })).not.toBeInTheDocument();
    expect(await screen.findByText('Team Cyan Pulsar')).toBeInTheDocument();
    await waitFor(() =>
      expect(FakeEventSource.instances[0]?.url).toBe(
        '/api/sessions/lq-abcd1234/events?role=p1',
      ),
    );
  });

  it('offers the spectator view from the gate', async () => {
    stubFetch(400);
    const user = userEvent.setup();
    const visited: string[] = [];
    render(
      <Session
        sessionId="lq-abcd1234"
        role="gamemaster"
        onNavigate={(path) => visited.push(path)}
      />,
    );

    await user.click(screen.getByRole('button', { name: /switch to view/i }));

    expect(visited).toEqual(['/lq-abcd1234/view']);
  });
});