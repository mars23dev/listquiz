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
    mineRank: 4,
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

describe('TripmineBoard mine marker', () => {
  it('marks the mine position from the start for a player', () => {
    const state: TripmineState = {
      tripmineSide: 'random',
      currentSearchedRank: 1,
      tripmineHit: false,
      isCompleted: false,
      mineRank: 2,
      items: [
        { id: 'm1', name: 'A', value: '', numericValue: 0, trueRank: 0, isTripmine: false, isRevealed: false },
        { id: 'm2', name: 'B', value: '', numericValue: 0, trueRank: 0, isTripmine: false, isRevealed: false },
      ],
    };
    render(
      <TripmineBoard
        tripmineState={state}
        onPickItem={vi.fn()}
        isMyTurn={false}
        currentRole="view"
        teams={TEAMS}
      />,
    );

    expect(screen.getByTitle(/rank 2 \(mine position\)/i)).toBeInTheDocument();
  });
});

describe('TripmineBoard gamemaster visibility', () => {
  function stateWithHiddenValue(): TripmineState {
    const state = tripmineState();
    state.items[1].value = '2800';
    return state;
  }

  it('shows the gamemaster every value, even before a pick', () => {
    render(
      <TripmineBoard
        tripmineState={stateWithHiddenValue()}
        onPickItem={vi.fn()}
        isMyTurn={false}
        currentRole="gamemaster"
        teams={TEAMS}
      />,
    );

    expect(screen.getByText('2920')).toBeInTheDocument();
    expect(screen.getByText('2800')).toBeInTheDocument();
  });

  it('still hides an unpicked value from a player', () => {
    render(
      <TripmineBoard
        tripmineState={stateWithHiddenValue()}
        onPickItem={vi.fn()}
        isMyTurn={false}
        currentRole="view"
        teams={TEAMS}
      />,
    );

    expect(screen.getByText('2920')).toBeInTheDocument();
    expect(screen.queryByText('2800')).not.toBeInTheDocument();
  });
});

describe('TripmineBoard item count', () => {
  it('renders a twenty-item board', () => {
    const items = Array.from({ length: 20 }, (_, index) => ({
      id: `m${index}`,
      name: `Item ${index}`,
      value: '',
      numericValue: index,
      trueRank: index + 1,
      isTripmine: false,
      isRevealed: false,
    }));
    const state: TripmineState = {
      tripmineSide: 'highest',
      currentSearchedRank: 1,
      tripmineHit: false,
      isCompleted: false,
      mineRank: 20,
      items,
    };

    render(
      <TripmineBoard
        tripmineState={state}
        onPickItem={vi.fn()}
        isMyTurn={false}
        currentRole="view"
        teams={TEAMS}
      />,
    );

    expect(
      screen.getAllByRole('button', { name: /hidden or already picked/i }),
    ).toHaveLength(20);
  });

  it('shows a bar position for every item', () => {
    const make = (count: number) => ({
      tripmineSide: 'highest' as const,
      currentSearchedRank: 1,
      tripmineHit: false,
      isCompleted: false,
      mineRank: count,
      items: Array.from({ length: count }, (_, index) => ({
        id: `m${index}`,
        name: `Item ${index}`,
        value: '',
        numericValue: index,
        trueRank: index + 1,
        isTripmine: false,
        isRevealed: false,
      })),
    });

    const { rerender } = render(
      <TripmineBoard
        tripmineState={make(20)}
        onPickItem={vi.fn()}
        isMyTurn={false}
        currentRole="view"
        teams={TEAMS}
      />,
    );
    expect(screen.getByText('#20')).toBeInTheDocument();
    expect(screen.queryByText('#21')).not.toBeInTheDocument();

    rerender(
      <TripmineBoard
        tripmineState={make(10)}
        onPickItem={vi.fn()}
        isMyTurn={false}
        currentRole="view"
        teams={TEAMS}
      />,
    );
    expect(screen.getByText('#10')).toBeInTheDocument();
    expect(screen.queryByText('#11')).not.toBeInTheDocument();
  });

  it('shows the value column name with the values (feature 017)', () => {
    const state: TripmineState = {
      tripmineSide: 'highest',
      currentSearchedRank: 1,
      tripmineHit: false,
      isCompleted: false,
      mineRank: 10,
      targetValue: '1969',
      items: [
        { id: 'm1', name: 'A', value: '', numericValue: 0, trueRank: 0, isTripmine: false, isRevealed: false },
      ],
    };
    render(
      <TripmineBoard
        tripmineState={state}
        onPickItem={vi.fn()}
        isMyTurn={false}
        currentRole="view"
        teams={TEAMS}
        columnName="Year"
      />,
    );

    expect(screen.getByText('Year')).toBeInTheDocument();
    expect(screen.queryByText(/VALUE:/i)).not.toBeInTheDocument();
    // The column name comes first, then the value ("Year 1969").
    expect(screen.getByLabelText('Year 1969')).toBeInTheDocument();
  });
});
