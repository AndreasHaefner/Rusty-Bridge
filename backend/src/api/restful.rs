use axum::{
    extract::{State,Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
    
};

use tower_cookies::{Cookie, Cookies, cookie::SameSite};
use uuid::Uuid;
use std::sync::Arc;
use shared::LobbyInfo; // Gehe davon aus, dass das dein Struct aus dem Frontend ist

use crate::{helper::CurrentUser, state::AppState};

// POST /api/auth/guest
pub async fn guest_login_handler(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
) -> impl IntoResponse {
    // 1. Anonymen User in DB erstellen
    let user = match state.user_manager.create_anonymous_user().await {
        Ok(u) => u,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    // 2. Token generieren (einfach eine neue UUID als String)
    let token = Uuid::new_v4().to_string();

    // 3. Session in der DB speichern
    if state.user_manager.create_session(user.id, &token).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
// Im guest_login_handler
let mut token_cookie = Cookie::new("session_token", token);
token_cookie.set_path("/");
token_cookie.set_same_site(SameSite::Lax);
cookies.add(token_cookie);

// HIER: Der fehlende Cookie für den Extraktor
let mut user_id_cookie = Cookie::new("user_id", user.id.to_string());
user_id_cookie.set_path("/");
user_id_cookie.set_same_site(SameSite::Lax);
cookies.add(user_id_cookie);

    // Wir senden einfach ein OK zurück. Das Frontend navigiert dann zu /hub.
    StatusCode::OK.into_response()
}

// POST /api/lobbies
pub async fn create_lobby_handler(
    State(state): State<Arc<AppState>>,
    CurrentUser(current_user): CurrentUser, // Schützt die Route: Nur User mit Cookie dürfen das!
) -> impl IntoResponse {
    match state.lobby_manager.create_lobby(current_user).await {
        // Das Frontend erwartet den neuen lobby_id String als Textantwort
        Ok(lobby_id) => (StatusCode::CREATED, lobby_id.to_string()).into_response(),
        Err(e) => {
            eprintln!("Fehler beim Erstellen der Lobby: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// POST /api/lobbies/:id/join
pub async fn join_lobby_rest_handler(
    State(state): State<Arc<AppState>>,
    Path(lobby_id): Path<Uuid>,
    CurrentUser(user): CurrentUser,
) -> impl IntoResponse {
        //No ws connection yet, therefor none
    match state.lobby_manager.join_lobby(lobby_id, user, None).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}



// GET /api/lobbies

pub async fn list_lobbies_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.lobby_manager.get_active_lobbies().await {
        Ok(lobbies) => {
            println!("Lobbys gefunden: {}", lobbies.len()); // Debug-Log
            Json(lobbies).into_response()
        },
        Err(e) => {
            eprintln!("DB-Fehler in list_lobbies: {:?}", e); // HIER siehst du den echten Fehler!
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        },
    }
}