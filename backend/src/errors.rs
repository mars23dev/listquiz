use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// One error type for the whole backend, mapped to the codes in
/// `specs/001-listquiz-game/contracts/rest-api.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    BadRequest,
    Unauthorized,
    NotFound,
    Conflict,
    TooSmall,
    TooLarge,
    NotReady,
}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::BadRequest => "bad_request",
            ErrorCode::Unauthorized => "unauthorized",
            ErrorCode::NotFound => "not_found",
            ErrorCode::Conflict => "conflict",
            ErrorCode::TooSmall => "too_small",
            ErrorCode::TooLarge => "too_large",
            ErrorCode::NotReady => "not_ready",
        }
    }

    pub fn status(self) -> StatusCode {
        match self {
            ErrorCode::BadRequest | ErrorCode::Unauthorized => StatusCode::BAD_REQUEST,
            ErrorCode::NotFound => StatusCode::NOT_FOUND,
            ErrorCode::Conflict | ErrorCode::NotReady => StatusCode::CONFLICT,
            ErrorCode::TooSmall | ErrorCode::TooLarge => StatusCode::UNPROCESSABLE_ENTITY,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorDetail,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: &'static str,
    message: String,
}

impl ApiError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        ApiError {
            code,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::BadRequest, message)
    }

    /// The product's own copy for a wrong password, kept verbatim from the draft.
    pub fn unauthorized() -> Self {
        Self::new(ErrorCode::Unauthorized, "Incorrect password!")
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Conflict, message)
    }

    pub fn not_ready(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotReady, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.code.status();
        let body = ErrorBody {
            error: ErrorDetail {
                code: self.code.as_str(),
                message: self.message,
            },
        };
        (status, Json(body)).into_response()
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for ApiError {}

pub type ApiResult<T> = Result<T, ApiError>;

impl From<rusqlite::Error> for ApiError {
    fn from(err: rusqlite::Error) -> Self {
        ApiError::new(ErrorCode::BadRequest, format!("storage error: {err}"))
    }
}
