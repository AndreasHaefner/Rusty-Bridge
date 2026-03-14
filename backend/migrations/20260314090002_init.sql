-- Created by AI

-- 1. Spiele-Tabelle (Das "Wohnzimmer")
CREATE TABLE IF NOT EXISTS games (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    status TEXT NOT NULL CHECK (status IN ('bidding', 'playing', 'finished')),
    current_turn_position TEXT, -- 'north', 'east', 'south', 'west'
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 2. Spieler-Tabelle
CREATE TABLE IF NOT EXISTS players (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    joined_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 3. Spiel-Teilnahme (Verknüpfung)
-- Damit wissen wir, welcher Spieler an welchem Tisch auf welcher Position sitzt
CREATE TABLE IF NOT EXISTS game_seats (
    game_id UUID REFERENCES games(id),
    player_id UUID REFERENCES players(id),
    position TEXT NOT NULL CHECK (position IN ('north', 'east', 'south', 'west')),
    PRIMARY KEY (game_id, position) -- Eine Position kann nur von einem Spieler besetzt sein
);

-- 4. Event-Log (Die "Single Source of Truth")
-- Hier speichern wir JEDEN Zug. Wenn Redis stirbt, lesen wir das hier ein.
CREATE TABLE IF NOT EXISTS game_events (
    id BIGSERIAL PRIMARY KEY,
    game_id UUID REFERENCES games(id),
    player_id UUID REFERENCES players(id),
    action_type TEXT NOT NULL,
    payload JSONB NOT NULL, -- Hier speichern wir die 'PlayCard' oder 'Bid' Daten
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Indexe für schnelles Abfragen der Historie
CREATE INDEX idx_game_events_game_id ON game_events(game_id);