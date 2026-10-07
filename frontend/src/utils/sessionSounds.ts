// Cue derivation for a running session (feature 004, US1).
//
// The stream carries only a revision and every value comes from a fresh state
// read, so cues are derived by comparing the state the room already rendered with
// the one that just arrived. A cue is emitted only for a genuinely consecutive,
// live update (`next.revision === prev.revision + 1`); the first load and any
// reconnect or heartbeat jump emit nothing, which is what keeps a reconnect from
// replaying a scoring sound. Only public data is read, so every role derives the
// same cue.

import type { SessionState } from '../types/session';

export type SessionCue = 'slide' | 'chime' | 'failure' | 'explosion' | 'remove' | 'bonus' | 'buzzer';

function barCount(state: SessionState): number {
  return (state.order?.left.length ?? 0) + (state.order?.right.length ?? 0);
}

function poolCount(state: SessionState): number {
  return state.order?.pool.length ?? 0;
}

function scoreRose(prev: SessionState, next: SessionState): boolean {
  return next.teams.some((team) => {
    const before = prev.teams.find((candidate) => candidate.id === team.id);
    return before !== undefined && team.score > before.score;
  });
}

function heartsDropped(prev: SessionState, next: SessionState): boolean {
  return next.teams.some((team) => {
    const before = prev.teams.find((candidate) => candidate.id === team.id);
    return before !== undefined && team.hearts < before.hearts;
  });
}

/**
 * The Tripmine item picked in this update, or null. A pick is the only thing that
 * sets `pointsAwarded` on an item, and the view only sets it (for a player) once
 * the item is revealed, so this reads the same for the gamemaster and a player.
 */
function newlyRevealedTripmineItem(
  prev: SessionState,
  next: SessionState,
): { isTripmine: boolean } | null {
  for (const item of next.tripmine?.items ?? []) {
    if (item.pointsAwarded === null) continue;
    const before = prev.tripmine?.items.find((candidate) => candidate.itemId === item.itemId);
    if (before?.pointsAwarded === null) {
      return { isTripmine: item.isTripmine };
    }
  }
  return null;
}

/**
 * One cue for the transition from `prev` to `next`, or null. The order of the
 * checks is the priority table in contracts/session-feedback.md §2.
 */
export function deriveSessionCue(
  prev: SessionState | null,
  next: SessionState,
): SessionCue | null {
  if (prev === null) return null;
  if (next.revision !== prev.revision + 1) return null;
  // Revealing values rearranges the Order view; it is not a game event.
  if (prev.revealValues !== next.revealValues) return null;

  if (prev.status !== 'finished' && next.status === 'finished') return 'bonus';

  // A new round resets the board; that is not a placement.
  if (
    prev.currentRound !== next.currentRound ||
    prev.round?.quizType !== next.round?.quizType
  ) {
    return null;
  }

  const reveal = newlyRevealedTripmineItem(prev, next);
  if (reveal) return reveal.isTripmine ? 'explosion' : 'chime';

  // The turn timer (feature 006): a timer appearing plays the start cue; a timer
  // clearing with the timeout message plays the timeout cue. This runs before the
  // heart-loss check so an Order timeout plays exactly one cue.
  if (!prev.timer && next.timer) return 'buzzer';
  if (
    prev.timer &&
    !next.timer &&
    (next.lastMessage ?? '').startsWith('Time ran out')
  ) {
    return 'failure';
  }

  if (heartsDropped(prev, next)) return 'failure';

  const barGrew = barCount(next) > barCount(prev);
  if (barGrew && scoreRose(prev, next)) return 'chime';

  const poolGrew = poolCount(next) > poolCount(prev);
  if (poolGrew && barCount(next) < barCount(prev)) return 'remove';

  if (barGrew) return 'slide';

  if (!prev.tripmine?.completed && next.tripmine?.completed) return 'chime';

  const pendingBefore = prev.order?.pending?.itemId ?? null;
  const pendingAfter = next.order?.pending?.itemId ?? null;
  if (pendingAfter !== null && pendingAfter !== pendingBefore) return 'slide';

  return null;
}
