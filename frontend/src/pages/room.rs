
use leptos::prelude::*;
use leptos_router::{components::*, path, hooks::{use_params_map, use_navigate}};
use leptos_use::use_websocket;
use codee::string::JsonSerdeCodec; 
use shared::{Card, PlayerAction, PlayingCommand, BiddingCommand, Bid, BidLevel, Suit, PublicGameState, LobbyInfo, GamePhaseData, BiddingState, PlayingState, Team, PlayerPosition};
use uuid::Uuid;
use crate::components::{compass, bidding, playing};

// ═══════════════════════════════════════════════════════════
// GAME ROOM
// ═══════════════════════════════════════════════════════════

#[component]
pub fn GameRoom() -> impl IntoView {
    let params   = use_params_map();
    let lobby_id = move || params.read().get("id").unwrap_or_default().clone();

    let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    let p_id = match storage.get_item("player_id").unwrap() {
        Some(id) if !id.is_empty() => id,
        _ => {
            let new_id = Uuid::new_v4().to_string();
            storage.set_item("player_id", &new_id).unwrap();
            new_id
        }
    };

    let ws_url = move || format!("/ws?lobby_id={}&player_id={}", lobby_id(), p_id);
    let ws = use_websocket::<PlayerAction, PublicGameState, JsonSerdeCodec>(&ws_url());
    let (game_state, set_game_state) = signal(None::<PublicGameState>);

    Effect::new(move |_| {
        if let Some(msg) = ws.message.get() { set_game_state.set(Some(msg)); }
    });

    let on_action = Callback::new(move |action: PlayerAction| { (ws.send)(&action); });

    view! {
        <div class="room-bg">
            {move || game_state.get().map(|s| view! {
                <compass::CompassRose current_turn=s.current_turn your_pos=s.your_pos />
            })}

            {move || match game_state.get() {
                None => view! {
                    <div class="waiting-screen">
                        <div class="waiting-card">
                            <div class="waiting-spinner">"♠ ♥ ♦ ♣"</div>
                            <p>"Warten auf Mitspieler..."</p>
                            <p class="waiting-sub">"Tisch: " {move || lobby_id()}</p>
                        </div>
                    </div>
                }.into_any(),

                Some(s) => match s.phase.clone() {
                    GamePhaseData::Bidding(bs) => view! {
                        <div class="game-wrapper">
                            <bidding::BiddingView state=bs pub_state=s.clone() current_turn=s.current_turn your_pos=s.your_pos on_action=on_action />
                        </div>
                    }.into_any(),

                    GamePhaseData::Playing(ps) => view! {
                        <div class="game-wrapper">
                            <playing::PlayingView play_state=ps pub_state=s.clone() on_action=on_action />
                        </div>
                    }.into_any(),

                    GamePhaseData::Finished { winner_team, score } => view! {
                        <div class="finished-screen">
                            <div class="finished-card">
                                <h2>"Spiel beendet!"</h2>
                                <div class="finished-suits">"♠ ♥ ♦ ♣"</div>
                                <p class="finished-winner">"Gewinner: "<strong>{format!("{:?}", winner_team)}</strong></p>
                                <p class="finished-score">"Punkte: "<strong>{score}</strong></p>
                            </div>
                        </div>
                    }.into_any(),
                }
            }}
        </div>
    }
}
