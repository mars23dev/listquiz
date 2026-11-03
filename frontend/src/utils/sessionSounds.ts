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

/**
 * True on the last five seconds of a countdown (5, 4, 3, 2, 1). A shorter timer
 * ticks on each of its remaining seconds, since they are all within the last five.
 */
export function isFinalTickSecond(secondsLeft: number): boolean {
  return secondsLeft >= 1 && secondsLeft <= 5;
}

function barCount(state: SessionState): number {
  return (state.order?.left.length ?? 0) + (state.order?.right.length ?? 0);
}

function poolCount(state: SessionState): number {
  return state.order?.pool.length ?? 0;
}

function connectMatchedCount(state: SessionState): number {
  return state.connect?.matched.length ?? 0;
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
 * The ordered cues for the transition from `prev` to `next`. The event cue (the
 * priority table in contracts/session-feedback.md §2) plays first; if the same
 * transition also finishes the game, the game-end cue is appended last, so a round
 * end always precedes the game end (feature 009). An empty list means no cue.
 */
export function deriveSessionCues(
  prev: SessionState | null,
  next: SessionState,
): SessionCue[] {
  if (prev === null) return [];
  if (next.revision !== prev.revision + 1) return [];
  // Revealing values rearranges the Order view; it is not a game event.
  if (prev.revealValues !== next.revealValues) return [];

  const finished = prev.status !== 'finished' && next.status === 'finished';
  // A new round resets the board; that is not a placement.
  const roundChanged =
    prev.currentRound !== next.currentRound ||
    prev.round?.quizType !== next.round?.quizType;

  let event: SessionCue | null = null;
  if (!roundChanged) {
    const reveal = newlyRevealedTripmineItem(prev, next);
    if (reveal) {
      event = reveal.isTripmine ? 'explosion' : 'chime';
    } else if (!prev.timer && next.timer) {
      // The turn timer (feature 006): a timer appearing plays the start cue.
      event = 'buzzer';
    } else if (
      prev.timer &&
      !next.timer &&
      (next.lastMessage ?? '').startsWith('Time ran out')
    ) {
      // A timeout cue; before the heart-loss check so an Order timeout is one cue.
      event = 'failure';
    } else if (heartsDropped(prev, next)) {
      event = 'failure';
    } else if (
      connectMatchedCount(next) > connectMatchedCount(prev) &&
      scoreRose(prev, next)
    ) {
      // Connect: a correct pair (feature 010) is a success.
      event = 'chime';
    } else {
      const barGrew = barCount(next) > barCount(prev);
      const poolGrew = poolCount(next) > poolCount(prev);
      if (barGrew && scoreRose(prev, next)) {
        event = 'chime';
      } else if (poolGrew && barCount(next) < barCount(prev)) {
        event = 'remove';
      } else if (barGrew) {
        event = 'slide';
      } else if (!prev.tripmine?.completed && next.tripmine?.completed) {
        event = 'chime';
      } else {
        const pendingBefore = prev.order?.pending?.itemId ?? null;
        const pendingAfter = next.order?.pending?.itemId ?? null;
        if (pendingAfter !== null && pendingAfter !== pendingBefore) {
          event = 'slide';
        }
      }
    }
  }

  const cues: SessionCue[] = [];
  if (event) cues.push(event);
  if (finished) cues.push('bonus');
  return cues;
}
