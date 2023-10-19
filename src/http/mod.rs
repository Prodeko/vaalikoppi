use std::sync::Arc;

use axum::{middleware::from_fn_with_state, Router};
use sqlx::{Pool, Postgres};
use tower_cookies::CookieManagerLayer;

use crate::{config::Config, middleware::require_admin_token::resolve_ctx};

mod index;
pub mod login;
mod static_files;
pub mod tokens;
pub mod user;

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Postgres>,
    pub config: Arc<Config>,
}

pub async fn serve(db: Pool<Postgres>, config: Config) {
    let state = AppState {
        config: Arc::new(config),
        db,
    };

    let app: Router = router()
        .layer(from_fn_with_state(state.clone(), resolve_ctx))
        .layer(CookieManagerLayer::new())
        .with_state(state);

    let address = &"0.0.0.0:80".parse().unwrap();

    axum::Server::bind(address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn router() -> Router<AppState> {
    index::router()
        .merge(tokens::router())
        .merge(login::router())
        .merge(user::router())
        .merge(static_files::router())
}
