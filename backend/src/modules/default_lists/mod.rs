use crate::errors::{ApiError, ApiResult};
use rusqlite::{Connection, OptionalExtension, params};

pub const MIN_ITEMS: usize = 11;
pub const MAX_ITEMS: usize = 100;
pub const MAX_LIST_TITLE_CHARS: usize = 80;
pub const MAX_COLUMN_NAME_CHARS: usize = 40;
pub const MAX_LIST_DESCRIPTION_CHARS: usize = 200;

/// Timestamps of the seeded lists are fixed so a fresh database is reproducible.
const SEED_TIME: i64 = 1_710_000_000_000;

pub struct DefaultColumn {
    pub id: &'static str,
    pub name: &'static str,
}

pub struct DefaultItem {
    pub id: &'static str,
    pub name: &'static str,
    pub icon_name: &'static str,
    /// (column id, display text, numeric value)
    pub values: &'static [(&'static str, &'static str, f64)],
}

pub struct DefaultList {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub order_direction: &'static str,
    pub columns: &'static [DefaultColumn],
    pub items: &'static [DefaultItem],
}

const HISTORY_COLUMNS: &[DefaultColumn] = &[DefaultColumn {
    id: "year",
    name: "Year",
}];

const HISTORY_ITEMS: &[DefaultItem] = &[
    DefaultItem {
        id: "h_1",
        name: "Gutenberg Printing Press",
        icon_name: "BookOpen",
        values: &[("year", "1440", 1440.0)],
    },
    DefaultItem {
        id: "h_2",
        name: "Eiffel Tower Completed",
        icon_name: "Landmark",
        values: &[("year", "1889", 1889.0)],
    },
    DefaultItem {
        id: "h_3",
        name: "First Modern Olympic Games",
        icon_name: "Trophy",
        values: &[("year", "1896", 1896.0)],
    },
    DefaultItem {
        id: "h_4",
        name: "Wright Brothers First Flight",
        icon_name: "Plane",
        values: &[("year", "1903", 1903.0)],
    },
    DefaultItem {
        id: "h_5",
        name: "Sinking of the Titanic",
        icon_name: "Compass",
        values: &[("year", "1912", 1912.0)],
    },
    DefaultItem {
        id: "h_6",
        name: "Discovery of Penicillin",
        icon_name: "FlaskConical",
        values: &[("year", "1928", 1928.0)],
    },
    DefaultItem {
        id: "h_7",
        name: "Yuri Gagarin Enters Space",
        icon_name: "Orbit",
        values: &[("year", "1961", 1961.0)],
    },
    DefaultItem {
        id: "h_anchor",
        name: "Apollo 11 Moon Landing",
        icon_name: "Rocket",
        values: &[("year", "1969", 1969.0)],
    },
    DefaultItem {
        id: "h_8",
        name: "Fall of the Berlin Wall",
        icon_name: "Shield",
        values: &[("year", "1989", 1989.0)],
    },
    DefaultItem {
        id: "h_9",
        name: "Launch of Wikipedia",
        icon_name: "Globe",
        values: &[("year", "2001", 2001.0)],
    },
    DefaultItem {
        id: "h_10",
        name: "Original iPhone Announced",
        icon_name: "Smartphone",
        values: &[("year", "2007", 2007.0)],
    },
];

const STATUE_COLUMNS: &[DefaultColumn] = &[
    DefaultColumn {
        id: "height",
        name: "Height (m)",
    },
    DefaultColumn {
        id: "year_built",
        name: "Year Built",
    },
];

const STATUE_ITEMS: &[DefaultItem] = &[
    DefaultItem {
        id: "s_1",
        name: "Colosseum (Rome)",
        icon_name: "Landmark",
        values: &[("height", "48 m", 48.0), ("year_built", "80 AD", 80.0)],
    },
    DefaultItem {
        id: "s_2",
        name: "Statue of Liberty (New York)",
        icon_name: "Landmark",
        values: &[("height", "93 m", 93.0), ("year_built", "1886", 1886.0)],
    },
    DefaultItem {
        id: "s_3",
        name: "Big Ben Clock Tower (London)",
        icon_name: "Clock",
        values: &[("height", "96 m", 96.0), ("year_built", "1859", 1859.0)],
    },
    DefaultItem {
        id: "s_4",
        name: "Great Pyramid of Giza (Egypt)",
        icon_name: "Triangle",
        values: &[
            ("height", "138 m", 138.0),
            ("year_built", "2560 BC", -2560.0),
        ],
    },
    DefaultItem {
        id: "s_5",
        name: "Statue of Unity (India)",
        icon_name: "Landmark",
        values: &[("height", "182 m", 182.0), ("year_built", "2018", 2018.0)],
    },
    DefaultItem {
        id: "s_6",
        name: "Golden Gate Bridge Tower",
        icon_name: "CableCar",
        values: &[("height", "227 m", 227.0), ("year_built", "1937", 1937.0)],
    },
    DefaultItem {
        id: "s_anchor",
        name: "Eiffel Tower (Paris)",
        icon_name: "Landmark",
        values: &[("height", "330 m", 330.0), ("year_built", "1889", 1889.0)],
    },
    DefaultItem {
        id: "s_7",
        name: "Empire State Building (New York)",
        icon_name: "Building2",
        values: &[("height", "381 m", 381.0), ("year_built", "1931", 1931.0)],
    },
    DefaultItem {
        id: "s_8",
        name: "Taipei 101 (Taipei)",
        icon_name: "Building2",
        values: &[("height", "508 m", 508.0), ("year_built", "2004", 2004.0)],
    },
    DefaultItem {
        id: "s_9",
        name: "Tokyo Skytree (Tokyo)",
        icon_name: "Radio",
        values: &[("height", "634 m", 634.0), ("year_built", "2012", 2012.0)],
    },
    DefaultItem {
        id: "s_10",
        name: "Burj Khalifa (Dubai)",
        icon_name: "Building2",
        values: &[("height", "828 m", 828.0), ("year_built", "2010", 2010.0)],
    },
];

