pub mod directory;
mod fallback;
pub mod file;
use axum::{routing::get, Router};
pub use fallback::*;
mod health;
use crate::application::Application;
pub use health::*;
pub fn create_router() -> Router<Application> {
    let api_router = Router::new()
        .route(
            "/v1/file",
            get(file::download_service_handler)
                .post(file::upload_service_handler)
                .delete(file::delete_service_handler),
        )
        .route(
            "/v1/directory",
            get(directory::list_service_handler)
                .put(directory::update_dir_service_handler)
                .post(directory::create_dir_service_handler)
                .delete(directory::remove_dir_service_handler),
        )
        .fallback(fallback_service_handler)
        .route("/v1/health_check", get(health_check_service_handler));
    return api_router;
}
