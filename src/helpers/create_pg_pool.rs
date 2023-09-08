use sqlx::{postgres::PgPoolOptions, Error, Pool, Postgres};

pub async fn create_pg_pool(max_connections: u32) -> Result<Pool<Postgres>, Error> {
    let pg_url = std::env::var("DATABASE_URL").expect("DATABASE_URL was not set");

    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(pg_url.as_str())
        .await
}
