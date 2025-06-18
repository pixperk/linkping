use axum::{extract::FromRequestParts, http::StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ClickEvent{
    pub slug : String,
    pub ip : String,
    pub user_agent: String,
    pub referer : Option<String>,
    pub timestamp : String,
}

impl<S> FromRequestParts<S> for ClickEvent
where 
    S : Send + Sync,
    {
        type Rejection = StatusCode;

        async  fn from_request_parts(parts: &mut axum::http::request::Parts,_state: &S,) -> Result<Self,Self::Rejection> {
           let headers = &parts.headers;
              let user_agent = headers.get("user-agent")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            let referer = headers.get("referer")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let ip = parts
                .extensions
                .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
                .map(|info| info.0.ip().to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            let slug = parts
                .uri
                .path()
                .trim_start_matches('/')
                .to_string();

            let timestamp = chrono::Utc::now().to_rfc3339();

            Ok(ClickEvent {
                slug,
                ip,
                user_agent,
                referer,
                timestamp,
            }) 
        }
    }