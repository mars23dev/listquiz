import { describe, expect, it, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ThemeProvider } from '../theme/ThemeProvider';
import { ThemeToggle } from './ThemeToggle';

describe('theme toggle', () => {
  beforeEach(() => {
    window.localStorage.clear();
    document.documentElement.removeAttribute('data-theme');
  });

  afterEach(() => {
    window.localStorage.clear();
    document.documentElement.removeAttribute('data-theme');
  });

  it('flips the data-theme attribute and persists the choice', async () => {
    const user = userEvent.setup();
    render(
      <ThemeProvider>
        <ThemeToggle />
      </ThemeProvider>,
    );

    const start = document.documentElement.getAttribute('data-theme');
    expect(start === 'dark' || start === 'light').toBe(true);

    const button = screen.getByRole('button', { name: /switch to .* theme/i });
    await user.click(button);

    const next = document.documentElement.getAttribute('data-theme');
    expect(next).not.toBe(start);
    expect(window.localStorage.getItem('listquiz-theme')).toBe(next);
  });
});
