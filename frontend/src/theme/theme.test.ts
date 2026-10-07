import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { REQUIRED_VARIABLES } from './theme';

// Read the source stylesheet directly: the palette contract is about the file,
// not about what a CSS pipeline happens to emit.
const css = readFileSync(resolve(process.cwd(), 'src/index.css'), 'utf8');

/** The declarations inside the rule whose selector starts at `selector`. */
function block(selector: string): string {
  const start = css.indexOf(selector);
  expect(start, `selector ${selector} is present`).toBeGreaterThan(-1);
  const open = css.indexOf('{', start);
  const close = css.indexOf('}', open);
  return css.slice(open + 1, close);
}

function valueOf(source: string, name: string): string | undefined {
  return source.match(new RegExp(`${name}:\\s*([^;]+);`))?.[1].trim();
}

describe('theme palette', () => {
  it('declares every required variable under both dark and light', () => {
    const dark = block(':root[data-theme="dark"]');
    const light = block(':root[data-theme="light"]');

    for (const name of REQUIRED_VARIABLES) {
      expect(dark, `dark declares ${name}`).toContain(`${name}:`);
      expect(light, `light declares ${name}`).toContain(`${name}:`);
    }
  });

  it('keeps the team accents identical in both themes', () => {
    const dark = block(':root[data-theme="dark"]');
    const light = block(':root[data-theme="light"]');
    for (const accent of [
      '--lq-team-cyan',
      '--lq-team-rose',
      '--lq-team-amber',
      '--lq-team-emerald',
    ]) {
      expect(valueOf(light, accent)).toBe(valueOf(dark, accent));
    }
  });
});
