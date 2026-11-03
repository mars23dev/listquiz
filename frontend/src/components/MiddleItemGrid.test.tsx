import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MiddleItemGrid } from './MiddleItemGrid';
import type { QuizItem } from '../types/quiz';

const ITEMS: QuizItem[] = [
  { id: 'a', name: 'Alpha', value: '10', numericValue: 10 },
  { id: 'b', name: 'Bravo', value: '20', numericValue: 20 },
];

function renderGrid(onSelectItem = vi.fn()) {
  render(
    <MiddleItemGrid
      items={ITEMS}
      placedIds={new Set()}
      onReturnToPool={() => {}}
      onDragStartItem={() => {}}
      onDragEndItem={() => {}}
      revealAllValues={false}
      onSelectItem={onSelectItem}
      selectedItemId={null}
    />,
  );
  return onSelectItem;
}

describe('pool grid keyboard operation', () => {
  it('focuses every pool item and selects one with Enter', async () => {
    const onSelectItem = renderGrid();
    const user = userEvent.setup();

    const alpha = screen.getByRole('button', { name: /select alpha/i });
    alpha.focus();
    expect(document.activeElement).toBe(alpha);

    await user.keyboard('{Enter}');
    expect(onSelectItem).toHaveBeenCalledWith(ITEMS[0]);
  });

  it('marks the selected item with aria-pressed', () => {
    render(
      <MiddleItemGrid
        items={ITEMS}
        placedIds={new Set()}
        onReturnToPool={() => {}}
        onDragStartItem={() => {}}
        onDragEndItem={() => {}}
        revealAllValues={false}
        onSelectItem={() => {}}
        selectedItemId="a"
      />,
    );
    expect(screen.getByRole('button', { name: /select alpha/i })).toHaveAttribute(
      'aria-pressed',
      'true',
    );
  });
});
