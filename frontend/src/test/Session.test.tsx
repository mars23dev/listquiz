import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor, within } from '@testing-library/react';
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

  addEventListener(_type: string, _listener: EventListenerOrEventListenerObject) {}

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

    expect(await screen.findByRole('heading', { name: 'Team Cyan Pulsar' })).toBeInTheDocument();
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

const ORDER_POOL = [
  { itemId: 'item_a', name: 'Alpha', value: '10', revealed: true },
  { itemId: 'item_b', name: 'Bravo', value: '20', revealed: true },
];

function orderState(overrides: Record<string, unknown> = {}) {
  return {
    ...SESSION_STATE,
    order: {
      anchor: { itemId: 'anchor', name: 'Anchor', value: '15' },
      pool: ORDER_POOL,
      left: [],
      right: [],
      pending: null,
      ...overrides,
    },
  };
}

/**
 * The server holds a pending item with slotIndex -1 until a placement is
 * confirmed, so the board must offer the ten slots before it draws the item.
 */
function pendingState(slotIndex = -1) {
  return orderState({
    pending: {
      itemId: 'item_a',
      name: 'Alpha',
      side: 'left',
      slotIndex,
      teamId: 'team_1',
    },
  });
}

describe('order board actions', () => {
  let posted: { url: string; body: Record<string, unknown>; passwordHeader: string | null }[] = [];
  let stateResponse: () => unknown = () => orderState();

  beforeEach(() => {
    FakeEventSource.instances = [];
    posted = [];
    stateResponse = () => orderState();
    vi.stubGlobal('EventSource', FakeEventSource as unknown as typeof EventSource);
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
        const url = String(input);
        if (url.includes('/unlock')) return jsonResponse({ ok: true });
        if (url.includes('/actions')) {
          posted.push({
            url,
            body: JSON.parse(String(init?.body ?? '{}')),
            passwordHeader: new Headers(init?.headers).get('X-Quiz-Password'),
          });
          return jsonResponse({ ok: true });
        }
        if (url.includes('/state')) return jsonResponse(stateResponse());
        return jsonResponse({ error: { code: 'bad_request', message: 'unexpected' } }, 400);
      }),
    );
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  async function renderUnlocked(role: 'gamemaster' | 'p1' | 'p2') {
    const user = userEvent.setup();
    render(<Session sessionId="lq-abcd1234" role={role} onNavigate={() => {}} />);
    if (role === 'gamemaster') {
      await user.type(screen.getByPlaceholderText('Enter password...'), 'letmein');
      await user.click(screen.getByRole('button', { name: /unlock/i }));
    }
    // Match the team card heading, not the waiting banner, which repeats the name.
    await screen.findByRole('heading', { name: 'Team Cyan Pulsar' });
    return user;
  }

  it('sends select_item when a pool item is clicked', async () => {
    const user = await renderUnlocked('gamemaster');

    await user.click(await screen.findByRole('button', { name: /select alpha/i }));

    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0].body).toEqual({
      action: 'select_item',
      itemId: 'item_a',
      teamId: 'team_1',
    });
    expect(posted[0].passwordHeader).toBe('letmein');
  });

  it('exposes every pool item and slot as a button reachable by keyboard', async () => {
    stateResponse = () => pendingState();
    const user = await renderUnlocked('gamemaster');

    const alpha = await screen.findByRole('button', { name: /select alpha/i });
    alpha.focus();
    expect(document.activeElement).toBe(alpha);

    // Enter is the keyboard equivalent of a click (FR-028).
    await user.keyboard('{Enter}');
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0].body).toMatchObject({ action: 'select_item', itemId: 'item_a' });

    // All ten bar slots are real buttons, so the bar is fully keyboard reachable.
    expect(screen.getAllByRole('button', { name: /slot \d/i })).toHaveLength(10);
  });

  it('sends choose_slot only after a slot is picked, then confirmed', async () => {
    stateResponse = () => pendingState();
    const user = await renderUnlocked('gamemaster');

    // Picking a slot is a local step: it must not reach the server on its own.
    const slot = await screen.findByRole('button', {
      name: /place the selected item in left slot 1/i,
    });
    slot.focus();
    await user.keyboard('{Enter}');
    expect(posted).toHaveLength(0);

    const confirm = await screen.findByRole('button', { name: /confirm placement/i });
    confirm.focus();
    await user.keyboard('{Enter}');

    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0].body).toEqual({
      action: 'choose_slot',
      itemId: 'item_a',
      side: 'left',
      slotIndex: 0,
      teamId: 'team_1',
    });
  });

  it('sends cancel_selection from the pending item', async () => {
    stateResponse = () => pendingState();
    const user = await renderUnlocked('gamemaster');

    const cancel = await screen.findByRole('button', { name: /cancel position/i });
    cancel.focus();
    await user.keyboard('{Enter}');

    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0].body).toEqual({
      action: 'cancel_selection',
      itemId: 'item_a',
      teamId: 'team_1',
    });
  });

  it('lets the gamemaster return a placed item to the pool', async () => {
    stateResponse = () => orderState({ left: [ORDER_POOL[0]] });
    const user = await renderUnlocked('gamemaster');

    await user.click(await screen.findByRole('button', { name: 'Return Alpha to pool' }));

    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0].body).toEqual({ action: 'return_to_pool', itemId: 'item_a', teamId: 'team_1' });
    expect(posted[0].passwordHeader).toBe('letmein');
  });

  it('offers the gamemaster reveal control in Tripmine rounds too', async () => {
    stateResponse = () => ({
      ...SESSION_STATE,
      round: { roundNumber: 1, quizType: 'TRIPMINE' },
      order: null,
      tripmine: { side: 'highest', searchedRank: 0, hit: false, completed: false, items: [] },
    });
    const user = await renderUnlocked('gamemaster');

    await user.click(await screen.findByRole('button', { name: /reveal values/i }));

    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0].body).toEqual({ action: 'reveal_values' });
    expect(posted[0].passwordHeader).toBe('letmein');
  });

  it('gives a player a read-only board with no action buttons', async () => {
    await renderUnlocked('p2');

    expect(await screen.findByText('Alpha')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /select alpha/i })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /reveal values/i })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /place the selected item/i })).not.toBeInTheDocument();
  });

  it('shows the draft waiting banner to a player who is not on turn', async () => {
    stateResponse = () => ({
      ...orderState(),
      teams: [
        { ...SESSION_STATE.teams[0], onTurn: false },
        {
          ...SESSION_STATE.teams[0],
          id: 'team_2',
          name: 'Team Ruby Spark',
          color: 'rose',
          primaryHex: '#f43f5e',
          onTurn: true,
        },
      ],
    });
    await renderUnlocked('p1');

    const banner = await screen.findByText(/to place an item/i);
    // The team name also appears on the team card, so check it inside the banner.
    expect(within(banner).getByText('Team Ruby Spark')).toBeInTheDocument();
  });

  it('does not show the waiting banner to the player whose team is on turn', async () => {
    await renderUnlocked('p1');
    expect(screen.queryByText(/to place an item/i)).not.toBeInTheDocument();
  });

  it('hides the waiting banner once an item is pending', async () => {
    stateResponse = () => pendingState();
    await renderUnlocked('p1');

    await screen.findByText('Alpha');
    expect(screen.queryByText(/to place an item/i)).not.toBeInTheDocument();
  });

  it('maps the dropped column to a valid gap on a partly-filled side', async () => {
    // Left side already holds one item; the far column is the far gap and the column
    // beside the anchor is the gap after the last item.
    stateResponse = () =>
      orderState({
        left: [ORDER_POOL[0]],
        pending: {
          itemId: 'item_b',
          name: 'Bravo',
          side: 'left',
          slotIndex: -1,
          teamId: 'team_1',
        },
      });
    const posted: { body: Record<string, unknown> }[] = [];
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
        const url = String(input);
        if (url.includes('/unlock')) return jsonResponse({ ok: true });
        if (url.includes('/actions')) {
          posted.push({ body: JSON.parse(String(init?.body ?? '{}')) });
          return jsonResponse({ ok: true });
        }
        if (url.includes('/state')) return jsonResponse(stateResponse());
        return jsonResponse({ error: { code: 'bad_request', message: 'unexpected' } }, 400);
      }),
    );
    const user = userEvent.setup();
    render(<Session sessionId="lq-abcd1234" role="gamemaster" onNavigate={() => {}} />);
    await user.type(screen.getByPlaceholderText('Enter password...'), 'letmein');
    await user.click(screen.getByRole('button', { name: /unlock/i }));

    await user.click(
      await screen.findByRole('button', { name: /place the selected item in left slot 5/i }),
    );
    await user.click(await screen.findByRole('button', { name: /confirm placement/i }));

    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0].body).toMatchObject({
      action: 'choose_slot',
      side: 'left',
      slotIndex: 1,
    });
    expect(screen.queryByText(/is not on the left side/i)).not.toBeInTheDocument();
  });
});