const CINEMA_COLUMNS: &[DefaultColumn] = &[
    DefaultColumn {
        id: "box_office",
        name: "Box Office ($)",
    },
    DefaultColumn {
        id: "release_year",
        name: "Release Year",
    },
];

const CINEMA_ITEMS: &[DefaultItem] = &[
    DefaultItem {
        id: "m_1",
        name: "Pulp Fiction",
        icon_name: "Film",
        values: &[
            ("box_office", "$213 Million", 213.0),
            ("release_year", "1994", 1994.0),
        ],
    },
    DefaultItem {
        id: "m_2",
        name: "Star Wars: A New Hope",
        icon_name: "Film",
        values: &[
            ("box_office", "$775 Million", 775.0),
            ("release_year", "1977", 1977.0),
        ],
    },
    DefaultItem {
        id: "m_3",
        name: "The Lion King",
        icon_name: "Film",
        values: &[
            ("box_office", "$968 Million", 968.0),
            ("release_year", "1994", 1994.0),
        ],
    },
    DefaultItem {
        id: "m_4",
        name: "The Dark Knight",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.01 Billion", 1010.0),
            ("release_year", "2008", 2008.0),
        ],
    },
    DefaultItem {
        id: "m_5",
        name: "Jurassic Park",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.05 Billion", 1050.0),
            ("release_year", "1993", 1993.0),
        ],
    },
    DefaultItem {
        id: "m_6",
        name: "Barbie",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.44 Billion", 1440.0),
            ("release_year", "2023", 2023.0),
        ],
    },
    DefaultItem {
        id: "m_anchor",
        name: "Titanic",
        icon_name: "Film",
        values: &[
            ("box_office", "$2.26 Billion", 2260.0),
            ("release_year", "1997", 1997.0),
        ],
    },
    DefaultItem {
        id: "m_7",
        name: "Top Gun: Maverick",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.49 Billion", 1490.0),
            ("release_year", "2022", 2022.0),
        ],
    },
    DefaultItem {
        id: "m_8",
        name: "Spider-Man: No Way Home",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.92 Billion", 1920.0),
            ("release_year", "2021", 2021.0),
        ],
    },
    DefaultItem {
        id: "m_9",
        name: "Avengers: Endgame",
        icon_name: "Film",
        values: &[
            ("box_office", "$2.80 Billion", 2800.0),
            ("release_year", "2019", 2019.0),
        ],
    },
    DefaultItem {
        id: "m_10",
        name: "Avatar",
        icon_name: "Film",
        values: &[
            ("box_office", "$2.92 Billion", 2920.0),
            ("release_year", "2009", 2009.0),
        ],
    },
];

/// The three lists the draft ships with, per the spec's Assumptions.
pub const DEFAULT_LISTS: [DefaultList; 3] = [
    DefaultList {
        id: "default_world_history",
        title: "World History & Inventions",
        description: "Key human milestones from the printing press to the smartphone",
        order_direction: "low_to_high",
        columns: HISTORY_COLUMNS,
        items: HISTORY_ITEMS,
    },
    DefaultList {
        id: "default_statues_monuments",
        title: "Iconic Statues & Architectural Marvels",
        description: "World wonders with height and construction date columns",
        order_direction: "low_to_high",
        columns: STATUE_COLUMNS,
        items: STATUE_ITEMS,
    },
    DefaultList {
        id: "default_cinema_box_office",
        title: "Blockbuster Cinema Box Office",
        description: "Record-setting movies with box office revenue and budget",
        order_direction: "low_to_high",
        columns: CINEMA_COLUMNS,
        items: CINEMA_ITEMS,
    },
];

/// One column as the API reports it.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ColumnSummary {
    pub id: String,
    pub name: String,
}

/// A list as the API reports it: enough to pick a round, never the items.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSummary {
    pub id: String,
    pub title: String,
    pub description: String,
    pub order_direction: String,
    pub columns: Vec<ColumnSummary>,
    pub item_count: i64,
    pub is_default: bool,
}

