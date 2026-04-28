use shared::{Bid, BiddingCommand, BiddingState, Card, GamePhaseData, PlayerAction, PlayerPosition, PublicGameState, Rank, Suit}; 
use serde::Deserialize;
use rand::seq::SliceRandom;
use uuid::Uuid;
use std::collections::HashMap;
use crate::game::lobby::{Lobby};
use crate::database::DbConfig;
use tokio::sync::{mpsc, Mutex};
use crate::dto::{LobbyManager};

pub struct Deck(pub Vec<Card>);

impl Deck {
    pub fn new() -> Self {
        let mut deck = Vec::with_capacity(52);
        for &suit in &[Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
            for &rank in &[
                Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six, 
                Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
                Rank::Jack, Rank::Queen, Rank::King, Rank::Ace
            ] {
                deck.push(Card { suit, rank });
            }
        }
        Deck(deck)
    }

    pub fn deal(mut self) -> [Vec<Card>; 4] {
        let mut rng = rand::rng();
        self.0.shuffle(&mut rng);

        let mut hands = self.0.chunks_exact(13).map(|chunk| chunk.to_vec());
        [
            hands.next().unwrap(),
            hands.next().unwrap(),
            hands.next().unwrap(),
            hands.next().unwrap(),
        ]
    }
}

#[derive(Deserialize)]
pub struct GameState {
    pub hands: HashMap<PlayerPosition, Vec<Card>>,
    pub table: HashMap<PlayerPosition, Card>,
    pub current_player: PlayerPosition,
    pub pot: i32,
    //pub phase: GamePhase,

    pub phase: GamePhaseData,

    //Bidding, maybe in other state Todo
   /*pub bidding_history: Vec<(PlayerPosition, BiddingCommand)>,
    pub highest_bid: Option<Bid>,
    pub highest_bidder: Option<PlayerPosition>,
    pub consecutive_passes: u8,
    pub is_doubled: bool,
    pub is_redoubled: bool,
    pub bidding_finished: bool,
     */ 
}
impl GameState {
    pub fn for_player(&self, player: PlayerPosition) -> PublicGameState {
        PublicGameState {
            pot: self.pot,
            my_hand: self.hands.get(&player).cloned().unwrap_or_default(),
            table: self.table.clone(),
            current_turn: self.current_player, 
            phase: self.phase.clone(),
            your_pos: player,
        }
    }
}
impl Default for GameState {
   fn default() -> Self {
        let dealt_hands = Deck::new().deal();
        let mut hands_map = HashMap::new();
        

        let positions = PlayerPosition::all();

        for (pos, cards) in positions.into_iter().zip(dealt_hands) {
            hands_map.insert(pos, cards);
        }

        Self {
            hands: hands_map,
            table: HashMap::new(),
            current_player: PlayerPosition::North,
            pot: 0,
            phase: GamePhaseData::Bidding(BiddingState::default()),
        }
    }
}


pub struct ActiveGame {
    pub game_tx: mpsc::Sender<(PlayerPosition, PlayerAction)>,
}


/* 

#[derive(Serialize)]

pub struct FilteredState {
    pub pot: i32,
    pub my_hand: Vec<Card>,
    pub table: HashMap<PlayerPosition, Card>, 
}
*/


// --- TESTS ---

#[cfg(test)]
mod tests {
    use super::*; 
    use std::time::Instant;

    #[test]
    fn test_entropy_bottleneck() {
        for i in 0..10000 {
            let start = Instant::now();
            let deck = Deck::new(); 
            let _ = deck.deal(); 
            let duration = start.elapsed();
            
            if duration.as_millis() > 1 {
                panic!("Lag bei Iteration {}: {:?}", i, duration);
            }
        }
    }
}

#[cfg(test)]
mod game_logic_tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_deck_integrity() {
        let deck = Deck::new();
        let hands = deck.deal();

        let mut all_cards: Vec<Card> = Vec::new();
        for hand in &hands {
            all_cards.extend(hand.iter());
        }
        assert_eq!(all_cards.len(), 52, "Es müssen genau 52 Karten sein");


        let unique_cards: HashSet<&Card> = all_cards.iter().collect();
        assert_eq!(unique_cards.len(), 52, "Das Deck enthält Duplikate!");
    }

    #[test]
    fn test_hand_size() {
        let deck = Deck::new();
        let hands = deck.deal();

        for (i, hand) in hands.iter().enumerate() {
            assert_eq!(hand.len(), 13, "Spieler {} hat nicht 13 Karten", i + 1);
        }
    }
}