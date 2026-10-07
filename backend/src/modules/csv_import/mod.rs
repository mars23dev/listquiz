//! CSV import and export for lists (US6, T110-T114, T118-T119).
//!
//! The `csv` crate does the tokenising, so a quoted field containing a comma
//! survives whole (research D-009). A value that yields no number keeps its display
//! text and is given a comparable value derived from its row position, and the
//! import report names every such row (research D-011, FR-057).

use serde::Serialize;

use crate::errors::{ApiError, ApiResult};

/// The most items a list may hold (FR-052, FR-059).
pub const MAX_ITEMS: usize = 100;
/// The fewest items a list may hold (FR-052, FR-059).
pub const MIN_ITEMS: usize = 11;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultedRow {
    pub position: usize,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub row_count: usize,
    pub defaulted_rows: Vec<DefaultedRow>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedColumn {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedValue {
    pub column_id: String,
    pub display: String,
    pub numeric: f64,
    pub was_defaulted: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedItem {
    pub name: String,
    pub icon_name: Option<String>,
    pub position: i64,
    pub values: Vec<ParsedValue>,
}

/// A list read out of a CSV, ready to be stored.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedList {
    pub title: String,
    pub order_direction: String,
    pub columns: Vec<ParsedColumn>,
    pub items: Vec<ParsedItem>,
    pub report: ImportReport,
}

/// A column id is a slug of its header, unique within the list (data-model.md).
pub fn slugify(name: &str) -> String {
    let mut slug = String::new();
    let mut last_underscore = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_underscore = false;
        } else if !last_underscore && !slug.is_empty() {
            slug.push('_');
            last_underscore = true;
        }
    }
    let slug = slug.trim_matches('_').to_string();
    if slug.is_empty() {
        "column".to_string()
    } else {
        slug
    }
}

/// The one numeric extraction rule (FR-057): strip everything except digits, `-`
/// and `.`, then parse. Returns `None` when nothing numeric is left.
pub fn parse_number(raw: &str) -> Option<f64> {
    let cleaned: String = raw
        .chars()
        .filter(|ch| ch.is_ascii_digit() || *ch == '-' || *ch == '.')
        .collect();
    if cleaned.is_empty() || cleaned == "-" || cleaned == "." {
        return None;
    }
    cleaned.parse::<f64>().ok()
}

