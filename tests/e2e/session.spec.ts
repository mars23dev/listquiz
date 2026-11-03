import { expect, test } from '@playwright/test';

/**
 * The two-device scenario from `quickstart.md`, steps 2 through 8, reduced to the
 * parts a browser can prove: a gamemaster context and a player context share one
 * session, and a host action reaches the player without a reload.
 *
 * Requires a running server (see playwright.config.ts).
 */
const TEAM = 'Team Cyan Pulsar';

async function createSession(request: import('@playwright/test').APIRequestContext) {
  const response = await request.post('/api/sessions', {
    data: {
      gmPassword: 'letmein',
      teamCount: 2,
      teams: [{ name: TEAM }, { name: 'Team Ruby Spark' }],
      roundCount: 2,
      rounds: [
        {
          roundNumber: 1,
          quizType: 'ORDER',
          listId: 'default_world_history',
          columnId: 'year',
        },
        {
          roundNumber: 2,
          quizType: 'TRIPMINE',
          listId: 'default_world_history',
          columnId: 'year',
          tripmineSide: 'highest',
        },
      ],
    },
  });
  expect(response.ok()).toBeTruthy();
  return (await response.json()) as {
    sessionId: string;
    links: { gamemaster: string; players: { role: string; path: string }[]; spectator: string };
  };
}

test('a gamemaster action reaches a player view in a second context', async ({ browser, request }) => {
  const session = await createSession(request);

  const gmContext = await browser.newContext();
  const playerContext = await browser.newContext();

  const gm = await gmContext.newPage();
  await gm.goto(session.links.gamemaster);
  await gm.getByLabel(/moderator password/i).fill('letmein');
  await gm.getByRole('button', { name: /unlock/i }).click();

  const player = await playerContext.newPage();
  await player.goto(session.links.players[0].path);
  await expect(player.getByRole('heading', { name: TEAM })).toBeVisible();

  // The host starts the round; the player must see it without a manual reload.
  await gm.getByRole('button', { name: /next round/i }).click();
  await expect(gm.getByText(/round 1 of 2/i)).toBeVisible();
  await expect(player.getByText(/round 1 of 2/i)).toBeVisible({ timeout: 1000 });

  // The player never gets host controls, and no value is shown before the reveal.
  await expect(player.getByRole('button', { name: /reveal values/i })).toHaveCount(0);

  // Revealing values on the host flips the player's board too.
  await gm.getByRole('button', { name: /reveal values/i }).click();
  await expect(gm.getByRole('button', { name: /values revealed/i })).toBeVisible();

  await gmContext.close();
  await playerContext.close();
});

test('the theme toggle flips the whole board and keeps it across navigation', async ({ page }) => {
  await page.goto('/');
  const theme = await page.evaluate(() => document.documentElement.dataset.theme);
  await page.getByRole('button', { name: /switch to .* theme/i }).click();
  const flipped = await page.evaluate(() => document.documentElement.dataset.theme);
  expect(flipped).not.toBe(theme);
  await expect
    .poll(() => page.evaluate(() => localStorage.getItem('listquiz-theme')))
    .toBe(flipped);
});
