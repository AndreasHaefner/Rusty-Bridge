use axum::{Json, response::IntoResponse};
use shared::AuthData;
use tower_cookies::{Cookie, Cookies, cookie::SameSite}; 

pub async fn login_handler(
    cookies: Cookies,
    auth_data: AuthData
) -> impl IntoResponse {

    let mut session_cookie = Cookie::new("session_token", auth_data.session_token);
    session_cookie.set_http_only(true);
    session_cookie.set_secure(true); 
    session_cookie.set_same_site(SameSite::Strict); 
    session_cookie.set_path("/");
    
  
    let mut userid_cookie = Cookie::new("user_id", auth_data.user_id.to_string());
    userid_cookie.set_http_only(true);
    userid_cookie.set_secure(true);
    userid_cookie.set_same_site(SameSite::Strict);
    userid_cookie.set_path("/");
    
    cookies.add(session_cookie);
    cookies.add(userid_cookie);
    Json("Login successfull")
}