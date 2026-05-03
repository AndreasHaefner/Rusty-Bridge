use std::collections::HashMap;
use tokio::sync::mpsc;
use shared::{Bid, Card, GamePhaseData, PlayerAction, PlayerPosition, PlayingCommand, PlayingState, PublicGameState, Suit, Team};
use crate::{dto::Player, game::{self, bidding, models::GameState, trick}};


type PlayerSenders = HashMap<PlayerPosition, mpsc::UnboundedSender<PublicGameState>>;
pub async fn game_loop(
    mut state: GameState, 
    mut rx: mpsc::Receiver<(PlayerPosition, PlayerAction)>,
    player_txs: PlayerSenders, 
) {
    for (pos, tx) in &player_txs {
        let _ = tx.send(state.for_player(*pos));
    }
    while let Some((player, action)) = rx.recv().await {
        let result = match action {
            PlayerAction::Playing(PlayingCommand::PlayCard { card }) => {
                process_play_card(&mut state, player, card)
            },
            PlayerAction::Bidding(bid_cmd) => { 
                
           let res = game::process_bid(&mut state, player, bid_cmd);
            if res.is_ok() {
                let finished = matches!(
                    &state.phase,
                    GamePhaseData::Bidding(bs) if bs.bidding_finished
                );
                if finished {
                    println!("Bidding beendet. Initialisiere Playing-Phase...");
                    bidding::finalize_bidding(&mut state);
                }
            }
            res
            }
        };

        if let Err(e) = result {
            println!("Fehler: {}", e);
            continue;
        }

        for (pos, tx) in &player_txs {
            let filtered = state.for_player(*pos); 
            let _ = tx.send(filtered); 
        }
    }
}
pub fn process_play_card(
    state: &mut GameState,
    player_position: PlayerPosition,
    card: Card,
) -> Result<(), String> {
    // Dummy-logic....
    let playing_state = match &state.phase {
        GamePhaseData::Playing(ps) => ps.clone(),
        _ => return Err("Nicht in der Spielphase!".to_string()),
    };

    let effective_player = if player_position == playing_state.declarer
        && state.current_player == playing_state.dummy
    {
        playing_state.dummy // Declarer uses dummy hand
    } else {
        player_position
    };

    if state.current_player != effective_player {
        return Err("Du bist nicht am Zug!".to_string());
    }

    // Remove card from hand
    let hand = state.hands.get_mut(&effective_player)
        .ok_or("Spieler hat keine Hand")?;

    // Check for suit played
    let led_suit = if state.table.is_empty() {
        None // None on the first move on trick
    } else {
        state.table.get(&playing_state.trick_lead).map(|c| c.suit)
    }; 

    trick::validate_card_play(hand, &card, led_suit)?;

    if let Some(pos) = hand.iter().position(|c| c == &card) {
        hand.remove(pos);
    } else {
        return Err("Karte nicht auf der Hand!".to_string());
    }

    state.table.insert(effective_player, card);
    state.current_player = state.current_player.next();

    // Trick complete (4 Cards)?
    if state.table.len() == 4 {
        resolve_trick(state);
    }

    Ok(())
}

fn resolve_trick(state: &mut GameState) {
    let GamePhaseData::Playing(ref mut ps) = state.phase else { return };
    let trump = ps.contract.suit;
    

    let winner = trick::evaluate_trick(&state.table, ps.trick_lead, trump);

    // Award points
    let GamePhaseData::Playing(ref mut ps) = state.phase else { return };
    match winner {
        PlayerPosition::North | PlayerPosition::South => ps.tricks_won_ns += 1,
        PlayerPosition::East  | PlayerPosition::West  => ps.tricks_won_ew += 1,
    }

    // Clear Table, winner leads next trick
    state.table.clear();
    ps.trick_lead = winner;
    state.current_player = winner;

    // Game ended? (13 Tricks)
    let total = ps.tricks_won_ns + ps.tricks_won_ew;
    if total == 13 {
        finalize_game(state);
    }
}

fn finalize_game(state: &mut GameState) {
    let GamePhaseData::Playing(ref ps) = state.phase else { return };
    
    let declarer = ps.declarer;
    let declarer_tricks = match declarer {
        PlayerPosition::North | PlayerPosition::South => ps.tricks_won_ns,
        PlayerPosition::East | PlayerPosition::West => ps.tricks_won_ew,
    };

    let is_vulnerable = false;

    let score_delta = calculate_bridge_score(
            &ps.contract, 
            ps.is_doubled,
            ps.is_redoubled,
            declarer_tricks, 
            is_vulnerable
        );
    let (winner_team, final_score) = if score_delta > 0 {
            // Declarer hat erfüllt
            let team = match declarer {
                PlayerPosition::North | PlayerPosition::South => Team::NordSouth,
                PlayerPosition::East | PlayerPosition::West => Team::EastWest,
            };
            (Some(team), score_delta)
        } else {
            // Declarer ist gefallen -> Verteidiger bekommen die Punkte
            let team = match declarer {
                PlayerPosition::North | PlayerPosition::South => Team::EastWest,
                PlayerPosition::East | PlayerPosition::West => Team::NordSouth,
            };
            (Some(team), -score_delta) // Punktestand für das Gewinnerteam ist positiv
        };

    println!("Spiel beendet! Gewinner: {:?}, Punkte: {}", winner_team, final_score);
    
    // Status sauber updaten
    state.phase = GamePhaseData::Finished { 
        winner_team, 
        score: final_score 
    }; 
}

pub fn calculate_bridge_score(
    contract: &Bid,
    is_doubled: bool,
    is_redoubled: bool,
    declarer_tricks: u8,
    is_vulnerable: bool
) -> i32 {
   
    let target_tricks = contract.level.val() + 6; 
    let mut score = 0;

    if declarer_tricks >= target_tricks {
        // All Good - Contact succeded
        let overtricks = declarer_tricks - target_tricks;
        
        // Contract Points
        let mut base_points = match contract.suit {
            Some(Suit::Clubs) | Some(Suit::Diamonds) => contract.level.val() as i32 * 20,
            Some(Suit::Hearts) | Some(Suit::Spades) => contract.level.val() as i32 * 30,
            None => 40 + (contract.level.val() as i32 - 1) * 30,
        };

        if is_doubled { base_points *= 2; score += 50; /* Insult bonus */ }
        if is_redoubled { base_points *= 4; score += 100; /* Insult bonus */ }
        score += base_points;

        // Game / Part-Score Bonus
        if base_points >= 100 {
            score += if is_vulnerable { 500 } else { 300 }; // Game Bonus
        } else {
            score += 50; // Part-Score Bonus
        }

        // Overtricks
        let overtrick_points = if is_redoubled {
            if is_vulnerable { 400 } else { 200 }
        } else if is_doubled {
            if is_vulnerable { 200 } else { 100 }
        } else {
            match contract.suit {
                Some(Suit::Clubs) | Some(Suit::Diamonds) => 20,
                Some(Suit::Hearts) | Some(Suit::Spades) | None => 30,
            }
        };
        score += overtricks as i32 * overtrick_points;

    } else {
       //Fallen - Contract Failed
        let undertricks = target_tricks - declarer_tricks;
        
        let penalty = if is_redoubled {
            if is_vulnerable { undertricks as i32 * 400 }
            else { undertricks as i32 * 200 }
        } else if is_doubled {
            if is_vulnerable { undertricks as i32 * 200 } 
            else { undertricks as i32 * 100 } 
        } else {
            if is_vulnerable { undertricks as i32 * 100 }
            else { undertricks as i32 * 50 }
        };
        
        score = -penalty;
    }

    score
}