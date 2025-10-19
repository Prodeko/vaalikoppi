use std::sync::Arc;

use axum::{middleware::from_fn_with_state, Router};
use sqlx::{Pool, Postgres};
use tower_cookies::CookieManagerLayer;

use crate::{config::Config, middleware::resolve_ctx::resolve_ctx};

pub mod audit;
mod index;
pub mod login;
mod static_files;
pub mod tokens;
pub mod user;
pub mod votes;
mod votings;

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Postgres>,
    pub config: Arc<Config>,
}

pub async fn serve(db: Pool<Postgres>, config: Config) {
    let port = config.port;

    let state = AppState {
        config: Arc::new(config),
        db,
    };

    let app: Router = router(state.clone())
        .layer(from_fn_with_state(state.clone(), resolve_ctx))
        .layer(CookieManagerLayer::new())
        .with_state(state);

    let address = &format!("0.0.0.0:{port}").parse().unwrap();

    axum::Server::bind(address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn router(state: AppState) -> Router<AppState> {
    index::router()
        .nest("/tokens", tokens::router(state.clone()))
        .merge(login::router())
        .nest("/user", user::router())
        .merge(static_files::router())
        .nest("/votings", votings::router(state.clone()))
        .merge(votes::router())
        .nest("/audit", audit::router(state.clone()))
}
