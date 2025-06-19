mod link;

use axum::Router;
use sqlx::PgPool;

use crate::routes::link::shorten_routes;

pub fn create_router(db : PgPool) -> Router {
    shorten_routes(db)
}
