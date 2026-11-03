import React, { useState } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { QuizItem, DragItemPayload } from '../types/quiz';
import { ItemIcon } from './ItemIcon';
import { MoveHorizontal, Check, X, AlertTriangle } from 'lucide-react';
import { NEUTRAL_BORDER, teamBorderStyle, type TeamColorSource } from '../utils/teamColors';

interface TopBarOrderingProps {
  anchorItem: QuizItem;
  leftItems: QuizItem[]; // Length 0..5
  rightItems: QuizItem[]; // Length 0..5
  pendingItem: { item: QuizItem; side: 'left' | 'right'; slotIndex: number } | null;
  onDropToPending: (item: QuizItem, side: 'left' | 'right', slotIndex: number) => void;
  /** Keyboard path for choosing a slot for the already selected item (FR-028). */
  onChooseSlot: (side: 'left' | 'right', slotIndex: number) => void;
  /** Teams for coloring a placed item's border (US2). */
  teams?: TeamColorSource[];
  /** The team on turn, used to color the pending preview (US2). */
  activeTeamId?: string | null;
  /**
   * True while the server holds a pending item awaiting a slot. Kept apart from
   * `pendingItem`, which is only set once a slot has been picked, so the slot
   * buttons are reachable before the item is drawn into the bar.
   */
  hasPendingItem?: boolean;
  /** False for players, who observe the board but cannot act on it. */
  interactive?: boolean;
  onConfirmPlacement: () => void;
  onCancelPlacement: () => void;
  draggingItem: QuizItem | null;
  onDragStartItem: (item: QuizItem) => void;
  onDragEndItem: () => void;
  revealAllValues: boolean;
  /** True when the session's reveal flag is set: borders go neutral and the bar is
   * the server's arranged layout. Distinct from `revealAllValues`, which is also
   * true for the gamemaster during normal play. */
  neutralBorders?: boolean;
  /** The round's value-column name (e.g. "Area (km²)"), shown with the values (feature 017). */
  columnName?: string;
  feedbackMessage?: string | null;
}

