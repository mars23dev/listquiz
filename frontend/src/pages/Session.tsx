import { useEffect, useMemo, useRef, useState } from 'react';
import { ArrowLeft, Layers, ShieldCheck, Tv, User } from 'lucide-react';
import { ApiError, deleteSession, postSessionAction, unlockSession } from '../api/client';
import { useSessionStream } from '../hooks/useSessionStream';
import { BottomTeams } from '../components/BottomTeams';
import { ConnectBoard } from '../components/ConnectBoard';
import { GameEndDialog } from '../components/GameEndDialog';
import { MiddleItemGrid } from '../components/MiddleItemGrid';
import { StageLighting } from '../components/StageLighting';
import { TopBarOrdering } from '../components/TopBarOrdering';
import { TripmineBoard } from '../components/TripmineBoard';
import {
  playBonusChime,
  playBuzzerSound,
  playChimeSound,
  playExplosionSound,
  playFailureSound,
  playRemoveSound,
  playSlideSound,
  playSnapSound,
  setSoundMuted,
  unlockAudio,
} from '../utils/audio';
import { positionForGap } from '../utils/orderBar';
import { deriveSessionCues, isFinalTickSecond } from '../utils/sessionSounds';
import type {
  QuizItem,
  Team as QuizTeam,
  TripmineItem as QuizTripmineItem,
  TripmineState as QuizTripmineState,
  UserRole,
} from '../types/quiz';
import type { OrderItem, SessionState, TeamView, TripmineView } from '../types/session';

interface SessionProps {
  sessionId: string;
  role: UserRole;
  onNavigate: (path: string) => void;
}

const WRONG_PASSWORD_COPY = 'Incorrect password!';

const TEAM_COLORS: QuizTeam['colorName'][] = ['cyan', 'rose', 'amber', 'emerald'];

/** The server's `color` name, clamped to the draft palette the team card understands. */
function toQuizTeam(team: TeamView): QuizTeam {
  const colorName = TEAM_COLORS.includes(team.color as QuizTeam['colorName'])
    ? (team.color as QuizTeam['colorName'])
    : 'cyan';
  return {
    id: team.id,
    name: team.name,
    colorName,
    primaryHex: team.primaryHex,
    accentGlow: team.accentGlow,
    score: team.score,
    hearts: team.hearts,
    isActiveTurn: team.onTurn,
    turnIndex: 0,
  };
}

/** The path segment for a role, matching the backend's link paths (FR-012). */
function rolePath(role: UserRole): string {
  return role === 'gamemaster' ? 'gamemaster' : role === 'view' ? 'view' : role;
}

/**
 * The board components predate the server-shaped state and take `QuizItem`, while
 * the API serves `OrderItem`. Values are hidden from players until the reveal
 * (FR-027), so a null value must survive the trip as an empty string rather than
 * the text "null". The server owns ordering, so `numericValue` is never read here.
 */
function toQuizItem(item: OrderItem): QuizItem {
  return {
    id: item.itemId,
    name: item.name,
    value: item.value ?? '',
    numericValue: 0,
    placedByTeamId: item.placedBy,
  };
}

interface ChosenSlot {
  itemId: string;
  side: 'left' | 'right';
  /** The bar column the player dropped on; mapped to a gap when confirming. */
  column: number;
}

/**
 * The server reports `revealed: true` for the gamemaster on every Tripmine item so
 * the answer key shows, but the board's "is this position picked" must follow the
 * points, which are only set once an item is really revealed.
 */
function toTripmineItem(
  item: TripmineView['items'][number],
  isGamemaster: boolean,
): QuizTripmineItem {
  const picked = isGamemaster ? item.pointsAwarded !== null : item.revealed;
  return {
    id: item.itemId,
    name: item.name,
    value: item.value ?? '',
    numericValue: 0,
    trueRank: item.trueRank ?? 0,
    isTripmine: item.isTripmine,
    isRevealed: picked,
    pointsAwarded: item.pointsAwarded ?? undefined,
    revealedByTeamId: item.pickedBy ?? undefined,
  };
}

