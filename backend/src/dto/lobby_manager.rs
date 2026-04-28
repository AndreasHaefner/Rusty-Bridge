use std::collections::HashMap;
use shared::{LobbyInfo, PlayerAction, PlayerPosition, PublicGameState};
use sqlx::PgPool;
use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use crate::dto::User;
pub struct Player {
    pub user_id: Uuid,        
    pub username: String,    
    pub pos: PlayerPosition,  // Position on the table
    pub tx: Option<mpsc::UnboundedSender<PublicGameState>>,
}

impl Player {
    pub fn from_user(user: User, pos: PlayerPosition) -> Self {
        Self { user_id: user.id, username: user.username, pos, tx: None }
    }
}


pub struct Lobby {
    pub lobby_id: Uuid,
    pub lobby_master: Player,
    pub players: HashMap<Uuid, Player>, 
    pub game_tx: mpsc::Sender<(PlayerPosition, PlayerAction)>,

}

pub struct LobbyManager {
    db: PgPool,
    active_games: Mutex<HashMap<Uuid, Lobby>>,
}

impl LobbyManager {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            active_games: Mutex::new(HashMap::new()),
        }
    }

    pub async fn activate_player_socket(
    &self, 
    lobby_id: Uuid, 
    user_id: Uuid, 
    tx: mpsc::UnboundedSender<PublicGameState>
) {
    let mut games = self.active_games.lock().await;
    if let Some(lobby) = games.get_mut(&lobby_id) {
        if let Some(player) = lobby.players.get_mut(&user_id) {
            player.tx = Some(tx); 
        }
    }
}

    pub async fn create_lobby(&self, creator: User) -> Result<Uuid, sqlx::Error> {
        let rec = sqlx::query!(
            "INSERT INTO lobbies (master_id, status) VALUES ($1, $2) RETURNING id",
            creator.id,
            "pending"
        )
        .fetch_one(&self.db)
        .await?;

        let mut players = HashMap::new();
        let pos = PlayerPosition::North; 
        
        players.insert(creator.id, Player::from_user(creator.clone(), pos));

        sqlx::query!(
            "INSERT INTO players (lobby_id, user_id, position) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            rec.id, creator.id, pos.to_string()
        ).execute(&self.db).await?;

        let (game_tx, _) = mpsc::channel(100);
        let mut games = self.active_games.lock().await;
        games.insert(rec.id, Lobby {
            lobby_id: rec.id,
            lobby_master: Player::from_user(creator, pos),
            players,
            game_tx,
        });

        Ok(rec.id)
    }



    pub async fn join_lobby(
        &self, 
        lobby_id: Uuid, 
        user: User, 
        tx: Option<mpsc::UnboundedSender<PublicGameState>> 
    ) -> Result<(PlayerPosition, bool), String> {
        let mut games = self.active_games.lock().await;
        let session = games.get_mut(&lobby_id).ok_or("Lobby existiert nicht")?;

        // is he already connected?
        if let Some(player_state) = session.players.get_mut(&user.id) {
            if tx.is_some() {
                player_state.tx = tx; 
            }
            return Ok((player_state.pos, session.players.len() == 4));
        }

        //try joining here
        if session.players.len() >= 4 {
            return Err("Lobby ist voll".to_string());
        }

        let taken_positions: Vec<PlayerPosition> = session.players.values().map(|s| s.pos).collect();
        let free_pos = PlayerPosition::all().into_iter()
            .find(|p| !taken_positions.contains(p))
            .ok_or("Keine Position frei")?;

        // store it in cache
        let mut new_player = Player::from_user(user.clone(), free_pos);
        new_player.tx = tx;
        session.players.insert(user.id, new_player);

        // Persist in DB
        let _ = sqlx::query!(
            "INSERT INTO players (lobby_id, user_id, position) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            lobby_id, user.id, free_pos.to_string()
        ).execute(&self.db).await;

        let is_full = session.players.len() == 4;
        Ok((free_pos, is_full))
    }


pub async fn get_active_lobbies(&self) -> Result<Vec<LobbyInfo>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT l.id, 
               u.username as "master_name?", -- Das "?" sagt sqlx: "Kann NULL sein"
               (SELECT COUNT(*) FROM players p WHERE p.lobby_id = l.id) as "player_count!"
        FROM lobbies l
        LEFT JOIN users u ON l.master_id = u.id
        WHERE l.status = 'pending'
        "#
    )
    .fetch_all(&self.db)
    .await?;

    Ok(rows.into_iter().map(|row| LobbyInfo {
        id: row.id.expect("invalid row id vom db"), 
        name: format!("Tisch von {}", row.master_name.as_deref().unwrap_or("Unbekannt")),
        players_count: row.player_count as u8,
    }).collect())
}

    pub async fn get_game_tx(&self, lobby_id: Uuid) -> Option<mpsc::Sender<(PlayerPosition, PlayerAction)>> {
        let games = self.active_games.lock().await;
        games.get(&lobby_id).map(|session| session.game_tx.clone())
    }

    pub async fn prepare_game_start(
        &self, 
        lobby_id: Uuid
    ) -> Option<(mpsc::Receiver<(PlayerPosition, PlayerAction)>, HashMap<PlayerPosition, mpsc::UnboundedSender<PublicGameState>>)> {
        let mut games = self.active_games.lock().await;
        let session = games.get_mut(&lobby_id)?;
        
        let (game_tx, game_rx) = mpsc::channel(100);
        session.game_tx = game_tx;
        
      let mut players_for_engine = HashMap::new();

    for player in session.players.values() {
        let sender = match &player.tx {
            Some(tx) => tx.clone(),
            None => {
                //ToDo - impl. Bots later. is  a voided channel for now
                let (dummy_tx, mut dummy_rx) = mpsc::unbounded_channel();
                
                tokio::spawn(async move {
                    while let Some(_) = dummy_rx.recv().await {}
                });
                
                println!("Player couldnt connect. Proceed without him");
                dummy_tx
            }
        };
        players_for_engine.insert(player.pos, sender);
    }
    
    Some((game_rx, players_for_engine))
}

}