function tripmineState() {
  return {
    ...SESSION_STATE,
    order: null,
    round: {
      roundNumber: 1,
      quizType: 'TRIPMINE',
      tripmineSide: 'highest',
      listTitle: 'Cinema',
      listId: 'list_2',
      columnId: 'box_office',
      columnName: 'Box Office ($)',
    },
    tripmine: {
      side: 'highest',
      searchedRank: 1,
      hit: false,
      completed: false,
      items: [
        {
          itemId: 'item_a',
          name: 'Alpha',
          value: '10',
          revealed: false,
          trueRank: 1,
          isTripmine: false,
          pointsAwarded: null,
        },
        {
          itemId: 'item_b',
          name: 'Bravo',
          value: '20',
          revealed: false,
          trueRank: 10,
          isTripmine: true,
          pointsAwarded: null,
        },
      ],
    },
  };
}

describe('tripmine board actions', () => {
  let posted: { url: string; body: Record<string, unknown> }[] = [];

  beforeEach(() => {
    FakeEventSource.instances = [];
    posted = [];
    vi.stubGlobal('EventSource', FakeEventSource as unknown as typeof EventSource);
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
        const url = String(input);
        if (url.includes('/unlock')) return jsonResponse({ ok: true });
        if (url.includes('/actions')) {
          posted.push({ url, body: JSON.parse(String(init?.body ?? '{}')) });
          return jsonResponse({ ok: true });
        }
        if (url.includes('/state')) return jsonResponse(tripmineState());
        return jsonResponse({ error: { code: 'bad_request', message: 'unexpected' } }, 400);
      }),
    );
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  async function renderBoard(role: 'gamemaster' | 'p1' | 'p2') {
    const user = userEvent.setup();
    render(<Session sessionId="lq-abcd1234" role={role} onNavigate={() => {}} />);
    if (role === 'gamemaster') {
      await user.type(screen.getByPlaceholderText('Enter password...'), 'letmein');
      await user.click(screen.getByRole('button', { name: /unlock/i }));
    }
    await screen.findByRole('heading', { name: 'Team Cyan Pulsar' });
    return user;
  }

  it('posts pick_tripitem when a cell is activated by keyboard', async () => {
    const user = await renderBoard('gamemaster');

    const cell = await screen.findByRole('button', { name: /pick alpha/i });
    cell.focus();
    await user.keyboard('{Enter}');

    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0].body).toEqual({
      action: 'pick_tripitem',
      itemId: 'item_a',
      teamId: 'team_1',
    });
  });

  it('gives a player a read-only grid', async () => {
    await renderBoard('p2');

    // A player's cells are present but disabled, so nothing can post an action.
    const cell = screen.queryByRole('button', { name: /alpha, hidden/i });
    expect(cell).not.toBeNull();
    expect(cell).toBeDisabled();
    expect(screen.queryByRole('button', { name: /pick alpha/i })).not.toBeInTheDocument();
  });
});

