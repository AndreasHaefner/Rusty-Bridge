mod pages;
mod components;

use leptos::prelude::*;
use leptos_router::{components::*, path};
use pages::*;


#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <main>
                <Routes fallback=|| view! { <div>"Seite nicht gefunden (404)"</div> }>
                    <Route path=path!("/") view=login::LoginScreen />
                    <Route path=path!("/hub") view=hub::Hub />
                    <Route path=path!("/room/:id") view=pages::room::GameRoom />
                </Routes>
            </main>
        </Router>
    }
} 
fn main() {
    mount_to_body(|| view! { <App /> })
}