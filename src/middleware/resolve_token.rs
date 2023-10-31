use axum::{
    async_trait,
    extract::{FromRequestParts, Path, State},
    http::{request::Parts, Request},
    middleware::Next,
    response::Response,
};

use crate::{
    error::{Error, Result},
    http::AppState,
    models::{Token, TokenState, VotingId},
};

pub async fn resolve_token<B>(
    Path(id): Path<VotingId>,
    state: State<AppState>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response> {
    let token = sqlx::query_as!(
        Token,
        "
        SELECT
            id,
            token,
            state AS \"state: TokenState\",
            alias
        FROM token
        WHERE id = $1
        ",
        id
    )
    .fetch_optional(&state.db)
    .await?;

    token.map(|t| {
        req.extensions_mut().insert(t);
    });

    Ok(next.run(req).await)
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Token {
    type Rejection = Error;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        parts
            .extensions
            .get::<Token>()
            .map(|token| token.clone())
            .ok_or(Error::TokenNotFound)
    }
}
