mod helpers;
mod http;
mod models;

use dotenv::dotenv;
use helpers::create_pg_pool;
use http::serve;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let pool = create_pg_pool(3)
        .await
        .expect("Failed to create connection pool!");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Running DB migrations failed");

    serve(pool).await;
}
