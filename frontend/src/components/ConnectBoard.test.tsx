import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ConnectBoard } from './ConnectBoard';
import type { ConnectView } from '../types/session';

const VIEW: ConnectView = {
  names: [
    { itemId: 'a', name: 'Alpha', iconName: 'BookOpen' },
    { itemId: 'b', name: 'Bravo', iconName: null },
  ],
  values: [
    { itemId: 'b', display: '200' },
    { itemId: 'a', display: '100' },
  ],
  matched: [],
  pending: { nameItemId: null, valueItemId: null },
};

describe('ConnectBoard', () => {
  it('shows names on the left and values on the right', () => {
    render(
      <ConnectBoard
        view={VIEW}
        interactive
        onPickName={vi.fn()}
        onPickValue={vi.fn()}
        onConfirm={vi.fn()}
      />,
    );

    expect(screen.getByText('Alpha')).toBeInTheDocument();
    expect(screen.getByText('Bravo')).toBeInTheDocument();
    expect(screen.getByText('100')).toBeInTheDocument();
    expect(screen.getByText('200')).toBeInTheDocument();
    // The middle box starts empty and Confirm is disabled.
    expect(screen.getByText(/pick a name/i)).toBeInTheDocument();
    expect(screen.getByText(/pick a value/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /confirm/i })).toBeDisabled();
  });

  it('stages a name and a value and confirms', async () => {
    const onPickName = vi.fn();
    const onPickValue = vi.fn();
    const onConfirm = vi.fn();
    const user = userEvent.setup();

    const { rerender } = render(
      <ConnectBoard
        view={VIEW}
        interactive
        onPickName={onPickName}
        onPickValue={onPickValue}
        onConfirm={onConfirm}
      />,
    );

    await user.click(screen.getByRole('button', { name: /alpha/i }));
    expect(onPickName).toHaveBeenCalledWith('a');

    rerender(
      <ConnectBoard
        view={{ ...VIEW, pending: { nameItemId: 'a', valueItemId: 'a' } }}
        interactive
        onPickName={onPickName}
        onPickValue={onPickValue}
        onConfirm={onConfirm}
      />,
    );

    const confirm = screen.getByRole('button', { name: /confirm/i });
    expect(confirm).toBeEnabled();
    await user.click(confirm);
    expect(onConfirm).toHaveBeenCalled();
  });
});
