import React from 'react';
import { motion } from 'motion/react';
import { Check, Link2 } from 'lucide-react';
import { ItemIcon } from './ItemIcon';
import type { ConnectView } from '../types/session';

interface ConnectBoardProps {
  view: ConnectView;
  /** False for players who are not on turn (read-only). */
  interactive: boolean;
  onPickName: (itemId: string) => void;
  onPickValue: (itemId: string) => void;
  onConfirm: () => void;
  /** Gamemaster only (feature 012): show each name's correct value beside it. */
  showAnswerKey?: boolean;
  /** The round's value-column name (e.g. "Area (km²)"), shown above the value column (feature 017). */
  columnName?: string;
}

/**
 * The Connect board (feature 010): ten names on the left (each with its icon) and the
 * ten values on the right in the list's order (no icons). The staged pair waits in the
 * middle box until confirmed. Values carry no icon, by design. The gamemaster may also
 * see each name's correct value as an answer key (feature 012).
 */
export const ConnectBoard: React.FC<ConnectBoardProps> = ({
  view,
  interactive,
  onPickName,
  onPickValue,
  onConfirm,
  showAnswerKey = false,
  columnName,
}) => {
  const matched = new Set(view.matched);
  const pendingName = view.pending.nameItemId
    ? view.names.find((name) => name.itemId === view.pending.nameItemId) ?? null
    : null;
  const pendingValue = view.pending.valueItemId
    ? view.values.find((value) => value.itemId === view.pending.valueItemId) ?? null
    : null;
  const canConfirm = interactive && pendingName !== null && pendingValue !== null;

  const cellClass = (isPicked: boolean, isMatched: boolean) =>
    `relative flex h-12 w-full items-center gap-2 rounded-xl border-2 px-2 text-left transition-all select-none ${
      isMatched
        ? 'border-slate-800/60 bg-slate-950/40 opacity-30'
        : isPicked
          ? 'border-emerald-400 bg-emerald-950/40 ring-2 ring-emerald-400/60'
          : 'border-slate-700/80 bg-slate-900/90 hover:border-cyan-400'
    }`;

  return (
    <div className="relative z-10 mx-auto w-full max-w-6xl px-2 sm:px-6 py-3">
      <h1 className="mb-3 text-center font-display text-xl sm:text-2xl font-black uppercase tracking-widest text-slate-100">
        Connect
      </h1>

      <div className="grid grid-cols-1 gap-3 sm:grid-cols-[1fr_auto_1fr]">
        {/* Left: names, each with its icon */}
        <div className="flex flex-col gap-2" aria-label="Names">
          {view.names.map((name) => {
            const isPicked = view.pending.nameItemId === name.itemId;
            const isMatched = matched.has(name.itemId);
            const answerKey = showAnswerKey
              ? (view.values.find((value) => value.itemId === name.itemId) ?? null)
              : null;
            return (
              <motion.button
                key={name.itemId}
                type="button"
                layout
                disabled={!interactive || isMatched}
                aria-pressed={isPicked}
                onClick={() => onPickName(name.itemId)}
                className={cellClass(isPicked, isMatched)}
              >
                <span className="shrink-0 text-cyan-300">
                  <ItemIcon name={name.iconName ?? undefined} className="w-5 h-5" />
                </span>
                <span className="truncate text-xs sm:text-sm font-semibold text-slate-100">
                  {name.name}
                </span>
                {answerKey && !isMatched ? (
                  <span className="ml-auto shrink-0 rounded bg-amber-400/10 px-1.5 py-0.5 font-mono text-[10px] font-bold tabular-nums text-amber-200">
                    {answerKey.display}
                  </span>
                ) : null}
              </motion.button>
            );
          })}
        </div>

        {/* Middle: the box the pair is pulled into */}
        <div className="flex flex-col items-center justify-center gap-3 rounded-2xl border border-slate-800 bg-slate-900/90 p-4 sm:w-56">
          <div className="flex items-center gap-1.5 text-xs font-bold uppercase tracking-wider text-slate-400">
            <Link2 className="w-3.5 h-3.5" />
            <span>Match</span>
          </div>
          <div className="w-full rounded-lg border border-dashed border-slate-700 bg-slate-950/60 p-2 text-center text-xs text-slate-300 min-h-[34px]">
            {pendingName ? pendingName.name : 'Pick a name'}
          </div>
          <div className="w-full rounded-lg border border-dashed border-slate-700 bg-slate-950/60 p-2 text-center text-xs text-slate-300 min-h-[34px]">
            {pendingValue ? pendingValue.display : 'Pick a value'}
          </div>
          <button
            type="button"
            onClick={onConfirm}
            disabled={!canConfirm}
            className="flex items-center justify-center gap-1 rounded-md bg-emerald-500 px-3 py-1.5 text-xs font-bold text-slate-950 shadow shadow-emerald-500/30 hover:bg-emerald-400 disabled:opacity-40"
          >
            <Check className="h-3 w-3 stroke-[3]" />
            <span>CONFIRM</span>
          </button>
        </div>

        {/* Right: values, no icons */}
        <div className="flex flex-col gap-2" aria-label="Values">
          {columnName ? (
            <div className="px-1 text-center font-mono text-[10px] font-bold uppercase tracking-wider text-amber-300/80">
              {columnName}
            </div>
          ) : null}
          {view.values.map((value) => {
            const isPicked = view.pending.valueItemId === value.itemId;
            const isMatched = matched.has(value.itemId);
            return (
              <motion.button
                key={value.itemId}
                type="button"
                layout
                disabled={!interactive || isMatched}
                aria-pressed={isPicked}
                onClick={() => onPickValue(value.itemId)}
                className={cellClass(isPicked, isMatched)}
              >
                <span className="truncate font-mono text-xs sm:text-sm font-bold tabular-nums text-amber-200">
                  {value.display}
                </span>
              </motion.button>
            );
          })}
        </div>
      </div>
    </div>
  );
};
