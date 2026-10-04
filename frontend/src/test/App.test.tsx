import { describe, expect, it, beforeEach, afterEach, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import App from '../App';

function setPath(path: string) {
  window.history.pushState({}, '', path);
}

describe('App shell', () => {
  beforeEach(() => {
    setPath('/');
    // The setup wizard loads the default lists, and list management loads the
    // folders; both answers have to match their endpoint's shape.
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL) => {
        const url = String(input);
        if (url === '/api/folders') {
          return new Response(
            JSON.stringify([
              { id: 'default', name: 'Default Lists', isDefault: true, locked: false },
            ]),
            { status: 200, headers: { 'Content-Type': 'application/json' } },
          );
        }
        return new Response(
          JSON.stringify({
            lists: [
              {
                id: 'default_world_history',
                title: 'World History & Inventions',
                description: '',
                orderDirection: 'low_to_high',
                itemCount: 12,
                isDefault: true,
                columns: [{ id: 'year', name: 'Year' }],
              },
            ],
          }),
          { status: 200, headers: { 'Content-Type': 'application/json' } },
        );
      }),
    );
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('renders the landing page at the root path', () => {
    render(<App />);

    const heading = screen.getByRole('heading', { level: 1 });
    expect(heading.textContent?.replace(/\s+/g, '')).toMatch(/^listquiz$/i);
  });

  it('navigates to list management without a page reload', async () => {
    render(<App />);

    screen.getByRole('button', { name: /manage lists/i }).click();

    expect(await screen.findByRole('heading', { name: /list management/i })).toBeInTheDocument();
    expect(window.location.pathname).toBe('/lists');
  });

  it('navigates to the setup wizard', async () => {
    render(<App />);

    screen.getByRole('button', { name: /host a quiz/i }).click();

    expect(
      await screen.findByRole('heading', { name: /step 1: participants & password/i }),
    ).toBeInTheDocument();
    expect(window.location.pathname).toBe('/setup');
  });

  it('routes a deep session link to the session view', () => {
    setPath('/lq-8fj2wkd9/p2');

    render(<App />);

    expect(screen.getByText('lq-8fj2wkd9 / p2')).toBeInTheDocument();
  });

  it('treats a bare session link as a player view, since the server redirects it', () => {
    setPath('/lq-8fj2wkd9');

    render(<App />);

    expect(screen.getByText('lq-8fj2wkd9 / p1')).toBeInTheDocument();
  });

  it('does not offer a demo session, because the product ships no session list', () => {
    render(<App />);

    expect(screen.queryByRole('button', { name: /quick start/i })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /demo/i })).not.toBeInTheDocument();
  });

  it('shows no timer anywhere on the landing page', () => {
    render(<App />);

    expect(screen.queryByText(/\d+:\d\d/)).not.toBeInTheDocument();
    expect(document.body.textContent).not.toMatch(/countdown/i);
  });

  it('uses history paths rather than hash fragments', async () => {
    render(<App />);

    screen.getByRole('button', { name: /manage lists/i }).click();
    await screen.findByRole('heading', { name: /list management/i });

    expect(window.location.hash).toBe('');
  });
});