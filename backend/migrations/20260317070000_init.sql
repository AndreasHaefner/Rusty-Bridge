-- Alles löschen, um sauber zu starten (nur für Entwicklung!)
DROP TABLE IF EXISTS game_events, games, players, lobbies, sessions, users CASCADE;

-- 1. USERS
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('anonymous', 'registered')),
    email TEXT UNIQUE,
    password_hash TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 2. SESSIONS
CREATE TABLE sessions (
    token VARCHAR(64) PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL
);

-- 3. LOBBIES
CREATE TABLE lobbies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    master_id UUID REFERENCES users(id) ON DELETE SET NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 4. PLAYERS
CREATE TABLE players (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lobby_id UUID NOT NULL REFERENCES lobbies(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    position TEXT NOT NULL CHECK (position IN ('North', 'East', 'South', 'West')),
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(lobby_id, user_id),
    UNIQUE(lobby_id, position)
);

-- 5. GAMES & EVENTS
CREATE TABLE games (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lobby_id UUID NOT NULL REFERENCES lobbies(id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK (status IN ('bidding', 'playing', 'finished')),
    current_turn_position TEXT CHECK (current_turn_position IN ('North', 'East', 'South', 'West')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE game_events (
    id BIGSERIAL PRIMARY KEY,
    game_id UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    player_id UUID REFERENCES players(id) ON DELETE SET NULL,
    action_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indizes
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_players_lobby_id ON players(lobby_id);