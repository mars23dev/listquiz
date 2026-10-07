// Provenance: item, list, team, and role types copied from
// the reference frontend's types/quiz.ts. That draft is a read-only reference (constitution
// v2.1.1) and is never imported or read at build or run time; this file is the only
// copy.
//
// Corrections against the draft:
//   - the draft's GameSessionState is gone: it carries the timer the spec removes
//     (FR-021) and it models client-side state the backend now owns. The
//     server-shaped state lives in ./session.ts.
//
// Additions from data-model.md (T027):
//   - Team.turnIndex, so turn order survives a round change (FR-019).
//   - RoundConfig.tripmineSide, the name the REST contract uses; the draft's
//     `tripmineType` was renamed so one name travels the whole stack.
//   - ListItemValue.wasDefaulted, so the list editor can show which values came
//     from CSV import defaults (FR-060).
//   - ScoreEvent, the audit row every score change appends (FR-023, FR-046).

export interface QuizItem {
  id: string;
  name: string;
  value: string; // Formatted value (e.g. "1969", "$2.26 B", "330 m")
  numericValue: number; // For chronological/numerical comparison
  category?: string;
  hint?: string;
  iconName?: string;
  customIconUrl?: string;
  /** The team that placed this item on the Order bar, if any (US2). */
  placedByTeamId?: string | null;
}

export interface QuizCategory {
  id: string;
  title: string;
  subtitle?: string;
  valueLabel?: string;
  anchorItem: QuizItem;
  poolItems: QuizItem[];
}

export interface ListColumn {
  id: string;
  name: string;
  type: 'number' | 'text';
}

export interface ListRowItem {
  id: string;
  iconName?: string;
  customIconUrl?: string;
  name: string;
  values: Record<string, string>; // columnId -> display string
  numericValues: Record<string, number>; // columnId -> numeric representation
}

/** One editable value cell. `wasDefaulted` marks a value filled in by a default. */
export interface ListItemValue {
  itemId: string;
  columnId: string;
  displayValue: string;
  numericValue: number;
  wasDefaulted: boolean;
}

export interface ManagedList {
  id: string;
  title: string;
  description?: string;
  orderDirection: 'low_to_high' | 'high_to_low';
  columns: ListColumn[]; // First value column is columns[0], further optional columns follow
  items: ListRowItem[]; // Minimum 11 items needed for a full 11-slot round
  createdAt: number;
  updatedAt: number;
  isDefault?: boolean;
}

export interface Team {
  id: string;
  name: string;
  colorName: 'cyan' | 'rose' | 'amber' | 'emerald';
  primaryHex: string;
  accentGlow: string;
  score: number;
  hearts: number; // 0 to 3
  isActiveTurn: boolean;
  turnIndex: number; // Position in the turn rotation, 0-based (FR-019)
  members?: string[];
}

/** Why a score or heart change happened. Only the gamemaster sees the reason. */
export type ScoreEventKind =
  | 'correct_placement'
  | 'wrong_placement'
  | 'tripmine_hit'
  | 'round_end'
  | 'manual_adjustment'
  | 'turn_pass';

/** One row of the session's score audit trail. */
export interface ScoreEvent {
  id: number;
  sessionId: string;
  roundNumber: number;
  teamId: string;
  kind: ScoreEventKind;
  pointsDelta: number;
  heartsDelta: number;
  itemId?: string;
  createdAt: number;
}

export type QuizType = 'ORDER' | 'TRIPMINE' | 'CONNECT';

export interface RoundConfig {
  roundNumber: number;
  quizType: QuizType;
  listId: string;
  selectedColumnId: string;
  tripmineSide?: 'highest' | 'lowest';
}

export interface TripmineItem {
  id: string;
  name: string;
  value: string;
  numericValue: number;
  iconName?: string;
  customIconUrl?: string;
  trueRank: number; // 1 (lowest) to 10 (highest)
  isTripmine: boolean;
  isRevealed: boolean;
  revealedByTeamId?: string;
  pointsAwarded?: number;
}

export interface TripmineState {
  tripmineSide: 'highest' | 'lowest' | 'random'; // gamemaster choice: highest, lowest, or random (Hard)
  currentSearchedRank: number; // 1..10
  items: TripmineItem[]; // 10 items in grid
  tripmineHit: boolean;
  tripmineHitByTeamId?: string;
  isCompleted: boolean;
  /** The mined item's value, shown as a public hint (FR-015). */
  targetValue?: string | null;
  /** The rank (1..=10) of the mine, shown to every role as a position only (010). */
  mineRank: number;
}

export interface SessionConfig {
  sessionId: string;
  gmPassword?: string;
  teamCount: number;
  teams: Team[];
  rounds: RoundConfig[];
  currentRoundIndex: number;
  createdAt: number;
}

export type UserRole = 'gamemaster' | 'p1' | 'p2' | 'p3' | 'p4' | 'view';

export type DropSide = 'left' | 'right';

export interface DragItemPayload {
  item: QuizItem;
  source: 'pool' | 'left' | 'right';
  sourceIndex?: number;
}
