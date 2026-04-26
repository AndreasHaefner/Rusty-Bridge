use shared::{PlayerPosition, BiddingCommand, Bid, Suit}; 
use crate::game::models::GameState;

pub fn process_bid(
    state: &mut GameState,
    player_pos: PlayerPosition,
    cmd: BiddingCommand,
) -> Result<(), String> {

    if state.current_player != player_pos {
        return Err("Du bist nicht am Zug!".to_string());
    }


    match cmd {
        BiddingCommand::Pass => {
            handle_pass(state)?;
        }
        BiddingCommand::MakeBid { ref bid } => {
        handle_make_bid(state, player_pos, bid)?;
    }
        BiddingCommand::Double => {
            handle_double(state)?;
        }
        BiddingCommand::Redouble => {
            handle_redouble(state)?;
        }
    }


    state.bidding_history.push((player_pos, cmd));


    if !state.bidding_finished {
        state.current_player = state.current_player.next();
    }

    Ok(())
}

fn handle_pass(state: &mut GameState) -> Result<(), String> {
    state.consecutive_passes += 1;


    let required_passes_to_end = if state.highest_bid.is_none() { 4 } else { 3 };

    if state.consecutive_passes >= required_passes_to_end {
        state.bidding_finished = true;
        

    }

    Ok(())
}

fn handle_make_bid(state: &mut GameState, player_pos: PlayerPosition, new_bid: &Bid) -> Result<(), String> {

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

fn handle_double(state: &mut GameState) -> Result<(), String> {
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


fn handle_redouble(state: &mut GameState) -> Result<(), String> {
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