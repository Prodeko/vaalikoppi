use askama::Template;
use axum::{
    debug_handler,
    extract::{Json, State},
    middleware::{from_fn, from_fn_with_state},
    response::Html,
    routing::{get, patch, post},
    Router,
};
use serde::Serialize;
use sqlx::{Postgres, QueryBuilder, Transaction};

use crate::{
    api_types::{ApiError, ApiResult},
    http::AppState,
    middleware::{require_is_admin::require_is_admin, resolve_token::resolve_token},
    models::{generate_token, LoginState, Token, TokenState, TokenUpdate},
};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/:id", patch(patch_token))
        .route_layer(from_fn_with_state(state, resolve_token))
        .route("/void-active", post(void_active_tokens))
        .route("/print", get(get_print_tokens))
        .route("/", get(get_tokens_page))
        .route("/", post(generate_tokens))
        .route_layer(from_fn(require_is_admin))
}

#[derive(Template)]
#[template(path = "components/admin-tokens.html")]
struct TokensTemplate {
    tokens: Vec<Token>,
    unactivated_token_count: i32,
    activated_token_count: i32,
    voided_token_count: i32,
}

#[derive(Template)]
#[template(path = "pages/admin-tokens.html")]
struct TokensPageTemplate {
    tokens: TokensTemplate,
    login_state: LoginState,
}

#[derive(Debug, Serialize)]
struct TokenInvalidateResult {
    count: i64,
}

#[debug_handler]
async fn void_active_tokens(state: State<AppState>) -> ApiResult<Json<TokenInvalidateResult>> {
    sqlx::query_as!(
        TokenInvalidateResult,
        "
        WITH updated_tokens AS (
            UPDATE token
            SET state = 'voided'::token_state
            WHERE state = 'activated'::token_state
            RETURNING id
        )
        SELECT count(id) as \"count!\"
        FROM updated_tokens
        "
    )
    .fetch_one(&state.db)
    .await
    .map(|t| Json(t))
    .map_err(|e| e.into())
}

#[derive(Template)]
#[template(path = "pages/admin-print-tokens.html")]
struct PrintTokensTemplate {
    tokens: Vec<Token>,
}

#[debug_handler]
async fn get_print_tokens(state: State<AppState>) -> ApiResult<Html<String>> {
    let tokens = sqlx::query_as!(
        Token,
        "
        select id, token, alias, state as \"state: TokenState\" from token
        where state = 'unactivated'::token_state;
        "
    )
    .fetch_all(&state.db)
    .await?;

    PrintTokensTemplate { tokens: tokens }
        .render()
        .map(|html| Html(html))
        .map_err(|_| ApiError::InternalServerError)
}

async fn get_tokens<'a>(conn: &mut Transaction<'a, Postgres>) -> ApiResult<TokensTemplate> {
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
    .fetch_all(&mut **conn)
    .await?;

    let mut unactivated_token_count = 0;
    let mut activated_token_count = 0;
    let mut voided_token_count = 0;

    tokens.iter().for_each(|t| match t.state {
        TokenState::Unactivated => unactivated_token_count += 1,
        TokenState::Activated => activated_token_count += 1,
        TokenState::Voided => voided_token_count += 1,
    });

    Ok(TokensTemplate {
        tokens,
        unactivated_token_count,
        activated_token_count,
        voided_token_count,
    })
}

#[debug_handler]
async fn get_tokens_page(state: State<AppState>) -> ApiResult<Html<String>> {
    let mut tx = state.db.begin().await?;
    let tokens_page_template = get_tokens(&mut tx).await?;

    let res = TokensPageTemplate {
        tokens: tokens_page_template,
        login_state: LoginState::Admin,
    }
    .render()
    .map(|html| Html(html))
    .map_err(|_| ApiError::InternalServerError);

    tx.commit().await?;

    res
}

#[debug_handler]
async fn patch_token(
    token: Token,
    state: State<AppState>,
    Json(token_update): Json<TokenUpdate>,
) -> ApiResult<Json<Token>> {
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
    fn handle_state_change(mut self, new_state: TokenState) -> ApiResult<Self> {
        match (self.state, new_state) {
            (_, TokenState::Voided) => {
                self.state = TokenState::Voided;
                Ok(self)
            }
            (TokenState::Unactivated, TokenState::Activated) => {
                self.state = TokenState::Activated;
                Ok(self)
            }
            _ => Err(ApiError::InvalidInput),
        }
    }
}

#[debug_handler]
async fn generate_tokens(state: State<AppState>) -> ApiResult<Html<String>> {
    // This could be passed with some request params if necessary
    let count = 100;

    if count == 0 {
        return Err(ApiError::InvalidInput);
    }

    let tokens = (0..count).map(|_| generate_token());

    let mut tx = state.db.begin().await?;

    let mut query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new("INSERT INTO token(token, state) ");

    query_builder.push_values(tokens, |mut b, token| {
        b.push_bind(token).push_bind(TokenState::Unactivated);
    });

    // TODO If tokens collide, the database will throw a duplicate key error which will return error code 500
    // to the admin's browser. This shouldn't break the application but the admin UX is bad.
    query_builder.build().execute(&mut *tx).await?;

    let res = get_tokens(&mut tx).await?.render().map(|html| Html(html))?;

    tx.commit().await?;

    Ok(res)
}
