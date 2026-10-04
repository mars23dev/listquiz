// The session state the backend serves, mirroring the Rust DTOs in
// backend/src/app_state.rs field for field, per
// specs/001-listquiz-game/contracts/realtime.md. If the contract changes, this file
// and that one change together.
import type { UserRole } from './quiz';

export interface TeamView {
  id: string;
  name: string;
  color: string;
  primaryHex: string;
  accentGlow: string;
  score: number;
  hearts: number;
  onTurn: boolean;
}

export interface OrderItem {
  itemId: string;
  name: string;
  /** Null for a player until the gamemaster reveals values (FR-027). */
  value: string | null;
  revealed: boolean;
}

export interface AnchorItem {
  itemId: string;
  name: string;
  /** Public by design: FR-026 requires the anchor to show its value. */
  value: string;
}

export interface PendingPlacement {
  itemId: string;
  name: string;
  side: 'left' | 'right';
  slotIndex: number;
  teamId: string;
}

export interface OrderView {
  anchor: AnchorItem;
  pool: OrderItem[];
  left: OrderItem[];
  right: OrderItem[];
  pending: PendingPlacement | null;
}

export interface TripmineItem {
  itemId: string;
  name: string;
  value: string | null;
  revealed: boolean;
  /** Only true for an item this view is allowed to know the answer for. */
  isTripmine: boolean;
  pointsAwarded: number | null;
}

export interface TripmineView {
  side: 'highest' | 'lowest';
  searchedRank: number;
  hit: boolean;
  completed: boolean;
  items: TripmineItem[];
}

export interface RoundSummary {
  roundNumber: number;
  quizType: 'ORDER' | 'TRIPMINE';
  tripmineSide: 'highest' | 'lowest' | null;
  listTitle: string;
  listId: string;
  columnId: string;
  columnName: string;
}

export interface PlayerLink {
  role: string;
  path: string;
  hasTeam: boolean;
}

export interface SessionLinks {
  gamemaster: string;
  players: PlayerLink[];
  spectator: string;
}

export interface SessionState {
  revision: number;
  sessionId: string;
  status: 'setup' | 'running' | 'finished';
  teamCount: number;
  roundCount: number;
  currentRound: number;
  muted: boolean;
  revealValues: boolean;
  lastMessage: string | null;
  round: RoundSummary | null;
  teams: TeamView[];
  order: OrderView | null;
  tripmine: TripmineView | null;
  links: SessionLinks;
}

export interface CreateSessionResponse {
  sessionId: string;
  links: SessionLinks;
}

export type ApiErrorCode =
  | 'bad_request'
  | 'unauthorized'
  | 'not_found'
  | 'conflict'
  | 'too_small'
  | 'too_large'
  | 'not_ready'
  | 'frontend_missing';

export interface ApiErrorBody {
  error: { code: ApiErrorCode; message: string };
}

export interface RoleQuery {
  role: UserRole;
}

// --- Lists (contracts/rest-api.md) ---------------------------------------

export interface ListColumnSummary {
  id: string;
  name: string;
}

export interface ListSummary {
  id: string;
  title: string;
  description: string;
  orderDirection: 'low_to_high' | 'high_to_low';
  columns: ListColumnSummary[];
  itemCount: number;
  isDefault: boolean;
}

export interface ListItemValueView {
  display: string;
  numeric: number;
  /** True when a CSV import filled this value in (FR-060). */
  wasDefaulted: boolean;
}

export interface ListItemView {
  id: string;
  name: string;
  iconName: string | null;
  iconImage: string | null;
  position: number;
  values: Record<string, ListItemValueView>;
}

export interface ListDetail extends ListSummary {
  folderId: string;
  items: ListItemView[];
}

export interface ImportReport {
  rowCount: number;
  defaultedRows: { position: number; raw: string }[];
  warnings: string[];
}