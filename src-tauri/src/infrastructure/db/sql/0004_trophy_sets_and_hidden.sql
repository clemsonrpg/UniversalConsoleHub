-- Universal Console Hub 3.0 — Universal Trophy Center Schema (Migration 0004)
-- Strictly additive & non-destructive over Migration 0003 (24 -> 25 STRICT tables).

-- Table #25: Trophy Sets (one canonical set per game/provider, e.g. PS3 NPWR00637_00)
CREATE TABLE trophy_sets (
    id TEXT PRIMARY KEY NOT NULL,
    console_id TEXT NOT NULL REFERENCES consoles(id) ON DELETE CASCADE,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    provider_id TEXT NOT NULL REFERENCES achievement_providers(id) ON DELETE RESTRICT,
    external_set_id TEXT NOT NULL,
    title TEXT NOT NULL,
    detail TEXT NOT NULL DEFAULT '',
    icon_path TEXT,
    total_count INTEGER NOT NULL DEFAULT 0 CHECK (total_count >= 0),
    supports_platinum INTEGER NOT NULL DEFAULT 0 CHECK (supports_platinum IN (0, 1)),
    updated_at TEXT NOT NULL CHECK (updated_at LIKE '____-__-__T__:__:__%Z'),
    UNIQUE(provider_id, external_set_id)
) STRICT;

-- Additive columns on `trophies` (#11) preserving existing rows and trg_trophies_immutable_origin
ALTER TABLE trophies
    ADD COLUMN trophy_set_id TEXT REFERENCES trophy_sets(id) ON DELETE CASCADE;

ALTER TABLE trophies
    ADD COLUMN is_hidden INTEGER NOT NULL DEFAULT 0 CHECK (is_hidden IN (0, 1));

-- Additive performance indexes
CREATE INDEX IF NOT EXISTS idx_trophy_sets_game
    ON trophy_sets(game_id);
CREATE INDEX IF NOT EXISTS idx_trophy_sets_console
    ON trophy_sets(console_id);
CREATE INDEX IF NOT EXISTS idx_trophies_set
    ON trophies(trophy_set_id) WHERE trophy_set_id IS NOT NULL;

-- Activate EmulatorAchievementProvider honestly for Phase 4 RPCS3 read-only trophy sync
-- (origin='emulator' is never mutated and remains protected by constraints)
UPDATE achievement_providers
SET availability_status = 'ready_offline',
    supports_sync = 1,
    notes = 'Leitura somente leitura de troféus locais RPCS3 (TROPCONF.SFM e TROPUSR.DAT em dev_hdd0/home/<user>/trophy/). Sem sincronização oficial PSN.'
WHERE slug = 'emulator';
