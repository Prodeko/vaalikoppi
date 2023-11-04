use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

pub type ApiResult<T> = core::result::Result<T, ApiError>;

#[derive(Serialize, Debug)]
pub enum AuthFailedError {
    MissingToken,
    InvalidToken,
}

#[serde_as]
#[derive(Serialize, Debug)]
pub enum ApiError {
    LoginFail,
    AuthFailed(AuthFailedError),
    InternalServerError,
    VotingNotFound,
    VotingAlreadyClosed,
    InvalidInput,
    AlreadyVoted,
    TokenNotFound,
    DatabaseError(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
    CorruptDatabaseError,
    TemplatingError(#[serde_as(as = "DisplayFromStr")] askama::Error),
    VotingAlgorithmError,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        println!("{:?}", self);
        (StatusCode::INTERNAL_SERVER_ERROR, "Unhandled client error").into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(val: sqlx::Error) -> Self {
        Self::DatabaseError(val)
    }
}

impl From<askama::Error> for ApiError {
    fn from(val: askama::Error) -> Self {
        Self::TemplatingError(val)
    }
}

impl core::fmt::Display for ApiError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}