function toTripmineState(view: TripmineView, isGamemaster: boolean): QuizTripmineState {
  return {
    tripmineSide: view.side === 'lowest' ? 'lowest' : view.side === 'random' ? 'random' : 'highest',
    currentSearchedRank: view.searchedRank,
    items: view.items.map((item) => toTripmineItem(item, isGamemaster)),
    tripmineHit: view.hit,
    isCompleted: view.completed,
    targetValue: view.targetValue,
    mineRank: view.mineRank,
  };
}

export function Session({ sessionId, role, onNavigate }: SessionProps) {
  const [password, setPassword] = useState('');
  const [unlocked, setUnlocked] = useState(false);
  const [unlocking, setUnlocking] = useState(false);
  const [passwordError, setPasswordError] = useState<string | null>(null);
  const needsPassword = role === 'gamemaster' && !unlocked;
  const {
    state,
    status,
    error: streamError,
    errorCode,
    refresh,
  } = useSessionStream(sessionId, role);

  useEffect(() => {
    if (needsPassword) return;
    void refresh();
  }, [needsPassword, refresh]);

  // FR-024b: the session's mute silences audio in every view, not just the host's.
  useEffect(() => {
    setSoundMuted(state?.muted ?? false);
  }, [state?.muted]);

  // US1: unlock audio on the first interaction (pointer or keyboard) so the
  // server-driven cues are audible; a suspended context produces no sound.
  useEffect(() => {
    const unlock = () => unlockAudio();
    window.addEventListener('pointerdown', unlock, { once: true });
    window.addEventListener('keydown', unlock, { once: true });
    return () => {
      window.removeEventListener('pointerdown', unlock);
      window.removeEventListener('keydown', unlock);
    };
  }, []);

  // US1: cues for each reflected event, derived by comparing the previous state with
  // the new one. A round end that also finishes the game plays its cue then the
  // game-end cue (feature 009). The first load and any reconnect emit nothing.
  const previousStateRef = useRef<SessionState | null>(null);
  useEffect(() => {
    if (!state) return;
    const cues = deriveSessionCues(previousStateRef.current, state);
    previousStateRef.current = state;
    for (const cue of cues) {
      switch (cue) {
        case 'slide':
          playSlideSound();
          break;
        case 'chime':
          playChimeSound();
          break;
        case 'failure':
          playFailureSound();
          break;
        case 'explosion':
          playExplosionSound();
          break;
        case 'remove':
          playRemoveSound();
          break;
        case 'bonus':
          playBonusChime();
          break;
        case 'buzzer':
          playBuzzerSound();
          break;
        default:
          break;
      }
    }
  }, [state]);

  const order = state?.order ?? null;
  const tripmine = state?.tripmine ?? null;
  const connect = state?.connect ?? null;
  const isGamemaster = role === 'gamemaster';
  const onTurnTeamId = useMemo(
    () => state?.teams.find((team) => team.onTurn)?.id ?? null,
    [state?.teams],
  );
  const activeTeamName = useMemo(
    () => state?.teams.find((team) => team.onTurn)?.name ?? null,
    [state?.teams],
  );
  const playerSlot = /^p([1-4])$/.exec(role);
  const playerTeam = playerSlot
    ? state?.teams[Number(playerSlot[1]) - 1]
    : undefined;
  const isPlayersTurn = playerTeam?.onTurn === true;
  // A board is interactive for the gamemaster or for the player role that owns the
  // team on turn (FR-001, FR-002). Every other view is read-only.
  const canAct = isGamemaster || isPlayersTurn;
  const teams = useMemo(() => (state?.teams ?? []).map(toQuizTeam), [state?.teams]);
  // Hearts are Order-specific; a Tripmine round hides them (feature 009).
  const showHearts = state?.round?.quizType !== 'TRIPMINE';

  // US2 (005) / US3 (008): the turn timer. The server owns the deadline and applies
  // the timeout itself, so the client only displays the countdown and ticks the last
  // five seconds; it never posts anything at zero.
  const timer = state?.timer ?? null;
  const [secondsLeft, setSecondsLeft] = useState(0);
  useEffect(() => {
    if (!timer) {
      setSecondsLeft(0);
      return;
    }
    setSecondsLeft(timer.secondsLeft);
    const id = window.setInterval(() => {
      setSecondsLeft((current) => (current > 0 ? current - 1 : 0));
    }, 1000);
    return () => window.clearInterval(id);
  }, [timer?.totalSeconds, timer?.secondsLeft, state?.revision, timer]);

  // A short tick on each of the last five seconds, once per second.
  const lastTickedRef = useRef<number | null>(null);
  useEffect(() => {
    if (!timer || !isFinalTickSecond(secondsLeft)) {
      lastTickedRef.current = null;
      return;
    }
    if (lastTickedRef.current === secondsLeft) return;
    lastTickedRef.current = secondsLeft;
    playSnapSound();
  }, [secondsLeft, timer]);

  // The slot a pending item is being dropped on is a client-side step: the server
  // only learns the slot once the placement is confirmed, so CONFIRM is a real
  // second step rather than a formality.
  const [chosenSlot, setChosenSlot] = useState<ChosenSlot | null>(null);
  const [draggingItem, setDraggingItem] = useState<QuizItem | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [tripmineFeedback, setTripmineFeedback] = useState<string | null>(null);

  const poolItems = useMemo(
    () => (order?.pool ?? []).map(toQuizItem),
    [order?.pool],
  );
  const leftItems = useMemo(() => (order?.left ?? []).map(toQuizItem), [order?.left]);
  const rightItems = useMemo(() => (order?.right ?? []).map(toQuizItem), [order?.right]);
  const anchorItem = useMemo(() => {
    if (!order) return null;
    return {
      id: order.anchor.itemId,
      name: order.anchor.name,
      value: order.anchor.value,
      numericValue: 0,
    } satisfies QuizItem;
  }, [order]);

  const pendingItem = useMemo(() => {
    const pending = order?.pending;
    if (!pending || !chosenSlot || chosenSlot.itemId !== pending.itemId) return null;
    return {
      item: { id: pending.itemId, name: pending.name, value: '', numericValue: 0 },
      side: chosenSlot.side,
      slotIndex: chosenSlot.column,
    };
  }, [order?.pending, chosenSlot]);

  const placedIds = useMemo(
    () => new Set([...leftItems, ...rightItems].map((item) => item.id)),
    [leftItems, rightItems],
  );

  const revealAllValues = isGamemaster || state?.revealValues === true;

  async function sendAction(action: Record<string, unknown>) {
    setActionError(null);
    // A player action names its role; the gamemaster action carries the password.
    const body = isGamemaster ? action : { ...action, role };
    try {
      await postSessionAction(sessionId, body, isGamemaster ? password : undefined);
      await refresh();
    } catch (cause) {
      setActionError(
        cause instanceof ApiError ? cause.message : 'The action could not be sent.',
      );
    }
  }

  function handleSelectItem(item: QuizItem) {
    if (!onTurnTeamId) return;
    setChosenSlot(null);
    void sendAction({ action: 'select_item', itemId: item.id, teamId: onTurnTeamId });
  }

  function handleDropToPending(item: QuizItem, side: 'left' | 'right', column: number) {
    setChosenSlot({ itemId: item.id, side, column });
    // Dragging a pool item straight onto a slot also has to select it first.
    if (order?.pending?.itemId !== item.id && onTurnTeamId) {
      void sendAction({ action: 'select_item', itemId: item.id, teamId: onTurnTeamId });
    }
  }

  function handleChooseSlot(side: 'left' | 'right', column: number) {
    const pending = order?.pending;
    if (!pending) return;
    setChosenSlot({ itemId: pending.itemId, side, column });
  }

  function handleConfirmPlacement() {
    const pending = order?.pending;
    if (!pending || !chosenSlot || !onTurnTeamId) return;
    // The dropped column is mapped to a gap on the contiguous side, so the server
    // receives a position the side actually has (contracts/order-placement.md).
    const sideLength = chosenSlot.side === 'left' ? leftItems.length : rightItems.length;
    const gap = positionForGap(chosenSlot.side, chosenSlot.column, sideLength);
    void sendAction({
      action: 'choose_slot',
      itemId: pending.itemId,
      side: chosenSlot.side,
      slotIndex: gap,
      teamId: onTurnTeamId,
    });
  }

  function handleCancelPlacement() {
    const pending = order?.pending;
    if (!pending || !onTurnTeamId) return;
    setChosenSlot(null);
    void sendAction({
      action: 'cancel_selection',
      itemId: pending.itemId,
      teamId: onTurnTeamId,
    });
  }

  function handleReturnToPool(itemId: string) {
    const pending = order?.pending;
    if (pending?.itemId === itemId) {
      void sendAction({
        action: 'cancel_selection',
        itemId,
        teamId: pending.teamId,
      });
      return;
    }
    if (!onTurnTeamId) return;
    void sendAction({ action: 'return_to_pool', itemId, teamId: onTurnTeamId });
  }

  async function handleRevealValues() {
    await sendAction({ action: 'reveal_values' });
  }

  // US2 (005): the gamemaster's turn-timer controls; US3 (006): fixed presets.
  function handleStartTimer(seconds: number) {
    void sendAction({ action: 'start_timer', seconds });
  }
  function handleStopTimer() {
    void sendAction({ action: 'stop_timer' });
  }

  async function handleToggleMute() {
    await sendAction({ action: 'set_muted', muted: !(state?.muted ?? false) });
  }

  function handlePassTurn() {
    void sendAction({ action: 'pass_turn' });
  }

  function handleNextRound() {
    void sendAction({ action: 'next_round' });
  }

  // Connect (feature 010): stage a name and a value, then confirm the pair.
  function handlePickName(itemId: string) {
    if (!onTurnTeamId) return;
    void sendAction({ action: 'pick_name', itemId, teamId: onTurnTeamId });
  }

  function handlePickValue(itemId: string) {
    if (!onTurnTeamId) return;
    void sendAction({ action: 'pick_value', itemId, teamId: onTurnTeamId });
  }

  function handleConfirmConnect() {
    if (!onTurnTeamId) return;
    void sendAction({ action: 'confirm_connect', teamId: onTurnTeamId });
  }

  // US3/US4 (005): dismissing the end dialog deletes the finished session, then
  // returns home. The delete is idempotent, so a missing session is fine.
  async function handleEndGame() {
    try {
      await deleteSession(sessionId);
    } catch {
      // Already gone, or the delete is refused; either way leave the finished game.
    }
    onNavigate('/');
  }

  function handleSetTurn(teamId: string) {
    void sendAction({ action: 'set_turn', teamId });
  }

  function handleAdjustScore(delta: number) {
    if (!onTurnTeamId) return;
    void sendAction({ action: 'adjust_score', teamId: onTurnTeamId, delta });
  }

  function handleAdjustHearts(delta: number) {
    if (!onTurnTeamId) return;
    void sendAction({ action: 'adjust_hearts', teamId: onTurnTeamId, delta });
  }

  function handlePickItem(item: QuizTripmineItem) {
    if (!onTurnTeamId) return;
    const teamName = activeTeamName ?? 'The active team';
    const unrevealedBefore = tripmine?.items.filter((entry) => !entry.revealed).length ?? 0;
    // The draft's own feedback copy, chosen from the same four outcomes the server
    // scores (FR-040 to FR-043).
    if (item.isTripmine) {
      setTripmineFeedback(
        `💥 BOOM! ${teamName} hit the Tripmine! (-10 pts, other teams +10 pts)`,
      );
    } else if (unrevealedBefore === 2) {
      setTripmineFeedback(`🎯 PERFECT DEFUSE! ${teamName} found the final item! (+20 pts)`);
    } else if (item.trueRank === tripmine?.searchedRank) {
      setTripmineFeedback(`🎯 TARGET HIT! ${teamName} found Rank #${item.trueRank}! (+10 pts)`);
    } else {
      setTripmineFeedback(`Safe pick! ${teamName} revealed Rank #${item.trueRank} (+5 pts)`);
    }
    void sendAction({ action: 'pick_tripitem', itemId: item.id, teamId: onTurnTeamId });
  }

  async function submitUnlock(event: React.FormEvent) {
    event.preventDefault();
    setUnlocking(true);
    try {
      await unlockSession(sessionId, password);
      setUnlocked(true);
      setPasswordError(null);
    } catch (cause) {
      setPasswordError(
        cause instanceof ApiError && cause.isWrongPassword
          ? WRONG_PASSWORD_COPY
          : WRONG_PASSWORD_COPY,
      );
    } finally {
      setUnlocking(false);
    }
  }

  if (errorCode === 'not_found') {
    return (
      <div className="flex min-h-screen flex-col items-center justify-center bg-slate-950 p-4 text-white">
        <h2 className="mb-2 text-xl font-bold">Session Not Found</h2>
        <p className="mb-4 text-xs text-slate-400">
          The session ID "{sessionId}" does not exist.
        </p>
        <button
          type="button"
          onClick={() => onNavigate('/')}
          className="rounded-lg bg-cyan-500 px-4 py-2 text-xs font-bold text-slate-950"
        >
          Return to ListQuiz Home
        </button>
      </div>
    );
  }

  if (needsPassword) {
    return (
      <main className="relative z-10 mx-auto flex min-h-screen w-full max-w-md flex-col justify-center px-6 py-12">
        <form
          onSubmit={submitUnlock}
          className="rounded-2xl border border-slate-800 bg-slate-900/90 p-6 shadow-2xl sm:p-8"
        >
          <h1 className="font-display text-2xl font-bold text-white">Gamemaster Access</h1>
          <p className="mt-1 text-sm text-slate-400">
            Enter the moderator password to unlock host controls.
          </p>

          <label className="mt-6 block text-sm font-semibold text-slate-200" htmlFor="gm-unlock">
            Moderator password
          </label>
          <input
            id="gm-unlock"
            type="password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            placeholder="Enter password..."
            className="mt-2 w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-2 text-slate-100"
          />

          {passwordError ? (
            <p role="alert" className="mt-3 text-sm font-semibold text-rose-400">
              {passwordError}
            </p>
          ) : null}

          <div className="mt-6 flex gap-2">
            <button
              type="button"
              onClick={() => onNavigate(`/${sessionId}/view`)}
              className="flex-1 rounded-lg border border-slate-700 px-4 py-2.5 text-sm font-semibold text-slate-300 hover:border-cyan-400"
            >
              Switch to View
            </button>
            <button
              type="submit"
              disabled={unlocking || !password}
              className="flex-1 rounded-lg bg-cyan-500 px-4 py-2.5 text-sm font-bold text-slate-950 hover:bg-cyan-400 disabled:opacity-50"
            >
              Unlock
            </button>
          </div>
        </form>
      </main>
    );
  }

  return (
    <div className="relative flex min-h-screen flex-col justify-between overflow-x-hidden bg-slate-950 text-slate-100 selection:bg-cyan-500/30">
      <StageLighting />

      <nav className="relative z-20 w-full border-b border-slate-800 bg-slate-950/90 px-3 py-2 backdrop-blur-md sm:px-6">
        <div className="mx-auto flex max-w-7xl flex-col items-center justify-between gap-2 md:flex-row">
          <div className="flex items-center gap-3">
            <button
              type="button"
              onClick={() => onNavigate('/')}
              className="flex items-center gap-1 text-xs text-slate-400 hover:text-white"
            >
              <ArrowLeft className="h-3.5 w-3.5" />
              <span>ListQuiz</span>
            </button>
            <span className="text-slate-700">/</span>
            <span className="font-mono text-xs font-bold text-cyan-400">{sessionId}</span>
            <span className="text-slate-700">·</span>
            <span className="flex items-center gap-1 text-xs font-semibold text-amber-400">
              <Layers className="h-3 w-3" /> Round {state?.currentRound ?? 0} of{' '}
              {state?.roundCount ?? 0} ({state?.round?.quizType ?? ''})
            </span>
          </div>

          {isGamemaster ? (
            <div className="flex items-center gap-1 rounded-lg border border-slate-800 bg-slate-900 p-1 text-xs">
              <span className="hidden px-1.5 font-mono text-[10px] uppercase text-slate-500 lg:inline">
                Role:
              </span>
              {(['gamemaster', 'p1', 'p2', 'p3', 'p4', 'view'] as UserRole[]).map((target) => (
                <button
                  key={target}
                  type="button"
                  aria-pressed={role === target}
                  onClick={() =>
                    window.open(`/${sessionId}/${rolePath(target)}`, '_blank', 'noopener')
                  }
                  className={`rounded px-2 py-1 font-bold ${
                    role === target
                      ? 'bg-cyan-500 text-slate-950'
                      : 'text-slate-300 hover:bg-slate-800'
                  }`}
                >
                  {target === 'gamemaster' ? 'GM' : target === 'view' ? 'View' : target.toUpperCase()}
                </button>
              ))}
            </div>
          ) : null}

          <div className="flex items-center gap-2 text-xs">
            {isGamemaster ? (
              <span className="flex items-center gap-1 font-bold text-amber-300">
                <ShieldCheck className="h-3.5 w-3.5" /> Moderator Controls Active
              </span>
            ) : role === 'view' ? (
              <span className="flex items-center gap-1 font-semibold text-cyan-300">
                <Tv className="h-3.5 w-3.5" /> Stage Broadcast Screen
              </span>
            ) : (
              <span
                className={`flex items-center gap-1 font-bold ${
                  isPlayersTurn ? 'animate-pulse text-emerald-400' : 'text-slate-400'
                }`}
              >
                <User className="h-3.5 w-3.5" />
                {playerTeam?.name} {isPlayersTurn ? '(YOUR TURN!)' : '(Waiting...)'}
              </span>
            )}
          </div>
        </div>
      </nav>

      {isGamemaster && state ? (
        <div className="relative z-20 border-b border-amber-500/40 bg-amber-950/40 px-3 py-2 backdrop-blur-md sm:px-6">
          <div className="mx-auto flex max-w-7xl flex-wrap items-center justify-end gap-2 text-xs">
            <button
              type="button"
              onClick={handlePassTurn}
              disabled={state.status === 'finished'}
              className="rounded-lg border border-amber-500/40 bg-slate-900 px-3 py-2 font-semibold text-amber-300 hover:bg-slate-800 disabled:opacity-50"
            >
              Pass Turn
            </button>

            {state.teams.length > 0 ? (
              <label className="flex items-center gap-1 text-slate-300">
                Turn
                <select
                  value={onTurnTeamId ?? ''}
                  onChange={(event) => handleSetTurn(event.target.value)}
                  disabled={state.status === 'finished'}
                  className="rounded border border-slate-600 bg-slate-900 px-2 py-1 text-slate-200"
                >
                  {state.teams.map((team) => (
                    <option
                      key={team.id}
                      value={team.id}
                      disabled={showHearts && team.hearts <= 0}
                    >
                      {team.name}
                    </option>
                  ))}
                </select>
              </label>
            ) : null}

            <div className="flex items-center gap-1 rounded-lg border border-slate-700 bg-slate-900/90 px-2 py-1">
              <button
                type="button"
                onClick={() => handleAdjustScore(50)}
                disabled={!onTurnTeamId}
                className="px-1 text-emerald-300 disabled:opacity-40"
              >
                +50
              </button>
              <button
                type="button"
                onClick={() => handleAdjustScore(-50)}
                disabled={!onTurnTeamId}
                className="px-1 text-rose-300 disabled:opacity-40"
              >
                -50
              </button>
              {showHearts && (
                <>
                  <button
                    type="button"
                    onClick={() => handleAdjustHearts(1)}
                    disabled={!onTurnTeamId}
                    className="px-1 text-rose-300 disabled:opacity-40"
                  >
                    +1♥
                  </button>
                  <button
                    type="button"
                    onClick={() => handleAdjustHearts(-1)}
                    disabled={!onTurnTeamId}
                    className="px-1 text-slate-300 disabled:opacity-40"
                  >
                    -1♥
                  </button>
                </>
              )}
            </div>

            <div className="flex items-center gap-1 rounded-lg border border-slate-700 bg-slate-900/90 px-2 py-1">
              <span className="text-xs text-slate-400">Timer</span>
              {[15, 30, 60].map((seconds) => (
                <button
                  key={seconds}
                  type="button"
                  onClick={() => handleStartTimer(seconds)}
                  className="rounded px-2 py-0.5 text-xs font-bold text-cyan-300 hover:bg-cyan-500/10"
                >
                  {seconds}s
                </button>
              ))}
              <button
                type="button"
                onClick={handleStopTimer}
                disabled={!timer}
                className="px-1 text-slate-300 disabled:opacity-40"
              >
                Stop
              </button>
            </div>

            <button
              type="button"
              onClick={handleNextRound}
              disabled={state.status === 'finished'}
              className="rounded-lg bg-cyan-500 px-3 py-2 font-bold text-slate-950 hover:bg-cyan-400 disabled:opacity-50"
            >
              {state.currentRound >= state.roundCount ? 'Finish Game' : 'Next Round'}
            </button>

            <button
              type="button"
              onClick={() => void handleRevealValues()}
              disabled={state.revealValues}
              className="rounded-lg border border-amber-400/60 bg-amber-400/10 px-3 py-2 font-bold text-amber-200 hover:bg-amber-400/20 disabled:opacity-50"
            >
              {state.revealValues ? 'Values revealed' : 'Reveal values'}
            </button>
            <button
              type="button"
              onClick={() => void handleToggleMute()}
              className="rounded-lg border border-slate-600 bg-slate-900 px-3 py-2 font-semibold text-slate-200 hover:bg-slate-800"
            >
              {state.muted ? 'Unmute' : 'Mute'}
            </button>
          </div>
        </div>
      ) : null}

      <main className="relative z-10 flex flex-1 flex-col px-3 pb-2 sm:px-6">
        <p className="mx-auto w-full max-w-7xl pt-2 font-mono text-xs text-slate-500">
          revision {state?.revision ?? 0} / {status} / round {state?.currentRound ?? 0} of{' '}
          {state?.roundCount ?? 0}
        </p>

        {timer ? (
          <div
            role="status"
            aria-label="Turn timer"
            className="mx-auto mt-2 w-fit rounded-xl border border-amber-400/50 bg-amber-400/10 px-4 py-1 font-mono text-sm font-bold text-amber-200"
          >
            {secondsLeft}s
          </div>
        ) : null}

        {streamError ? (
          <p role="alert" className="mt-4 rounded-lg border border-rose-500/40 bg-rose-500/10 px-4 py-3 text-sm text-rose-200">
            {streamError}
          </p>
        ) : null}

        {actionError ? (
          <p role="alert" className="mt-4 rounded-lg border border-amber-500/40 bg-amber-500/10 px-4 py-3 text-sm text-amber-200">
            {actionError}
          </p>
        ) : null}

        {/* US2 (FR-006 to FR-008): before any round starts, the host gets an obvious
            start control and the room gets a waiting message in the board area. */}
        {state && !connect && !order && !tripmine && state.status !== 'finished' ? (
          <div className="flex flex-1 flex-col items-center justify-center gap-4 py-20 text-center">
            {isGamemaster ? (
              <button
                type="button"
                onClick={handleNextRound}
                className="rounded-2xl bg-cyan-500 px-12 py-7 text-2xl font-black uppercase tracking-widest text-slate-950 shadow-2xl shadow-cyan-500/30 hover:bg-cyan-400 focus:outline-none focus-visible:ring-4 focus-visible:ring-cyan-300"
              >
                Start Round
              </button>
            ) : (
              <p className="font-mono text-sm text-slate-400">
                Waiting for the game to be started…
              </p>
            )}
          </div>
        ) : null}

      {order && anchorItem ? (
        <section className="mt-6">
          {/* The draft's waiting banner, shown to players when it is not their turn. */}
          {playerSlot && !isPlayersTurn && order.pending === null && activeTeamName ? (
            <div className="text-center py-2 px-4 mx-auto my-1 rounded-xl bg-slate-900/80 border border-slate-800 text-xs text-slate-400 font-mono">
              Waiting for{' '}
              <span className="text-cyan-400 font-bold">{activeTeamName}</span> to place an
              item...
            </div>
          ) : null}

          {isGamemaster && order.pending && !chosenSlot ? (
            <p role="status" className="my-2 text-center text-sm font-semibold text-emerald-200">
              Selected {order.pending.name}. Choose a bar slot.
            </p>
          ) : null}

          <TopBarOrdering
            anchorItem={anchorItem}
            leftItems={leftItems}
            rightItems={rightItems}
            pendingItem={pendingItem}
            teams={teams}
            activeTeamId={onTurnTeamId}
            hasPendingItem={Boolean(order.pending)}
            interactive={canAct}
            onChooseSlot={handleChooseSlot}
            onDropToPending={handleDropToPending}
            onConfirmPlacement={handleConfirmPlacement}
            onCancelPlacement={handleCancelPlacement}
            draggingItem={draggingItem}
            onDragStartItem={setDraggingItem}
            onDragEndItem={() => setDraggingItem(null)}
            revealAllValues={revealAllValues}
            neutralBorders={state?.revealValues === true}
          />

          {isGamemaster && (leftItems.length > 0 || rightItems.length > 0) ? (
            <div
              aria-label="Placed item actions"
              className="mt-3 flex flex-wrap justify-end gap-2"
            >
              {[...leftItems, ...rightItems].map((item) => (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => handleReturnToPool(item.id)}
                  className="rounded-lg border border-slate-700 px-3 py-1.5 text-xs font-semibold text-slate-300 hover:border-cyan-400 hover:text-white focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-300"
                >
                  Return {item.name} to pool
                </button>
              ))}
            </div>
          ) : null}

          <MiddleItemGrid
            items={poolItems}
            placedIds={placedIds}
            onReturnToPool={handleReturnToPool}
            onDragStartItem={setDraggingItem}
            onDragEndItem={() => setDraggingItem(null)}
            revealAllValues={revealAllValues}
            onSelectItem={handleSelectItem}
            selectedItemId={order.pending?.itemId ?? null}
            interactive={canAct}
          />
        </section>
      ) : null}

      {tripmine ? (
        <section className="mt-6">
          <TripmineBoard
            tripmineState={toTripmineState(tripmine, isGamemaster)}
            onPickItem={handlePickItem}
            isMyTurn={canAct}
            currentRole={role}
            teams={teams}
            feedbackMessage={tripmineFeedback}
          />
        </section>
      ) : null}

      {connect ? (
        <section className="mt-6">
          <ConnectBoard
            view={connect}
            interactive={canAct}
            onPickName={handlePickName}
            onPickValue={handlePickValue}
            onConfirm={handleConfirmConnect}
            showAnswerKey={isGamemaster}
          />
        </section>
      ) : null}

      {state && state.status === 'finished' ? (
        <GameEndDialog teams={teams} onDismiss={() => void handleEndGame()} />
      ) : null}
      </main>

      {state ? (
        <BottomTeams
          teams={teams}
          activeTeamCount={state.teamCount}
          currentRole={role}
          showHearts={showHearts}
        />
      ) : null}
    </div>
  );
}
