import { describe, expect, it, vi } from 'vitest';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { GameEndDialog } from './GameEndDialog';

const TEAMS = [
  { id: 'team_1', name: 'Cyan Pulsar', score: 30, primaryHex: '#06b6d4' },
  { id: 'team_2', name: 'Ruby Spark', score: 70, primaryHex: '#f43f5e' },
];

describe('GameEndDialog', () => {
  it('ranks teams by points and shows points only', () => {
    render(<GameEndDialog teams={TEAMS} onDismiss={() => {}} />);

    const dialog = screen.getByRole('dialog');
    const entries = within(dialog).getAllByRole('listitem');
    expect(entries[0]).toHaveTextContent('Ruby Spark');
    expect(entries[0]).toHaveTextContent('70 pts');
    expect(entries[1]).toHaveTextContent('Cyan Pulsar');
    expect(entries[1]).toHaveTextContent('30 pts');

    expect(within(dialog).queryByText(/heart/i)).not.toBeInTheDocument();
  });

  it('dismisses by keyboard', async () => {
    const onDismiss = vi.fn();
    render(<GameEndDialog teams={TEAMS} onDismiss={onDismiss} />);
    const user = userEvent.setup();

    const button = screen.getByRole('button', { name: /back to home/i });
    button.focus();
    await user.keyboard('{Enter}');

    expect(onDismiss).toHaveBeenCalled();
  });
});
