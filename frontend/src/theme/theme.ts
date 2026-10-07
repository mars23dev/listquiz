// Theme model (T145, T146). Kept free of React so the rules are plain functions.

export type Theme = 'dark' | 'light';

export const THEME_STORAGE_KEY = 'listquiz-theme';

/** Every palette variable both themes must declare (checked by theme.test.ts). */
export const REQUIRED_VARIABLES = [
  '--lq-bg',
  '--lq-surface',
  '--lq-surface-muted',
  '--lq-text',
  '--lq-muted',
  '--lq-border',
  '--lq-team-cyan',
  '--lq-team-rose',
  '--lq-team-amber',
  '--lq-team-emerald',
  '--lq-status-good',
  '--lq-status-warn',
  '--lq-status-bad',
];

/** The operating system preference, defaulting to the draft's dark board. */
export function systemTheme(): Theme {
  if (typeof window !== 'undefined' && typeof window.matchMedia === 'function') {
    return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
  }
  return 'dark';
}

/** The locally stored override, or `null` when the visitor has not chosen one. */
export function readStoredTheme(): Theme | null {
  try {
    const value = window.localStorage.getItem(THEME_STORAGE_KEY);
    return value === 'dark' || value === 'light' ? value : null;
  } catch {
    return null;
  }
}

export function storeTheme(theme: Theme): void {
  try {
    window.localStorage.setItem(THEME_STORAGE_KEY, theme);
  } catch {
    // A browser that refuses storage simply keeps the in-memory choice.
  }
}

/** The theme to start with: the stored override, else the system preference. */
export function initialTheme(): Theme {
  return readStoredTheme() ?? systemTheme();
}

/** Apply a theme to the document root, which is what the CSS keys off. */
export function applyTheme(theme: Theme): void {
  document.documentElement.setAttribute('data-theme', theme);
}
