use std::collections::HashMap;
use tokio::sync::mpsc;
use shared::{Card, GamePhaseData, PlayerAction, PlayerPosition, PlayingCommand, PlayingState, PublicGameState};
use crate::{dto::Player, game::{self, bidding, models::GameState}};


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
pub fn process_play_card(state: &mut GameState, player_position: PlayerPosition, card: Card) -> Result<(), String> {


    if state.current_player != player_position {
        return Err("Du bist nicht am Zug!".to_string());
    }


    let hand = state.hands.get_mut(&player_position).ok_or("Spieler hat keine Hand")?;
    
    //find and remove the card from the hand here
    if let Some(pos) = hand.iter().position(|c| c == &card) {
        hand.remove(pos);
    } else {
        return Err("Karte nicht auf der Hand gefunden!".to_string());
    }

    state.table.insert(player_position, card);
    state.current_player = state.current_player.next();

    Ok(())

}