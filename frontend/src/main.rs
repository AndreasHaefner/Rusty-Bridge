use leptos::prelude::*;
use leptos_use::{use_websocket};
use codee::string::JsonSerdeCodec; 
use shared::{Card, PlayerAction, PlayingCommand, Suit, PublicGameState};


#[component]
fn App() -> impl IntoView {
    let ws = use_websocket::<String, PublicGameState, JsonSerdeCodec>("/ws");
    let (game_state, set_game_state) = signal(None::<PublicGameState>);

    Effect::new(move |_| {
        match ws.message.get() {
            Some(msg) => set_game_state.set(Some(msg)),
            None => println!("no GameState received"),
        }
    });

    let play_card = move |card: Card| {
        let action = PlayerAction::Playing(PlayingCommand::PlayCard { card });
        match serde_json::to_string(&action) {
            Ok(json) => {
                (ws.send)(&json);
            },
            Err(e) => println!("error: {}", e), //maybe ToDo -- Remove?!
        }
    };
   view! {
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
                fallback=move || view! { <div class="ready"></div> }
            >
                <div class="loading">"Warten auf Mitspieler (Lobby)..."</div>
            </Show>
        </div>
    }
} // Hier endet die App-Funktion
    
fn main() {
    mount_to_body(|| view! { <App /> })
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