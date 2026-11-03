# Research: Quick Game Connect And Random Lists

## D-1801: Connect is offered by the landing page and accepted end to end

**Decision**: Add a Connect quick-game choice on `Landing.tsx` and widen the quick game-type union from
`'ORDER' | 'TRIPMINE'` to include `'CONNECT'` in `App.tsx` and `api/client.ts`. The backend already
handles Connect in `quick_game::build_request`.

**Rationale**: The backend supports Connect quick games; only the frontend entry point omitted it
(FR-001, FR-002). Widening the existing union is the smallest change (Principle VII).

**Alternatives considered**: a new endpoint (unnecessary — the existing quick endpoint accepts CONNECT).

## D-1802: The quick game list is chosen at random among eligible lists

**Decision**: Change `quick_game::build_request` to take an rng and pick uniformly at random among the
lists that fit the game type (enough items with values and at least one column), instead of the first
fitting list. The route passes `&mut rand::rng()`.

**Rationale**: The user wants variety between quick games (FR-003, FR-007). The fitting-list predicate
already exists; collecting the eligible lists and choosing one at random is the smallest change
(Principle VII). Passing an rng keeps the rule deterministic and testable with a seed (Principle I/II).

**Alternatives considered**: picking the first list (the current bug); shuffling all lists and taking the
first (equivalent but more work).

## D-1803: Single fit and no fit are deterministic and explicit

**Decision**: When exactly one list fits, use it; when none fits, return `not_ready` with a clear message.

**Rationale**: The existing error path already covers "no list large enough"; with a random choice the
single-fit case must still be deterministic (FR-005, FR-006). No new behavior beyond the selection.

**Alternatives considered**: always erroring when fewer than two fit (wrong — one fitting list is valid).

## D-1804: No other quick game behavior changes

**Decision**: Keep one team, one round, immediate start, and the player view unchanged.

**Rationale**: Only the list choice and the offered types change (FR-008; Principle VII).

**Alternatives considered**: none needed.

## D-1805: Tests cover the new rules

**Decision**: Add `cargo test` cases for the random choice (seeded rng varies across seeds; a single
eligible list is always chosen; no eligible list errors) and a `vitest` case that the landing page offers
Connect and the quick game still starts.

**Rationale**: The rules are business logic the constitution requires to be tested (Principles I, II).

**Alternatives considered**: manual checks only (rejected — the rules are unit-testable).
