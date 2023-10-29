use askama::Template;
use axum::{response::Html, routing::get, Router};

use crate::{ctx::Ctx, models::Token};

use super::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_root))
}

#[derive(Template)]
#[template(path = "index.html")] // using the template in this path, relative
                                 // to the `templates` dir in the crate root
enum ClientState {
    LoggedIn {
        is_valid_token: bool,
        user_alias: String,
    },
    NotLoggedIn,
}

async fn get_root(context: Ctx) -> Html<String> {
    let token = context.token();

    let state: ClientState = match token {
        None => ClientState::NotLoggedIn {},
        Some(Token {
            is_activated: false,
            ..
        }) => ClientState::NotLoggedIn {},
        Some(Token {
            is_trashed: true, ..
        }) => ClientState::NotLoggedIn {},
        Some(Token {
            is_activated: true,
            is_trashed: false,
            alias,
            ..
        }) => ClientState::LoggedIn {
            is_valid_token: true,
            user_alias: alias.unwrap_or("".to_string()),
        },
    };

    Html(state.render().unwrap())
}
