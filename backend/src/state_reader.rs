//! Reading a session out of the database into the shape the frontend consumes
//! (T037). Every handler answers from here, so the REST endpoint and the SSE
//! `state` frame cannot drift apart.

use rusqlite::{Connection, OptionalExtension, params};

use crate::app_state::{SessionState, TeamView, UserRole, build_links, redact};
use crate::errors::{ApiError, ApiResult};

/// One `game_session` row plus the flags a view needs.
struct SessionRow {
    revision: i64,
    status: String,
    team_count: i32,
    round_count: i32,
    current_round: i32,
    muted: bool,
    reveal_values: bool,
    active_team_id: Option<String>,
    last_message: Option<String>,
}

/// Load the full, unredacted state and redact it for `role`.
pub fn read_state(conn: &Connection, session_id: &str, role: UserRole) -> ApiResult<SessionState> {
    let row: Option<SessionRow> = conn
        .query_row(
            "SELECT revision, status, team_count, round_count, current_round,
                    muted, reveal_values, active_team_id, last_message
             FROM game_session WHERE id = ?1",
            params![session_id],
            |row| {
                Ok(SessionRow {
                    revision: row.get(0)?,
                    status: row.get(1)?,
                    team_count: row.get(2)?,
                    round_count: row.get(3)?,
                    current_round: row.get(4)?,
                    muted: row.get::<_, i64>(5)? != 0,
                    reveal_values: row.get::<_, i64>(6)? != 0,
                    active_team_id: row.get(7)?,
                    last_message: row.get(8)?,
                })
            },
        )
        .optional()?;
    let row =
        row.ok_or_else(|| ApiError::not_found(format!("No session with id '{session_id}'.")))?;

    let teams = read_teams(conn, session_id, row.active_team_id.as_deref())?;
    let round = read_current_round(conn, session_id, row.current_round)?;
    // Load every value; `redact` decides who may see it. Filtering during the
    // read would make the gamemaster's own view unrecoverable.
    let order = read_order(conn, session_id)?;
    let tripmine = read_tripmine(conn, session_id)?;

    let state = SessionState {
        revision: row.revision,
        session_id: session_id.to_string(),
        status: row.status,
        team_count: row.team_count,
        round_count: row.round_count,
        current_round: row.current_round,
        muted: row.muted,
        reveal_values: row.reveal_values,
        last_message: row.last_message,
        round,
        teams,
        order,
        tripmine,
        links: build_links(session_id, row.team_count),
    };
    Ok(redact(&state, role))
}

fn read_teams(
    conn: &Connection,
    session_id: &str,
    active_team_id: Option<&str>,
) -> ApiResult<Vec<TeamView>> {
    let mut statement = conn
        .prepare(
            "SELECT id, name, color, primary_hex, accent_glow, score, hearts
             FROM team WHERE session_id = ?1 ORDER BY turn_index",
        )
        .map_err(ApiError::from)?;
    let teams = statement
        .query_map(params![session_id], |row| {
            let id: String = row.get(0)?;
            Ok(TeamView {
                on_turn: active_team_id == Some(id.as_str()),
                id,
                name: row.get(1)?,
                color: row.get(2)?,
                primary_hex: row.get(3)?,
                accent_glow: row.get(4)?,
                score: row.get(5)?,
                hearts: row.get(6)?,
            })
        })
        .map_err(ApiError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(ApiError::from)?;
    Ok(teams)
}

fn read_current_round(
    conn: &Connection,
    session_id: &str,
    current_round: i32,
) -> ApiResult<Option<crate::app_state::RoundSummary>> {
    if current_round <= 0 {
        return Ok(None);
    }
    let summary = conn
        .query_row(
            "SELECT rc.round_number, rc.quiz_type, rc.tripmine_side, rc.list_id,
                    rc.column_id, l.title, c.name
             FROM round_config rc
             LEFT JOIN quiz_list l ON l.id = rc.list_id
             LEFT JOIN list_item_column c ON c.list_id = rc.list_id AND c.id = rc.column_id
             WHERE rc.session_id = ?1 AND rc.round_number = ?2",
            params![session_id, current_round],
            |row| {
                Ok(crate::app_state::RoundSummary {
                    round_number: row.get(0)?,
                    quiz_type: row.get(1)?,
                    tripmine_side: row.get(2)?,
                    list_id: row.get(3)?,
                    column_id: row.get(4)?,
                    list_title: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                    column_name: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                })
            },
        )
        .optional()?;
    Ok(summary)
}

/// The items behind the stored Order layout, with names and values joined in.
/// The stored ids are the source of truth for what is on the bar; a missing id
/// (list deleted mid-round) is skipped rather than shown as a blank row.
fn read_order_items(
    conn: &Connection,
    column_id: &str,
    ids: &[String],
) -> ApiResult<Vec<crate::app_state::OrderItem>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut items = Vec::with_capacity(ids.len());
    for id in ids {
        let item = conn
            .query_row(
                "SELECT i.name, v.display
                 FROM item i
                 LEFT JOIN item_value v ON v.item_id = i.id AND v.column_id = ?2
                 WHERE i.id = ?1",
                params![id, column_id],
                |row| {
                    Ok(crate::app_state::OrderItem {
                        item_id: id.clone(),
                        name: row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                        value: row.get::<_, Option<String>>(1)?,
                        // Always populated here; redaction happens later.
                        revealed: true,
                    })
                },
            )
            .optional()?;
        if let Some(item) = item {
            items.push(item);
        }
    }
    Ok(items)
}

