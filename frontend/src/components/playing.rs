
use leptos::prelude::*;
use leptos_router::{components::*, path, hooks::{use_params_map, use_navigate}};
use leptos_use::use_websocket;
use codee::string::JsonSerdeCodec; 
use shared::{Card, PlayerAction, PlayingCommand, BiddingCommand, Bid, BidLevel, Suit, PublicGameState, LobbyInfo, GamePhaseData, BiddingState, PlayingState, Team, PlayerPosition};
use uuid::Uuid;
use reqwest::Client;
use crate::components::helpers;
use crate::components::hand;
use crate::components::table;

// ═══════════════════════════════════════════════════════════
// SPIELPHASE — Positionslogik
//
// Du sitzt immer unten. Die vier absoluten Positionen werden
// relativ zu dir gemappt:
//   pos_bottom = du
//   pos_top    = gegenüber (2 Schritte im UZS)
//   pos_right  = rechts (1 Schritt)
//   pos_left   = links (3 Schritte = 1 gegen UZS)
// ═══════════════════════════════════════════════════════════

#[component]
pub fn PlayingView(
    play_state: PlayingState,
    pub_state: PublicGameState,
    on_action: Callback<PlayerAction>,
) -> impl IntoView {
    let your_pos     = pub_state.your_pos;
    let current_turn = pub_state.current_turn;
    let dummy_pos    = play_state.dummy;
    let declarer_pos = play_state.declarer;
    let dummy_hand   = pub_state.dummy_hand.clone().unwrap_or_default();
    let dummy_revealed = pub_state.dummy_hand.is_some();

    // Relative Positionen
    let pos_right  = your_pos.next();
    let pos_top    = your_pos.next().next();
    let pos_left   = your_pos.next().next().next();

    // Hilfsclosure: rendert eine Position korrekt
    let render = move |abs_pos: PlayerPosition, vertical: bool| {
        let is_me     = abs_pos == your_pos;
        let is_dummy  = abs_pos == dummy_pos;
        let my_turn   = current_turn == your_pos;
        let decl_dummy = your_pos == declarer_pos && current_turn == dummy_pos;

        let label = match abs_pos {
            PlayerPosition::North => "Nord", PlayerPosition::East => "Ost",
            PlayerPosition::South => "Süd",  PlayerPosition::West => "West",
        };

        if is_me {
            let clickable = my_turn;
            view! {
                <hand::HandView
                    cards=pub_state.my_hand.clone()
                    label=format!("{} (Du)", label)
                    clickable=clickable
                    on_action=on_action
                />
            }.into_any()
        } else if is_dummy && dummy_revealed {
            view! {
                <hand::DummyHandView
                    cards=dummy_hand.clone()
                    label=format!("{} (Dummy)", label)
                    clickable=decl_dummy
                    on_action=on_action
                />
            }.into_any()
        } else if is_dummy {
            let opp_count = pub_state.opponent_card_counts
                .get(&abs_pos)
                .copied()
                .unwrap_or(13);
            view! { <hand::OpponentHandView card_count=opp_count label=format!("{} (Dummy)", label) vertical=vertical /> }.into_any()
        } else {
             let opp_count = pub_state.opponent_card_counts
                .get(&abs_pos)
                .copied()
                .unwrap_or(13);
            view! { <hand::OpponentHandView card_count=opp_count label=label.to_string() vertical=vertical /> }.into_any()
        }
    };

    view! {
        <div class="playing-layout">
            <div class="contract-bar">
                <span class="contract-label">"Kontrakt: "</span>
                <span class="contract-value">{helpers::format_bid(&play_state.contract)}</span>
                {if play_state.is_doubled   { view!{ <span class="badge-x">"X"</span>   }.into_any() } else { view!{<span></span>}.into_any() }}
                {if play_state.is_redoubled { view!{ <span class="badge-xx">"XX"</span> }.into_any() } else { view!{<span></span>}.into_any() }}
                <span class="contract-by">"von " {format!("{:?}", play_state.declarer)}</span>
                <span class="tricks-score">
                    "N/S: " <strong>{play_state.tricks_won_ns}</strong>
                    " · E/W: " <strong>{play_state.tricks_won_ew}</strong>
                </span>
            </div>

            <div class="field-north">{render(pos_top, false)}</div>

            <div class="field-middle">
                <div class="field-west">{render(pos_left, true)}</div>
                <table::TableView table=pub_state.table.clone() your_pos=your_pos />
                <div class="field-east">{render(pos_right, true)}</div>
            </div>

            <div class="field-south">{render(your_pos, false)}</div>
        </div>
    }
}