describe('role views and session controls', () => {
  let posted: { url: string; body: Record<string, unknown> }[] = [];

  beforeEach(() => {
    FakeEventSource.instances = [];
    posted = [];
    vi.stubGlobal('EventSource', FakeEventSource as unknown as typeof EventSource);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('shows the perspective banner for each role', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL) => {
        const url = String(input);
        if (url.includes('/unlock')) return jsonResponse({ ok: true });
        if (url.includes('/state')) return jsonResponse(orderState());
        return jsonResponse({ error: { code: 'bad_request', message: 'unexpected' } }, 400);
      }),
    );

    const user = userEvent.setup();
    const { unmount } = render(
      <Session sessionId="lq-abcd1234" role="gamemaster" onNavigate={() => {}} />,
    );
    await user.type(screen.getByPlaceholderText('Enter password...'), 'letmein');
    await user.click(screen.getByRole('button', { name: /unlock/i }));
    expect(await screen.findByText(/moderator controls active/i)).toBeInTheDocument();
    unmount();
  });

  it('shows the stage broadcast banner and no host controls to the spectator', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL) => {
        const url = String(input);
        if (url.includes('/state')) return jsonResponse(pendingState());
        return jsonResponse({ error: { code: 'bad_request', message: 'unexpected' } }, 400);
      }),
    );

    render(<Session sessionId="lq-abcd1234" role="view" onNavigate={() => {}} />);

    expect(await screen.findByText(/stage broadcast screen/i)).toBeInTheDocument();
    expect(screen.queryByText(/moderator controls active/i)).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /reveal values/i })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /select alpha/i })).not.toBeInTheDocument();
  });

  it('posts set_muted from the gamemaster dashboard', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
        const url = String(input);
        if (url.includes('/unlock')) return jsonResponse({ ok: true });
        if (url.includes('/actions')) {
          posted.push({ url, body: JSON.parse(String(init?.body ?? '{}')) });
          return jsonResponse({ ok: true });
        }
        if (url.includes('/state')) return jsonResponse(orderState());
        return jsonResponse({ error: { code: 'bad_request', message: 'unexpected' } }, 400);
      }),
    );

    const user = userEvent.setup();
    render(<Session sessionId="lq-abcd1234" role="gamemaster" onNavigate={() => {}} />);
    await user.type(screen.getByPlaceholderText('Enter password...'), 'letmein');
    await user.click(screen.getByRole('button', { name: /unlock/i }));

    await user.click(await screen.findByRole('button', { name: /^mute$/i }));

    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0].body).toEqual({ action: 'set_muted', muted: true });
  });

  it('renders a not-found view for an unknown session', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () =>
        jsonResponse({ error: { code: 'not_found', message: 'Unknown session.' } }, 404),
      ),
    );

    render(<Session sessionId="lq-nope" role="p1" onNavigate={() => {}} />);

    expect(await screen.findByText(/session not found/i)).toBeInTheDocument();
    expect(screen.getByText(/lq-nope/)).toBeInTheDocument();
  });
});

