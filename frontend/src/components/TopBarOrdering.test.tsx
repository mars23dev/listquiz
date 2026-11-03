import { describe, expect, it, vi } from 'vitest';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { TopBarOrdering } from './TopBarOrdering';
import type { QuizItem } from '../types/quiz';

const ANCHOR: QuizItem = { id: 'anchor', name: 'Anchor', value: '15', numericValue: 15 };

function renderBar(overrides: Partial<Parameters<typeof TopBarOrdering>[0]> = {}) {
  const handlers = {
    onChooseSlot: vi.fn(),
    onConfirmPlacement: vi.fn(),
    onCancelPlacement: vi.fn(),
  };
  render(
    <TopBarOrdering
      anchorItem={ANCHOR}
      leftItems={[]}
      rightItems={[]}
      pendingItem={null}
      hasPendingItem
      interactive
      draggingItem={null}
      onChooseSlot={handlers.onChooseSlot}
      onDropToPending={() => {}}
      onConfirmPlacement={handlers.onConfirmPlacement}
      onCancelPlacement={handlers.onCancelPlacement}
      onDragStartItem={() => {}}
      onDragEndItem={() => {}}
      revealAllValues={false}
      {...overrides}
    />,
  );
  return handlers;
}

describe('order bar keyboard operation', () => {
  it('shows the value column name with the values (feature 017)', () => {
    renderBar({ columnName: 'Area (km²)' });

    expect(screen.getByText('Area (km²)')).toBeInTheDocument();
  });

  it('shows a revealed slot value in the middle of the card, like the anchor (feature 017)', () => {
    const item: QuizItem = { id: 'i1', name: 'Alpha', value: '100', numericValue: 100 };
    renderBar({ leftItems: [item], revealAllValues: true, columnName: 'Area (km²)' });

    // The value and its column name sit with the name in the card's middle content,
    // not in a footer line.
    const name = screen.getByText('Alpha');
    const middle = name.parentElement as HTMLElement;
    expect(within(middle).getByText('100')).toBeInTheDocument();
    expect(within(middle).getByText('Area (km²)')).toBeInTheDocument();
  });

  it('every bar slot is a button and a slot is chosen with the keyboard', async () => {
    const handlers = renderBar();
    const user = userEvent.setup();

    expect(screen.getAllByRole('button', { name: /slot \d/i })).toHaveLength(10);

    const slot = screen.getByRole('button', { name: /place the selected item in left slot 1/i });
    slot.focus();
    expect(document.activeElement).toBe(slot);
    await user.keyboard('{Enter}');

    expect(handlers.onChooseSlot).toHaveBeenCalledWith('left', 0);
  });

  it('confirms or cancels a pending placement by keystroke', async () => {
    const pendingItem = {
      item: { id: 'a', name: 'Alpha', value: '', numericValue: 0 },
      side: 'left' as const,
      slotIndex: 0,
    };
    const handlers = renderBar({ pendingItem });
    const user = userEvent.setup();

    const confirm = screen.getByRole('button', { name: /confirm placement/i });
    confirm.focus();
    await user.keyboard('{Enter}');
    expect(handlers.onConfirmPlacement).toHaveBeenCalled();

    const cancel = screen.getByRole('button', { name: /cancel position/i });
    cancel.focus();
    await user.keyboard('{Enter}');
    expect(handlers.onCancelPlacement).toHaveBeenCalled();
  });
});

describe('order bar team-colored borders', () => {
  const TEAMS = [
    { id: 'team_1', primaryHex: '#06b6d4' },
    { id: 'team_2', primaryHex: '#f43f5e' },
  ];
  const PLACED: QuizItem = {
    id: 'x',
    name: 'Placed',
    value: '10',
    numericValue: 10,
    placedByTeamId: 'team_2',
  };

  it('frames a placed item with the placing team color', () => {
    renderBar({ leftItems: [PLACED], teams: TEAMS, hasPendingItem: false });
    const slot = screen.getByRole('button', { name: /holds Placed/i });
    expect(slot.style.borderColor).toBe('rgb(244, 63, 94)');
  });

  it('frames the pending preview with the active team color', () => {
    renderBar({
      pendingItem: { item: { id: 'a', name: 'Alpha', value: '', numericValue: 0 }, side: 'left', slotIndex: 0 },
      teams: TEAMS,
      activeTeamId: 'team_1',
    });
    const slot = screen.getByRole('button', { name: /holds Alpha/i });
    expect(slot.style.borderColor).toBe('rgb(6, 182, 212)');
  });

  it('uses a neutral border when values are revealed', () => {
    renderBar({ leftItems: [PLACED], teams: TEAMS, neutralBorders: true, hasPendingItem: false });
    const slot = screen.getByRole('button', { name: /holds Placed/i });
    expect(slot.style.borderColor).toBe('rgb(71, 85, 105)');
  });

  it('keeps the team color for the gamemaster who merely sees values', () => {
    renderBar({
      leftItems: [PLACED],
      teams: TEAMS,
      revealAllValues: true,
      neutralBorders: false,
      hasPendingItem: false,
    });
    const slot = screen.getByRole('button', { name: /holds Placed/i });
    expect(slot.style.borderColor).toBe('rgb(244, 63, 94)');
  });
});