/// The `{ "lists": [...] }` envelope the list endpoints return.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ListSummaries {
    pub lists: Vec<ListSummary>,
}

/// Every list in `folder_id` with its columns, oldest first. Reads the database
/// rather than the seed constants, so a folder created later answers the same
/// way the seeded one does.
pub fn list_summaries(conn: &Connection, folder_id: &str) -> ApiResult<ListSummaries> {
    let mut lists = Vec::new();
    let mut statement = conn.prepare(
        "SELECT id, title, description, order_direction, item_count, is_default
         FROM quiz_list WHERE folder_id = ?1 ORDER BY created_at ASC, title ASC",
    )?;
    let rows = statement.query_map(params![folder_id], |row| {
        Ok(ListSummary {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            order_direction: row.get(3)?,
            columns: Vec::new(),
            item_count: row.get(4)?,
            is_default: row.get::<_, i64>(5)? != 0,
        })
    })?;
    for row in rows {
        let mut list = row?;
        list.columns = columns_of(conn, &list.id)?;
        lists.push(list);
    }
    Ok(ListSummaries { lists })
}

/// A list's columns in display order; `columns[0]` is the default value column.
fn columns_of(conn: &Connection, list_id: &str) -> ApiResult<Vec<ColumnSummary>> {
    let mut statement = conn.prepare(
        "SELECT id, name FROM list_item_column WHERE list_id = ?1 ORDER BY position ASC",
    )?;
    let rows = statement.query_map(params![list_id], |row| {
        Ok(ColumnSummary {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;
    let columns = rows.collect::<Result<Vec<_>, _>>()?;
    Ok(columns)
}

/// Insert the seeded lists into `folder_id`. Idempotent, and a failure to write the
/// seed is reported rather than swallowed.
pub fn seed_default_lists(conn: &Connection, folder_id: &str) -> ApiResult<()> {
    let tx = conn.unchecked_transaction()?;
    for list in DEFAULT_LISTS.iter() {
        tx.execute(
            "INSERT OR IGNORE INTO quiz_list
                (id, folder_id, title, description, order_direction, item_count, is_default, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?7)",
            params![
                list.id,
                folder_id,
                list.title,
                list.description,
                list.order_direction,
                list.items.len() as i64,
                SEED_TIME,
            ],
        )?;
        for (position, column) in list.columns.iter().enumerate() {
            tx.execute(
                "INSERT OR REPLACE INTO list_item_column (list_id, id, name, position)
                 VALUES (?1, ?2, ?3, ?4)",
                params![list.id, column.id, column.name, position as i64],
            )?;
        }
        for (position, item) in list.items.iter().enumerate() {
            tx.execute(
                "INSERT OR IGNORE INTO item (id, list_id, name, position, icon_name)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![item.id, list.id, item.name, position as i64, item.icon_name],
            )?;
            for (column_id, display, numeric) in item.values.iter() {
                tx.execute(
                    "INSERT OR REPLACE INTO item_value (item_id, column_id, display, numeric, was_defaulted)
                     VALUES (?1, ?2, ?3, ?4, 0)",
                    params![item.id, column_id, display, numeric],
                )?;
            }
        }
    }
    tx.commit()?;
    Ok(())
}

/// Read a list's items ordered by position.
pub fn list_item_count(conn: &Connection, list_id: &str) -> ApiResult<usize> {
    let count = conn
        .query_row(
            "SELECT COUNT(*) FROM item WHERE list_id = ?1",
            params![list_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .unwrap_or(0);
    Ok(count as usize)
}

/// An Order round needs one anchor plus ten candidates, hence the minimum of 11
/// (spec Assumptions).
pub fn validate_list_size(count: usize) -> ApiResult<()> {
    if count < MIN_ITEMS {
        return Err(ApiError::new(
            crate::errors::ErrorCode::TooSmall,
            format!("A list needs at least {MIN_ITEMS} items; this one has {count}."),
        ));
    }
    if count > MAX_ITEMS {
        return Err(ApiError::new(
            crate::errors::ErrorCode::TooLarge,
            format!("A list cannot hold more than {MAX_ITEMS} items; this one has {count}."),
        ));
    }
    Ok(())
}

pub fn validate_list_title(title: &str) -> ApiResult<()> {
    let trimmed = title.trim();
    let len = trimmed.chars().count();
    if len == 0 {
        return Err(ApiError::bad_request("List title cannot be empty."));
    }
    if len > MAX_LIST_TITLE_CHARS {
        return Err(ApiError::bad_request(format!(
            "List title cannot exceed {MAX_LIST_TITLE_CHARS} characters."
        )));
    }
    Ok(())
}

pub fn validate_column_name(name: &str) -> ApiResult<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(ApiError::bad_request("Column name cannot be empty."));
    }
    if trimmed.chars().count() > MAX_COLUMN_NAME_CHARS {
        return Err(ApiError::bad_request(format!(
            "Column name cannot exceed {MAX_COLUMN_NAME_CHARS} characters."
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
