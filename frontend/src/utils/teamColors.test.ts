import { describe, expect, it } from 'vitest';
import { NEUTRAL_BORDER, neutralBorderStyle, teamBorderStyle } from './teamColors';

const TEAMS = [
  { id: 'team_1', primaryHex: '#06b6d4', accentGlow: 'rgba(6, 182, 212, 0.45)' },
  { id: 'team_2', primaryHex: '#f43f5e', accentGlow: 'rgba(244, 63, 94, 0.45)' },
];

describe('teamBorderStyle', () => {
  it('frames an item with its team color', () => {
    const style = teamBorderStyle('team_2', TEAMS);
    expect(style.borderColor).toBe('#f43f5e');
    expect(style.boxShadow).toContain('rgba(244, 63, 94, 0.45)');
  });

  it('falls back to neutral for an unknown or absent team', () => {
    expect(teamBorderStyle(null, TEAMS).borderColor).toBe(NEUTRAL_BORDER);
    expect(teamBorderStyle('team_9', TEAMS).borderColor).toBe(NEUTRAL_BORDER);
    expect(teamBorderStyle(undefined, TEAMS).borderColor).toBe(NEUTRAL_BORDER);
  });

  it('offers a neutral style for the reveal', () => {
    expect(neutralBorderStyle().borderColor).toBe(NEUTRAL_BORDER);
  });
});
