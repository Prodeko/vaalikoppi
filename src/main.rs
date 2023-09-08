use dotenv::dotenv;
use vaalikoppi::helpers::create_pg_pool;
use vaalikoppi::http;

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

    http::serve(pool).await;
}
