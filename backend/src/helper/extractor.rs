use std::sync::Arc;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::StatusCode;
use axum::http::request::Parts;
use uuid::Uuid;
use crate::state::AppState;
use crate::dto::User;
use tower_cookies::{Cookies, Cookie}; 
pub struct CurrentUser(pub User);


impl<S> FromRequestParts<S> for CurrentUser 
where 
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = Arc::from_ref(state);
        
        // Get Cookie From Request
        let cookies = Cookies::from_request_parts(parts, &state).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        // Extract UserID and Token from Cookie
        let user_id_str = cookies.get("user_id").map(|c| c.value().to_string()).ok_or(StatusCode::UNAUTHORIZED)?;
        let token = cookies.get("session_token").map(|c| c.value().to_string()).ok_or(StatusCode::UNAUTHORIZED)?;
        
        let user_id = Uuid::parse_str(&user_id_str).map_err(|_| StatusCode::BAD_REQUEST)?;

    
        match state.user_manager.verify_session(user_id, &token).await {
            Ok(Some(user)) => Ok(CurrentUser(user)),
            _ => Err(StatusCode::UNAUTHORIZED),
        }
    }
}