use axum::{
    middleware::{from_fn, from_fn_with_state},
    Router,
};

use crate::middleware::require_admin_token::{require_admin, resolve_ctx};

use super::AppState;

pub mod index;
pub mod login;
pub mod tokens;

pub fn router(state: AppState) -> Router<AppState> {
    tokens::router()
        .route_layer(from_fn(require_admin))
        .layer(from_fn_with_state(state, resolve_ctx))
        .merge(login::router())
}
