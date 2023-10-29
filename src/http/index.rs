use askama::Template;
use axum::{extract::State, response::Html, routing::get, Router};
use sqlx::{Pool, Postgres};
use tower_cookies::Cookies;

use crate::models::Token;

use super::{user::USER_TOKEN, AppState};

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

async fn get_root(state: State<AppState>, cookies: Cookies) -> Html<String> {
    let token_cookie = cookies.get(USER_TOKEN);

    async fn get_token_object(
        token_cookie: Option<tower_cookies::Cookie<'_>>,
        db: &Pool<Postgres>,
    ) -> Option<Token> {
        let cookie = token_cookie?;
        let result = sqlx::query!("SELECT * FROM token WHERE id = $1", cookie.value())
            .fetch_one(db)
            .await;

        result.map_or(None, |r| {
            Some(Token {
                id: r.id,
                is_activated: r.is_activated,
                is_trashed: r.is_trashed,
                alias: r.alias,
            })
        })
    }

    let token = get_token_object(token_cookie, &state.db).await;
    println!("{:?}", token);

    let state: ClientState;

    match token {
        Some(token) => {
            if (token.is_activated && !token.is_trashed) {
                state = ClientState::LoggedIn {
                    is_valid_token: true,
                    user_alias: token.alias.unwrap_or("".to_string()),
                }
            } else {
                state = ClientState::NotLoggedIn {}
            }
        }
        None => state = ClientState::NotLoggedIn {},
    }

    Html(state.render().unwrap())
}
