import { useEffect, type ReactNode } from 'react';
import { applyTheme } from './theme';

/**
 * The app ships the dark theme only (feature 005, US6): the light/dark toggle was
 * removed, so this simply pins the document to dark. It stays a provider so the
 * router keeps the same mount shape.
 */
export function ThemeProvider({ children }: { children: ReactNode }) {
  useEffect(() => {
    applyTheme('dark');
  }, []);

  return <>{children}</>;
}
