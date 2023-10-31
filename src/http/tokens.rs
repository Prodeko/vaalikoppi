use askama::Template;
use axum::{
    debug_handler,
    extract::{Json, State},
    middleware::{from_fn, from_fn_with_state},
    response::Html,
    routing::{get, patch, post},
    Router,
};
use serde::Deserialize;
use sqlx::{Postgres, QueryBuilder};

use crate::{
    error::{Error, Result},
    http::AppState,
    middleware::{require_admin_token::require_admin, resolve_token::resolve_token},
    models::{generate_token, Token, TokenState, TokenUpdate},
};

#[derive(Deserialize)]
struct GenerateTokenInput {
    count: u32,
}

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/:id", patch(patch_token))
        .route_layer(from_fn_with_state(state, resolve_token))
        .route("/", get(get_tokens))
        .route("/", post(generate_tokens))
        .route_layer(from_fn(require_admin))
}

#[derive(Template)]
#[template(path = "admin-tokens.html")]
struct TokensTemplate {
    tokens: Vec<Token>,
    unactivated_token_count: i32,
    activated_token_count: i32,
    voided_token_count: i32,
}

#[debug_handler]
async fn get_tokens(state: State<AppState>) -> Result<Html<String>> {
    let tokens = sqlx::query_as!(
        Token,
        "
        SELECT
            id,
            token,
            state AS \"state: TokenState\",
            alias
        FROM token
        "
    )
    .fetch_all(&state.db)
    .await?;
    let mut unactivated_token_count = 0;
    let mut activated_token_count = 0;
    let mut voided_token_count = 0;

    tokens.iter().for_each(|t| match t.state {
        TokenState::Unactivated => unactivated_token_count += 1,
        TokenState::Activated => activated_token_count += 1,
        TokenState::Voided => voided_token_count += 1,
    });

    TokensTemplate {
        tokens,
        unactivated_token_count,
        activated_token_count,
        voided_token_count,
    }
    .render()
    .map(|html| Html(html))
    .map_err(|_| Error::InternalServerError)
}

#[debug_handler]
async fn patch_token(
    token: Token,
    state: State<AppState>,
    Json(token_update): Json<TokenUpdate>,
) -> Result<Json<Token>> {
    let state_changed_token = token
        .handle_state_change(token_update.state)
        .map(|t| Json(t))?;

    sqlx::query_as!(
        Token,
        "
        UPDATE token
        SET state = $2
        WHERE id = $1
        RETURNING
            id,
            token,
            state AS \"state: TokenState\",
            alias
        ",
        state_changed_token.id,
        state_changed_token.state as TokenState
    )
    .fetch_one(&state.db)
    .await
    .map(|t| Json(t))
    .map_err(|e| e.into())
}

impl Token {
    fn handle_state_change(mut self, new_state: TokenState) -> Result<Self> {
        match (self.state, new_state) {
            (_, TokenState::Voided) => {
                self.state = TokenState::Voided;
                Ok(self)
            }
            (TokenState::Unactivated, TokenState::Activated) => {
                self.state = TokenState::Activated;
                Ok(self)
            }
            _ => Err(Error::InvalidInput),
        }
    }
}

#[debug_handler]
async fn generate_tokens(state: State<AppState>, Json(input): Json<GenerateTokenInput>) {
    if input.count == 0 {
        return;
    }

    let tokens = (0..input.count).map(|_| generate_token());

    let mut query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new("INSERT INTO token(token, state) ");

    query_builder.push_values(tokens, |mut b, token| {
        b.push_bind(token).push_bind(TokenState::Unactivated);
    });

    let result = query_builder.build().execute(&state.0.db).await;

    match result {
        Ok(res) => println!(
            "Successfully nserted {} tokens into database",
            res.rows_affected()
        ),
        Err(err) => print!("Error while trying to insert tokens to database: {}", err),
    }
}
