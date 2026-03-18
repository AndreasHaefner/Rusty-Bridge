use crate::dto::LobbyManager;
use crate::dto::UserManager;

pub struct AppState {
    pub lobby_manager: LobbyManager,
    pub user_manager: UserManager,
}