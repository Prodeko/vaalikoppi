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
    TokenUnactivated,
    TokenVoided,
    WrongAdminToken,
}

#[derive(Serialize, Debug)]
pub enum InvalidAliasError {
    AliasAlreadyInUse,
    BadAlias,
}

#[serde_as]
#[derive(Serialize, Debug)]
pub enum ApiError {
    AuthFailed(AuthFailedError),
    InternalServerError,
    VotingNotFound,
    VotingAlreadyClosed,
    VotingNotOpen,
    NotAllActiveTokensHaveVoted,
    InvalidInput,
    AlreadyVoted,
    TokenNotFound,
    DatabaseError(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
    CorruptDatabaseError,
    TemplatingError(#[serde_as(as = "DisplayFromStr")] askama::Error),
    VotingAlgorithmError(&'static str),
    InvalidAlias(InvalidAliasError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        println!("{:?}", self);
        match self {
            ApiError::InvalidAlias(InvalidAliasError::AliasAlreadyInUse) => {
                (StatusCode::BAD_REQUEST, "Alias already in use").into_response()
            }
            ApiError::InvalidAlias(InvalidAliasError::BadAlias) => {
                (StatusCode::BAD_REQUEST, "Bad alias").into_response()
            }
            ApiError::AuthFailed(AuthFailedError::MissingToken) => {
                (StatusCode::UNAUTHORIZED, "Token missing").into_response()
            }
            ApiError::AuthFailed(AuthFailedError::InvalidToken) => {
                (StatusCode::UNAUTHORIZED, "Token invalid").into_response()
            }
            ApiError::AuthFailed(AuthFailedError::TokenUnactivated) => {
                (StatusCode::UNAUTHORIZED, "Token unactivated").into_response()
            }
            ApiError::AuthFailed(AuthFailedError::TokenVoided) => {
                (StatusCode::UNAUTHORIZED, "Token voided").into_response()
            }
            ApiError::AuthFailed(AuthFailedError::WrongAdminToken) => {
                (StatusCode::UNAUTHORIZED, "Wrong admin token").into_response()
            }
            ApiError::VotingNotFound => {
                (StatusCode::BAD_REQUEST, "Voting is not open").into_response()
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Unhandled client error").into_response(),
        }
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
