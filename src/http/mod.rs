use std::sync::Arc;

use axum::Router;
use sqlx::{Pool, Postgres};
use tower_cookies::CookieManagerLayer;

use crate::config::Config;

pub mod admin;
mod index;
mod static_files;

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

    let app: Router = router().layer(CookieManagerLayer::new()).with_state(state);

    let address = &"0.0.0.0:80".parse().unwrap();
    axum::Server::bind(address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn router() -> Router<AppState> {
    index::router()
        .nest("/admin", admin::router())
        .merge(static_files::router())
}
