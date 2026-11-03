import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { BottomTeams } from './BottomTeams';
import type { Team } from '../types/quiz';

const TEAMS: Team[] = [
  {
    id: 'team_1',
    name: 'One',
    colorName: 'cyan',
    primaryHex: '#06b6d4',
    accentGlow: 'rgba(6, 182, 212, 0.45)',
    score: 10,
    hearts: 2,
    isActiveTurn: true,
    turnIndex: 0,
  },
];

describe('BottomTeams hearts', () => {
  it('shows the heart indicator by default', () => {
    render(<BottomTeams teams={TEAMS} activeTeamCount={1} />);
    expect(screen.getByTitle(/Hearts remaining/i)).toBeInTheDocument();
  });

  it('hides the heart indicator when showHearts is false', () => {
    render(<BottomTeams teams={TEAMS} activeTeamCount={1} showHearts={false} />);
    expect(screen.queryByTitle(/Hearts remaining/i)).not.toBeInTheDocument();
  });
});
