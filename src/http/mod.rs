use axum::Router;
use sqlx::{Pool, Postgres};

mod index;

pub async fn serve(_db: Pool<Postgres>) {
    let app = router();
    let address = &"0.0.0.0:80".parse().unwrap();
    axum::Server::bind(address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn router() -> Router {
    index::router()
}
