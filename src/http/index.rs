use askama::Template;
use axum::{response::Html, routing::get, Router};

use crate::{
    ctx::Ctx,
    models::{LoginState, VotingState},
};
use axum::extract::State;

use crate::api_types::{ApiError, ApiResult};

use super::{
    votings::{
        get_admin_votings_list_template, get_votings_list_template, AdminVotingListTemplate,
        VotingListTemplate,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_root))
}

#[derive(Template)]
#[template(path = "pages/login.html")]
struct LoginTemplate {
    login_state: LoginState,
}

#[derive(Template)]
#[template(path = "pages/voter-home.html")]
struct VotingTemplate {
    pub login_state: LoginState,
    pub votings_list_template: VotingListTemplate,
}

#[derive(Template)]
#[template(path = "pages/admin-home.html")]
struct AdminVotingTemplate {
    pub login_state: LoginState,
    pub votings_list_template: AdminVotingListTemplate,
}

async fn get_root(context: Ctx, state: State<AppState>) -> ApiResult<Html<String>> {
    let template = async {
        match context.login_state() {
            LoginState::NotLoggedIn => LoginTemplate {
                login_state: context.login_state(),
            }
            .render()
            .map_err(|_| ApiError::InternalServerError),
            LoginState::Voter { .. } => {
                let votings_list_template =
                    get_votings_list_template(state.db.clone(), context.login_state(), None)
                        .await?;

                VotingTemplate {
                    login_state: context.login_state(),
                    votings_list_template,
                }
                .render()
                .map_err(|e| e.into())
            }
            LoginState::Admin => {
                let votings_list_template =
                    get_admin_votings_list_template(state.db.clone(), context.login_state())
                        .await?;

                AdminVotingTemplate {
                    login_state: context.login_state(),
                    votings_list_template,
                }
                .render()
                .map_err(|e| e.into())
            }
        }
    }
    .await?;

    Ok(Html(template))
}
