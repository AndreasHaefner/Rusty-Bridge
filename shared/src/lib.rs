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


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GamePhaseData {
    Bidding(BiddingState),
    Playing(PlayingState),
    Finished { winner_team: Option<Team>, score: i32 },
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Team{
    NordSouth,
    EastWest
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BiddingState {
    pub history: Vec<(PlayerPosition, BiddingCommand)>,
    pub highest_bid: Option<Bid>,
    pub highest_bidder: Option<PlayerPosition>,
    pub consecutive_passes: u8,
    pub bidding_finished: bool,
    pub is_doubled: bool,
    pub is_redoubled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayingState {
    pub contract: Bid,
    pub is_doubled: bool,
    pub is_redoubled: bool,
    pub declarer: PlayerPosition,
    pub dummy: PlayerPosition,
    pub tricks_won_ns: u8,
    pub tricks_won_ew: u8,
    pub trick_lead: PlayerPosition,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BidLevel{
    One =1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7
}
impl BidLevel {
    pub fn val(&self) -> u8 {
        self.clone() as u8
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bid{
     pub level: BidLevel, 
     pub suit: Option<Suit> 
}

impl Bid {
    pub fn value(&self) -> u8 {
        let suit_value = match self.suit {
            Some(Suit::Clubs) => 0,
            Some(Suit::Diamonds) => 1,
            Some(Suit::Hearts) => 2,
            Some(Suit::Spades) => 3,
            None => 4,
        };
        (self.level.val() * 5) + suit_value     
    }

    pub fn is_higher_than(&self, other: &Bid) -> bool {
        self.value() > other.value()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BiddingCommand {
    MakeBid{bid: Bid},
    Pass,
    Redouble,
    Double
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
    pub current_phase: GamePhaseData,
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
    pub dummy_hand: Option<Vec<Card>>, 
    pub table: HashMap<PlayerPosition, Card>,
    pub current_turn: PlayerPosition,
    pub phase: GamePhaseData,
    pub your_pos: PlayerPosition, 
    pub opponent_card_counts: HashMap<PlayerPosition, usize>,
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