export const TopBarOrdering: React.FC<TopBarOrderingProps> = ({
  anchorItem,
  leftItems,
  rightItems,
  pendingItem,
  onDropToPending,
  onChooseSlot,
  teams = [],
  activeTeamId = null,
  hasPendingItem = false,
  interactive = true,
  onConfirmPlacement,
  onCancelPlacement,
  draggingItem,
  onDragStartItem,
  onDragEndItem,
  revealAllValues,
  neutralBorders = false,
  feedbackMessage,
  columnName,
}) => {
  // Currently hovered slot while dragging
  const [hoveredSlot, setHoveredSlot] = useState<{ side: 'left' | 'right'; slotIndex: number } | null>(null);

  // Compute preview lists when dragging an item over slots or viewing pending item
  const computeDisplayLeft = (): { items: QuizItem[]; isPendingInList: boolean } => {
    // If currently dragging an item over the left side, show live rearrangement preview
    if (draggingItem && hoveredSlot && hoveredSlot.side === 'left') {
      const base = leftItems.filter((i) => i.id !== draggingItem.id);
      const k = base.length;
      if (k < 5) {
        let res: QuizItem[];
        if (hoveredSlot.slotIndex <= 4 - k) {
          res = [draggingItem, ...base];
        } else {
          const occ = hoveredSlot.slotIndex - (5 - k);
          res = [...base];
          res.splice(occ + 1, 0, draggingItem);
        }
        return { items: res, isPendingInList: false };
      }
    }

    // If pendingItem is on the left and not currently being dragged elsewhere
    if (pendingItem && pendingItem.side === 'left') {
      const base = leftItems.filter((i) => i.id !== pendingItem.item.id);
      const k = base.length;
      let res: QuizItem[];
      if (pendingItem.slotIndex <= 4 - k) {
        res = [pendingItem.item, ...base];
      } else {
        const occ = pendingItem.slotIndex - (5 - k);
        res = [...base];
        res.splice(occ + 1, 0, pendingItem.item);
      }
      return { items: res, isPendingInList: true };
    }

    return { items: leftItems, isPendingInList: false };
  };

  const computeDisplayRight = (): { items: QuizItem[]; isPendingInList: boolean } => {
    // If currently dragging an item over the right side, show live rearrangement preview
    if (draggingItem && hoveredSlot && hoveredSlot.side === 'right') {
      const base = rightItems.filter((i) => i.id !== draggingItem.id);
      const m = base.length;
      if (m < 5) {
        let res: QuizItem[];
        if (hoveredSlot.slotIndex >= m) {
          res = [...base, draggingItem];
        } else {
          res = [...base];
          res.splice(hoveredSlot.slotIndex, 0, draggingItem);
        }
        return { items: res, isPendingInList: false };
      }
    }

    // If pendingItem is on the right and not currently being dragged elsewhere
    if (pendingItem && pendingItem.side === 'right') {
      const base = rightItems.filter((i) => i.id !== pendingItem.item.id);
      const m = base.length;
      let res: QuizItem[];
      if (pendingItem.slotIndex >= m) {
        res = [...base, pendingItem.item];
      } else {
        res = [...base];
        res.splice(pendingItem.slotIndex, 0, pendingItem.item);
      }
      return { items: res, isPendingInList: true };
    }

    return { items: rightItems, isPendingInList: false };
  };

  const displayLeft = computeDisplayLeft();
  const displayRight = computeDisplayRight();

  const handleDragOver = (e: React.DragEvent, side: 'left' | 'right', slotIndex: number) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    if (!hoveredSlot || hoveredSlot.side !== side || hoveredSlot.slotIndex !== slotIndex) {
      setHoveredSlot({ side, slotIndex });
    }
  };

  const handleDragLeave = () => {
    setHoveredSlot(null);
  };

  const handleDrop = (e: React.DragEvent, targetSide: 'left' | 'right', targetSlotIndex: number) => {
    e.preventDefault();
    setHoveredSlot(null);
    const dataStr = e.dataTransfer.getData('application/json');
    if (!dataStr) return;

    try {
      const payload: DragItemPayload = JSON.parse(dataStr);
      onDropToPending(payload.item, targetSide, targetSlotIndex);
    } catch {
      // Ignored
    }
  };

  const handleDragStartPending = (e: React.DragEvent, item: QuizItem) => {
    onDragStartItem(item);
    const payload: DragItemPayload = {
      item,
      source: pendingItem?.side ?? 'left',
    };
    e.dataTransfer.setData('application/json', JSON.stringify(payload));
    e.dataTransfer.effectAllowed = 'move';
  };

  return (
    <header className="relative z-10 w-full px-2 sm:px-6 pt-4 pb-2">
      <div className="max-w-7xl mx-auto">
        {/* The Order Header - at the very top of the entire page */}
        <div className="relative flex items-center justify-between mb-3 px-1">
          {/* Far Left: Lowest Item Hint */}
          <div className="flex items-center gap-1.5 text-xs font-semibold text-cyan-400">
            <span className="w-1.5 h-1.5 rounded-full bg-cyan-400" />
            <span className="uppercase tracking-wider">Lowest</span>
          </div>

          {/* Centered Headline: Order */}
          <h1 className="absolute left-1/2 -translate-x-1/2 text-xl sm:text-2xl font-black uppercase tracking-widest text-slate-100 font-['Syne']">
            Order
          </h1>

          {/* Far Right: Highest Item Hint */}
          <div className="flex items-center gap-1.5 text-xs font-semibold text-rose-400">
            <span className="uppercase tracking-wider">Highest</span>
            <span className="w-1.5 h-1.5 rounded-full bg-rose-400" />
          </div>
        </div>

        {/* Feedback Toast */}
        <AnimatePresence>
          {feedbackMessage && (
            <motion.div
              initial={{ opacity: 0, y: -8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -8 }}
              className="mb-2 flex items-center justify-center gap-2 py-1 px-3 rounded-lg bg-rose-500/20 border border-rose-500/40 text-rose-300 text-xs font-bold font-mono text-center mx-auto w-fit"
            >
              <AlertTriangle className="w-3.5 h-3.5 text-rose-400" />
              <span>{feedbackMessage}</span>
            </motion.div>
          )}
        </AnimatePresence>

        {/* The 11-Slot Master Bar */}
        <div className="relative rounded-2xl bg-slate-900/90 border border-slate-800 shadow-2xl p-2 sm:p-2.5 backdrop-blur-xl">
          {/* Runway center line */}
          <div className="absolute top-1/2 left-4 right-4 h-[2px] bg-gradient-to-r from-cyan-500/20 via-amber-500/40 to-rose-500/20 -translate-y-1/2 pointer-events-none" />

          {/* Slots Container - 11 columns */}
          <div className="grid grid-cols-11 gap-1.5 sm:gap-2 items-stretch min-h-[142px]">
            {/* 5 LEFT SLOTS */}
            {[0, 1, 2, 3, 4].map((slotIdx) => {
              const k = displayLeft.items.length;
              const hasItem = slotIdx >= 5 - k;
              const itemIndexInArray = slotIdx - (5 - k);
              const item = hasItem ? displayLeft.items[itemIndexInArray] : null;
              const isHovered = hoveredSlot?.side === 'left' && hoveredSlot.slotIndex === slotIdx;
              const isDirectNeighbor = slotIdx === 4;

              // Check if this item is the currently pending (unconfirmed) item
              const isPending = pendingItem && item && item.id === pendingItem.item.id;
              // Check if currently dragging so confirm button is hidden while dragging
              const isBeingDragged = draggingItem && item && item.id === draggingItem.id;

              // The border names the team that placed the item; the pending preview
              // is framed by the team on turn. On reveal every border goes neutral.
              const ownedTeamId = isPending ? activeTeamId : item?.placedByTeamId ?? null;
              const borderStyle = neutralBorders
                ? { borderColor: NEUTRAL_BORDER }
                : ownedTeamId
                  ? teamBorderStyle(ownedTeamId, teams)
                  : undefined;

              const slotClassName = `group relative flex flex-col justify-between rounded-xl transition-all duration-150 select-none p-1.5 text-center border ${
                isPending
                  ? 'border-emerald-400 bg-emerald-950/40 shadow-lg shadow-emerald-500/25 ring-2 ring-emerald-400/60 scale-[1.02]'
                  : isHovered
                  ? 'border-cyan-400 bg-cyan-950/40 shadow-lg shadow-cyan-500/25 ring-2 ring-cyan-400/50 scale-[1.02]'
                  : item
                  ? 'border-cyan-600/40 bg-slate-800/90 shadow-md'
                  : isDirectNeighbor
                  ? 'border-dashed border-cyan-800/50 bg-slate-950/40 hover:border-cyan-600/60'
                  : 'border-dashed border-slate-800/80 bg-slate-950/30 hover:border-slate-700'
              }`;

              // The selected item is rendered in its candidate slot; placement
              // controls live outside the slot buttons to keep HTML valid.
              const canPlaceHere =
                interactive &&
                (hasPendingItem || Boolean(pendingItem) || Boolean(draggingItem)) &&
                !isPending &&
                !isBeingDragged;

              const slotBody = (
                <>
                  {item ? (
                    <motion.div
                      layout
                      draggable={Boolean(isPending)}
                      onDragStart={(e) => {
                        if (isPending) {
                          handleDragStartPending(e as unknown as React.DragEvent, item);
                        }
                      }}
                      onDragEnd={() => onDragEndItem()}
                      initial={{ scale: 0.9, opacity: 0 }}
                      animate={{ scale: 1, opacity: 1 }}
                      exit={{ scale: 0.85, opacity: 0 }}
                      transition={{ duration: 0.15 }}
                      className={`flex-1 flex flex-col justify-center items-center py-1 ${
                        isPending ? 'cursor-grab active:cursor-grabbing' : ''
                      }`}
                    >
                      <div className="flex flex-col items-center">
                        <div className={`mb-1 ${isPending ? 'text-emerald-300' : 'text-cyan-300'}`}>
                          <ItemIcon name={item.iconName} className="w-5 h-5" />
                        </div>
                        <p className="text-[11px] sm:text-xs font-semibold text-slate-100 line-clamp-2 leading-snug px-0.5 text-center">
                          {item.name}
                        </p>

                        {/* Revealed value in the middle, like the anchor (feature 017) */}
                        {revealAllValues && (
                          <div className="mt-2 py-0.5 px-1.5 rounded bg-slate-800/80 border border-slate-600/60">
                            <span className="text-xs sm:text-sm font-black font-mono tabular-nums text-cyan-300">
                              {item.value}
                            </span>
                          </div>
                        )}
                        {revealAllValues && columnName ? (
                          <span className="mt-1 font-mono text-[9px] uppercase tracking-wider text-slate-400">
                            {columnName}
                          </span>
                        ) : null}
                      </div>
                    </motion.div>
                  ) : (
                    <div className="flex-1 flex flex-col items-center justify-center text-slate-700">
                      <MoveHorizontal className="w-3.5 h-3.5 opacity-30" />
                    </div>
                  )}
                </>
              );

              return (
                <button
                  key={`left-slot-${slotIdx}`}
                  type="button"
                  disabled={!interactive}
                  aria-disabled={!canPlaceHere}
                  style={borderStyle}
                  aria-label={
                    canPlaceHere
                      ? `Place the selected item in left slot ${slotIdx + 1}`
                      : `Left slot ${slotIdx + 1}, ${item ? `holds ${item.name}` : 'empty'}`
                  }
                  onClick={() => {
                    if (hasPendingItem || pendingItem) onChooseSlot('left', slotIdx);
                  }}
                  onDragOver={(e) => handleDragOver(e, 'left', slotIdx)}
                  onDragLeave={handleDragLeave}
                  onDrop={(e) => handleDrop(e, 'left', slotIdx)}
                  className={`${slotClassName} focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-300 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-950 ${
                    canPlaceHere ? 'cursor-pointer hover:border-cyan-300' : 'cursor-default'
                  }`}
                >
                  {slotBody}
                </button>
              );
            })}

            {/* CENTER STATIC REVEALED ANCHOR ITEM */}
            <div className="relative flex flex-col justify-center items-center rounded-xl bg-gradient-to-b from-amber-500/20 via-amber-950/30 to-slate-900 border-2 border-amber-400/90 shadow-xl shadow-amber-500/20 p-2 text-center select-none ring-2 ring-amber-400/30">
              <div className="text-amber-400 mb-1">
                <ItemIcon name={anchorItem.iconName} className="w-5 h-5" />
              </div>
              
              <h4 className="text-[11px] sm:text-xs font-bold text-amber-100 line-clamp-3 leading-snug px-0.5">
                {anchorItem.name}
              </h4>

              <div className="mt-2 py-0.5 px-1.5 rounded bg-amber-400/20 border border-amber-400/50">
                <span className="text-xs sm:text-sm font-black font-mono tabular-nums text-amber-300">
                  {anchorItem.value}
                </span>
              </div>
              {columnName ? (
                <span className="mt-1 font-mono text-[9px] uppercase tracking-wider text-amber-400/80">
                  {columnName}
                </span>
              ) : null}
            </div>

            {/* 5 RIGHT SLOTS */}
            {[0, 1, 2, 3, 4].map((slotIdx) => {
              const hasItem = slotIdx < displayRight.items.length;
              const item = hasItem ? displayRight.items[slotIdx] : null;
              const isHovered = hoveredSlot?.side === 'right' && hoveredSlot.slotIndex === slotIdx;
              const isDirectNeighbor = slotIdx === 0;

              // Check if this item is the currently pending (unconfirmed) item
              const isPending = pendingItem && item && item.id === pendingItem.item.id;
              // Check if currently dragging so confirm button is hidden while dragging
              const isBeingDragged = draggingItem && item && item.id === draggingItem.id;

              // The border names the team that placed the item; the pending preview
              // is framed by the team on turn. On reveal every border goes neutral.
              const ownedTeamId = isPending ? activeTeamId : item?.placedByTeamId ?? null;
              const borderStyle = neutralBorders
                ? { borderColor: NEUTRAL_BORDER }
                : ownedTeamId
                  ? teamBorderStyle(ownedTeamId, teams)
                  : undefined;

              const slotClassName = `group relative flex flex-col justify-between rounded-xl transition-all duration-150 select-none p-1.5 text-center border ${
                isPending
                  ? 'border-emerald-400 bg-emerald-950/40 shadow-lg shadow-emerald-500/25 ring-2 ring-emerald-400/60 scale-[1.02]'
                  : isHovered
                  ? 'border-rose-400 bg-rose-950/40 shadow-lg shadow-rose-500/25 ring-2 ring-rose-400/50 scale-[1.02]'
                  : item
                  ? 'border-rose-600/40 bg-slate-800/90 shadow-md'
                  : isDirectNeighbor
                  ? 'border-dashed border-rose-800/50 bg-slate-950/40 hover:border-rose-600/60'
                  : 'border-dashed border-slate-800/80 bg-slate-950/30 hover:border-slate-700'
              }`;

              const canPlaceHere =
                interactive &&
                (hasPendingItem || Boolean(pendingItem) || Boolean(draggingItem)) &&
                !isPending &&
                !isBeingDragged;

              const slotBody = (
                <>
                  {item ? (
                    <motion.div
                      layout
                      draggable={Boolean(isPending)}
                      onDragStart={(e) => {
                        if (isPending) {
                          handleDragStartPending(e as unknown as React.DragEvent, item);
                        }
                      }}
                      onDragEnd={() => onDragEndItem()}
                      initial={{ scale: 0.9, opacity: 0 }}
                      animate={{ scale: 1, opacity: 1 }}
                      exit={{ scale: 0.85, opacity: 0 }}
                      transition={{ duration: 0.15 }}
                      className={`flex-1 flex flex-col justify-center items-center py-1 ${
                        isPending ? 'cursor-grab active:cursor-grabbing' : ''
                      }`}
                    >
                      <div className="flex flex-col items-center">
                        <div className={`mb-1 ${isPending ? 'text-emerald-300' : 'text-rose-300'}`}>
                          <ItemIcon name={item.iconName} className="w-5 h-5" />
                        </div>
                        <p className="text-[11px] sm:text-xs font-semibold text-slate-100 line-clamp-2 leading-snug px-0.5 text-center">
                          {item.name}
                        </p>

                        {/* Revealed value in the middle, like the anchor (feature 017) */}
                        {revealAllValues && (
                          <div className="mt-2 py-0.5 px-1.5 rounded bg-slate-800/80 border border-slate-600/60">
                            <span className="text-xs sm:text-sm font-black font-mono tabular-nums text-rose-300">
                              {item.value}
                            </span>
                          </div>
                        )}
                        {revealAllValues && columnName ? (
                          <span className="mt-1 font-mono text-[9px] uppercase tracking-wider text-slate-400">
                            {columnName}
                          </span>
                        ) : null}
                      </div>
                    </motion.div>
                  ) : (
                    <div className="flex-1 flex flex-col items-center justify-center text-slate-700">
                      <MoveHorizontal className="w-3.5 h-3.5 opacity-30" />
                    </div>
                  )}
                </>
              );

              return (
                <button
                  key={`right-slot-${slotIdx}`}
                  type="button"
                  disabled={!interactive}
                  aria-disabled={!canPlaceHere}
                  style={borderStyle}
                  aria-label={
                    canPlaceHere
                      ? `Place the selected item in right slot ${slotIdx + 1}`
                      : `Right slot ${slotIdx + 1}, ${item ? `holds ${item.name}` : 'empty'}`
                  }
                  onClick={() => {
                    if (hasPendingItem || pendingItem) onChooseSlot('right', slotIdx);
                  }}
                  onDragOver={(e) => handleDragOver(e, 'right', slotIdx)}
                  onDragLeave={handleDragLeave}
                  onDrop={(e) => handleDrop(e, 'right', slotIdx)}
                  className={`${slotClassName} focus:outline-none focus-visible:ring-2 focus-visible:ring-rose-300 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-950 ${
                    canPlaceHere ? 'cursor-pointer hover:border-rose-300' : 'cursor-default'
                  }`}
                >
                  {slotBody}
                </button>
              );
            })}
          </div>
        </div>
        {interactive && hasPendingItem ? (
          <div className="mt-2 flex justify-end gap-2" aria-label="Placement actions">
            <button
              type="button"
              onClick={onCancelPlacement}
              aria-label="Cancel position"
              className="flex items-center justify-center gap-1 rounded-md border border-slate-700 px-3 py-1.5 text-xs font-bold text-slate-300 hover:border-rose-400 hover:text-rose-300 focus:outline-none focus-visible:ring-2 focus-visible:ring-rose-300"
            >
              <X className="h-3 w-3" />
              <span>CANCEL</span>
            </button>
            {pendingItem ? (
              <button
                type="button"
                onClick={onConfirmPlacement}
                aria-label="Confirm placement"
                className="flex items-center justify-center gap-1 rounded-md bg-emerald-500 px-3 py-1.5 text-xs font-bold text-slate-950 shadow shadow-emerald-500/30 hover:bg-emerald-400 focus:outline-none focus-visible:ring-2 focus-visible:ring-emerald-300"
              >
                <Check className="h-3 w-3 stroke-[3]" />
                <span>CONFIRM PLACEMENT</span>
              </button>
            ) : null}
          </div>
        ) : null}
      </div>
    </header>
  );
};
