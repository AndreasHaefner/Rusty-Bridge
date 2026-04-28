
use std::collections::HashMap;
use tokio::sync::mpsc;
use shared::{PlayerPosition, PublicGameState};


pub struct Lobby {
    pub players: HashMap<PlayerPosition, mpsc::UnboundedSender<PublicGameState>>,
}

impl Lobby {
    pub fn new() -> Self {
        Self { players: HashMap::new() }
    }

    pub fn add_player(&mut self, tx: mpsc::UnboundedSender<PublicGameState>) -> Option<PlayerPosition> {

        let positions = PlayerPosition::all();
        for pos in positions {
            if !self.players.contains_key(&pos) {
                self.players.insert(pos, tx);
                return Some(pos);
            }
        }
        None // Lobby full, can't add anything
    }

    pub fn is_full(&self) -> bool {
        self.players.len() == 4
    }
}

