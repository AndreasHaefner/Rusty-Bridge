
use leptos::prelude::*;
use leptos_router::{components::*, path, hooks::{use_params_map, use_navigate}};
use leptos_use::use_websocket;
use codee::string::JsonSerdeCodec; 
use shared::{Card, PlayerAction, PlayingCommand, BiddingCommand, Bid, BidLevel, Suit, PublicGameState, LobbyInfo, GamePhaseData, BiddingState, PlayingState, Team, PlayerPosition};
use uuid::Uuid;
use reqwest::Client;

// ═══════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════

pub fn suit_symbol(suit: Suit) -> &'static str {
    match suit { Suit::Spades=>"♠", Suit::Hearts=>"♥", Suit::Diamonds=>"♦", Suit::Clubs=>"♣" }
}

pub fn suit_color(suit: Suit) -> &'static str {
    match suit { Suit::Hearts | Suit::Diamonds => "red", _ => "black" }
}

pub fn format_bid(bid: &Bid) -> String {
    let l = match bid.level {
        BidLevel::One=>"1", BidLevel::Two=>"2", BidLevel::Three=>"3",
        BidLevel::Four=>"4", BidLevel::Five=>"5", BidLevel::Six=>"6", BidLevel::Seven=>"7",
    };
    let s = match bid.suit {
        Some(Suit::Clubs)=>"♣", Some(Suit::Diamonds)=>"♦",
        Some(Suit::Hearts)=>"♥", Some(Suit::Spades)=>"♠", None=>"NT",
    };
    format!("{}{}", l, s)
}

pub fn rank_str(card: &Card) -> &'static str {
    use shared::Rank::*;
    match card.rank {
        Two=>"2", Three=>"3", Four=>"4", Five=>"5", Six=>"6",
        Seven=>"7", Eight=>"8", Nine=>"9", Ten=>"10",
        Jack=>"J", Queen=>"Q", King=>"K", Ace=>"A",
    }
}

pub fn rank_value(card: &Card) -> u8 {
    use shared::Rank::*;
    match card.rank {
        Two=>2, Three=>3, Four=>4, Five=>5, Six=>6, Seven=>7, Eight=>8,
        Nine=>9, Ten=>10, Jack=>11, Queen=>12, King=>13, Ace=>14,
    }
}

/// Bridge-Standard Farbreihenfolge: ♠ ♥ ♦ ♣
pub fn suit_order(suit: Suit) -> u8 {
    match suit { Suit::Spades=>0, Suit::Hearts=>1, Suit::Diamonds=>2, Suit::Clubs=>3 }
}

/// Sortiert nach Farbe (♠♥♦♣), innerhalb Farbe Ass→2
pub fn sort_hand(cards: &mut Vec<Card>) {
    cards.sort_by(|a, b| {
        suit_order(a.suit).cmp(&suit_order(b.suit))
            .then(rank_value(b).cmp(&rank_value(a)))
    });
}

/// Gruppiert in 4 Farb-Spalten (♠ ♥ ♦ ♣), jede absteigend sortiert
pub fn group_by_suit(cards: Vec<Card>) -> [(Suit, Vec<Card>); 4] {
    [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs].map(|suit| {
        let mut group: Vec<Card> = cards.iter().filter(|c| c.suit == suit).cloned().collect();
        group.sort_by(|a, b| rank_value(b).cmp(&rank_value(a)));
        (suit, group)
    })
}

// ═══════════════════════════════════════════════════════════
// KOMPASS — relativ zur eigenen Position
//
// Dein Platz ist immer optisch unten. Die Rose dreht sich so,
// dass deine Position am unteren Pol liegt.
//
// North=unten → 0°  East=unten → 90°  South=unten → 180°  West=unten → 270°
// Die Label werden gegenrotiert damit sie aufrecht bleiben.
// ═══════════════════════════════════════════════════════════

pub fn pos_index(pos: PlayerPosition) -> i32 {
    match pos { PlayerPosition::North=>0, PlayerPosition::East=>1, PlayerPosition::South=>2, PlayerPosition::West=>3 }
}
