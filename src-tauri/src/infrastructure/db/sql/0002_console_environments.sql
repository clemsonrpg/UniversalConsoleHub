-- Universal Console Hub 3.0 — Console Environments & Verification Schema (Migration 0002)
-- Strictly additive & non-destructive over Migration 0001 (19 -> 22 STRICT tables).
-- Note: `game_versions` already exists as Table #8 in Migration 0001 and is preserved intact.

-- Table #20: Independent discovery directories per console
CREATE TABLE console_scan_paths (
    id TEXT PRIMARY KEY NOT NULL,
    console_id TEXT NOT NULL REFERENCES consoles(id) ON DELETE CASCADE,
    directory_path TEXT NOT NULL,
    recursive INTEGER NOT NULL DEFAULT 1 CHECK (recursive IN (0, 1)),
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    last_scanned_at TEXT CHECK (last_scanned_at IS NULL OR last_scanned_at LIKE '____-__-__T__:__:__%Z'),
    created_at TEXT NOT NULL CHECK (created_at LIKE '____-__-__T__:__:__%Z'),
    UNIQUE(console_id, directory_path)
) STRICT;

-- Table #21: Independent per-console configuration settings
CREATE TABLE console_settings (
    id TEXT PRIMARY KEY NOT NULL,
    console_id TEXT NOT NULL REFERENCES consoles(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL CHECK (updated_at LIKE '____-__-__T__:__:__%Z'),
    UNIQUE(console_id, key)
) STRICT;

-- Table #22: Granular independent runtime verification records per console, emulator, feature, version, and optional game
-- Supports both console-level checks (game_id IS NULL, scope_key = 'console')
-- and per-title checks such as `per_game_compatibility` (game_id = <games.id>, scope_key = game_id).
CREATE TABLE console_runtime_verifications (
    id TEXT PRIMARY KEY NOT NULL,
    console_id TEXT NOT NULL REFERENCES consoles(id) ON DELETE CASCADE,
    emulator_id TEXT NOT NULL REFERENCES emulators(id) ON DELETE CASCADE,
    game_id TEXT REFERENCES games(id) ON DELETE CASCADE,
    scope_key TEXT NOT NULL DEFAULT 'console',
    feature_key TEXT NOT NULL CHECK (feature_key IN (
        'emulator_launch',
        'system_boot',
        'original_dashboard',
        'game_boot',
        'per_game_compatibility',
        'session_monitoring',
        'hub_return'
    )),
    emulator_version TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN (
        'unverified',
        'documented_upstream',
        'files_verified',
        'runtime_confirmed',
        'failed'
    )),
    evidence_notes TEXT NOT NULL DEFAULT '',
    verified_at TEXT NOT NULL CHECK (verified_at LIKE '____-__-__T__:__:__%Z'),
    CHECK ((game_id IS NULL AND scope_key = 'console') OR (game_id IS NOT NULL AND scope_key = game_id)),
    UNIQUE(console_id, emulator_id, feature_key, emulator_version, scope_key)
) STRICT;

-- Additive indexes on Migration 0002 tables
CREATE INDEX IF NOT EXISTS idx_console_scan_paths_console
    ON console_scan_paths(console_id);
CREATE INDEX IF NOT EXISTS idx_console_settings_console
    ON console_settings(console_id);
CREATE INDEX IF NOT EXISTS idx_console_runtime_verifications_console_emu
    ON console_runtime_verifications(console_id, emulator_id);
CREATE INDEX IF NOT EXISTS idx_console_runtime_verifications_game
    ON console_runtime_verifications(game_id) WHERE game_id IS NOT NULL;

-- Additive non-destructive indexes on existing Migration 0001 tables (`game_versions` #8 and `game_sessions` #9)
CREATE INDEX IF NOT EXISTS idx_game_versions_file_path
    ON game_versions(file_path);
CREATE INDEX IF NOT EXISTS idx_game_versions_serial
    ON game_versions(serial_code) WHERE serial_code IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_game_sessions_open
    ON game_sessions(ended_at) WHERE ended_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_game_sessions_game_profile
    ON game_sessions(game_id, profile_id, started_at DESC);
