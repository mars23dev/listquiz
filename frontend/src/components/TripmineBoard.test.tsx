import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { TripmineBoard } from './TripmineBoard';
import type { TripmineState } from '../types/quiz';

const TEAMS = [
  { id: 'team_1', primaryHex: '#06b6d4' },
  { id: 'team_2', primaryHex: '#f43f5e' },
];

function tripmineState(): TripmineState {
  return {
    tripmineSide: 'highest',
    currentSearchedRank: 5,
    tripmineHit: false,
    isCompleted: false,
    items: [
      {
        id: 'm1',
        name: 'Picked',
        value: '2920',
        numericValue: 2920,
        trueRank: 10,
        isTripmine: false,
        isRevealed: true,
        revealedByTeamId: 'team_2',
        pointsAwarded: 5,
      },
      {
        id: 'm2',
        name: 'Hidden',
        value: '',
        numericValue: 0,
        trueRank: 4,
        isTripmine: false,
        isRevealed: false,
      },
    ],
  };
}

describe('TripmineBoard team-colored borders', () => {
  it('frames a revealed item with the picking team color', () => {
    render(
      <TripmineBoard
        tripmineState={tripmineState()}
        onPickItem={vi.fn()}
        isMyTurn={false}
        currentRole="view"
        teams={TEAMS}
      />,
    );

    const picked = screen.getByRole('button', { name: /Picked, hidden or already picked/i });
    expect(picked.style.borderColor).toBe('rgb(244, 63, 94)');

    const hidden = screen.getByRole('button', { name: /Hidden, hidden or already picked/i });
    expect(hidden.style.borderColor).toBe('');
  });
});