fn parse_ids(raw: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(raw).unwrap_or_default()
}

fn read_order(
    conn: &Connection,
    session_id: &str,
) -> ApiResult<Option<crate::app_state::OrderView>> {
    let row: Option<(String, f64, String, String, String, Option<String>)> = conn
        .query_row(
            "SELECT anchor_item_id, anchor_value, pool, left_ids, right_ids, pending
             FROM order_state WHERE session_id = ?1",
            params![session_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .optional()?;
    let Some((anchor_item_id, anchor_value, pool, left, right, pending)) = row else {
        return Ok(None);
    };

    let column_id: Option<String> = conn
        .query_row(
            "SELECT column_id FROM round_config
             WHERE session_id = ?1 AND round_number = (
                 SELECT current_round FROM game_session WHERE id = ?1
             )",
            params![session_id],
            |row| row.get(0),
        )
        .optional()?
        .flatten();
    let column_id = column_id.unwrap_or_default();

    let anchor = conn
        .query_row(
            "SELECT i.name, v.display
             FROM item i
             LEFT JOIN item_value v ON v.item_id = i.id AND v.column_id = ?2
             WHERE i.id = ?1",
            params![anchor_item_id, column_id],
            |row| {
                let name = row.get::<_, Option<String>>(0)?.unwrap_or_default();
                let display = row.get::<_, Option<String>>(1)?;
                Ok(crate::app_state::AnchorItem {
                    item_id: anchor_item_id.clone(),
                    name,
                    value: display.unwrap_or_else(|| format_value(anchor_value)),
                })
            },
        )
        .optional()?;

    Ok(Some(crate::app_state::OrderView {
        anchor: anchor.unwrap_or(crate::app_state::AnchorItem {
            item_id: anchor_item_id,
            name: String::new(),
            value: format_value(anchor_value),
        }),
        pool: read_order_items(conn, &column_id, &parse_ids(&pool))?,
        left: read_order_items(conn, &column_id, &parse_ids(&left))?,
        right: read_order_items(conn, &column_id, &parse_ids(&right))?,
        pending: pending
            .as_deref()
            .and_then(|raw| serde_json::from_str(raw).ok()),
    }))
}

fn read_tripmine(
    conn: &Connection,
    session_id: &str,
) -> ApiResult<Option<crate::app_state::TripmineView>> {
    let row: Option<(String, i32, i64, Option<String>, i64)> = conn
        .query_row(
            "SELECT side, searched_rank, hit, hit_by, completed
             FROM tripmine_state WHERE session_id = ?1",
            params![session_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .optional()?;
    let Some((side, searched_rank, hit, _hit_by, completed)) = row else {
        return Ok(None);
    };

    let column_id: Option<String> = conn
        .query_row(
            "SELECT column_id FROM round_config
             WHERE session_id = ?1 AND round_number = (
                 SELECT current_round FROM game_session WHERE id = ?1
             )",
            params![session_id],
            |row| row.get(0),
        )
        .optional()?
        .flatten();
    let column_id = column_id.unwrap_or_default();

    let raw_items: String = conn
        .query_row(
            "SELECT items FROM tripmine_state WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "[]".to_string());

    let slots: Vec<crate::modules::tripmine_round::TripmineSlot> =
        serde_json::from_str(&raw_items).unwrap_or_default();

    let mut items = Vec::with_capacity(slots.len());
    for slot in slots {
        let (name, display) = conn
            .query_row(
                "SELECT i.name, v.display
                 FROM item i
                 LEFT JOIN item_value v ON v.item_id = i.id AND v.column_id = ?2
                 WHERE i.id = ?1",
                params![slot.item.item_id, column_id],
                |row| {
                    Ok((
                        row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                        row.get::<_, Option<String>>(1)?,
                    ))
                },
            )
            .optional()?
            .unwrap_or((slot.item.name.clone(), Some(slot.item.display.clone())));

        items.push(crate::app_state::TripmineItem {
            item_id: slot.item.item_id,
            name,
            value: display,
            revealed: slot.revealed,
            true_rank: Some(slot.true_rank),
            is_tripmine: slot.is_tripmine,
            points_awarded: slot.points_awarded,
        });
    }

    Ok(Some(crate::app_state::TripmineView {
        side,
        searched_rank,
        hit: hit != 0,
        completed: completed != 0,
        target_value: items
            .iter()
            .find(|item| item.is_tripmine)
            .and_then(|item| item.value.clone()),
        items,
    }))
}

/// One decimal place at most, without a trailing `.0`.
pub fn format_value(value: f64) -> String {
    if (value.fract()).abs() < f64::EPSILON {
        format!("{}", value as i64)
    } else {
        let rounded = (value * 10.0).round() / 10.0;
        format!("{rounded}")
    }
}
