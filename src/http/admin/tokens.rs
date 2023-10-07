use axum::{extract::Json, routing::post, Extension, Router};
use serde::Deserialize;
use sqlx::{Pool, Postgres, QueryBuilder};

use crate::{helpers::generate_token, models::Token};

#[derive(Deserialize)]
struct GenerateTokenInput {
    count: u32,
}

pub fn router() -> Router {
    Router::new().route("/tokens", post(generate_tokens))
}

async fn generate_tokens(db: Extension<Pool<Postgres>>, Json(input): Json<GenerateTokenInput>) {
    if input.count == 0 {
        return;
    }

    let tokens = (0..input.count)
        .map(|_| generate_token())
        .map(|token_id| Token {
            id: token_id,
            is_activated: false,
            is_trashed: false,
        });

    let mut query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new("INSERT INTO token(id, is_activated, is_trashed) ");

    query_builder.push_values(tokens, |mut b, token| {
        b.push_bind(token.id)
            .push_bind(token.is_activated)
            .push_bind(token.is_trashed);
    });

    let result = query_builder.build().execute(&db.0).await;

    match result {
        Ok(res) => println!(
            "Successfully nserted {} tokens into database",
            res.rows_affected()
        ),
        Err(err) => print!("Error while trying to insert tokens to database: {}", err),
    }
}
