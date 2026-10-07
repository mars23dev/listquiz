import React from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { TripmineState, TripmineItem, Team, UserRole } from '../types/quiz';
import { ItemIcon } from './ItemIcon';
import { teamBorderStyle, type TeamColorSource } from '../utils/teamColors';
import {
  Bomb,
  Target,
  Trophy,
  AlertTriangle,
  CheckCircle2,
  HelpCircle,
  Sparkles,
  Flame,
  ArrowRight,
  ShieldAlert,
} from 'lucide-react';

interface TripmineBoardProps {
  tripmineState: TripmineState;
  onPickItem: (item: TripmineItem) => void;
  isMyTurn: boolean;
  activeTeam?: Team;
  currentRole: UserRole;
  /** Teams for coloring a revealed item's border (US2). */
  teams?: TeamColorSource[];
  feedbackMessage?: string | null;
}

export const TripmineBoard: React.FC<TripmineBoardProps> = ({
  tripmineState,
  onPickItem,
  isMyTurn,
  activeTeam,
  currentRole,
  teams = [],
  feedbackMessage,
}) => {
  const isGamemaster = currentRole === 'gamemaster';
  const { currentSearchedRank, tripmineSide, items, tripmineHit, isCompleted, mineRank } = tripmineState;
  // A 20-item round uses a denser grid and smaller cells so it fits one screen.
  const dense = items.length > 10;

  // Count remaining unrevealed items
  const unrevealedItems = items.filter((i) => !i.isRevealed);
  const isFinalTwo = unrevealedItems.length === 2;

  return (
    <div className="relative z-10 w-full px-2 sm:px-6 py-2 max-w-7xl mx-auto">
      {/* Top Header: TRIPMINE Title + Target HUD */}
      <div className="relative flex flex-col sm:flex-row items-center justify-between gap-3 mb-3 px-1">
        {/* Left: Direction / Tripmine Rule */}
        <div className="flex items-center gap-2 text-xs">
          <div className="flex items-center gap-1.5 py-1 px-2.5 rounded-lg bg-rose-950/60 border border-rose-500/40 text-rose-300 font-mono font-bold">
            <Bomb className="w-3.5 h-3.5 text-rose-400" />
            <span>
              TRIPMINE:{' '}
              {tripmineSide === 'highest'
                ? '#10 (HIGHEST)'
                : tripmineSide === 'lowest'
                  ? '#1 (LOWEST)'
                  : 'RANDOM (HARD)'}
            </span>
            {tripmineState.targetValue ? (
              <span
                className="text-amber-300"
                aria-label={`Tripmine value ${tripmineState.targetValue}`}
              >
                · VALUE: {tripmineState.targetValue}
              </span>
            ) : null}
          </div>
        </div>

        {/* Center: Headline & Current Target */}
        <div className="text-center">
          <h1 className="text-xl sm:text-2xl font-black uppercase tracking-widest text-white font-['Syne'] leading-none">
            TRIP<span className="text-rose-500">MINE</span>
          </h1>
          <div className="mt-1 flex items-center justify-center gap-1.5 text-xs font-mono font-bold">
            <span className="text-slate-400">SEARCHING FOR:</span>
            <span className="text-amber-300 bg-amber-400/20 border border-amber-400/50 px-2 py-0.5 rounded animate-pulse">
              RANK #{currentSearchedRank} {currentSearchedRank === 1 ? '(LOWEST)' : currentSearchedRank === 10 ? '(HIGHEST)' : ''}
            </span>
            {isFinalTwo && !isCompleted && (
              <span className="text-emerald-400 font-black px-1.5 py-0.5 rounded bg-emerald-500/20 border border-emerald-500/40 text-[10px]">
                FINAL 2 (20 PTS!)
              </span>
            )}
          </div>
        </div>

        {/* Right: Round Status */}
        <div className="flex items-center gap-2 text-xs font-mono">
          <span className="text-slate-400">Remaining:</span>
          <span className="text-cyan-300 font-bold font-mono px-2 py-0.5 rounded bg-slate-900 border border-slate-700">
            {unrevealedItems.length} / 10
          </span>
        </div>
      </div>

      {/* Rank Progress Ladder (one position per item: 10 or 20) */}
      <div className="mb-3 px-2 py-1.5 rounded-xl bg-slate-900/80 border border-slate-800 flex items-center justify-between gap-1 overflow-x-auto">
        {Array.from({ length: items.length }, (_, index) => index + 1).map((rank) => {
          const matchingItem = items.find((i) => i.trueRank === rank);
          const isRev = matchingItem?.isRevealed;
          const isTarget = currentSearchedRank === rank && !isCompleted;
          const isMine = matchingItem?.isTripmine;
          // The mine's position is public from the start (feature 010): a position,
          // not the item.
          const isMineRank = mineRank === rank;

          let badgeStyle = 'bg-slate-950 text-slate-500 border-slate-800';
          if (isRev) {
            if (isMine) {
              badgeStyle = 'bg-rose-500 text-slate-950 border-rose-400 font-black';
            } else {
              badgeStyle = 'bg-emerald-500/20 text-emerald-300 border-emerald-500/50 font-bold';
            }
          } else if (isTarget) {
            badgeStyle = 'bg-amber-400 text-slate-950 border-amber-300 font-black ring-2 ring-amber-400/60 animate-bounce';
          } else if (isMineRank) {
            badgeStyle = 'bg-rose-950/70 text-rose-300 border-rose-500/60 font-bold';
          }

          return (
            <div
              key={rank}
              className={`flex-1 py-1 px-1.5 rounded-lg border text-center font-mono text-[11px] transition-all min-w-[32px] ${badgeStyle}`}
              title={
                matchingItem
                  ? `Rank ${rank}: ${matchingItem.name}${isMine ? ' (TRIPMINE)' : ''}`
                  : isMineRank
                    ? `Rank ${rank} (mine position)`
                    : `Rank ${rank}`
              }
            >
              <span className="block font-bold">#{rank}</span>
              {isRev && isMine ? (
                <Bomb className="w-3 h-3 mx-auto mt-0.5 text-slate-950" />
              ) : isRev ? (
                <CheckCircle2 className="w-3 h-3 mx-auto mt-0.5 text-emerald-400" />
              ) : isTarget ? (
                <Target className="w-3 h-3 mx-auto mt-0.5 text-slate-950" />
              ) : isMineRank ? (
                <Bomb className="w-3 h-3 mx-auto mt-0.5 text-rose-400" />
              ) : null}
            </div>
          );
        })}
      </div>

      {/* Turn or Status Banner */}
      <AnimatePresence>
        {feedbackMessage && (
          <motion.div
            initial={{ opacity: 0, y: -6 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -6 }}
            className={`mb-3 py-1.5 px-4 rounded-xl text-xs font-bold font-mono text-center mx-auto w-fit border shadow-lg ${
              tripmineHit
                ? 'bg-rose-500/30 border-rose-500 text-rose-200'
                : 'bg-amber-500/20 border-amber-400/50 text-amber-200'
            }`}
          >
            {feedbackMessage}
          </motion.div>
        )}
      </AnimatePresence>

      {/* Tripmine grid: 5 columns for 10 items, 10 columns for 20 so all fit. */}
      <div
        className={`grid gap-3 ${
          dense ? 'grid-cols-4 sm:grid-cols-5 md:grid-cols-10' : 'grid-cols-2 sm:grid-cols-3 md:grid-cols-5'
        }`}
      >
        {items.map((item, index) => {
          const isRevealed = item.isRevealed;
          const isMine = item.isTripmine;
          const isTarget = item.trueRank === currentSearchedRank;
          const canClick = isMyTurn && !isRevealed && !isCompleted;

          // The border names the team that picked the item (US2); mine/safe stays on
          // the rank badge, the icon, and the point text (FR-009).
          const pickedBy = isRevealed ? item.revealedByTeamId : undefined;
          const borderStyle = pickedBy ? teamBorderStyle(pickedBy, teams) : undefined;

          return (
            <motion.button
              key={item.id}
              layout
              type="button"
              disabled={!canClick}
              style={borderStyle}
              aria-label={
                canClick ? `Pick ${item.name}` : `${item.name}, hidden or already picked`
              }
              initial={{ opacity: 0, scale: 0.95 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ duration: 0.18, delay: index * 0.02 }}
              onClick={() => {
                if (canClick) {
                  onPickItem(item);
                }
              }}
              className={`relative flex flex-col justify-between rounded-xl border transition-all select-none text-left focus:outline-none focus-visible:ring-2 focus-visible:ring-amber-300 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-950 ${
                dense ? 'p-1.5 min-h-[64px]' : 'p-3 min-h-[120px]'
              } ${
                isRevealed && isMine
                  ? 'border-rose-500 bg-rose-950/80 shadow-2xl shadow-rose-500/40 ring-4 ring-rose-500/60'
                  : isRevealed
                  ? 'border-emerald-500/60 bg-slate-900/95 shadow-md'
                  : canClick
                  ? 'border-slate-700/80 bg-slate-900/90 hover:border-amber-400 hover:bg-slate-850 hover:shadow-amber-400/20 cursor-pointer active:scale-95'
                  : 'border-slate-800/80 bg-slate-950/60 opacity-80 cursor-default'
              }`}
            >
              {/* Card Header */}
              <div className="flex items-center justify-between text-[11px] mb-1">
                {isRevealed ? (
                  <span
                    className={`font-mono font-black px-1.5 py-0.5 rounded text-[10px] ${
                      isMine
                        ? 'bg-rose-500 text-slate-950'
                        : 'bg-emerald-500 text-slate-950'
                    }`}
                  >
                    RANK #{item.trueRank}
                  </span>
                ) : isGamemaster ? (
                  <span className="font-mono text-[10px] text-amber-300 font-bold">
                    GM: #{item.trueRank} {isMine ? '⚠️ MINE' : ''}
                  </span>
                ) : (
                  <span className="font-mono text-[10px] text-slate-500">
                    Item #{index + 1}
                  </span>
                )}

                {/* Status Icon */}
                {isRevealed && isMine ? (
                  <Bomb className="w-4 h-4 text-rose-400 animate-bounce" />
                ) : isRevealed ? (
                  <CheckCircle2 className="w-4 h-4 text-emerald-400" />
                ) : (
                  <HelpCircle className="w-3.5 h-3.5 text-slate-600" />
                )}
              </div>

              {/* Center Content: Icon + Name */}
              <div className="flex flex-col items-center justify-center my-1 text-center">
                <div
                  className={`p-2 rounded-lg mb-1.5 ${
                    isRevealed && isMine
                      ? 'bg-rose-500/20 text-rose-300'
                      : isRevealed
                      ? 'bg-emerald-500/20 text-emerald-300'
                      : 'bg-slate-800 text-cyan-300'
                  }`}
                >
                  <ItemIcon
                    name={item.iconName}
                    customIconUrl={item.customIconUrl}
                    className="w-5 h-5"
                  />
                </div>

                <h3 className="text-xs sm:text-[13px] font-bold text-slate-100 leading-snug line-clamp-2">
                  {item.name}
                </h3>
              </div>

              {/* Card Footer: Points awarded / Reveal value / Click affordance */}
              <div className="pt-1.5 border-t border-slate-800/80 mt-1 flex flex-col items-center justify-center">
                {isRevealed ? (
                  <div className="w-full flex items-center justify-between text-[11px] font-mono">
                    <span className="text-slate-400 truncate font-semibold">
                      {item.value}
                    </span>
                    <span
                      className={`font-black ${
                        isMine
                          ? 'text-rose-400'
                          : item.pointsAwarded && item.pointsAwarded >= 20
                          ? 'text-emerald-300'
                          : 'text-amber-300'
                      }`}
                    >
                      {isMine ? '-10 PTS' : `+${item.pointsAwarded || 5} PTS`}
                    </span>
                  </div>
                ) : isGamemaster ? (
                  <div className="w-full flex items-center justify-between text-[11px] font-mono">
                    <span className="text-slate-400 truncate font-semibold">
                      {item.value}
                    </span>
                    <span className="text-slate-600 font-black">GM</span>
                  </div>
                ) : canClick ? (
                  <span className="text-[10px] font-bold text-amber-300 uppercase tracking-wider animate-pulse flex items-center gap-1">
                    <span>Click to Pick</span>
                    <ArrowRight className="w-3 h-3" />
                  </span>
                ) : (
                  <span className="text-[10px] text-slate-600 font-mono">
                    Hidden
                  </span>
                )}
              </div>

              {/* Mine Explosion Flash Overlay */}
              {isRevealed && isMine && (
                <div className="absolute inset-0 bg-rose-500/20 rounded-xl pointer-events-none animate-pulse" />
              )}
            </motion.button>
          );
        })}
      </div>
    </div>
  );
};
