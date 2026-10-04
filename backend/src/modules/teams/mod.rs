use crate::errors::{ApiError, ApiResult};
use serde::{Deserialize, Serialize};

pub const MIN_TEAMS: i32 = 1;
pub const MAX_TEAMS: i32 = 4;
pub const MAX_TEAM_NAME_CHARS: usize = 40;

pub const GLOW_ALPHA: f64 = 0.45;

/// One of the four built-in palettes, copied from the draft's `INITIAL_TEAMS`.
/// Every team starts with `Team <Colour> <Noun>` and 3 hearts (FR-042, FR-047).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub color_name: &'static str,
    pub primary_hex: &'static str,
    pub accent_glow: &'static str,
}

pub const INITIAL_TEAMS: [TeamSpec; 4] = [
    TeamSpec {
        id: "team_1",
        name: "Team Cyan Pulsar",
        color_name: "cyan",
        primary_hex: "#06b6d4",
        accent_glow: "rgba(6, 182, 212, 0.45)",
    },
    TeamSpec {
        id: "team_2",
        name: "Team Ruby Spark",
        color_name: "rose",
        primary_hex: "#f43f5e",
        accent_glow: "rgba(244, 63, 94, 0.45)",
    },
    TeamSpec {
        id: "team_3",
        name: "Team Amber Apex",
        color_name: "amber",
        primary_hex: "#f59e0b",
        accent_glow: "rgba(245, 158, 11, 0.45)",
    },
    TeamSpec {
        id: "team_4",
        name: "Team Emerald Volt",
        color_name: "emerald",
        primary_hex: "#10b981",
        accent_glow: "rgba(16, 185, 129, 0.45)",
    },
];

/// A team row as it is about to be inserted into `team`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewTeam {
    pub id: String,
    pub name: String,
    pub color_name: String,
    pub primary_hex: String,
    pub accent_glow: String,
    pub score: i32,
    pub hearts: i32,
    pub turn_index: i32,
}

/// A gamemaster-supplied colour override from the Apply Palette control (FR-048).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamColorInput {
    pub name: Option<String>,
    pub primary_hex: String,
}

pub fn team_id(index: usize) -> String {
    format!("team_{}", index + 1)
}

pub fn validate_team_count(count: i32) -> ApiResult<()> {
    if !(MIN_TEAMS..=MAX_TEAMS).contains(&count) {
        return Err(ApiError::bad_request(format!(
            "Team count must be between {MIN_TEAMS} and {MAX_TEAMS}."
        )));
    }
    Ok(())
}

pub fn validate_team_name(name: &str) -> ApiResult<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(ApiError::bad_request("Team name cannot be empty."));
    }
    if trimmed.chars().count() > MAX_TEAM_NAME_CHARS {
        return Err(ApiError::bad_request(format!(
            "Team name cannot exceed {MAX_TEAM_NAME_CHARS} characters."
        )));
    }
    Ok(())
}

/// Accepts `#rgb` and `#rrggbb`, returns the six-digit lowercase form.
pub fn normalize_hex(hex: &str) -> ApiResult<String> {
    let trimmed = hex.trim();
    let body = trimmed
        .strip_prefix('#')
        .ok_or_else(|| ApiError::bad_request("Team colour must be a hex value such as #06b6d4."))?;
    let valid = match body.len() {
        3 => body.chars().all(|c| c.is_ascii_hexdigit()),
        6 => body.chars().all(|c| c.is_ascii_hexdigit()),
        _ => false,
    };
    if !valid {
        return Err(ApiError::bad_request(format!(
            "'{trimmed}' is not a valid hex colour."
        )));
    }
    let expanded = if body.len() == 3 {
        body.chars().flat_map(|c| [c, c]).collect::<String>()
    } else {
        body.to_string()
    };
    Ok(format!("#{}", expanded.to_lowercase()))
}

/// The glow colour is derived from the team colour, never stored twice (FR-046).
pub fn glow_from_hex(hex: &str) -> ApiResult<String> {
    let hex = normalize_hex(hex)?;
    let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(0);
    Ok(format!("rgba({r}, {g}, {b}, {GLOW_ALPHA})"))
}

/// Build the team rows for a new session. `overrides` may be shorter than `count`;
/// missing entries keep the built-in palette.
pub fn build_teams(count: i32, overrides: &[TeamColorInput]) -> ApiResult<Vec<NewTeam>> {
    validate_team_count(count)?;
    let count = count as usize;
    if overrides.len() > count {
        return Err(ApiError::bad_request(format!(
            "Received {} team colours for {count} teams.",
            overrides.len()
        )));
    }

    let mut teams = Vec::with_capacity(count);
    for index in 0..count {
        let spec = INITIAL_TEAMS[index];
        let custom = overrides.get(index);
        let (name, primary_hex) = match custom {
            None => (spec.name.to_string(), spec.primary_hex.to_string()),
            Some(input) => {
                let name = match &input.name {
                    Some(name) => {
                        validate_team_name(name)?;
                        name.trim().to_string()
                    }
                    None => spec.name.to_string(),
                };
                (name, normalize_hex(&input.primary_hex)?)
            }
        };
        let accent_glow = glow_from_hex(&primary_hex)?;
        teams.push(NewTeam {
            id: team_id(index),
            name,
            color_name: spec.color_name.to_string(),
            primary_hex,
            accent_glow,
            score: 0,
            hearts: crate::modules::scoring::MAX_HEARTS,
            turn_index: index as i32,
        });
    }
    Ok(teams)
}

#[cfg(test)]
mod tests;
