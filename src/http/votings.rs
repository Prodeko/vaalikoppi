use askama::Template;
use axum::{
    extract::{Path, State},
    middleware::from_fn,
    response::Html,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::{Pool, Postgres};

use crate::{
    middleware::require_admin_token::require_admin,
    models::{Voting, VotingResult},
};

use super::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:id", post(post_voting))
        .route("/:id", put(put_voting))
        .route("/:id", delete(delete_voting))
        .route_layer(from_fn(require_admin))
        .route("/", get(get_votings))
}

async fn post_voting(state: State<AppState>, id: Path<u64>) -> Html<String> {
    Html("hello world".to_string())
}
async fn put_voting(state: State<AppState>, id: Path<u64>) -> Html<String> {
    Html("hello world".to_string())
}
async fn delete_voting(state: State<AppState>, id: Path<u64>) -> Html<String> {
    Html("hello world".to_string())
}
async fn get_votings(state: State<AppState>) -> askama::Html {
    get_all_votings_html(state.db.clone()).await
}

#[derive(Template)]
#[template(path = "voting-list.html", ext = "html")]
struct VotingsTemplate<'a> {
    open_votings: Vec<Voting>,
    closed_votings: Vec<Voting>,
    ended_votings: Vec<VotingResult<'a>>,
    csrf_token: &'a str,
    is_admin: bool,
}

async fn get_all_votings_html(db: Pool<Postgres>) -> askama::Html {
    let template: VotingsTemplate = todo!();
    //template.render()
}
