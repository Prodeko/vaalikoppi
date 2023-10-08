use std::sync::Arc;

use axum::{Extension, Router};
use sqlx::{Pool, Postgres};
use tower::ServiceBuilder;

use crate::config::Config;

mod admin;
mod index;
mod static_files;

#[derive(Clone)]
pub struct Context {
    db: Pool<Postgres>,
    config: Arc<Config>,
}

pub async fn serve(db: Pool<Postgres>, config: Config) {
    let app = router().layer(ServiceBuilder::new().layer(Extension(Context {
        config: Arc::new(config),
        db,
    })));
    let address = &"0.0.0.0:80".parse().unwrap();
    axum::Server::bind(address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn router() -> Router {
    index::router()
        .merge(admin::tokens::router())
        .merge(static_files::router())
}