/// Parse a CSV body into a list, with the report naming any defaulted rows.
pub fn parse_csv(title: &str, order_direction: &str, body: &str) -> ApiResult<ParsedList> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(body.as_bytes());

    let headers: Vec<String> = reader
        .headers()
        .map_err(|error| {
            ApiError::bad_request(format!("The CSV header could not be read: {error}"))
        })?
        .iter()
        .map(str::to_string)
        .collect();
    if headers.is_empty() {
        return Err(ApiError::bad_request("The CSV has no header row."));
    }

    // An optional leading icon column is detected case-insensitively (FR-057).
    let has_icon = headers[0].trim().eq_ignore_ascii_case("icon");
    let name_index = if has_icon { 1 } else { 0 };
    if headers.len() <= name_index {
        return Err(ApiError::bad_request(
            "The CSV needs a name column and at least one value column.",
        ));
    }

    let mut columns = Vec::new();
    let mut used_slugs: Vec<String> = Vec::new();
    for (offset, header) in headers.iter().skip(name_index + 1).enumerate() {
        let mut id = slugify(header);
        while used_slugs.contains(&id) {
            id.push('_');
        }
        used_slugs.push(id.clone());
        columns.push(ParsedColumn {
            id,
            name: header.trim().to_string(),
        });
        let _ = offset;
    }
    if columns.is_empty() {
        return Err(ApiError::bad_request(
            "The CSV needs at least one value column.",
        ));
    }

    let mut items = Vec::new();
    let mut defaulted_rows = Vec::new();
    let mut warnings = Vec::new();
    for (position, record) in reader.records().enumerate() {
        let record = record.map_err(|error| {
            ApiError::bad_request(format!("Row {} could not be read: {error}", position + 2))
        })?;
        let name = record
            .get(name_index)
            .unwrap_or_default()
            .trim()
            .to_string();
        if name.is_empty() {
            return Err(ApiError::bad_request(format!(
                "Row {} has no item name.",
                position + 2
            )));
        }
        let icon_name = if has_icon {
            let icon = record.get(0).unwrap_or_default().trim();
            if icon.is_empty() {
                None
            } else {
                Some(icon.to_string())
            }
        } else {
            None
        };

        let mut values = Vec::new();
        for (column_offset, column) in columns.iter().enumerate() {
            let raw = record
                .get(name_index + 1 + column_offset)
                .unwrap_or_default()
                .trim()
                .to_string();
            let (numeric, was_defaulted) = match parse_number(&raw) {
                Some(value) => (value, false),
                None => {
                    // A row that cannot be read still imports: the comparable value
                    // is its position, so the list stays playable (D-011).
                    let position_value = (position as f64 + 1.0) * 10.0;
                    defaulted_rows.push(DefaultedRow {
                        position: position + 1,
                        raw: raw.clone(),
                    });
                    warnings.push(format!(
                        "Row {}: value '{raw}' is not numeric; {position_value} was used instead.",
                        position + 2
                    ));
                    (position_value, true)
                }
            };
            values.push(ParsedValue {
                column_id: column.id.clone(),
                display: raw,
                numeric,
                was_defaulted,
            });
        }

        items.push(ParsedItem {
            name,
            icon_name,
            position: position as i64,
            values,
        });
    }

    validate_count(items.len())?;
    let row_count = items.len();

    Ok(ParsedList {
        title: title.to_string(),
        order_direction: order_direction.to_string(),
        columns,
        items,
        report: ImportReport {
            row_count,
            defaulted_rows,
            warnings,
        },
    })
}

fn validate_count(count: usize) -> ApiResult<()> {
    if count < MIN_ITEMS {
        return Err(ApiError::new(
            crate::errors::ErrorCode::TooSmall,
            format!("A list needs between {MIN_ITEMS} and {MAX_ITEMS} items; the CSV has {count}."),
        ));
    }
    if count > MAX_ITEMS {
        return Err(ApiError::new(
            crate::errors::ErrorCode::TooLarge,
            format!("A list needs between {MIN_ITEMS} and {MAX_ITEMS} items; the CSV has {count}."),
        ));
    }
    Ok(())
}

/// One exportable item: its icon, name, and a display value per column.
#[derive(Debug, Clone)]
pub struct ExportItem {
    pub icon_name: Option<String>,
    pub name: String,
    pub values: Vec<String>,
}

/// Render a list as CSV with the header `Icon,Name,<column names...>` (FR-058).
pub fn export_csv(column_names: &[String], items: &[ExportItem]) -> ApiResult<String> {
    let mut writer = csv::WriterBuilder::new().from_writer(Vec::new());

    let mut header = vec!["Icon".to_string(), "Name".to_string()];
    header.extend(column_names.iter().cloned());
    writer
        .write_record(&header)
        .map_err(|error| ApiError::new(crate::errors::ErrorCode::BadRequest, error.to_string()))?;

    for item in items {
        let mut record = vec![
            item.icon_name.clone().unwrap_or_default(),
            item.name.clone(),
        ];
        record.extend(item.values.iter().cloned());
        writer.write_record(&record).map_err(|error| {
            ApiError::new(crate::errors::ErrorCode::BadRequest, error.to_string())
        })?;
    }

    let bytes = writer
        .into_inner()
        .map_err(|error| ApiError::new(crate::errors::ErrorCode::BadRequest, error.to_string()))?;
    String::from_utf8(bytes)
        .map_err(|error| ApiError::new(crate::errors::ErrorCode::BadRequest, error.to_string()))
}

#[cfg(test)]
mod tests;
