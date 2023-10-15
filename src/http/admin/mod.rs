use axum::Router;

use super::AppState;

pub mod index;
pub mod login;
pub mod tokens;

pub fn router() -> Router<AppState> {
    tokens::router().merge(login::router())
}
