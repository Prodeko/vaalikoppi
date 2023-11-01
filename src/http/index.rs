use askama::Template;
use axum::{response::Html, routing::get, Router};

use crate::{
    ctx::Ctx,
    models::{LoginState},
};
use axum::extract::State;

use crate::api_types::{ApiError, ApiResult};

use super::{
    votings::{get_votings_list_template, VotingListTemplate},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_root))
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {}

#[derive(Template)]
#[template(path = "voting.html")]
struct VotingTemplate {
    alias: String,
    votings_list_template: VotingListTemplate,
}

async fn get_root(context: Ctx, state: State<AppState>) -> ApiResult<Html<String>> {
    let template = async {
        match context.login_state() {
            LoginState::NotLoggedIn => LoginTemplate {}
                .render()
                .map_err(|_| ApiError::InternalServerError),
            LoginState::Voter { alias, .. } => {
                let votings_list_template =
                    get_votings_list_template(state.db.clone(), false).await?;

                VotingTemplate {
                    alias,
                    votings_list_template,
                }
                .render()
                .map_err(|e| e.into())
            }
            LoginState::Admin => Ok("This is admin-votings.html".to_string()),
        }
    }
    .await?;

    Ok(Html(template))
}
