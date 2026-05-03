use leptos::prelude::*;
use leptos_router::{components::*, path, hooks::{use_params_map, use_navigate}};
use shared::{Card, PlayerAction, PlayingCommand, BiddingCommand, Bid, BidLevel, Suit, PublicGameState, LobbyInfo, GamePhaseData, BiddingState, PlayingState, Team, PlayerPosition};
use uuid::Uuid;
use reqwest::Client;



// ─── Hub ──────────────────────────────────────────────────────────────────────

#[component]
pub fn Hub() -> impl IntoView {
    let navigate = use_navigate();
    let lobbies = LocalResource::new(move || async move {
        let origin = web_sys::window().unwrap().location().origin().unwrap();
        let url = format!("{}/api/lobbies", origin);
        let client = Client::new();
        let res = client.get(&url).fetch_credentials_include().send().await;
        if let Ok(response) = res {
            response.json::<Vec<LobbyInfo>>().await.unwrap_or_default()
        } else { vec![] }
    });

    let navigate_clone = navigate.clone();
    let join_action = Action::new_local(move |lobby_id: &Uuid| {
        let lobby_id = *lobby_id;
        let nav = navigate_clone.clone();
        async move {
            let origin = web_sys::window().unwrap().location().origin().unwrap();
            let url = format!("{}/api/lobbies/{}/join", origin, lobby_id);
            let client = Client::new();
            let res = client.post(url).fetch_credentials_same_origin().send().await;
            if res.is_ok() && res.unwrap().status().is_success() {
                nav(&format!("/room/{}", lobby_id), Default::default());
            }
        }
    });

    let navigate_clone2 = navigate.clone();
    let create_lobby = Action::new_local(move |_: &()| {
        let nav = navigate_clone2.clone();
        async move {
            let origin = web_sys::window().unwrap().location().origin().unwrap();
            let url = format!("{}/api/lobbies", origin);
            let client = Client::new();
            if let Ok(res) = client.post(&url).fetch_credentials_same_origin().send().await {
                if let Ok(new_id) = res.text().await {
                    nav(&format!("/room/{}", new_id), Default::default());
                }
            }
        }
    });

    view! {
        <div class="hub-bg">
            <div class="hub-container">
                <h1 class="hub-title">"Rusty Bridge"</h1>
                <button class="btn-primary" on:click=move |_| { create_lobby.dispatch(()); }>
                    "+ Neuen Tisch erstellen"
                </button>
                <div class="hub-divider"><span>"Offene Tische"</span></div>
                <Transition fallback=move || view! { <p class="hub-loading">"Lade Tische..."</p> }>
                    <div class="lobby-list">
                        {move || lobbies.get().map(|list| {
                            if list.is_empty() {
                                view! { <p class="hub-empty">"Keine offenen Tische."</p> }.into_any()
                            } else {
                                list.into_iter().map(|lobby| {
                                    let id = lobby.id;
                                    let count = lobby.players_count;
                                    view! {
                                        <div class="lobby-card">
                                            <div class="lobby-card-left">
                                                <span class="lobby-name">{lobby.name.clone()}</span>
                                            </div>
                                            <div class="lobby-card-right">
                                                <span class="lobby-count">
                                                    {count} "/4"
                                                    <span class="lobby-pips">
                                                        {(0..4u8).map(|i| view! {
                                                            <span class=if i < count as u8 { "pip filled" } else { "pip" }>"●"</span>
                                                        }).collect_view()}
                                                    </span>
                                                </span>
                                                <button class="btn-join" on:click=move |_| { join_action.dispatch(id); }>
                                                    "Beitreten →"
                                                </button>
                                            </div>
                                        </div>
                                    }
                                }).collect_view().into_any()
                            }
                        })}
                    </div>
                </Transition>
            </div>
        </div>
    }
}