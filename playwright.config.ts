import { defineConfig, devices } from '@playwright/test';

/**
 * Two-context end-to-end harness (T151). It runs against a backend that is already
 * serving the built frontend, because the product has no standalone frontend
 * (Principle IV). Start it first:
 *
 *   npm --prefix frontend run build
 *   cargo run --manifest-path backend/Cargo.toml --release
 *
 * then: npx playwright test (after `npm i -D @playwright/test && npx playwright install`).
 */
export default defineConfig({
  testDir: './tests/e2e',
  timeout: 30_000,
  fullyParallel: false,
  reporter: 'list',
  use: {
    baseURL: process.env.LISTQUIZ_URL ?? 'http://localhost:8080',
    headless: true,
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
});
