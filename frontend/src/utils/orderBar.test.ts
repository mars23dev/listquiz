import { describe, expect, it } from 'vitest';
import { BAR_COLUMNS, positionForGap } from './orderBar';

const COLUMNS = Array.from({ length: BAR_COLUMNS }, (_, index) => index);

describe('positionForGap', () => {
  it('maps every column to the single gap 0 on an empty side', () => {
    for (const side of ['left', 'right'] as const) {
      for (const column of COLUMNS) {
        expect(positionForGap(side, column, 0)).toBe(0);
      }
    }
  });

  it('maps a side with n items to a gap in 0..=n', () => {
    for (const side of ['left', 'right'] as const) {
      for (let length = 0; length <= BAR_COLUMNS; length += 1) {
        for (const column of COLUMNS) {
          const gap = positionForGap(side, column, length);
          expect(gap).toBeGreaterThanOrEqual(0);
          expect(gap).toBeLessThanOrEqual(length);
        }
      }
    }
  });

  it('reaches both ends of a partly-filled left side', () => {
    // Left side with 2 items: the far columns are the outer gap, the column beside
    // the anchor is the gap after the last item.
    const gaps = COLUMNS.map((column) => positionForGap('left', column, 2));
    expect(gaps[0]).toBe(0);
    expect(gaps[4]).toBe(2);
  });

  it('clamps a right-side drop beyond the last item to the end gap', () => {
    // Right side with 2 items: columns past the items all mean "after the last".
    expect(positionForGap('right', 0, 2)).toBe(0);
    expect(positionForGap('right', 4, 2)).toBe(2);
  });

  it('never exceeds the side length even if the column is out of range', () => {
    expect(positionForGap('left', 99, 1)).toBe(1);
    expect(positionForGap('right', 99, 1)).toBe(1);
  });
});
