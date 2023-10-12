use axum::Router;
use tower_http::services::ServeDir;

use super::AppState;

pub fn router() -> Router<AppState> {
    Router::new().nest_service("/static", ServeDir::new("src/static"))
}
