
use leptos::prelude::*;
use leptos_router::{components::*, path, hooks::{use_params_map, use_navigate}};
use leptos_use::use_websocket;
use codee::string::JsonSerdeCodec; 
use shared::{Card, PlayerAction, PlayingCommand, BiddingCommand, Bid, BidLevel, Suit, PublicGameState, LobbyInfo, GamePhaseData, BiddingState, PlayingState, Team, PlayerPosition};
use uuid::Uuid;
use reqwest::Client;
use crate::components::helpers;
// ═══════════════════════════════════════════════════════════
// SPIELKARTE
// ═══════════════════════════════════════════════════════════

#[component]
pub fn PlayingCard(card: Card, clickable: bool, on_click: Option<Callback<Card>>) -> impl IntoView {
    let sym = helpers::suit_symbol(card.suit);
    let col = helpers::suit_color(card.suit);
    let rnk = helpers::rank_str(&card);
    let handle_click = move |_| {
        if clickable { if let Some(cb) = on_click { cb.run(card); } }
    };
    view! {
        <div
            class=move || format!("playing-card {} {}", col, if clickable {"clickable"} else {""})
            on:click=handle_click
        >
            <div class="card-corner top-left">
                <div class="card-rank">{rnk}</div>
                <div class="card-suit-small">{sym}</div>
            </div>
            <div class="card-center-suit">{sym}</div>
            <div class="card-corner bottom-right">
                <div class="card-rank">{rnk}</div>
                <div class="card-suit-small">{sym}</div>
            </div>
        </div>
    }
}

#[component]
pub fn CardBack() -> impl IntoView {
    view! {
        <div class="playing-card card-back">
            <div class="card-back-pattern">"✦"</div>
        </div>
    }
}