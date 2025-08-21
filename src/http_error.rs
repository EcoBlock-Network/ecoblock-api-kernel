use axum::response::{IntoResponse, Response};
use axum::Json;
use axum::http::StatusCode;
use serde::Serialize;
use sqlx::Error as SqlxError;

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    code: Option<String>,
}

#[derive(Debug)]
pub struct AppError {
    pub status: StatusCode,
    pub message: String,
    pub code: Option<String>,
}

impl AppError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
    Self { status, message: message.into(), code: None }
    }
    
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
    let body = ErrorBody { error: self.message, code: self.code };
    (self.status, Json(body)).into_response()
    }
}

impl From<(StatusCode, String)> for AppError {
    fn from((status, msg): (StatusCode, String)) -> Self {
    AppError::new(status, msg)
    }
}

impl From<SqlxError> for AppError {
    fn from(e: SqlxError) -> Self {
        use sqlx::Error::*;
        match e {
            RowNotFound => AppError::new(StatusCode::NOT_FOUND, "notFound").with_code("not_found"),
            Database(db) => {
                if let Some(code) = db.code() {
                    if code == "23505" {
                        if let Some(cons) = db.constraint() {
                            let code_str = match cons {
                                "users_username_key" | "users_username_unique" => "duplicate_username",
                                "users_email_key" | "users_email_unique" => "duplicate_email",
                                other => {
                                    if other.contains("username") {
                                        "duplicate_username"
                                    } else if other.contains("email") {
                                        "duplicate_email"
                                    } else {
                                        "duplicate_key"
                                    }
                                }
                            };
                            return AppError { status: StatusCode::CONFLICT, message: "duplicateKey".to_string(), code: Some(code_str.to_string()) };
                        }
                        return AppError { status: StatusCode::CONFLICT, message: "duplicateKey".to_string(), code: Some("duplicate_key".to_string()) };
                    }
                }
                AppError::new(StatusCode::INTERNAL_SERVER_ERROR, db.message().to_string())
            }
            other => AppError::new(StatusCode::INTERNAL_SERVER_ERROR, other.to_string()),
        }
    }
}
