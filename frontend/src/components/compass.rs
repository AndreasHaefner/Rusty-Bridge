
use leptos::prelude::*;
use leptos_router::{components::*, path, hooks::{use_params_map, use_navigate}};
use leptos_use::use_websocket;
use codee::string::JsonSerdeCodec; 
use shared::{Card, PlayerAction, PlayingCommand, BiddingCommand, Bid, BidLevel, Suit, PublicGameState, LobbyInfo, GamePhaseData, BiddingState, PlayingState, Team, PlayerPosition};
use uuid::Uuid;
use reqwest::Client;
use crate::components::helpers;

#[component]
pub fn CompassRose(current_turn: PlayerPosition, your_pos: PlayerPosition) -> impl IntoView {
    // Rose dreht sich so, dass your_pos unten ist (= Südpol der Rose)
    // South ist bei 0° unten → dein Offset = (index(your_pos) - 2) * 90
    let rotation = (helpers::pos_index(your_pos) - 2) * 90;
    let ring_style   = format!("transform: rotate({}deg);", rotation);
    let label_style  = format!("transform: rotate({}deg) translate(-50%, -50%);", -rotation);

    let dir_cls = move |pos: PlayerPosition| {
        let base = match pos {
            PlayerPosition::North => "compass-dir north",
            PlayerPosition::East  => "compass-dir east",
            PlayerPosition::South => "compass-dir south",
            PlayerPosition::West  => "compass-dir west",
        };
        if current_turn == pos { format!("{} active", base) } else { base.to_string() }
    };

    let lbl_cls = move |pos: PlayerPosition| {
        if your_pos == pos { "compass-label you" } else { "compass-label" }
    };

    let lbl = |pos: PlayerPosition| match pos {
        PlayerPosition::North=>"N", PlayerPosition::East=>"E",
        PlayerPosition::South=>"S", PlayerPosition::West=>"W",
    };

    view! {
        <div class="compass">
            <div class="compass-ring" style=ring_style>
                {[PlayerPosition::North, PlayerPosition::East, PlayerPosition::South, PlayerPosition::West]
                    .into_iter().map(|pos| {
                        let dc = dir_cls(pos);
                        let lc = lbl_cls(pos);
                        let ls = label_style.clone();
                        view! {
                            <div class=dc>
                                <span class=lc style=ls>{lbl(pos)}</span>
                            </div>
                        }
                    }).collect_view()
                }
                <div class="compass-center">"✦"</div>
            </div>
        </div>
    }
}