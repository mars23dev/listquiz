import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { TripmineBoard } from './TripmineBoard';
import type { TripmineState } from '../types/quiz';

const STATE: TripmineState = {
  tripmineSide: 'highest',
  currentSearchedRank: 1,
  tripmineHit: false,
  isCompleted: false,
  items: [
    {
      id: 'a',
      name: 'Alpha',
      value: '10',
      numericValue: 10,
      trueRank: 1,
      isTripmine: false,
      isRevealed: false,
    },
    {
      id: 'b',
      name: 'Bravo',
      value: '20',
      numericValue: 20,
      trueRank: 10,
      isTripmine: true,
      isRevealed: false,
    },
  ],
};

describe('tripmine grid keyboard operation', () => {
  it('reaches a cell and picks it with the keyboard', async () => {
    const onPickItem = vi.fn();
    const user = userEvent.setup();
    render(
      <TripmineBoard
        tripmineState={STATE}
        onPickItem={onPickItem}
        isMyTurn
        currentRole="gamemaster"
      />,
    );

    const cell = screen.getByRole('button', { name: /pick alpha/i });
    cell.focus();
    expect(document.activeElement).toBe(cell);

    await user.keyboard('{Enter}');
    expect(onPickItem).toHaveBeenCalledWith(STATE.items[0]);
  });

  it('renders no hidden in-round side switch', () => {
    render(
      <TripmineBoard
        tripmineState={STATE}
        onPickItem={() => {}}
        isMyTurn
        currentRole="gamemaster"
      />,
    );
    expect(screen.queryByRole('button', { name: /flip rule/i })).not.toBeInTheDocument();
  });
});
