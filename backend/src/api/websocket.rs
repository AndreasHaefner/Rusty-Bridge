

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Query},
    response::IntoResponse,
};
use crate::{helper::CurrentUser, state::AppState};
use shared::{ PlayerAction, PublicGameState};


use std::{collections::HashMap, sync::Arc};

use crate::{ game::{ GameState}};
use futures_util::{SinkExt,StreamExt};
use uuid::Uuid;
use tokio::sync::{mpsc, };
use axum::extract::State; 



pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
    current_user_wrap : CurrentUser
) -> impl IntoResponse {
    let lobby_id = params.get("lobby_id").and_then(|s| Uuid::parse_str(s).ok());
  match (lobby_id, current_user_wrap) {
        (Some(l_id), current_user_wrap) => {
            ws.on_upgrade(move |socket| handle_socket(socket, state, l_id, current_user_wrap))
        }
        _ => {print!("Error from parsing get");
            (axum::http::StatusCode::BAD_REQUEST, "Lobby- oder Player-ID fehlt/ungültig").into_response()},
    }
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>, lobby_id: Uuid, CurrentUser(current_user) : CurrentUser) {
    println!("DEBUG: handle_socket gestartet für {}", lobby_id);
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<PublicGameState>();
    
    // Connect to the lobby
    let (my_pos, is_full) = match state.lobby_manager.join_lobby(lobby_id, current_user, Some(tx)).await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Lobby-Beitritt fehlgeschlagen: {:?}", e);
            return;
        }
    };

    println!("Spieler auf Position {:?} der Lobby {} beigetreten.", my_pos, lobby_id);

    // Lobby full, start game
    if is_full {
        println!("🎉 Tisch {} ist voll! Initialisiere Spiel...", lobby_id);
        
    
        if let Some((game_rx, players)) = state.lobby_manager.prepare_game_start(lobby_id).await {
            tokio::spawn(async move {
                crate::game::engine::game_loop(
                    GameState::default(), 
                    game_rx, 
                    players
                ).await;
            });
        }
    }

    // Receiver lopp Engine => browser
    let mut send_task = tokio::spawn(async move {
        while let Some(update) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&update) {
                if ws_sender.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Sending loop Browser => Enginge
    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = ws_receiver.next().await {
            if let Ok(action) = serde_json::from_str::<PlayerAction>(&text) {
              
                if let Some(game_tx) = state_clone.lobby_manager.get_game_tx(lobby_id).await {
                    let _ = game_tx.send((my_pos, action)).await;
                }
            }
            else {
                eprintln!("Error parsing WebSocket command: {}", text); 
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => println!("Sende-Einheit für {:?} beendet", my_pos),
        _ = &mut recv_task => {
            println!("Empfangs-Einheit für {:?} beendet. Cleanup...", my_pos);
           // ToDo Cleanup table after player left
        },
    }
}

