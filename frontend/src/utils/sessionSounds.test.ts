import { describe, expect, it } from 'vitest';
import { deriveSessionCue } from './sessionSounds';
import type { OrderItem, PendingPlacement, SessionState, TripmineItem } from '../types/session';

function item(itemId: string): OrderItem {
  return { itemId, name: itemId, value: null, revealed: false, placedBy: null };
}

function pending(itemId: string): PendingPlacement {
  return { itemId, name: itemId, side: 'left', slotIndex: 0, teamId: 'team_1' };
}

function tripmineItem(overrides: Partial<TripmineItem> = {}): TripmineItem {
  return {
    itemId: 'mine',
    name: 'Mine',
    value: null,
    revealed: false,
    trueRank: null,
    isTripmine: false,
    pointsAwarded: null,
    pickedBy: null,
    ...overrides,
  };
}

function state(overrides: Partial<SessionState> = {}): SessionState {
  return {
    revision: 1,
    sessionId: 'lq-abcd1234',
    status: 'running',
    teamCount: 2,
    roundCount: 3,
    currentRound: 1,
    muted: false,
    revealValues: false,
    lastMessage: null,
    timer: null,
    round: {
      roundNumber: 1,
      quizType: 'ORDER',
      tripmineSide: null,
      listTitle: 'List',
      listId: 'list_1',
      columnId: 'col_1',
      columnName: 'Column',
    },
    teams: [
      { id: 'team_1', name: 'One', color: 'cyan', primaryHex: '#06b6d4', accentGlow: '', score: 0, hearts: 3, onTurn: true },
      { id: 'team_2', name: 'Two', color: 'rose', primaryHex: '#f43f5e', accentGlow: '', score: 0, hearts: 3, onTurn: false },
    ],
    order: { anchor: { itemId: 'a', name: 'A', value: '5' }, pool: [], left: [], right: [], pending: null },
    tripmine: null,
    links: { gamemaster: '', players: [], spectator: '' },
    ...overrides,
  };
}

