mod shorten;

use axum::Router;
use sqlx::PgPool;

use crate::routes::shorten::shorten_routes;

pub fn create_router(db : PgPool) -> Router {
    shorten_routes(db)
}
