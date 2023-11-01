pub mod api_types;
mod config;
mod ctx;
mod helpers;
mod http;
mod middleware;
mod models;

use dotenv::dotenv;
use envconfig::Envconfig;

use helpers::create_pg_pool;
use http::serve;

use config::Config;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let config = Config::init_from_env().unwrap();

    let pool = create_pg_pool(&config.database_url, 3)
        .await
        .expect("Failed to create connection pool!");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Running DB migrations failed");

    serve(pool, config).await;
}