describe('player turn interaction', () => {
  let posted: { body: Record<string, unknown>; passwordHeader: string | null }[] = [];

  function stub() {
    posted = [];
    vi.stubGlobal('EventSource', FakeEventSource as unknown as typeof EventSource);
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
        const url = String(input);
        if (url.includes('/actions')) {
          posted.push({
            body: JSON.parse(String(init?.body ?? '{}')),
            passwordHeader: new Headers(init?.headers).get('X-Quiz-Password'),
          });
          return jsonResponse({ ok: true });
        }
        if (url.includes('/state')) return jsonResponse(orderState());
        return jsonResponse({ error: { code: 'bad_request', message: 'unexpected' } }, 400);
      }),
    );
  }

  beforeEach(() => {
    FakeEventSource.instances = [];
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('lets the team on turn act from its player role, with the role and no password', async () => {
    stub();
    const user = userEvent.setup();
    render(<Session sessionId="lq-abcd1234" role="p1" onNavigate={() => {}} />);

    await user.click(await screen.findByRole('button', { name: /select alpha/i }));

    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0].body).toEqual({
      action: 'select_item',
      itemId: 'item_a',
      teamId: 'team_1',
      role: 'p1',
    });
    expect(posted[0].passwordHeader).toBeNull();
  });

  it('keeps a player whose team is not on turn read-only', async () => {
    stub();
    render(<Session sessionId="lq-abcd1234" role="p2" onNavigate={() => {}} />);

    await screen.findByRole('heading', { name: 'Team Cyan Pulsar' });
    expect(screen.queryByRole('button', { name: /select alpha/i })).not.toBeInTheDocument();
  });

  it('keeps the spectator read-only', async () => {
    stub();
    render(<Session sessionId="lq-abcd1234" role="view" onNavigate={() => {}} />);

    await screen.findByText(/stage broadcast screen/i);
    expect(screen.queryByRole('button', { name: /select alpha/i })).not.toBeInTheDocument();
  });
});

