use askama::Template;
use axum::{response::Html, routing::get, Router};

use super::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_root))
}

#[derive(Template)]
#[template(path = "index.html")] // using the template in this path, relative
                                 // to the `templates` dir in the crate root

struct HelloTemplate<'a> {
    is_valid_token: &'a bool,
}

async fn get_root() -> Html<String> {
    let root = HelloTemplate {
        is_valid_token: &false,
    };
    Html(root.render().unwrap())
}
