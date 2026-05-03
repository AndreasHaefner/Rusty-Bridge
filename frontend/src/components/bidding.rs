
use leptos::prelude::*;
use leptos_router::{components::*, path, hooks::{use_params_map, use_navigate}};
use leptos_use::use_websocket;
use codee::string::JsonSerdeCodec; 
use shared::{Card, PlayerAction, PlayingCommand, BiddingCommand, Bid, BidLevel, Suit, PublicGameState, LobbyInfo, GamePhaseData, BiddingState, PlayingState, Team, PlayerPosition};
use uuid::Uuid;
use reqwest::Client;
use crate::components::helpers;
use crate::components::hand;

// ═══════════════════════════════════════════════════════════
// BIETPHASE
// ═══════════════════════════════════════════════════════════

#[component]
pub fn BiddingView(
    state: BiddingState,
    current_turn: PlayerPosition,
    pub_state: PublicGameState,
    your_pos: PlayerPosition,
    on_action: Callback<PlayerAction>,
) -> impl IntoView {
    let is_my_turn = current_turn == your_pos;
    let pass     = move |_| on_action.run(PlayerAction::Bidding(BiddingCommand::Pass));
    let double   = move |_| on_action.run(PlayerAction::Bidding(BiddingCommand::Double));
    let redouble = move |_| on_action.run(PlayerAction::Bidding(BiddingCommand::Redouble));
    let (level, set_level) = signal(1u8);
    let (suit, set_suit)   = signal(Some(Suit::Spades));
    let make_bid = move |_| {
        let bid_level = match level.get() {
            1=>BidLevel::One, 2=>BidLevel::Two, 3=>BidLevel::Three,
            4=>BidLevel::Four, 5=>BidLevel::Five, 6=>BidLevel::Six, _=>BidLevel::Seven,
        };
        on_action.run(PlayerAction::Bidding(BiddingCommand::MakeBid {
            bid: Bid { level: bid_level, suit: suit.get() }
        }));
    };

    view! {
        <div class="bidding-layout">
        <div class="bidding-panel">
            <div class="bidding-header">
                <h3 class="panel-title">"Reizphase"</h3>
                {if let Some(b) = state.highest_bid.clone() {
                    view! { <div class="current-bid">"Höchstgebot: "<span class="bid-badge">{helpers::format_bid(&b)}</span></div> }.into_any()
                } else {
                    view! { <div class="current-bid">"Noch kein Gebot"</div> }.into_any()
                }}
            </div>

            {move || if is_my_turn {
                view! {
                    <div class="bidding-controls">
                        <div class="bid-builder">
                            <select class="bid-select" on:change=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<u8>() { set_level.set(v); }
                            }>
                                <option value="1">"1"</option><option value="2">"2"</option>
                                <option value="3">"3"</option><option value="4">"4"</option>
                                <option value="5">"5"</option><option value="6">"6"</option>
                                <option value="7">"7"</option>
                            </select>
                            <select class="bid-select" on:change=move |ev| {
                                let s = match event_target_value(&ev).as_str() {
                                    "C"=>Some(Suit::Clubs), "D"=>Some(Suit::Diamonds),
                                    "H"=>Some(Suit::Hearts), "S"=>Some(Suit::Spades), _=>None,
                                };
                                set_suit.set(s);
                            }>
                                <option value="S">"♠ Pik"</option>
                                <option value="H">"♥ Coeur"</option>
                                <option value="D">"♦ Karo"</option>
                                <option value="C">"♣ Treff"</option>
                                <option value="NT">"NT"</option>
                            </select>
                            <button class="btn-bid" on:click=make_bid>"Bieten"</button>
                        </div>
                        <div class="bid-actions">
                            <button class="btn-pass"     on:click=pass>"Passen"</button>
                            <button class="btn-double"   on:click=double>"Kontra"</button>
                            <button class="btn-redouble" on:click=redouble>"Rekontra"</button>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="waiting-turn">"Warte auf " {format!("{:?}", current_turn)} "..."</div>
                }.into_any()
            }}

            <div class="bid-history">
                <div class="bid-history-header">"Verlauf"</div>
                <div class="bid-history-grid">
                    <div class="bid-col-header">"N"</div>
                    <div class="bid-col-header">"E"</div>
                    <div class="bid-col-header">"S"</div>
                    <div class="bid-col-header">"W"</div>
                    {state.history.iter().map(|(pos, cmd)| {
                        let cmd_str = match cmd {
                            BiddingCommand::MakeBid { bid } => helpers::format_bid(bid),
                            BiddingCommand::Pass      => "Pas".to_string(),
                            BiddingCommand::Double    => "X".to_string(),
                            BiddingCommand::Redouble  => "XX".to_string(),
                        };
                        let pos_class = match pos {
                            PlayerPosition::North => "bid-history-entry hist-n",
                            PlayerPosition::East  => "bid-history-entry hist-e",
                            PlayerPosition::South => "bid-history-entry hist-s",
                            PlayerPosition::West  => "bid-history-entry hist-w",
                        };
                        view! { <div class=pos_class>{cmd_str}</div> }
                    }).collect_view()}
                </div>
            </div>
        </div>
        <div class="bidding-hand-container" style="margin-top: 2rem;">
                <hand::HandView
                    cards=pub_state.my_hand.clone()
                    label="Deine Hand".to_string()
                    clickable=false 
                    on_action=on_action
                />
        </div>
    </div>
        
    }
}