describe('gamemaster dashboard controls', () => {
  let posted: { url: string; body: Record<string, unknown> }[] = [];

  function stub(state: unknown) {
    posted = [];
    vi.stubGlobal('EventSource', FakeEventSource as unknown as typeof EventSource);
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
        const url = String(input);
        if (url.includes('/unlock')) return jsonResponse({ ok: true });
        if (url.includes('/actions')) {
          posted.push({ url, body: JSON.parse(String(init?.body ?? '{}')) });
          return jsonResponse({ ok: true });
        }
        if (url.includes('/state')) return jsonResponse(state);
        return jsonResponse({ error: { code: 'bad_request', message: 'unexpected' } }, 400);
      }),
    );
  }

  beforeEach(() => {
    FakeEventSource.instances = [];
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('sends pass_turn, next_round, and score adjustments', async () => {
    stub(orderState());
    const user = userEvent.setup();
    render(<Session sessionId="lq-abcd1234" role="gamemaster" onNavigate={() => {}} />);
    await user.type(screen.getByPlaceholderText('Enter password...'), 'letmein');
    await user.click(screen.getByRole('button', { name: /unlock/i }));

    await user.click(await screen.findByRole('button', { name: /pass turn/i }));
    await screen.findByRole('button', { name: /next round/i });
    await user.click(screen.getByRole('button', { name: /next round/i }));
    await user.click(screen.getByRole('button', { name: /\+50/i }));

    await waitFor(() => expect(posted).toHaveLength(3));
    expect(posted.map((call) => call.body.action)).toEqual([
      'pass_turn',
      'next_round',
      'adjust_score',
    ]);
    expect(posted[2].body).toEqual({ action: 'adjust_score', teamId: 'team_1', delta: 50 });
  });

  it('starts the timer from a fixed preset and offers no duration input', async () => {
    stub(orderState());
    const user = userEvent.setup();
    render(<Session sessionId="lq-abcd1234" role="gamemaster" onNavigate={() => {}} />);
    await user.type(screen.getByPlaceholderText('Enter password...'), 'letmein');
    await user.click(screen.getByRole('button', { name: /unlock/i }));

    await screen.findByRole('button', { name: '15s' });
    expect(screen.getByRole('button', { name: '30s' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '60s' })).toBeInTheDocument();
    expect(screen.queryByRole('spinbutton')).not.toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: '30s' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0].body).toEqual({ action: 'start_timer', seconds: 30 });
  });

  it('disables the turn option of a team with no hearts (US1)', async () => {
    const withOutTeam = {
      ...orderState(),
      teams: [
        {
          id: 'team_1',
          name: 'Team Cyan Pulsar',
          color: 'cyan',
          primaryHex: '#06b6d4',
          accentGlow: 'rgba(6, 182, 212, 0.45)',
          score: 0,
          hearts: 3,
          onTurn: true,
        },
        {
          id: 'team_2',
          name: 'Team Ruby Spark',
          color: 'rose',
          primaryHex: '#f43f5e',
          accentGlow: 'rgba(244, 63, 94, 0.45)',
          score: 0,
          hearts: 0,
          onTurn: false,
        },
      ],
    };
    stub(withOutTeam);
    const user = userEvent.setup();
    render(<Session sessionId="lq-abcd1234" role="gamemaster" onNavigate={() => {}} />);
    await user.type(screen.getByPlaceholderText('Enter password...'), 'letmein');
    await user.click(screen.getByRole('button', { name: /unlock/i }));

    const turn = await screen.findByRole('combobox', { name: /turn/i });
    expect(within(turn).getByRole('option', { name: 'Team Ruby Spark' })).toBeDisabled();
    expect(within(turn).getByRole('option', { name: 'Team Cyan Pulsar' })).toBeEnabled();
  });

  it('shows the final ranking once the game is finished', async () => {
    stub({ ...orderState(), status: 'finished', revealValues: true });

    render(<Session sessionId="lq-abcd1234" role="view" onNavigate={() => {}} />);

    expect(await screen.findByText(/final ranking/i)).toBeInTheDocument();
    expect(screen.getAllByText(/Team Cyan Pulsar/).length).toBeGreaterThan(0);
  });
});

