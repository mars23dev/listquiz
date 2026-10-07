import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react';
import { applyTheme, initialTheme, readStoredTheme, storeTheme, type Theme } from './theme';

interface ThemeContextValue {
  theme: Theme;
  setTheme: (theme: Theme) => void;
  toggleTheme: () => void;
}

const ThemeContext = createContext<ThemeContextValue | null>(null);

/**
 * Applies the theme attribute on first render, then keeps it and the stored
 * override in step (FR-063). State is held above the router, so switching pages
 * or roles never loses it.
 */
export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setTheme] = useState<Theme>(() => {
    const start = initialTheme();
    if (typeof document !== 'undefined') {
      applyTheme(start);
    }
    return start;
  });

  useEffect(() => {
    applyTheme(theme);
    storeTheme(theme);
  }, [theme]);

  const value = useMemo<ThemeContextValue>(
    () => ({
      theme,
      setTheme,
      toggleTheme: () => {
        setTheme((current) => (current === 'dark' ? 'light' : 'dark'));
      },
    }),
    [theme],
  );

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

export function useTheme(): ThemeContextValue {
  const context = useContext(ThemeContext);
  if (context) {
    return context;
  }
  // A component rendered on its own (a unit test, an isolated mount) still gets a
  // working toggle: it reads and writes the document attribute directly.
  return {
    theme: readStoredTheme() ?? initialTheme(),
    setTheme: (next) => {
      applyTheme(next);
      storeTheme(next);
    },
    toggleTheme: () => {
      const current =
        document.documentElement.getAttribute('data-theme') === 'light' ? 'light' : 'dark';
      const next = current === 'dark' ? 'light' : 'dark';
      applyTheme(next);
      storeTheme(next);
    },
  };
}
