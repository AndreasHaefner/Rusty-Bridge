use leptos::prelude::*;
use leptos_router::{components::*, path, hooks::{use_params_map, use_navigate}};
use leptos_use::use_websocket;
use codee::string::JsonSerdeCodec; 
use shared::{Card, PlayerAction, PlayingCommand, BiddingCommand, Bid, BidLevel, Suit, PublicGameState, LobbyInfo, GamePhaseData, BiddingState, PlayingState, Team, PlayerPosition};
use uuid::Uuid;
use reqwest::Client;


// ─── Login ────────────────────────────────────────────────────────────────────

#[component]
pub fn LoginScreen() -> impl IntoView {
    let navigate = use_navigate();
    let login_action = Action::new_local(move |_: &()| {
        let navigate = navigate.clone();
        async move {
            let origin = web_sys::window().unwrap().location().origin().unwrap();
            let url = format!("{}/api/auth/guest", origin);
            let client = Client::new();
            let res = client.post(&url).fetch_credentials_include().send().await;
            match res {
                Ok(response) if response.status().is_success() => {
                    navigate("/hub", Default::default());
                }
                _ => web_sys::console::error_1(&"Login fehlgeschlagen!".into()),
            }
        }
    });

    view! {
        <div class="login-bg">
            <div class="login-card">
                <div class="login-suit-row">
                    <span class="suit black">"♠"</span>
                    <span class="suit red">"♥"</span>
                    <span class="suit red">"♦"</span>
                    <span class="suit black">"♣"</span>
                </div>
                <h1 class="login-title">"Rusty Bridge"</h1>
                <p class="login-sub">
                    "Made with "
                    <span class="strikethrough">"love"</span>
                    <span class="rust-overlay">
                        <span class="rust-logo-mini"></span>
                        <span class="rust-text-mini">"Rust"</span>
                    </span>
                </p>
                <button class="btn-primary" on:click=move |_| { login_action.dispatch(()); }>
                    "Als Gast spielen"
                </button>
            </div>
        </div>
    }
}
