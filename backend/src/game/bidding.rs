use core::panic;

use shared::{Bid, BiddingCommand, BiddingState, GamePhaseData, PlayerPosition, PlayingState, Suit}; 
use crate::game::models::GameState;

pub fn process_bid(
    state: &mut GameState,
    player_pos: PlayerPosition,
    cmd: BiddingCommand,
) -> Result<(), String> {


    if state.current_player != player_pos {
        return Err("Du bist nicht am Zug!".to_string());
    }

    let bidding_state = match &mut state.phase {
        GamePhaseData::Bidding(bs) => bs,
        _ => return Err("Wir sind nicht in der Reizphase!".to_string()),
    };

    if bidding_state.bidding_finished {
        return Err("Reizen ist bereits beendet!".to_string());
    }


    match cmd {
        BiddingCommand::Pass => {
            handle_pass(bidding_state)?;
        }
        BiddingCommand::MakeBid { ref bid } => {
        handle_make_bid(bidding_state, player_pos, bid)?;
    }
        BiddingCommand::Double => {
            handle_double(bidding_state)?;
        }
        BiddingCommand::Redouble => {
            handle_redouble(bidding_state)?;
        }
    }


    bidding_state.history.push((player_pos, cmd));


    if !bidding_state.bidding_finished {
        state.current_player = state.current_player.next();
    }

    Ok(())
}

fn handle_pass(state: &mut BiddingState) -> Result<(), String> {
    state.consecutive_passes += 1;


    let required_passes_to_end = if state.highest_bid.is_none() { 4 } else { 3 };

    if state.consecutive_passes >= required_passes_to_end {
        state.bidding_finished = true;
        

    }

    Ok(())
}

fn handle_make_bid(state: &mut BiddingState, player_pos: PlayerPosition, new_bid: &Bid) -> Result<(), String> {

    if let Some(current_highest) = &state.highest_bid {
        if !new_bid.is_higher_than(current_highest) {
            return Err("Gebot muss höher sein als das aktuelle Höchstgebot.".to_string());
        }
    }

    state.highest_bid = Some(new_bid.clone());
    state.highest_bidder = Some(player_pos);
    

    state.consecutive_passes = 0;
    state.is_doubled = false;
    state.is_redoubled = false;

    Ok(())
}

fn handle_double(state: &mut BiddingState) -> Result<(), String> {
    if state.highest_bid.is_none() {
        return Err("Du kannst nur kontrieren, wenn bereits ein Gebot abgegeben wurde.".to_string());
    }
    if state.is_doubled || state.is_redoubled {
        return Err("Gebot ist bereits kontriert oder rekontriert.".to_string());
    }
    

    state.is_doubled = true;
    state.consecutive_passes = 0; 
    Ok(())
}


fn handle_redouble(state: &mut BiddingState) -> Result<(), String> {
    if !state.is_doubled {
        return Err("Du kannst nur rekontrieren, wenn vorher kontriert wurde.".to_string());
    }
    if state.is_redoubled {
        return Err("Gebot ist bereits rekontriert.".to_string());
    }



    state.is_redoubled = true;
    state.consecutive_passes = 0; 
    Ok(())
}

pub fn finalize_bidding(state: &mut GameState) {
   let bidding_state = match &mut state.phase {
        GamePhaseData::Bidding(bs) => bs,
        _ => panic!("Not in Bidding State"),
    };
    if let Some(final_bid) = bidding_state.highest_bid.clone() {
        let declarer = determine_declarer(bidding_state);
        
        // Dummy is the player on the team of the declarer
        let dummy = declarer.next().next(); 

        state.phase = GamePhaseData::Playing(PlayingState {
            contract: final_bid,
            is_doubled: bidding_state.is_doubled,
            is_redoubled: bidding_state.is_redoubled,
            declarer,
            dummy,
            table: std::collections::HashMap::new(),
            tricks_won_ns: 0,
            tricks_won_ew: 0,
        });

        // The Player to the left of declarer starts the game
        state.current_player = declarer.next();
    } else {
        // Everyone passed; no game is going to happen
        state.phase = GamePhaseData::Finished {
            winner_team: None,
            score: 0,
        };
    }
}

fn determine_declarer(state: &BiddingState) -> PlayerPosition {
    let final_bid = state.highest_bid.as_ref().expect("Bidding beendet ohne Gebot?");
    let winning_team = state.highest_bidder.expect("Kein Bieter?");
    
    let partner = winning_team.next().next();

    for (pos, cmd) in &state.history {
        if let BiddingCommand::MakeBid { bid } = cmd {
            if bid.suit == final_bid.suit && (*pos == winning_team || *pos == partner) {
                return *pos;
            }
        }
    }
    panic!("Err determining the winner");// Fallback (We should never end here
}