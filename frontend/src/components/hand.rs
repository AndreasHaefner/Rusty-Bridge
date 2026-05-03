
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
// HÄNDE
// ═══════════════════════════════════════════════════════════

/// Eigene/klickbare Hand: aufgefächert, nach ♠♥♦♣ sortiert
#[component]
pub fn HandView(
    cards: Vec<Card>,
    label: String,
    clickable: bool,
    on_action: Callback<PlayerAction>,
) -> impl IntoView {
    let mut sorted = cards.clone();
    helpers::sort_hand(&mut sorted);
    view! {
        <div class="hand-area">
            <div class="hand-label">{label}</div>
            <div class="hand-cards">
                {sorted.into_iter().map(|card| {
                    let cb = Callback::new(move |c: Card| {
                        on_action.run(PlayerAction::Playing(PlayingCommand::PlayCard { card: c }));
                    });
                    view! { <card::PlayingCard card=card clickable=clickable on_click=Some(cb) /> }
                }).collect_view()}
            </div>
        </div>
    }
}

/// Verdeckte Gegner-Hand
#[component]
pub fn OpponentHandView(card_count: usize, label: String, vertical: bool) -> impl IntoView {
    view! {
        <div class=move || format!("hand-area opponent {}", if vertical {"vertical"} else {""})>
            <div class="hand-label">{label.clone()}</div>
            <div class=move || format!("hand-cards {}", if vertical {"vertical"} else {""})>
                {(0..card_count).map(|_| view! { <card::CardBack /> }).collect_view()}
            </div>
        </div>
    }
}

/// Dummy-Hand: 4 Spalten à Farbe, Karten vertikal gestapelt mit sichtbarem Rand
/// Klassische Bridge-Darstellung (wie auf BBO/Bridgebase)
#[component]
pub fn DummyHandView(
    cards: Vec<Card>,
    label: String,
    clickable: bool,
    on_action: Callback<PlayerAction>,
) -> impl IntoView {
    let groups = helpers::group_by_suit(cards);
    view! {
        <div class="dummy-hand-area">
            <div class="hand-label">{label}</div>
            <div class="dummy-columns">
                {groups.into_iter().map(|(suit, suit_cards)| {
                    let sym = helpers::suit_symbol(suit);
                    let col = helpers::suit_color(suit);
                    let empty = suit_cards.is_empty();
                    view! {
                        <div class="dummy-column">
                            <div class=format!("dummy-suit-header {}", col)>{sym}</div>
                            <div class="dummy-column-cards">
                                {if empty {
                                    view! { <div class="dummy-empty-suit">"—"</div> }.into_any()
                                } else {
                                    suit_cards.into_iter().map(|card| {
                                        let cb = Callback::new(move |c: Card| {
                                            on_action.run(PlayerAction::Playing(
                                                PlayingCommand::PlayCard { card: c }
                                            ));
                                        });
                                        view! {
                                            <card::PlayingCard card=card clickable=clickable on_click=Some(cb) />
                                        }
                                    }).collect_view().into_any()
                                }}
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