describe('round start affordance', () => {
  const NOT_STARTED = { ...SESSION_STATE, status: 'setup', currentRound: 0, order: null, tripmine: null };

  /** A Connect round is active: the pre-round prompt must behave as for Order/Tripmine. */
  function connectState() {
    return {
      ...SESSION_STATE,
      order: null,
      tripmine: null,
      connect: {
        names: [
          { itemId: 'a', name: 'Alpha', iconName: 'BookOpen' },
          { itemId: 'b', name: 'Bravo', iconName: null },
        ],
        values: [
          { itemId: 'b', display: '200' },
          { itemId: 'a', display: '100' },
        ],
        matched: [],
        pending: { nameItemId: null, valueItemId: null },
      },
    };
  }

  function stub(state: unknown, onAction?: (body: Record<string, unknown>) => void) {
    vi.stubGlobal('EventSource', FakeEventSource as unknown as typeof EventSource);
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
        const url = String(input);
        if (url.includes('/unlock')) return jsonResponse({ ok: true });
        if (url.includes('/actions')) {
          onAction?.(JSON.parse(String(init?.body ?? '{}')));
          return jsonResponse({ ok: true });
        }
        if (url.includes('/state')) return jsonResponse(state);
        return jsonResponse({ error: { code: 'bad_request', message: 'unexpected' } }, 400);
      }),
    );
  }

  beforeEach(() => {
    FakeEventSource.instances = [];
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('gives the gamemaster a start control that starts the first round', async () => {
    let action: Record<string, unknown> | null = null;
    stub(NOT_STARTED, (body) => {
      action = body;
    });
    const user = userEvent.setup();
    render(<Session sessionId="lq-abcd1234" role="gamemaster" onNavigate={() => {}} />);
    await user.type(screen.getByPlaceholderText('Enter password...'), 'letmein');
    await user.click(screen.getByRole('button', { name: /unlock/i }));

    const start = await screen.findByRole('button', { name: /start round/i });
    await user.click(start);

    await waitFor(() => expect(action).not.toBeNull());
    expect(action).toEqual({ action: 'next_round' });
  });

  it('shows players and spectators a waiting message before the round starts', async () => {
    stub(NOT_STARTED);
    render(<Session sessionId="lq-abcd1234" role="p1" onNavigate={() => {}} />);

    expect(await screen.findByText(/waiting for the game to be started/i)).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /start round/i })).not.toBeInTheDocument();
  });

  it('hides the start control and the waiting message once a board exists', async () => {
    stub(orderState());
    const user = userEvent.setup();
    render(<Session sessionId="lq-abcd1234" role="gamemaster" onNavigate={() => {}} />);
    await user.type(screen.getByPlaceholderText('Enter password...'), 'letmein');
    await user.click(screen.getByRole('button', { name: /unlock/i }));

    await screen.findByRole('heading', { name: 'Team Cyan Pulsar' });
    expect(screen.queryByRole('button', { name: /start round/i })).not.toBeInTheDocument();
    expect(screen.queryByText(/waiting for the game to be started/i)).not.toBeInTheDocument();
  });

  it('hides the start control and the waiting message once a Connect board exists (US1)', async () => {
    stub(connectState());
    const user = userEvent.setup();
    render(<Session sessionId="lq-abcd1234" role="gamemaster" onNavigate={() => {}} />);
    await user.type(screen.getByPlaceholderText('Enter password...'), 'letmein');
    await user.click(screen.getByRole('button', { name: /unlock/i }));

    // The Connect board renders once the round is active.
    expect(await screen.findByText('Alpha')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /start round/i })).not.toBeInTheDocument();
    expect(screen.queryByText(/waiting for the game to be started/i)).not.toBeInTheDocument();
  });

  it('does not show a player the waiting message once a Connect board exists (US1)', async () => {
    stub(connectState());
    render(<Session sessionId="lq-abcd1234" role="p1" onNavigate={() => {}} />);

    expect(await screen.findByText('Alpha')).toBeInTheDocument();
    expect(screen.queryByText(/waiting for the game to be started/i)).not.toBeInTheDocument();
  });

  it('shows the pre-round prompt only while no game type is active (US2)', async () => {
    for (const state of [orderState(), tripmineState(), connectState()]) {
      stub(state);
      const { unmount } = render(<Session sessionId="lq-abcd1234" role="p1" onNavigate={() => {}} />);
      expect(await screen.findByText('Alpha')).toBeInTheDocument();
      expect(screen.queryByText(/waiting for the game to be started/i)).not.toBeInTheDocument();
      unmount();
    }

    stub(NOT_STARTED);
    render(<Session sessionId="lq-abcd1234" role="p1" onNavigate={() => {}} />);
    expect(await screen.findByText(/waiting for the game to be started/i)).toBeInTheDocument();
  });

  it('shows the Connect answer key only to the gamemaster (US2)', async () => {
    stub(connectState());
    const user = userEvent.setup();
    const gm = render(<Session sessionId="lq-abcd1234" role="gamemaster" onNavigate={() => {}} />);
    await user.type(screen.getByPlaceholderText('Enter password...'), 'letmein');
    await user.click(screen.getByRole('button', { name: /unlock/i }));

    // Alpha's value is 100; the gamemaster sees it on the name row.
    const gmAlpha = await screen.findByRole('button', { name: /alpha/i });
    expect(within(gmAlpha).getByText('100')).toBeInTheDocument();
    gm.unmount();

    stub(connectState());
    const player = render(<Session sessionId="lq-abcd1234" role="p1" onNavigate={() => {}} />);
    const playerAlpha = await screen.findByRole('button', { name: /alpha/i });
    expect(within(playerAlpha).queryByText('100')).not.toBeInTheDocument();
    player.unmount();

    stub(connectState());
    const spectator = render(<Session sessionId="lq-abcd1234" role="view" onNavigate={() => {}} />);
    const viewAlpha = await screen.findByRole('button', { name: /alpha/i });
    expect(within(viewAlpha).queryByText('100')).not.toBeInTheDocument();
    spectator.unmount();
  });
});

