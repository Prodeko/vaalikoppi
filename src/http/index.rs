use axum::{response::Html, routing::get, Router};

pub fn router() -> Router {
    Router::new().route("/", get(get_root))
}

async fn get_root() -> Html<&'static str> {
    let root_html = include_str!("../templates/index.html");
    Html(root_html)
}