describe('deriveSessionCue', () => {
  it('plays nothing on the first load or a non-consecutive revision', () => {
    const first = state({ revision: 1 });
    expect(deriveSessionCue(null, first)).toBeNull();
    expect(deriveSessionCue(first, state({ revision: 3 }))).toBeNull();
  });

  it('plays nothing when only the reveal flag or mute toggles', () => {
    const base = state({ revision: 1 });
    expect(deriveSessionCue(base, state({ revision: 2, revealValues: true }))).toBeNull();
    expect(deriveSessionCue(base, state({ revision: 2, muted: true }))).toBeNull();
  });

  it('plays nothing for a new round or a turn-only change', () => {
    const base = state({ revision: 1 });
    expect(deriveSessionCue(base, state({ revision: 2, currentRound: 2 }))).toBeNull();
    const turned = state({
      revision: 2,
      teams: [
        { id: 'team_1', name: 'One', color: 'cyan', primaryHex: '#06b6d4', accentGlow: '', score: 0, hearts: 3, onTurn: false },
        { id: 'team_2', name: 'Two', color: 'rose', primaryHex: '#f43f5e', accentGlow: '', score: 0, hearts: 3, onTurn: true },
      ],
    });
    expect(deriveSessionCue(base, turned)).toBeNull();
  });

  it('plays a slide when an item is selected', () => {
    const base = state({
      revision: 1,
      order: { anchor: { itemId: 'a', name: 'A', value: '5' }, pool: [item('x')], left: [], right: [], pending: null },
    });
    const selected = state({
      revision: 2,
      order: { anchor: { itemId: 'a', name: 'A', value: '5' }, pool: [item('x')], left: [], right: [], pending: pending('x') },
    });
    expect(deriveSessionCue(base, selected)).toBe('slide');
  });

  it('plays a chime for a correct placement', () => {
    const base = state({ revision: 1 });
    const placed = state({
      revision: 2,
      teams: [
        { id: 'team_1', name: 'One', color: 'cyan', primaryHex: '#06b6d4', accentGlow: '', score: 10, hearts: 3, onTurn: false },
        { id: 'team_2', name: 'Two', color: 'rose', primaryHex: '#f43f5e', accentGlow: '', score: 0, hearts: 3, onTurn: true },
      ],
      order: { anchor: { itemId: 'a', name: 'A', value: '5' }, pool: [], left: [item('x')], right: [], pending: null },
    });
    expect(deriveSessionCue(base, placed)).toBe('chime');
  });

  it('plays a failure cue when a heart is lost', () => {
    const base = state({ revision: 1 });
    const wrong = state({
      revision: 2,
      teams: [
        { id: 'team_1', name: 'One', color: 'cyan', primaryHex: '#06b6d4', accentGlow: '', score: 0, hearts: 2, onTurn: false },
        { id: 'team_2', name: 'Two', color: 'rose', primaryHex: '#f43f5e', accentGlow: '', score: 0, hearts: 3, onTurn: true },
      ],
    });
    expect(deriveSessionCue(base, wrong)).toBe('failure');
  });

  it('plays an explosion when the mine is picked', () => {
    const base = state({
      revision: 1,
      tripmine: { side: 'highest', searchedRank: 1, hit: false, completed: false, targetValue: null, items: [tripmineItem({ itemId: 'mine', isTripmine: true })] },
    });
    const hit = state({
      revision: 2,
      tripmine: { side: 'highest', searchedRank: 1, hit: true, completed: true, targetValue: null, items: [tripmineItem({ itemId: 'mine', isTripmine: true, revealed: true, pointsAwarded: -10 })] },
    });
    expect(deriveSessionCue(base, hit)).toBe('explosion');
  });

  it('plays a chime for a safe Tripmine reveal', () => {
    const base = state({
      revision: 1,
      tripmine: { side: 'highest', searchedRank: 1, hit: false, completed: false, targetValue: null, items: [tripmineItem({ itemId: 'safe' })] },
    });
    const safe = state({
      revision: 2,
      tripmine: { side: 'highest', searchedRank: 1, hit: false, completed: false, targetValue: null, items: [tripmineItem({ itemId: 'safe', revealed: true, pointsAwarded: 5 })] },
    });
    expect(deriveSessionCue(base, safe)).toBe('chime');
  });

  it('plays a chime when the Tripmine round completes without a new reveal', () => {
    const base = state({
      revision: 1,
      tripmine: { side: 'highest', searchedRank: 1, hit: false, completed: false, targetValue: null, items: [tripmineItem({ itemId: 'x', revealed: true, pointsAwarded: 5 })] },
    });
    const done = state({
      revision: 2,
      tripmine: { side: 'highest', searchedRank: 1, hit: false, completed: true, targetValue: null, items: [tripmineItem({ itemId: 'x', revealed: true, pointsAwarded: 5 })] },
    });
    expect(deriveSessionCue(base, done)).toBe('chime');
  });

  it('plays a remove cue when an item returns to the pool', () => {
    const base = state({
      revision: 1,
      order: { anchor: { itemId: 'a', name: 'A', value: '5' }, pool: [], left: [item('x')], right: [], pending: null },
    });
    const returned = state({
      revision: 2,
      order: { anchor: { itemId: 'a', name: 'A', value: '5' }, pool: [item('x')], left: [], right: [], pending: null },
    });
    expect(deriveSessionCue(base, returned)).toBe('remove');
  });

  it('plays a bonus when the session finishes', () => {
    const base = state({ revision: 1 });
    const finished = state({ revision: 2, status: 'finished' });
    expect(deriveSessionCue(base, finished)).toBe('bonus');
  });

  it('plays a start cue when a timer appears', () => {
    const base = state({ revision: 1, timer: null });
    const started = state({ revision: 2, timer: { totalSeconds: 20, secondsLeft: 20 } });
    expect(deriveSessionCue(base, started)).toBe('buzzer');
  });

  it('plays a timeout cue when the timer clears with the timeout message', () => {
    const base = state({ revision: 1, timer: { totalSeconds: 20, secondsLeft: 0 } });
    const timedOut = state({
      revision: 2,
      timer: null,
      lastMessage: 'Time ran out! -1 heart',
    });
    expect(deriveSessionCue(base, timedOut)).toBe('failure');
  });

  it('plays nothing when a timer clears for another reason', () => {
    const base = state({ revision: 1, timer: { totalSeconds: 20, secondsLeft: 10 } });
    const passed = state({ revision: 2, timer: null, lastMessage: null });
    expect(deriveSessionCue(base, passed)).toBeNull();
  });

  it('plays one cue for an Order timeout that also drops a heart', () => {
    const base = state({
      revision: 1,
      timer: { totalSeconds: 20, secondsLeft: 0 },
      teams: [
        { id: 'team_1', name: 'One', color: 'cyan', primaryHex: '#06b6d4', accentGlow: '', score: 0, hearts: 3, onTurn: true },
        { id: 'team_2', name: 'Two', color: 'rose', primaryHex: '#f43f5e', accentGlow: '', score: 0, hearts: 3, onTurn: false },
      ],
    });
    const timedOut = state({
      revision: 2,
      timer: null,
      lastMessage: 'Time ran out! -1 heart',
      teams: [
        { id: 'team_1', name: 'One', color: 'cyan', primaryHex: '#06b6d4', accentGlow: '', score: 0, hearts: 2, onTurn: false },
        { id: 'team_2', name: 'Two', color: 'rose', primaryHex: '#f43f5e', accentGlow: '', score: 0, hearts: 3, onTurn: true },
      ],
    });
    expect(deriveSessionCue(base, timedOut)).toBe('failure');
  });
});
