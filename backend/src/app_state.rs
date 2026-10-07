use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tokio::sync::broadcast;

use crate::errors::{ApiError, ApiResult};

/// In-process pub/sub for live session updates (D-005). One channel carries
/// `(session_id, revision)`; every subscriber filters for its own session, so a
/// ten-phone room costs ten small receivers.
#[derive(Clone)]
pub struct Broadcaster {
    tx: broadcast::Sender<Update>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Update {
    pub session_id: String,
    pub revision: i64,
}

impl Default for Broadcaster {
    fn default() -> Self {
        Self::new()
    }
}

impl Broadcaster {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(256);
        Broadcaster { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Update> {
        self.tx.subscribe()
    }

    pub fn receiver_count(&self) -> usize {
        self.tx.receiver_count()
    }

    /// Announce a state change for `session_id` at the revision the database just
    /// recorded, so the stream never reports a number `GET /state` does not.
    pub fn publish(&self, session_id: &str, revision: i64) {
        let _ = self.tx.send(Update {
            session_id: session_id.to_string(),
            revision,
        });
    }
}

/// Shared application state. The database connection sits behind a mutex because
/// SQLite is synchronous; no handler holds the lock across an await point.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub hub: Broadcaster,
    pub dist_dir: PathBuf,
    /// How often an idle event stream emits a named heartbeat frame.
    pub heartbeat_interval: std::time::Duration,
}

impl AppState {
    pub fn new(conn: Connection, dist_dir: PathBuf) -> Self {
        AppState {
            db: Arc::new(Mutex::new(conn)),
            hub: Broadcaster::new(),
            dist_dir,
            heartbeat_interval: crate::routes::stream::HEARTBEAT_INTERVAL,
        }
    }

    /// Shorten the heartbeat interval. Only tests do this; a deployed server keeps
    /// the default so an idle stream is not chatty.
    #[cfg(test)]
    pub fn with_heartbeat_interval(mut self, interval: std::time::Duration) -> Self {
        self.heartbeat_interval = interval;
        self
    }

    /// Run `f` against the database. Used by every handler that reads or writes.
    pub fn with_db<T>(&self, f: impl FnOnce(&Connection) -> T) -> T {
        let guard = self.db.lock().expect("database mutex poisoned");
        f(&guard)
    }

    /// The current revision of a session, or `0` when it does not exist.
    pub fn current_revision(&self, session_id: &str) -> i64 {
        self.with_db(|conn| {
            conn.query_row(
                "SELECT revision FROM game_session WHERE id = ?1",
                [session_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
        })
    }

    /// Reject an unknown session before a stream opens (FR-013).
    pub fn require_session(&self, session_id: &str) -> ApiResult<()> {
        if self.with_db(|conn| {
            conn.query_row(
                "SELECT 1 FROM game_session WHERE id = ?1",
                [session_id],
                |_| Ok(()),
            )
            .is_ok()
        }) {
            Ok(())
        } else {
            Err(ApiError::not_found(format!(
                "No session with id '{session_id}'."
            )))
        }
    }

    /// Record a new revision for the session and announce it to live clients.
    /// The counter lives in the database, so it keeps growing across restarts
    /// and never moves backwards for a client that reloads the page.
    pub fn bump_revision(&self, session_id: &str) -> ApiResult<i64> {
        let revision = self.with_db(|conn| {
            conn.query_row(
                "UPDATE game_session SET revision = revision + 1 WHERE id = ?1 RETURNING revision",
                [session_id],
                |row| row.get::<_, i64>(0),
            )
        })?;
        self.hub.publish(session_id, revision);
        Ok(revision)
    }
}

use serde::{Deserialize, Serialize};

/// Which view of a session a reader holds. `view` is the spectator slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Gamemaster,
    P1,
    P2,
    P3,
    P4,
    View,
}

impl UserRole {
    pub fn as_str(self) -> &'static str {
        match self {
            UserRole::Gamemaster => "gamemaster",
            UserRole::P1 => "p1",
            UserRole::P2 => "p2",
            UserRole::P3 => "p3",
            UserRole::P4 => "p4",
            UserRole::View => "view",
        }
    }

