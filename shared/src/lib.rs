use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize,Hash)]
pub enum Suit { Clubs, Diamonds, Hearts, Spades }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")] 
pub enum Rank {
    Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten,
    Jack, Queen, King, Ace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

#[derive(Serialize)]
pub struct PrivateHandData {
    pub my_hand: Vec<Card>,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum PlayerPosition {
    North,
    East,
    South,
    West,
}

impl PlayerPosition {
    pub fn next(&self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East  => Self::South,
            Self::South => Self::West,
            Self::West  => Self::North,
        }
    }
    pub fn all() -> [Self; 4] {
        [Self::North, Self::East, Self::South, Self::West]
    }
    pub fn cycle() -> impl Iterator<Item = Self> {
        [Self::North, Self::East, Self::South, Self::West]
            .into_iter()
            .cycle()
    }
    pub fn partner(&self) -> Self {
        match self {
            PlayerPosition::North => PlayerPosition::South,
            PlayerPosition::South => PlayerPosition::North,
            PlayerPosition::East => PlayerPosition::West,
            PlayerPosition::West => PlayerPosition::East,
        }
    }
}


impl fmt::Display for PlayerPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        
        write!(f, "{:?}", self)
    }
}



#[derive(Serialize,Deserialize,  Debug, Clone)]
pub enum GamePhase {
    Bidding,
    Playing,
    Finished,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BiddingCommand {
    Bid { level: u8, suit: Suit },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlayingCommand {
    PlayCard { card: Card },
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlayerAction {
    Playing(PlayingCommand),
    Bidding(BiddingCommand),
}

#[derive(Serialize)]
pub struct ServerPush {
    pub current_phase: GamePhase,
    pub current_turn: PlayerPosition,
    pub update_data: GameUpdateData,
}
#[derive(Serialize)]
pub struct GameUpdateData {
    pub table_cards: HashMap<PlayerPosition, Card>, 
    pub last_action: Option<ActionInfo>,       
    pub scores: Vec<Score>,                         
    pub cards_left: CountCards,                        
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActionInfo {
    Action(PlayerPosition, PlayerAction),
    StatusMessage(String),
}
#[derive(Serialize)]
pub struct Score {
    pub player_score: usize
}
#[derive(Serialize)]
pub struct CountCards {
    pub cards_left: usize
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PublicGameState {
    pub pot: i32,
    pub my_hand: Vec<Card>,
    pub table: HashMap<PlayerPosition, Card>,
    pub current_turn: PlayerPosition,
    pub phase: GamePhase,
}



#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct LobbyInfo {
    pub id: Uuid,
    pub name: String,
    pub players_count: u8,
}


#[derive(Serialize, Deserialize)]
pub struct IdResponse {
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthData {
    pub user_id: Uuid,
    pub session_token: String,
}


