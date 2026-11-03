// Team-colored item borders (feature 004, US2).
//
// A placed (Order) or revealed (Tripmine) item is framed with the color of the team
// that acted on it, taken from the team's own scoreboard color. This one helper
// keeps the border vocabulary identical across both games and gives the reveal case
// a single neutral fallback.

export interface TeamColorSource {
  id: string;
  primaryHex: string;
  accentGlow?: string;
}

export interface TeamBorderStyle {
  borderColor: string;
  boxShadow?: string;
}

/** Slate-600: the neutral frame used when no team owns the item (or on reveal). */
export const NEUTRAL_BORDER = '#475569';

export function neutralBorderStyle(): TeamBorderStyle {
  return { borderColor: NEUTRAL_BORDER };
}

/**
 * The border style for an item placed or picked by `teamId`. Unknown or absent ids
 * (a pool item, or a team that is no longer in the session) get the neutral frame.
 */
export function teamBorderStyle(
  teamId: string | null | undefined,
  teams: TeamColorSource[],
): TeamBorderStyle {
  const team = teamId ? teams.find((candidate) => candidate.id === teamId) : undefined;
  if (!team) return neutralBorderStyle();
  return {
    borderColor: team.primaryHex,
    ...(team.accentGlow ? { boxShadow: `0 8px 18px -6px ${team.accentGlow}` } : {}),
  };
}
