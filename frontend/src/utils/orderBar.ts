/**
 * Order bar geometry (feature 003).
 *
 * A bar side is a contiguous run of items next to the anchor, not a set of independent
 * slots. A new item is dropped into a gap: before the first item, between two items,
 * or after the last item. A side with n items therefore has n + 1 gaps, numbered
 * `0..=n`, and that gap is the insertion position the server expects.
 *
 * The bar still renders a fixed five columns per side (the draft's layout): the left
 * side packs its items against the anchor and the right side packs them from the
 * anchor outward. `positionForGap` translates the column a player drops on into the
 * matching gap, so the client never sends a position the side does not have.
 */

export type BarSideName = 'left' | 'right';

/** Columns rendered per side of the anchor. */
export const BAR_COLUMNS = 5;

/**
 * The gap (insertion position, `0..=sideLength`) a dropped `column` maps to on the
 * named side. The left side is right-aligned against the anchor; the right side is
 * left-aligned. The result is clamped so a drop can never name a position outside the
 * side's `0..=sideLength` range.
 */
export function positionForGap(
  side: BarSideName,
  column: number,
  sideLength: number,
): number {
  const length = Math.max(0, Math.min(sideLength, BAR_COLUMNS));
  const raw = side === 'right' ? column : column - (BAR_COLUMNS - length) + 1;
  return Math.max(0, Math.min(raw, length));
}
