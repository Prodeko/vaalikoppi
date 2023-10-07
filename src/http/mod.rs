use axum::{Extension, Router};
use sqlx::{Pool, Postgres};
use tower::ServiceBuilder;

mod admin;
mod index;

pub async fn serve(db: Pool<Postgres>) {
    let app = router().layer(ServiceBuilder::new().layer(Extension(db)));
    let address = &"0.0.0.0:80".parse().unwrap();
    axum::Server::bind(address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn router() -> Router {
    index::router().merge(admin::tokens::router())
}
