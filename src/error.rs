use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub type Result<T> = core::result::Result<T, Error>;

pub enum AuthFailedError {
    MissingToken,
    InvalidToken,
}

pub enum Error {
    LoginFail,
    AuthFailed(AuthFailedError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, "Unhandled client error").into_response()
    }
}
