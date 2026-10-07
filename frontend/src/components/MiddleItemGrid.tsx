import React from 'react';
import { motion } from 'motion/react';
import { QuizItem, DragItemPayload } from '../types/quiz';
import { ItemIcon } from './ItemIcon';

interface MiddleItemGridProps {
  items: QuizItem[]; // The 10 pool items
  placedIds: Set<string>;
  onReturnToPool: (itemId: string) => void;
  onDragStartItem: (item: QuizItem) => void;
  onDragEndItem: () => void;
  revealAllValues: boolean;
  /** Keyboard/click path: pick a pool item to make it the pending item (FR-028). */
  onSelectItem: (item: QuizItem) => void;
  /** Id of the pending item, highlighted so the selection is visible. */
  selectedItemId?: string | null;
  /** False for players, who observe the pool but cannot act on it. */
  interactive?: boolean;
}

export const MiddleItemGrid: React.FC<MiddleItemGridProps> = ({
  items,
  placedIds,
  onReturnToPool,
  onDragStartItem,
  onDragEndItem,
  revealAllValues,
  onSelectItem,
  selectedItemId,
  interactive = true,
}) => {
  const handleDragStart = (e: React.DragEvent, item: QuizItem) => {
    onDragStartItem(item);
    const payload: DragItemPayload = {
      item,
      source: 'pool',
    };
    e.dataTransfer.setData('application/json', JSON.stringify(payload));
    e.dataTransfer.effectAllowed = 'copyMove';
  };

  const handleDragEnd = () => {
    onDragEndItem();
  };

  const handleDropOnGrid = (e: React.DragEvent) => {
    e.preventDefault();
    const dataStr = e.dataTransfer.getData('application/json');
    if (!dataStr) return;
    try {
      const payload: DragItemPayload = JSON.parse(dataStr);
      // Dropping an item already in the pool back onto the pool has no effect.
      // Bar-origin drops return a placed item or cancel an unconfirmed selection.
      if (payload.source !== 'pool') onReturnToPool(payload.item.id);
    } catch {
      // Ignored
    }
  };

  const handleDragOverGrid = (e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
  };

  return (
    <section 
      onDragOver={handleDragOverGrid}
      onDrop={handleDropOnGrid}
      className="relative z-10 w-full px-2 sm:px-6 py-4"
    >
      <div className="max-w-7xl mx-auto">
        {/* 10 Items Grid (5 columns x 2 rows) */}
        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-5 gap-3">
          {items.map((item, index) => {
            const isPlaced = placedIds.has(item.id);
            const isSelected = selectedItemId === item.id;
            const isReadOnly = isPlaced || !interactive;

            const card = (
              <>
                {/* Icon */}
                <div
                  className={`mb-2 p-2 rounded-lg transition-transform ${
                    isPlaced
                      ? 'text-slate-600'
                      : 'bg-slate-800/80 text-cyan-300 group-hover:scale-110 group-hover:text-cyan-400'
                  }`}
                >
                  <ItemIcon name={item.iconName} className="w-5 h-5" />
                </div>

                {/* Name */}
                <h3 className="text-xs sm:text-[13px] font-bold text-slate-100 text-center leading-snug group-hover:text-white transition-colors">
                  {item.name}
                </h3>

                {/* Reveal Value if Game is Over */}
                {revealAllValues && (
                  <div className="mt-1.5 text-[11px] font-mono font-bold tabular-nums text-amber-300">
                    {item.value}
                  </div>
                )}
              </>
            );

            return (
              <motion.div
                key={item.id}
                layout
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.16, delay: index * 0.015 }}
                className={`group relative rounded-xl border transition-all duration-150 min-h-[96px] ${
                  isPlaced
                    ? 'border-slate-800/60 bg-slate-950/40 opacity-20 select-none'
                    : !interactive
                    ? 'border-slate-700/80 bg-slate-900/90 shadow-lg shadow-black/40'
                    : isSelected
                    ? 'border-emerald-400 bg-emerald-950/40 shadow-lg shadow-emerald-500/25 ring-2 ring-emerald-400/60'
                    : 'border-slate-700/80 bg-slate-900/90 shadow-lg shadow-black/40 hover:border-cyan-400 hover:bg-slate-850 hover:shadow-cyan-500/10'
                }`}
              >
                {isReadOnly ? (
                  <div
                    className="flex flex-col items-center justify-center h-full p-3.5"
                    aria-hidden="true"
                  >
                    {card}
                  </div>
                ) : (
                  <button
                    type="button"
                    draggable
                    onDragStart={(e) => handleDragStart(e as unknown as React.DragEvent, item)}
                    onDragEnd={handleDragEnd}
                    onClick={() => onSelectItem(item)}
                    aria-pressed={isSelected}
                    aria-label={`Select ${item.name} for placement`}
                    className="flex w-full flex-col items-center justify-center rounded-xl p-3.5 cursor-grab active:cursor-grabbing focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-300 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-950"
                  >
                    {card}
                  </button>
                )}
              </motion.div>
            );
          })}
        </div>
      </div>
    </section>
  );
};
