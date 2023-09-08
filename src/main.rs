use axum::{routing::get, Router};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;

mod models;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let pg_url = std::env::var("DATABASE_URL").expect("DATABASE_URL was not set");

    let pool = PgPoolOptions::new()
        .max_connections(3)
        .connect(pg_url.as_str())
        .await
        .unwrap();

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Running DB migrations failed");

    let app = Router::new().route("/", get(|| async { "Hello, World!" }));
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap()
}
