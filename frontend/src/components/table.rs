
use leptos::prelude::*;
use leptos_router::{components::*, path, hooks::{use_params_map, use_navigate}};
use leptos_use::use_websocket;
use codee::string::JsonSerdeCodec; 
use shared::{Card, PlayerAction, PlayingCommand, BiddingCommand, Bid, BidLevel, Suit, PublicGameState, LobbyInfo, GamePhaseData, BiddingState, PlayingState, Team, PlayerPosition};
use uuid::Uuid;
use reqwest::Client;
use crate::components::helpers;
use crate::components::card;
// ═══════════════════════════════════════════════════════════
// TISCH
// ═══════════════════════════════════════════════════════════

#[component]
pub fn TableView(table: std::collections::HashMap<PlayerPosition, Card>,  your_pos: PlayerPosition) -> impl IntoView {
    let get_card = move |pos: PlayerPosition| table.get(&pos).cloned();
    let slot_south = your_pos;
    let slot_east  = your_pos.next();
    let slot_north = your_pos.next().next();
    let slot_west  = your_pos.next().next().next();
      let slot = move |pos: PlayerPosition| match get_card(pos) {
        Some(card) => view! { <card::PlayingCard card=card clickable=false on_click=None /> }.into_any(),
        None       => view! { <div class="table-slot-empty"></div> }.into_any(),
    };
    view! {
        <div class="table-area">
            <div class="table-felt">
                <div class="table-north">{slot(slot_north)}</div>
                <div class="table-middle">
                    <div class="table-west">{slot(slot_west)}</div>
                    <div class="table-center-dot">"♦"</div>
                    <div class="table-east">{slot(slot_east)}</div>
                </div>
                <div class="table-south">{slot(slot_south)}</div>
            </div>
        </div>
    }
}