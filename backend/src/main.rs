mod database;


use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use shared::{ PlayerPosition, PlayerAction, PublicGameState};
use database::DbConfig;
use std::{collections::HashMap, sync::Arc};
mod game;
use game::{models::{AppState}, engine, lobby::{Lobby}};
use tokio::sync::{mpsc, Mutex};
use axum::extract::State; 

use crate::game::GameState;
use futures_util::{SinkExt,StreamExt};


#[tokio::main]
async fn main() {
    //test -connect db and Cache (redis)
    let config = DbConfig::init().await.expect("Datenbank-Initialisierung fehlgeschlagen");

    config.redis_test_conn().await.expect("Redis Healthcheck fehlgeschlagen");
    
   let (game_tx, game_rx) = tokio::sync::mpsc::channel::<(PlayerPosition, PlayerAction)>(100);

    let shared_state = Arc::new(AppState {
        db: config,
        lobby: Mutex::new(Lobby::new()),
        game_tx, // Sender
        game_rx: Mutex::new(Some(game_rx)), // Receiver
    });
let app = Router::new()
    .route("/ws", get(ws_handler))
    .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}



async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>, 
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
    
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<PublicGameState>();
    
    let my_pos = {
        let mut lobby = state.lobby.lock().await;
        let pos = lobby.add_player(tx).expect("Lobby ist voll");
        
        if lobby.is_full() {
            let players = std::mem::replace(&mut lobby.players, HashMap::new());
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            let game_rx = state.game_rx.lock().await.take().expect("...");
            let initial_state = GameState::default(); 
            
            tokio::spawn(async move {
                engine::game_loop(initial_state, game_rx, players).await;
            });
}
        pos
    };

  
    tokio::spawn(async move {
    println!("WebSocket-Sender für Position gestartet!"); // Debugging ,ToDo -- Remove
    while let Some(update) = rx.recv().await {
        println!("Sende State an Client..."); 
        if let Ok(json) = serde_json::to_string(&update) {
            let _ = ws_sender.send(Message::Text(json.into())).await;
        }
    }

    });

    // 4. Sendeschleife: Nachrichten vom WebSocket in den Game-Loop werfen
    while let Some(Ok(Message::Text(text))) = ws_receiver.next().await {
        if let Ok(action) = serde_json::from_str::<PlayerAction>(&text) {
            let _ = state.game_tx.send((my_pos, action)).await;
        }
    }
}
    