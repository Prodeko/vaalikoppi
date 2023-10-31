use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Serialize, Debug)]
pub enum AuthFailedError {
    MissingToken,
    InvalidToken,
}

#[serde_as]
#[derive(Serialize, Debug)]
pub enum Error {
    LoginFail,
    AuthFailed(AuthFailedError),
    InternalServerError,
    VotingNotFound,
    VotingAlreadyClosed,
    InvalidInput,
    AlreadyVoted,
    DatabaseError(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
    CorruptDatabaseError,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("{:?}", self);
        (StatusCode::INTERNAL_SERVER_ERROR, "Unhandled client error").into_response()
    }
}

impl From<sqlx::Error> for Error {
    fn from(val: sqlx::Error) -> Self {
        Self::DatabaseError(val)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}
