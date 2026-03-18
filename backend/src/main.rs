mod database;
mod dto;
mod game;
mod api;
mod state;
mod helper;

use database::DbConfig;

use state::AppState;
use api::{websocket, restful};

use axum::{
    Router, routing::{get, post}
};
use tower_cookies::CookieManagerLayer;
use std::{ sync::Arc};

use crate::dto::{LobbyManager, User, UserManager, user_manager};
#[tokio::main]
async fn main() {
    //test -connect db and Cache (redis)
    println!("Main-Funktion gestartet!");
    let config = DbConfig::init().await.expect("Datenbank-Initialisierung fehlgeschlagen");

    config.redis_test_conn().await.expect("Redis Healthcheck fehlgeschlagen");
    println!("Redis Connected!");

    // LobbyManager mit dem DB-Pool erstellen
    let lobby_manager = LobbyManager::new(config.get_db().clone());
    let user_manager = UserManager::new(config.get_db().clone());
      println!("lobby_manager Created!");
    let shared_state = Arc::new(AppState {
        lobby_manager,
        user_manager
    });
    println!("shared_state Created!");
    

    let app = Router::new()
    .route("/ws", get(websocket::ws_handler))
    .route("/api/lobbies", post(restful::create_lobby_handler)) 
    .route("/api/lobbies", get(restful::list_lobbies_handler))
    .route("/api/lobbies/{id}/join", post(restful::join_lobby_rest_handler))
    // Creates User/Login
    .route("/api/auth/guest", post(restful::guest_login_handler)) //ToDo .route("/api/auth/new/register", axum::routing::get(restful::new_register_user))
  // Not Needed? .route("/api//lobbies/join",  axum::routing::get(restful::create_lobby_handler))
    //ToDo .route("/api/auth/login", axum::routing::get(restful::user_login))
    .layer(CookieManagerLayer::new()) //Important for user auth
    .with_state(shared_state);
        println!("app created");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
