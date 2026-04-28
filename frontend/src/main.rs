use leptos::prelude::*;
use leptos_router::{components::*, path, hooks::{use_params_map, use_navigate}};
use leptos_use::use_websocket;
use codee::string::JsonSerdeCodec; 
use shared::{Card, PlayerAction, PlayingCommand, BiddingCommand, Bid, BidLevel, Suit, PublicGameState, LobbyInfo, GamePhaseData, BiddingState, PlayingState, Team, PlayerPosition};
use uuid::Uuid;
use reqwest::Client;

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
            let origin = web_sys::window().unwrap().location().origin().unwrap();
            let url = format!("{}/api/auth/guest", origin);
            
            let client = Client::new();
            let res = client.post(&url)
                .fetch_credentials_include() 
                .send()
                .await; 

            match res {
                Ok(response) => {
                    if response.status().is_success() {
                        navigate("/hub", Default::default());
                    } else {
                        web_sys::console::error_1(&"Login fehlgeschlagen!".into());
                    }
                }
                Err(e) => web_sys::console::error_1(&format!("Netzwerkfehler: {:?}", e).into()),
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

#[component]
fn Hub() -> impl IntoView {
    let navigate = use_navigate();

    let lobbies = LocalResource::new(move || async move {
        let origin = web_sys::window().unwrap().location().origin().unwrap();
        let url = format!("{}/api/lobbies", origin);
        
        let client = Client::new(); 
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
            
            let client = Client::new();
            if let Ok(res) = client.post(&url)
                .fetch_credentials_same_origin()
                .send()
                .await {
                if let Ok(new_id) = res.text().await {
                    nav(&format!("/room/{}", new_id), Default::default());
                }
            }
        }
    });

    view! {
        <div class="hub-container">
            <h2>"Offene Tische"</h2>
            <button on:click=move |_| { create_lobby.dispatch(()); }>"Neuen Tisch erstellen"</button>
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
                                    <div class="lobby-card">
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


fn format_bid(bid: &Bid) -> String {
    let level_str = match bid.level {
        BidLevel::One => "1", BidLevel::Two => "2", BidLevel::Three => "3",
        BidLevel::Four => "4", BidLevel::Five => "5", BidLevel::Six => "6", BidLevel::Seven => "7",
    };
    let suit_str = match bid.suit {
        Some(Suit::Clubs) => "♣", Some(Suit::Diamonds) => "♦",
        Some(Suit::Hearts) => "♥", Some(Suit::Spades) => "♠", None => "NT",
    };
    format!("{} {}", level_str, suit_str)
}

#[component]
fn BiddingView(state: BiddingState, current_turn: PlayerPosition, on_action: Callback<PlayerAction>) -> impl IntoView {
    let pass = move |_| on_action.run(PlayerAction::Bidding(BiddingCommand::Pass));
    let double = move |_| on_action.run(PlayerAction::Bidding(BiddingCommand::Double));
    let redouble = move |_| on_action.run(PlayerAction::Bidding(BiddingCommand::Redouble));

    let (level, set_level) = signal(1u8); 
    let (suit, set_suit) = signal(Some(Suit::Spades)); 

    let make_bid = move |_| {
        let bid_level = match level.get() {
            1 => BidLevel::One, 2 => BidLevel::Two, 3 => BidLevel::Three,
            4 => BidLevel::Four, 5 => BidLevel::Five, 6 => BidLevel::Six, _ => BidLevel::Seven,
        };
        let bid = Bid { level: bid_level, suit: suit.get() };
        on_action.run(PlayerAction::Bidding(BiddingCommand::MakeBid { bid }));
    };

    view! {
        <div class="bidding-view">
            <h3>"Reizphase"</h3>
            <div class="highest-bid">
                <strong>"Aktuelles Höchstgebot: "</strong>
                {match state.highest_bid {
                    Some(b) => format_bid(&b),
                    None => "Noch kein Gebot".to_string()
                }}
            </div>
            
            <div class="bidding-controls">
                <button on:click=pass>"Passen"</button>
                <button on:click=double>"Kontra"</button>
                <button on:click=redouble>"Rekontra"</button>
                
                <div class="make-bid-form">
                    <select on:change=move |ev| {
                        if let Ok(val) = event_target_value(&ev).parse::<u8>() { set_level.set(val); }
                    }>
                        <option value="1">"1"</option><option value="2">"2"</option>
                        <option value="3">"3"</option><option value="4">"4"</option>
                        <option value="5">"5"</option><option value="6">"6"</option>
                        <option value="7">"7"</option>
                    </select>
                    
                    <select on:change=move |ev| {
                        let val = event_target_value(&ev);
                        let s = match val.as_str() {
                            "Clubs" => Some(Suit::Clubs), "Diamonds" => Some(Suit::Diamonds),
                            "Hearts" => Some(Suit::Hearts), "Spades" => Some(Suit::Spades), _ => None,
                        };
                        set_suit.set(s);
                    }>
                        <option value="Clubs">"Treff (♣)"</option>
                        <option value="Diamonds">"Karo (♦)"</option>
                        <option value="Hearts">"Coeur (♥)"</option>
                        <option value="Spades">"Pik (♠)"</option>
                        <option value="NoTrump">"Sans Atout (NT)"</option>
                    </select>
                    <button on:click=make_bid class="bid-btn">"Bieten"</button>
                </div>
            </div>

            <div class="bidding-history">
                <h4>"Historie"</h4>
                <ul>
                    {state.history.into_iter().map(|(pos, cmd)| {
                        let cmd_str = match cmd {
                            BiddingCommand::MakeBid { bid } => format!("bietet {}", format_bid(&bid)),
                            BiddingCommand::Pass => "passt".to_string(),
                            BiddingCommand::Double => "kontriert".to_string(),
                            BiddingCommand::Redouble => "rekontriert".to_string(),
                        };
                        view! { <li><strong>{format!("{:?}", pos)}</strong> ": " {cmd_str}</li> }
                    }).collect_view()}
                </ul>
            </div>
        </div>
    }
}

#[component]
fn PlayingView(state: PlayingState) -> impl IntoView {
    view! {
        <div class="playing-view">
            <h3>"Spielphase"</h3>
            <div class="contract-info">
                <strong>"Kontrakt: "</strong> {format_bid(&state.contract)}
                " von " <strong>{format!("{:?}", state.declarer)}</strong>
            </div>
            
            <div class="table-view">
                <h4>"Tisch (Gespielte Karten)"</h4>
                <div class="played-cards">
                    {if state.table.is_empty() {
                        view! { <p>"Noch keine Karten auf dem Tisch."</p> }.into_any()
                    } else {
                        state.table.into_iter().map(|(pos, card)| {
                            let color = match card.suit {
                                Suit::Hearts | Suit::Diamonds => "#d32f2f", _ => "#212121",
                            };
                            let symbol = match card.suit {
                                Suit::Spades => "♠", Suit::Hearts => "♥", Suit::Diamonds => "♦", Suit::Clubs => "♣",
                            };
                            view! { 
                                <div class="table-card" style=format!("color: {color};")>
                                    <div class="pos-label">{format!("{:?}", pos)}</div>
                                    <div class="card-face">{format!("{:?} {}", card.rank, symbol)}</div>
                                </div> 
                            }
                        }).collect_view().into_any()
                    }}
                </div>
            </div>

            <div class="tricks-info">
                <span>"Stiche N/S: " <strong>{state.tricks_won_ns}</strong></span>
                <span>" | E/W: " <strong>{state.tricks_won_ew}</strong></span>
            </div>
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
let ws = use_websocket::<PlayerAction, PublicGameState, JsonSerdeCodec>(&ws_url());
    
    let (game_state, set_game_state) = signal(None::<PublicGameState>);

    Effect::new(move |_| {
        if let Some(msg) = ws.message.get() {
            set_game_state.set(Some(msg));
        }
    });

   let on_action = Callback::new(move |action: PlayerAction| {
        (ws.send)(&action);
    });
    view! {
    <div class="room">
        <h2>"Spieltisch: " {move || lobby_id()}</h2>
        
        {move || game_state.get().map(|s| {

            let is_playing = matches!(s.phase, GamePhaseData::Playing(_));
            
            let phase_content = match s.phase.clone() {
                GamePhaseData::Bidding(bs) => view! { 
                    <BiddingView state=bs current_turn=s.current_turn on_action=on_action /> 
                }.into_any(),
                GamePhaseData::Playing(ps) => view! { 
                    <PlayingView state=ps /> 
                }.into_any(),
                GamePhaseData::Finished { winner_team, score } => view! { 
                    <div class="finished-view">
                        <h3>"Spiel beendet!"</h3>
                        <p>"Gewinner: " <strong>{format!("{:?}", winner_team)}</strong></p>
                        <p>"Punkte: " {score}</p>
                    </div> 
                }.into_any(),
            };

            view! {
                <div class="my-pos">
                    "Deine Position: " <strong>{format!("{:?}", s.your_pos)}</strong>
                </div>

                <div class="game-container">
                    <div class="info-bar">
                        "Am Zug ist: " <strong class="highlight-turn">{format!("{:?}", s.current_turn)}</strong>
                    </div>
                    
                    {phase_content}

                    <HandView 
                        cards=s.my_hand.clone() 
                        title="Deine Hand".to_string() 
                        is_playing=is_playing 
                        on_action=on_action 
                    />
                </div>
            }
        })}

        <Show 
            when=move || game_state.get().is_none()
            fallback=|| view! { <div class="ready"></div> }
        >
            <div class="loading">"Warten auf Mitspieler (Lobby)..."</div>
        </Show>
    </div>
    }

    }


#[component]
fn HandView(cards: Vec<Card>, title: String, is_playing: bool, on_action: Callback<PlayerAction>) -> impl IntoView {
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
                        Suit::Spades => "♠", Suit::Hearts => "♥", Suit::Diamonds => "♦", Suit::Clubs => "♣",
                    };
                    
                    let card_clone = card.clone();
                    let on_card_click = move |_| {
                        if is_playing {
                            on_action.run(PlayerAction::Playing(PlayingCommand::PlayCard { card: card_clone }));
                        }
                    };

                    view! {
                        <div 
                            class="card-item" 
                            class:playable=is_playing
                            style=format!("color: {color};")
                            on:click=on_card_click
                        >
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