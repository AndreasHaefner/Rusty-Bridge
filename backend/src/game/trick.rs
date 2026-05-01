use std::collections::HashMap;
use shared::{Card, PlayerPosition, PlayingState, Suit};

/// Are all Cards following the rules?
/// Returns Err if valid card-play is violated
pub fn validate_card_play(
    hand: &[Card],
    card: &Card,
    led_suit: Option<Suit>,
) -> Result<(), String> {
    let Some(led) = led_suit else {
        return Ok(()); // First Player in trick - everything goes
    };

    if card.suit == led {
        return Ok(()); // Everything okay - suit gets followed
    }

    let has_led_suit = hand.iter().any(|c| c.suit == led);
    if has_led_suit {
        return Err(format!("Du musst {:?} bedienen!", led));
    }

    Ok(()) // No Cards in the played suit, so the play is valid
}

/// Determins winner of played trick (4 Cards played)
/// Returns Position of winning player
pub fn evaluate_trick(
    table: &HashMap<PlayerPosition, Card>,
    lead: PlayerPosition,       // Who played the first card?
    trump: Option<Suit>,        // Trump (None = Sans Atous)
) -> PlayerPosition {
    // Determined first played suit
    let led_suit = table[&lead].suit;

    let mut winner = lead;
    let mut winning_card = &table[&lead];

    for pos in PlayerPosition::all() {
        if pos == lead {
            continue;
        }
        let card = &table[&pos];
        if beats(card, winning_card, led_suit, trump) {
            winner = pos;
            winning_card = card;
        }
    }

    winner
}

/// Does 'challenger' beat the currently winning card?
fn beats(
    challenger: &Card,
    current_winner: &Card,
    led_suit: Suit,
    trump: Option<Suit>,
) -> bool {
    let challenger_is_trump = Some(challenger.suit) == trump;
    let winner_is_trump = Some(current_winner.suit) == trump;

    match (challenger_is_trump, winner_is_trump) {
        // Both trump -> higest wins
        (true, true) => rank_value(challenger) > rank_value(current_winner),
        // Only challenger trump -> is the new winning card
        (true, false) => true,
        // Only current winnering card is trump -> stays winning
        (false, true) => false,
        // No trump played at all -> highest Card value wins
        (false, false) => {
            challenger.suit == led_suit
                && rank_value(challenger) > rank_value(current_winner)
        }
    }
}


// ToDo maybe this shouldn't be here, it may be better in shared instead. OR maybe Rank should be a u8 with a serde mapping of written numbers 
fn rank_value(card: &Card) -> u8 {
    use shared::Rank::*;
    match card.rank {
        Two => 2, Three => 3, Four => 4, Five => 5, Six => 6,
        Seven => 7, Eight => 8, Nine => 9, Ten => 10,
        Jack => 11, Queen => 12, King => 13, Ace => 14,
    }
}