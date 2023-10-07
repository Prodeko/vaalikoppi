use askama::Template;
use axum::{response::Html, routing::get, Router};

pub fn router() -> Router {
    Router::new().route("/", get(get_root))
}

#[derive(Template)]
#[template(path = "index.html")] // using the template in this path, relative
                                 // to the `templates` dir in the crate root

struct HelloTemplate {}

async fn get_root() -> Html<String> {
    let root = HelloTemplate {};
    Html(root.render().unwrap())
}
