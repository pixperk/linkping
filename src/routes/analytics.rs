use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use validator::Validate;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{
    errors::AppError,
    services::analytics::get_analytics_data,
    models::analytics::{AnalyticsRequest, AnalyticsData},
};

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub timestamp: String,
    pub data: T,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub timestamp: String,
    pub error: String,
    pub status_code: u16,
}

pub async fn analytics_handler(
    State(db): State<sqlx::PgPool>,
    Path(slug): Path<String>,
    Query(params): Query<AnalyticsRequest>,
) -> Result<Json<ApiResponse<AnalyticsData>>, (StatusCode, Json<ErrorResponse>)> {
    // Validate the request parameters
    if let Err(e) = params.validate() {
        let error_response = ErrorResponse {
            success: false,
            timestamp: Utc::now().to_rfc3339(),
            error: format!("Validation error: {}", e),
            status_code: StatusCode::BAD_REQUEST.as_u16(),
        };
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }
    
    // Also validate the date range relationship if both dates are provided
    if let Err(e) = params.validate_date_range() {
        let error_response = ErrorResponse {
            success: false,
            timestamp: Utc::now().to_rfc3339(),
            error: format!("Date validation error: {}", e),
            status_code: StatusCode::BAD_REQUEST.as_u16(),
        };
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }
    
    // Get analytics data with query parameters
    match get_analytics_data(&db, slug, &params).await {
        Ok(analytics_data) => {
            let response = ApiResponse {
                success: true,
                timestamp: Utc::now().to_rfc3339(),
                data: analytics_data,
            };
            Ok(Json(response))
        },
        Err(e) => {
            let status_code = match &e {
                AppError::NotFound(_) => StatusCode::NOT_FOUND,
                AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
                AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            };
            
            let error_response = ErrorResponse {
                success: false,
                timestamp: Utc::now().to_rfc3339(),
                error: e.to_string(),
                status_code: status_code.as_u16(),
            };
            
            Err((status_code, Json(error_response)))
        }
    }
}
