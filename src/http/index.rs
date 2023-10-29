use askama::Template;
use axum::{response::Html, routing::get, Router};

use crate::{ctx::Ctx, models::Token};
use axum::extract::State;

use crate::error::{Error, Result};

use super::{
    votings::{get_votings_list_template, VotingListTemplate},
    AppState,
};

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
        votings: VotingListTemplate,
        is_admin: bool,
    },
    NotLoggedIn,
}

async fn get_root(context: Ctx, state: State<AppState>) -> Result<Html<String>> {
    let token = context.token();

    let state: ClientState = async {
        let client_state: Result<ClientState> = match token {
            None => Ok(ClientState::NotLoggedIn {}),
            Some(Token {
                is_activated: false,
                ..
            }) => Ok(ClientState::NotLoggedIn {}),
            Some(Token {
                is_trashed: true, ..
            }) => Ok(ClientState::NotLoggedIn {}),
            Some(Token {
                is_activated: true,
                is_trashed: false,
                alias,
                ..
            }) => {
                let votings_list_template =
                    get_votings_list_template(state.db.clone(), context.is_admin()).await?;

                Ok(ClientState::LoggedIn {
                    is_valid_token: true,
                    user_alias: alias.unwrap_or("".to_string()),
                    votings: votings_list_template,
                    is_admin: context.is_admin(),
                })
            }
        };

        client_state
    }
    .await?;

    state
        .render()
        .map(|r| Html(r))
        .map_err(|_| Error::InternalServerError)
}
