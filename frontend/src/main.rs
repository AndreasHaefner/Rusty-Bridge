
use leptos::prelude::*;
use leptos_router::{components::*, path, hooks::{use_params_map, use_navigate}};
use leptos_use::use_websocket;
use codee::string::JsonSerdeCodec; 
use shared::{Card, PlayerAction, PlayingCommand, Suit, PublicGameState, LobbyInfo};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use reqwest::{Client};
#[component]
pub fn App() -> impl IntoView {

    view! {
        <Router>
            <main>
                <Routes fallback=|| view! { <div>"Seite nicht gefunden (404)"</div> }>
                    <Route path=path!("/") view=LoginScreen />
                    <Route path=path!("/hub") view=Hub />
                    <Route path=path!("/room/:id") view=GameRoom />
                </Routes>
            </main>
        </Router>
    }
} 

#[component]
fn LoginScreen() -> impl IntoView {
    let navigate = use_navigate();


  let login_action = Action::new_local(move |_: &()| {
    let navigate = navigate.clone();
    async move {

        web_sys::console::log_1(&"Button geklickt! Sende Request an /api/auth/guest...".into());
        let origin = web_sys::window()
            .unwrap()
            .location()
            .origin()
            .unwrap();
            
        let url = format!("{}/api/auth/guest", origin);
        
        let client = reqwest::Client::new();
        let res = client.post(&url)
    .fetch_credentials_include() 
    .send()
    .await; 
        match res {
            Ok(response) => {
                let status = response.status();
                web_sys::console::log_1(&format!("Antwort vom Server: {}", status).into());
                
                if status.is_success() {
                    web_sys::console::log_1(&"Login erfolgreich, leite zum Hub weiter...".into());
                    navigate("/hub", Default::default());
                } else {
                    web_sys::console::error_1(&"Login fehlgeschlagen! Status war nicht 2xx.".into());
                }
            }
            Err(e) => {
                web_sys::console::error_1(&format!("Netzwerkfehler (Server offline oder falsche URL?): {:?}", e).into());
            }
        }
    }
});

    view! {
        <div class="login-container">
            <h1>"Willkommen bei Rusty Bridge"</h1>
            <button on:click=move |_| { login_action.dispatch(()); }>
                "Als Gast spielen"
            </button>
        </div>
    }
}

// --- 3. DER HUB (REST GET & POST) ---
#[component]
fn Hub() -> impl IntoView {
    let navigate = use_navigate();

  
  let lobbies = LocalResource::new(move || async move {
    let origin = web_sys::window().unwrap().location().origin().unwrap();
    let url = format!("{}/api/lobbies", origin);
    
    let client = reqwest::Client::new(); 
    let res = client.get(&url)         
        .fetch_credentials_include()   
        .send()
        .await;
        
    if let Ok(response) = res {
        response.json::<Vec<LobbyInfo>>().await.unwrap_or_default()
    } else {
        vec![] 
    }
});

let navigate_clone = navigate.clone();
    let join_action = Action::new_local(move |lobby_id: &Uuid| {
        let lobby_id = *lobby_id;
        let nav = navigate_clone.clone();
        async move {
            let origin = web_sys::window().unwrap().location().origin().unwrap();
            let url = format!("{}/api/lobbies/{}/join", origin, lobby_id);
            
            let client = reqwest::Client::new();
            // Erstelle den Client
    let client = Client::new();

    let res = client.post(url)
        .fetch_credentials_same_origin()
        .send()
        .await;
         

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
            
            let client = reqwest::Client::new();
            if let Ok(res) =   client.post(&url)
            .fetch_credentials_same_origin()
            .send()
            .await {
                if let Ok(new_id) = res.text().await {
                    nav(&format!("/room/{}", new_id), Default::default());
                }
            }
        }
    });
    // View
    view! {
        <div class="hub-container">
            <h2>"Offene Tische"</h2>
            
            <button on:click=move |_| { create_lobby.dispatch(()); }>
                "Neuen Tisch erstellen"
            </button>

            <hr />

            <Transition fallback=move || view! { <p>"Lade Tische..."</p> }>
                <div class="lobby-list">
                    {move || lobbies.get().map(|list| {
                        if list.is_empty() {
                            view! { <p>"Keine aktiven Lobbys gefunden."</p> }.into_any()
                        } else {
                            list.into_iter().map(|lobby| {
                                let id = lobby.id;
                                view! {
                                    <div class="lobby-card" style="border: 1px solid #ccc; margin: 10px; padding: 10px;">
                                        <strong>{lobby.name}</strong>
                                        <span>" (" {lobby.players_count} "/4 Spieler)"</span>
                                        <button on:click=move |_| { join_action.dispatch(id); }>
                                            "Beitreten"
                                        </button>
                                    </div>
                                }
                            }).collect_view().into_any()
                        }
                    })}
                </div>
            </Transition>
        </div>
    }
} 



#[component]
fn GameRoom() -> impl IntoView {
    let params = use_params_map();
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

        //Starts websockets here
    let ws = use_websocket::<String, PublicGameState, JsonSerdeCodec>(&ws_url());
    let (game_state, set_game_state) = signal(None::<PublicGameState>);

    Effect::new(move |_| {
        if let Some(msg) = ws.message.get() {
            set_game_state.set(Some(msg));
        }
    });

    let play_card = move |card: Card| {
        let action = PlayerAction::Playing(PlayingCommand::PlayCard { card });
        if let Ok(json) = serde_json::to_string(&action) {
            (ws.send)(&json);
        }
    };


    view! {
        <div class="room">
            <h2>"Spieltisch: " {move || lobby_id()}</h2>

            <div class="game-container">
                {move || game_state.get().map(|s| view! {
                    <div class="info-bar">
                        "Phase: " {format!("{:?}", s.phase)} 
                        " | Dran ist: " {format!("{:?}", s.current_turn)}
                    </div>
                    
                    <div class="table-view">
                        <h3>"Tisch"</h3>
                        <div class="played-cards">
                            {s.table.into_iter().map(|(pos, card)| {
                                view! { <div>{format!("{:?}: {:?}", pos, card)}</div> }
                            }).collect_view()}
                        </div>
                    </div>

                    <HandView cards=s.my_hand title="Deine Hand".to_string() />
                })}

                <Show 
                    when=move || game_state.get().is_none()
                    fallback=|| view! { <div class="ready"></div> }
                >
                    <div class="loading">"Warten auf Mitspieler (Lobby)..."</div>
                </Show>
            </div>
        </div>
    }
}


#[component]
fn HandView(cards: Vec<Card>, title: String) -> impl IntoView {
    view! {
        <div class="hand-view">
            <h3 class="hand-title">{title}</h3>
            
            <div class="cards-container">
                {cards.into_iter().map(|card| {
                    let color = match card.suit {
                        Suit::Hearts | Suit::Diamonds => "#d32f2f",
                        _ => "#212121",
                    };
                    let symbol = match card.suit {
                        Suit::Spades => "♠",
                        Suit::Hearts => "♥",
                        Suit::Diamonds => "♦",
                        Suit::Clubs => "♣",
                    };
                    
                    view! {
                        <div class="card-item" style=format!("color: {color};")>
                            <div class="card-symbol">{symbol}</div>
                            <div class="card-rank">{format!("{:?}", card.rank)}</div>
                            <div class="card-symbol rotate-180">{symbol}</div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}



fn main() {
    mount_to_body(|| view! { <App /> })
}