describe('turn timer (005 US2)', () => {
  beforeEach(() => {
    FakeEventSource.instances = [];
    vi.stubGlobal('EventSource', FakeEventSource as unknown as typeof EventSource);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  function stubWithTimer(timer: { totalSeconds: number; secondsLeft: number }) {
    const posted: Record<string, unknown>[] = [];
    const withTimer = { ...SESSION_STATE, timer };
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
        const url = String(input);
        if (url.includes('/actions')) {
          posted.push(JSON.parse(String(init?.body ?? '{}')));
          return jsonResponse({ ok: true });
        }
        if (url.includes('/state')) return jsonResponse(withTimer);
        return jsonResponse({ error: { code: 'bad_request', message: 'unexpected' } }, 400);
      }),
    );
    return posted;
  }

  it('shows a countdown for a running timer', async () => {
    stubWithTimer({ totalSeconds: 20, secondsLeft: 20 });
    render(<Session sessionId="lq-abcd1234" role="view" onNavigate={() => {}} />);

    expect(await screen.findByLabelText('Turn timer')).toHaveTextContent(/\d+s/);
  });

  it('posts nothing and shows no error bar when the countdown reaches zero', async () => {
    const posted = stubWithTimer({ totalSeconds: 20, secondsLeft: 0 });
    render(<Session sessionId="lq-abcd1234" role="p1" onNavigate={() => {}} />);

    await screen.findByRole('heading', { name: 'Team Cyan Pulsar' });
    await waitFor(() => expect(posted).toHaveLength(0));
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    expect(screen.queryByText(/has not run out yet/i)).not.toBeInTheDocument();
  });
});
