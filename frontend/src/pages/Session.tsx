import { useEffect, useMemo, useState } from 'react';
import { ApiError, postSessionAction, unlockSession } from '../api/client';
import { useSessionStream } from '../hooks/useSessionStream';
import { MiddleItemGrid } from '../components/MiddleItemGrid';
import { TopBarOrdering } from '../components/TopBarOrdering';
import type { QuizItem, UserRole } from '../types/quiz';
import type { OrderItem } from '../types/session';

interface SessionProps {
  sessionId: string;
  role: UserRole;
  onNavigate: (path: string) => void;
}

const WRONG_PASSWORD_COPY = 'Incorrect password!';

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
  };
}

interface ChosenSlot {
  itemId: string;
  side: 'left' | 'right';
  slotIndex: number;
}

export function Session({ sessionId, role, onNavigate }: SessionProps) {
  const [password, setPassword] = useState('');
  const [unlocked, setUnlocked] = useState(false);
  const [unlocking, setUnlocking] = useState(false);
  const [passwordError, setPasswordError] = useState<string | null>(null);
  const needsPassword = role === 'gamemaster' && !unlocked;
  const { state, status, error: streamError, refresh } = useSessionStream(sessionId, role);

  useEffect(() => {
    if (needsPassword) return;
    void refresh();
  }, [needsPassword, refresh]);

  const order = state?.order ?? null;
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

  // The slot a pending item is being dropped on is a client-side step: the server
  // only learns the slot once the placement is confirmed, so CONFIRM is a real
  // second step rather than a formality.
  const [chosenSlot, setChosenSlot] = useState<ChosenSlot | null>(null);
  const [draggingItem, setDraggingItem] = useState<QuizItem | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

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
      slotIndex: chosenSlot.slotIndex,
    };
  }, [order?.pending, chosenSlot]);

  const placedIds = useMemo(
    () => new Set([...leftItems, ...rightItems].map((item) => item.id)),
    [leftItems, rightItems],
  );

  const revealAllValues = isGamemaster || state?.revealValues === true;

  async function sendAction(action: Record<string, unknown>) {
    setActionError(null);
    try {
      await postSessionAction(sessionId, action, isGamemaster ? password : undefined);
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

  function handleDropToPending(item: QuizItem, side: 'left' | 'right', slotIndex: number) {
    setChosenSlot({ itemId: item.id, side, slotIndex });
    // Dragging a pool item straight onto a slot also has to select it first.
    if (order?.pending?.itemId !== item.id && onTurnTeamId) {
      void sendAction({ action: 'select_item', itemId: item.id, teamId: onTurnTeamId });
    }
  }

  function handleChooseSlot(side: 'left' | 'right', slotIndex: number) {
    const pending = order?.pending;
    if (!pending) return;
    setChosenSlot({ itemId: pending.itemId, side, slotIndex });
  }

  function handleConfirmPlacement() {
    const pending = order?.pending;
    if (!pending || !chosenSlot || !onTurnTeamId) return;
    void sendAction({
      action: 'choose_slot',
      itemId: pending.itemId,
      side: chosenSlot.side,
      slotIndex: chosenSlot.slotIndex,
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
    <main className="relative z-10 mx-auto w-full max-w-7xl px-6 py-8">
      <p className="font-mono text-sm text-slate-400">
        {sessionId} / {role}
      </p>

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

      {state ? (
        <section className="mt-6 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
          <p className="col-span-full font-mono text-xs text-slate-500">
            revision {state.revision} / {status} / {state.status} / round{' '}
            {state.currentRound} of {state.roundCount}
          </p>
          {state.teams.map((team) => (
            <article
              key={team.id}
              className="rounded-xl border border-slate-800 bg-slate-900/90 p-4"
              style={{ borderColor: team.primaryHex }}
            >
              <h2 className="text-sm font-bold text-white">{team.name}</h2>
              <p className="mt-1 font-mono text-xs text-slate-400">
                {team.score} pts / {team.hearts} hearts
              </p>
              {team.onTurn ? (
                <p className="mt-2 font-mono text-[10px] uppercase text-cyan-300">
                  On turn
                </p>
              ) : null}
            </article>
          ))}
        </section>
      ) : null}

      {isGamemaster && state ? (
        <section className="mt-4 flex flex-wrap items-center justify-end gap-2">
          {/* Global reveal applies to Order and Tripmine values (FR-024). */}
          <button
            type="button"
            onClick={() => void handleRevealValues()}
            disabled={state.revealValues}
            className="rounded-lg border border-amber-400/60 bg-amber-400/10 px-4 py-2 text-sm font-bold text-amber-200 hover:bg-amber-400/20 disabled:opacity-50"
          >
            {state.revealValues ? 'Values revealed' : 'Reveal values'}
          </button>
        </section>
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
            hasPendingItem={Boolean(order.pending)}
            interactive={isGamemaster}
            onChooseSlot={handleChooseSlot}
            onDropToPending={handleDropToPending}
            onConfirmPlacement={handleConfirmPlacement}
            onCancelPlacement={handleCancelPlacement}
            draggingItem={draggingItem}
            onDragStartItem={setDraggingItem}
            onDragEndItem={() => setDraggingItem(null)}
            revealAllValues={revealAllValues}
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
            interactive={isGamemaster}
          />
        </section>
      ) : null}
    </main>
  );
}