    pub fn parse(raw: &str) -> Option<UserRole> {
        match raw {
            "gamemaster" => Some(UserRole::Gamemaster),
            "p1" => Some(UserRole::P1),
            "p2" => Some(UserRole::P2),
            "p3" => Some(UserRole::P3),
            "p4" => Some(UserRole::P4),
            "view" => Some(UserRole::View),
            _ => None,
        }
    }

    /// The team slot a player role occupies, or `None` for the gamemaster and the
    /// spectator. A role beyond the session's team count has no team and its view
    /// is read-only.
    pub fn team_index(self) -> Option<i32> {
        match self {
            UserRole::P1 => Some(1),
            UserRole::P2 => Some(2),
            UserRole::P3 => Some(3),
            UserRole::P4 => Some(4),
            _ => None,
        }
    }
}

// Every struct below mirrors `specs/001-listquiz-game/contracts/realtime.md`. The
// contract is the wire format, so the field names and the camelCase spelling come
// from there rather than from Rust conventions.

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerLink {
    pub role: String,
    pub path: String,
    pub has_team: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionLinks {
    pub gamemaster: String,
    pub players: Vec<PlayerLink>,
    pub spectator: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamView {
    pub id: String,
    pub name: String,
    pub color: String,
    pub primary_hex: String,
    pub accent_glow: String,
    pub score: i32,
    pub hearts: i32,
    pub on_turn: bool,
}

/// An item on the Order bar or in its pool. `value` is `None` for a player until
/// the gamemaster reveals values (FR-027).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderItem {
    pub item_id: String,
    pub name: String,
    pub value: Option<String>,
    pub revealed: bool,
    /// The team that placed this item on the bar, or `null` while it is in the pool
    /// (feature 004, US2). Public: the room watched the placement.
    pub placed_by: Option<String>,
}

/// The anchor is public by design: FR-026 says it *must* show its value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnchorItem {
    pub item_id: String,
    pub name: String,
    pub value: String,
}

/// The item currently being placed, with the slot the gamemaster proposed. A player
/// view only ever sees its own team's placement (contracts/realtime.md).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingPlacement {
    pub item_id: String,
    pub name: String,
    pub side: String,
    pub slot_index: i32,
    pub team_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderView {
    pub anchor: AnchorItem,
    pub pool: Vec<OrderItem>,
    pub left: Vec<OrderItem>,
    pub right: Vec<OrderItem>,
    pub pending: Option<PendingPlacement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripmineItem {
    pub item_id: String,
    pub name: String,
    pub value: Option<String>,
    pub revealed: bool,
    /// Position 1-10 in the ordered grid. A pick reveals it (FR-039), so it is
    /// `null` for a player until the item is revealed, and always present for the
    /// gamemaster.
    pub true_rank: Option<i32>,
    /// The answer key. Only `true` for an item the player has already revealed.
    pub is_tripmine: bool,
    pub points_awarded: Option<i32>,
    /// The team that picked this item, or `null` while it is still hidden
    /// (feature 004, US2). Public once revealed.
    pub picked_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripmineView {
    pub side: String,
    pub searched_rank: i32,
    pub hit: bool,
    pub completed: bool,
    /// The mined item's value for the round's column: a public hint (FR-015), shown
    /// to every role even while per-cell values stay hidden.
    pub target_value: Option<String>,
    pub items: Vec<TripmineItem>,
}

/// The gamemaster's turn timer as every role sees it (feature 005, US2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerView {
    pub total_seconds: u32,
    pub seconds_left: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoundSummary {
    pub round_number: i32,
    pub quiz_type: String,
    pub tripmine_side: Option<String>,
    pub list_title: String,
    pub list_id: String,
    pub column_id: String,
    pub column_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionState {
    pub revision: i64,
    pub session_id: String,
    pub status: String,
    pub team_count: i32,
    pub round_count: i32,
    pub current_round: i32,
    pub muted: bool,
    pub reveal_values: bool,
    pub last_message: Option<String>,
    /// The running turn timer, or `null` (feature 005, US2). Public to every role.
    pub timer: Option<TimerView>,
    pub round: Option<RoundSummary>,
    pub teams: Vec<TeamView>,
    pub order: Option<OrderView>,
    pub tripmine: Option<TripmineView>,
    pub links: SessionLinks,
}

impl SessionState {
    pub fn team(&self, team_id: &str) -> Option<&TeamView> {
        self.teams.iter().find(|t| t.id == team_id)
    }

    /// The team holding the turn, derived from the teams the session reports.
    pub fn active_team_id(&self) -> Option<&str> {
        self.teams.iter().find(|t| t.on_turn).map(|t| t.id.as_str())
    }

    /// True when this role owns the active team and therefore acts this turn.
    pub fn role_has_turn(&self, role: UserRole) -> bool {
        match role.team_index() {
            Some(index) if index <= self.team_count => {
                self.active_team_id() == Some(&*format!("team_{index}"))
            }
            _ => false,
        }
    }

    /// True when this role is a player slot beyond the configured team count, which
    /// renders read-only with no badge and no turn emphasis (FR-009).
    pub fn role_is_unassigned(&self, role: UserRole) -> bool {
        match role.team_index() {
            Some(index) => index > self.team_count,
            None => false,
        }
    }
}

/// The draft's five role links are always present, whatever the team count
/// (FR-009): one gamemaster path, all four player paths, one spectator path.
pub fn build_links(session_id: &str, team_count: i32) -> SessionLinks {
    SessionLinks {
        gamemaster: format!("/{session_id}/gamemaster"),
        players: (1..=4)
            .map(|index| PlayerLink {
                role: format!("p{index}"),
                path: format!("/{session_id}/p{index}"),
                has_team: index <= team_count,
            })
            .collect(),
        spectator: format!("/{session_id}/view"),
    }
}

/// Redact a full state for one role. This is the only place a value can be hidden,
/// so there is no code path by which a player view renders an unrevealed number
/// (contracts/realtime.md).
///
/// Rules, straight from the contract's table:
///
/// | Field | Gamemaster | Player / spectator |
/// |---|---|---|
/// | `order.pool/left/right[].value` | present | `null` until `revealValues` |
/// | `order.anchor.value` | present | present (FR-026) |
/// | `tripmine.items[].value` | present | `null` until that item is revealed |
/// | `tripmine.items[].isTripmine` | present | only once that item is revealed |
///
/// The bar itself is public: every view sees which items have been placed, which is
/// how the room watches an order round fill up.
pub fn redact(state: &SessionState, role: UserRole) -> SessionState {
    let is_gamemaster = role == UserRole::Gamemaster;
    let reveal = state.reveal_values;
    let show_values = is_gamemaster || reveal;

    let order = state.order.as_ref().map(|order| OrderView {
        anchor: order.anchor.clone(),
        pool: order
            .pool
            .iter()
            .map(|i| reveal_order_item(i, show_values))
            .collect(),
        left: order
            .left
            .iter()
            .map(|i| reveal_order_item(i, show_values))
            .collect(),
        right: order
            .right
            .iter()
            .map(|i| reveal_order_item(i, show_values))
            .collect(),
        pending: order.pending.as_ref().map(|p| PendingPlacement {
            team_id: if is_gamemaster || p.team_id == acting_team(state, role) {
                p.team_id.clone()
            } else {
                // A player view never learns which team proposed another team's
                // placement (contracts/realtime.md).
                String::new()
            },
            ..p.clone()
        }),
    });

    let tripmine = state.tripmine.as_ref().map(|tripmine| TripmineView {
        side: tripmine.side.clone(),
        searched_rank: tripmine.searched_rank,
        hit: tripmine.hit,
        completed: tripmine.completed,
        target_value: tripmine.target_value.clone(),
        items: tripmine
            .items
            .iter()
            .map(|item| {
                let revealed = is_gamemaster || item.revealed;
                TripmineItem {
                    item_id: item.item_id.clone(),
                    name: item.name.clone(),
                    value: if revealed || reveal {
                        item.value.clone()
                    } else {
                        None
                    },
                    revealed,
                    true_rank: if revealed { item.true_rank } else { None },
                    is_tripmine: if revealed { item.is_tripmine } else { false },
                    points_awarded: if revealed { item.points_awarded } else { None },
                    picked_by: if revealed {
                        item.picked_by.clone()
                    } else {
                        None
                    },
                }
            })
            .collect(),
    });

    SessionState {
        order,
        tripmine,
        ..state.clone()
    }
}

fn reveal_order_item(item: &OrderItem, show_values: bool) -> OrderItem {
    OrderItem {
        item_id: item.item_id.clone(),
        name: item.name.clone(),
        value: if show_values {
            item.value.clone()
        } else {
            None
        },
        revealed: show_values,
        placed_by: item.placed_by.clone(),
    }
}

/// The team id a role acts for, or an empty string when it acts for nobody.
fn acting_team(state: &SessionState, role: UserRole) -> String {
    match role.team_index() {
        Some(index) if index <= state.team_count => format!("team_{index}"),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order_item(id: &str, name: &str, value: &str) -> OrderItem {
        OrderItem {
            item_id: id.to_string(),
            name: name.to_string(),
            value: Some(value.to_string()),
            revealed: true,
            placed_by: None,
        }
    }

    fn team(id: &str, on_turn: bool) -> TeamView {
        TeamView {
            id: id.to_string(),
            name: id.to_string(),
            color: "cyan".to_string(),
            primary_hex: "#06b6d4".to_string(),
            accent_glow: "rgba(6, 182, 212, 0.45)".to_string(),
            score: 10,
            hearts: 2,
            on_turn,
        }
    }

    fn order_state() -> SessionState {
        SessionState {
            revision: 7,
            session_id: "s1".to_string(),
            status: "running".to_string(),
            team_count: 3,
            round_count: 5,
            current_round: 1,
            muted: false,
            reveal_values: false,
            last_message: None,
            timer: None,
            round: Some(RoundSummary {
                round_number: 1,
                quiz_type: "ORDER".to_string(),
                tripmine_side: None,
                list_title: "World History".to_string(),
                list_id: "list_1".to_string(),
                column_id: "year".to_string(),
                column_name: "Year".to_string(),
            }),
            teams: vec![
                team("team_1", false),
                team("team_2", true),
                team("team_3", false),
            ],
            order: Some(OrderView {
                anchor: AnchorItem {
                    item_id: "a".to_string(),
                    name: "Apollo 11".to_string(),
                    value: "1969".to_string(),
                },
                pool: vec![
                    order_item("i1", "Gutenberg", "1440"),
                    order_item("i2", "Titanic", "1912"),
                ],
                left: vec![order_item("i1", "Gutenberg", "1440")],
                right: vec![order_item("i2", "Titanic", "1912")],
                pending: Some(PendingPlacement {
                    item_id: "i3".to_string(),
                    name: "iPhone".to_string(),
                    side: "right".to_string(),
                    slot_index: 2,
                    team_id: "team_2".to_string(),
                }),
            }),
            tripmine: None,
            links: build_links("s1", 3),
        }
    }

    fn tripmine_state() -> SessionState {
        let mut state = order_state();
        state.round = Some(RoundSummary {
            round_number: 2,
            quiz_type: "TRIPMINE".to_string(),
            tripmine_side: Some("highest".to_string()),
            list_title: "Cinema".to_string(),
            list_id: "list_2".to_string(),
            column_id: "box_office".to_string(),
            column_name: "Box Office ($)".to_string(),
        });
        state.order = None;
        state.tripmine = Some(TripmineView {
            side: "highest".to_string(),
            searched_rank: 5,
            hit: false,
            completed: false,
            target_value: Some("2920".to_string()),
            items: vec![
                TripmineItem {
                    item_id: "m1".to_string(),
                    name: "Avatar".to_string(),
                    value: Some("2920".to_string()),
                    revealed: true,
                    true_rank: Some(10),
                    is_tripmine: true,
                    points_awarded: None,
                    picked_by: None,
                },
                TripmineItem {
                    item_id: "m2".to_string(),
                    name: "Endgame".to_string(),
                    value: Some("2800".to_string()),
                    revealed: false,
                    true_rank: Some(4),
                    is_tripmine: false,
                    points_awarded: None,
                    picked_by: None,
                },
            ],
        });
        state
    }

    fn order_of(state: &SessionState) -> &OrderView {
        state.order.as_ref().expect("an order round")
    }

    fn tripmine_of(state: &SessionState) -> &TripmineView {
        state.tripmine.as_ref().expect("a tripmine round")
    }

    #[test]
    fn five_links_always_exist_regardless_of_team_count() {
        for count in 1..=4 {
            let links = build_links("s1", count);
            assert_eq!(links.players.len(), 4);
            assert_eq!(links.gamemaster, "/s1/gamemaster");
            assert_eq!(links.spectator, "/s1/view");
            assert_eq!(
                links
                    .players
                    .iter()
                    .map(|l| l.role.as_str())
                    .collect::<Vec<_>>(),
                ["p1", "p2", "p3", "p4"]
            );
        }
    }

    #[test]
    fn player_links_beyond_the_team_count_carry_no_team() {
        let links = build_links("s1", 2);

        let with_team: Vec<&str> = links
            .players
            .iter()
            .filter(|l| l.has_team)
            .map(|l| l.role.as_str())
            .collect();
        let without_team: Vec<&str> = links
            .players
            .iter()
            .filter(|l| !l.has_team)
            .map(|l| l.role.as_str())
            .collect();
        assert_eq!(with_team, ["p1", "p2"]);
        assert_eq!(without_team, ["p3", "p4"]);
    }

    #[test]
    fn a_one_team_session_still_links_p2_to_p4() {
        let links = build_links("s1", 1);

        assert_eq!(links.players.len(), 4);
        assert!(links.players[0].has_team);
        assert!(links.players[1..].iter().all(|l| !l.has_team));
    }

    #[test]
    fn link_paths_carry_the_session_id() {
        let links = build_links("lq-8fj2wkd9", 4);

        assert_eq!(links.gamemaster, "/lq-8fj2wkd9/gamemaster");
        assert_eq!(links.players[2].path, "/lq-8fj2wkd9/p3");
        assert_eq!(links.spectator, "/lq-8fj2wkd9/view");
    }

    #[test]
    fn the_state_serialises_with_camel_case_field_names() {
        let json = serde_json::to_value(order_state()).unwrap();

        assert!(json.get("sessionId").is_some());
        assert!(json.get("revealValues").is_some());
        assert!(json.get("lastMessage").is_some());
        assert!(json["order"].is_object());
        assert!(json["tripmine"].is_null());
        assert!(json["teams"][0].get("onTurn").is_some());
        assert!(json["order"]["pool"][0].get("itemId").is_some());
        assert!(json["links"]["gamemaster"].is_string());
    }

    #[test]
    fn the_gamemaster_sees_every_value_and_the_answer_key() {
        let state = order_state();

        let gm = redact(&state, UserRole::Gamemaster);

        let order = order_of(&gm);
        assert!(order.pool.iter().all(|i| i.value.is_some()));
        assert!(order.left.iter().all(|i| i.value.is_some()));
        assert!(order.right.iter().all(|i| i.value.is_some()));
        assert!(order.pool.iter().all(|i| i.revealed));
    }

    #[test]
    fn a_player_sees_no_pool_or_bar_values_before_the_reveal() {
        let state = order_state();

        let p2 = redact(&state, UserRole::P2);

        let order = order_of(&p2);
        assert!(order.pool.iter().all(|i| i.value.is_none()));
        assert!(order.left.iter().all(|i| i.value.is_none()));
        assert!(order.right.iter().all(|i| i.value.is_none()));
        assert!(order.pool.iter().all(|i| !i.revealed));
    }

    #[test]
    fn a_player_still_sees_the_anchor_value() {
        let state = order_state();

        for role in [
            UserRole::P1,
            UserRole::P2,
            UserRole::P3,
            UserRole::P4,
            UserRole::View,
        ] {
            let redacted = redact(&state, role);
            assert_eq!(
                order_of(&redacted).anchor.value,
                "1969",
                "FR-026 makes the anchor value public"
            );
        }
    }

    #[test]
    fn revealing_values_shows_them_in_every_view() {
        let mut state = order_state();
        state.reveal_values = true;

        let spectator = redact(&state, UserRole::View);

        let order = order_of(&spectator);
        assert!(order.pool.iter().all(|i| i.value.is_some()));
        assert!(order.left.iter().all(|i| i.value.is_some()));
    }

    #[test]
    fn every_view_sees_which_items_have_been_placed() {
        let state = order_state();

        let spectator = redact(&state, UserRole::View);

        let order = order_of(&spectator);
        assert_eq!(order.left.len(), 1);
        assert_eq!(order.right.len(), 1);
        assert_eq!(order.left[0].name, "Gutenberg");
    }

    #[test]
    fn a_pending_placement_names_only_its_own_team_to_a_player() {
        let state = order_state();

        let own = redact(&state, UserRole::P2);
        let other = redact(&state, UserRole::P1);

        assert_eq!(order_of(&own).pending.as_ref().unwrap().team_id, "team_2");
        assert_eq!(order_of(&other).pending.as_ref().unwrap().team_id, "");
    }

    #[test]
    fn the_gamemaster_sees_the_pending_placement_team() {
        let state = order_state();

        let gm = redact(&state, UserRole::Gamemaster);

        assert_eq!(order_of(&gm).pending.as_ref().unwrap().team_id, "team_2");
    }

    #[test]
    fn a_player_never_receives_the_answer_key_of_an_unrevealed_item() {
        let state = tripmine_state();

        let spectator = redact(&state, UserRole::View);

        let tripmine = tripmine_of(&spectator);
        for item in tripmine.items.iter().filter(|i| !i.revealed) {
            assert!(
                !item.is_tripmine,
                "{} is unrevealed, so it must not carry the answer",
                item.name
            );
            assert!(item.points_awarded.is_none());
        }
    }

    #[test]
    fn the_gamemaster_sees_the_tripmine_answer_key() {
        let state = tripmine_state();

        let gm = redact(&state, UserRole::Gamemaster);

        let tripmine = tripmine_of(&gm);
        assert!(
            tripmine
                .items
                .iter()
                .all(|i| i.is_tripmine || i.item_id == "m2")
        );
        assert!(tripmine.items[0].is_tripmine);
    }

    #[test]
    fn a_player_sees_a_revealed_tripmine_items_value_and_a_hidden_one_stays_null() {
        let state = tripmine_state();

        let p2 = redact(&state, UserRole::P2);

        let tripmine = tripmine_of(&p2);
        assert_eq!(tripmine.items[0].value.as_deref(), Some("2920"));
        assert!(
            tripmine.items[0].is_tripmine,
            "it is revealed, so the answer may show"
        );
        assert_eq!(tripmine.items[1].value, None);
        assert!(!tripmine.items[1].is_tripmine);
    }

    #[test]
    fn the_searched_rank_is_public() {
        let state = tripmine_state();

        let spectator = redact(&state, UserRole::View);

        assert_eq!(tripmine_of(&spectator).searched_rank, 5);
    }

    #[test]
    fn reveal_values_also_unlocks_tripmine_numbers() {
        let mut state = tripmine_state();
        state.reveal_values = true;

        let spectator = redact(&state, UserRole::View);

        let tripmine = tripmine_of(&spectator);
        assert!(tripmine.items.iter().all(|i| i.value.is_some()));
    }

    #[test]
    fn the_turn_belongs_to_the_matching_player_slot() {
        let state = order_state();

        assert!(state.role_has_turn(UserRole::P2));
        assert!(!state.role_has_turn(UserRole::P1));
        assert!(!state.role_has_turn(UserRole::View));
        assert!(!state.role_has_turn(UserRole::Gamemaster));
    }

    #[test]
    fn a_player_slot_beyond_the_team_count_is_flagged_unassigned() {
        let state = order_state();

        assert!(!state.role_is_unassigned(UserRole::P2));
        assert!(state.role_is_unassigned(UserRole::P4));
        assert!(!state.role_is_unassigned(UserRole::View));
        assert!(!state.role_has_turn(UserRole::P4));
    }

    #[test]
    fn redaction_leaves_the_revision_scores_and_teams_untouched() {
        let state = order_state();

        let p1 = redact(&state, UserRole::P1);

        assert_eq!(p1.revision, state.revision);
        assert_eq!(p1.current_round, state.current_round);
        assert_eq!(p1.teams, state.teams);
        assert_eq!(p1.links, state.links);
        assert_eq!(p1.round, state.round);
    }

    #[test]
    fn role_parsing_round_trips() {
        for role in [
            UserRole::Gamemaster,
            UserRole::P1,
            UserRole::P2,
            UserRole::P3,
            UserRole::P4,
            UserRole::View,
        ] {
            assert_eq!(UserRole::parse(role.as_str()), Some(role));
        }
        assert_eq!(UserRole::parse("admin"), None);
        assert_eq!(UserRole::parse("P1"), None);
    }

    #[test]
    fn the_placing_team_is_public_and_pool_items_have_none() {
        let mut state = order_state();
        state
            .order
            .as_mut()
            .expect("an order board")
            .left
            .iter_mut()
            .for_each(|item| item.placed_by = Some("team_2".to_string()));

        let for_player = redact(&state, UserRole::P1);
        let order = for_player.order.expect("an order view");
        assert_eq!(order.left[0].placed_by.as_deref(), Some("team_2"));
        assert!(
            order.pool.iter().all(|item| item.placed_by.is_none()),
            "a pool item carries no placing team"
        );
    }

    #[test]
    fn the_picking_team_is_exposed_only_once_revealed() {
        let mut state = tripmine_state();
        let view = state.tripmine.as_mut().expect("a tripmine view");
        view.items[0].picked_by = Some("team_1".to_string());
        view.items[1].picked_by = Some("team_2".to_string());

        let for_player = redact(&state, UserRole::P1);
        let items = for_player.tripmine.expect("a tripmine view").items;
        assert_eq!(items[0].picked_by.as_deref(), Some("team_1"), "revealed");
        assert_eq!(items[1].picked_by, None, "still hidden");
    }
}

#[cfg(test)]
mod hub_tests {
    use super::*;

    #[tokio::test]
    async fn a_publish_reaches_every_subscriber() {
        let hub = Broadcaster::new();
        let mut a = hub.subscribe();
        let mut b = hub.subscribe();

        hub.publish("s1", 7);

        let for_a = a.recv().await.unwrap();
        let for_b = b.recv().await.unwrap();
        assert_eq!(for_a.session_id, "s1");
        assert_eq!(for_a.revision, 7);
        assert_eq!(for_b.session_id, "s1");
        assert_eq!(for_b.revision, 7);
        assert_eq!(hub.receiver_count(), 2);
    }

    #[tokio::test]
    async fn a_subscriber_can_skip_another_sessions_updates() {
        let hub = Broadcaster::new();
        let mut mine = hub.subscribe();

        hub.publish("other", 3);
        hub.publish("s1", 4);

        // The SSE handler does exactly this loop: consume, keep only my session.
        let mut seen = Vec::new();
        while let Ok(update) = mine.try_recv() {
            if update.session_id == "s1" {
                seen.push(update);
            }
        }
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].revision, 4);
    }

    #[test]
    fn a_revision_is_read_from_the_database_and_keeps_growing() {
        let dir = std::env::temp_dir().join(format!("lq-rev-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("listquiz.db");
        let state = AppState::new(crate::db::open_db(&path).unwrap(), dir.clone());

        state.with_db(|conn| crate::db::insert_test_session(conn, "s1", 41));

        assert_eq!(state.current_revision("s1"), 41);
        assert_eq!(state.bump_revision("s1").unwrap(), 42);
        assert_eq!(state.current_revision("s1"), 42);
        assert_eq!(state.current_revision("missing"), 0);
        assert!(state.require_session("s1").is_ok());
        assert!(state.require_session("missing").is_err());
        assert!(state.bump_revision("missing").is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
