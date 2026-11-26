-- Universal Console Hub 3.0 — Real Console Profiles & Multi-User Readiness (Migration 0003)
-- Strictly additive & non-destructive over Migration 0002 (22 -> 24 STRICT tables).

-- Table #23: Persistent Console Profile Snapshot (Console identity, NEVER player identity)
CREATE TABLE console_profiles (
    id TEXT PRIMARY KEY NOT NULL,
    console_id TEXT NOT NULL UNIQUE REFERENCES consoles(id) ON DELETE CASCADE,
    emulator_id TEXT REFERENCES emulators(id) ON DELETE SET NULL,
    emulator_status TEXT NOT NULL DEFAULT 'unconfigured' CHECK (emulator_status IN (
        'unconfigured',
        'available',
        'missing_on_disk'
    )),
    detected_emulator_version TEXT,
    detected_firmware_version TEXT,
    emulator_root_dir TEXT,
    native_save_count INTEGER NOT NULL DEFAULT 0 CHECK (native_save_count >= 0),
    native_trophy_dir_present INTEGER NOT NULL DEFAULT 0 CHECK (native_trophy_dir_present IN (0, 1)),
    trophy_sync_readiness TEXT NOT NULL DEFAULT 'unavailable' CHECK (trophy_sync_readiness IN (
        'unavailable',
        'phase4_planned',
        'ready'
    )),
    native_metadata_json TEXT NOT NULL DEFAULT '{}',
    sync_status TEXT NOT NULL DEFAULT 'never_synced' CHECK (sync_status IN (
        'never_synced',
        'synced',
        'partial',
        'error'
    )),
    sync_diagnostics TEXT NOT NULL DEFAULT '',
    last_synced_at TEXT CHECK (last_synced_at IS NULL OR last_synced_at LIKE '____-__-__T__:__:__%Z'),
    updated_at TEXT NOT NULL CHECK (updated_at LIKE '____-__-__T__:__:__%Z')
) STRICT;

-- Table #24: Real Local Console Users (only populated when the platform exposes real local users)
CREATE TABLE console_local_users (
    id TEXT PRIMARY KEY NOT NULL,
    console_id TEXT NOT NULL REFERENCES consoles(id) ON DELETE CASCADE,
    native_user_key TEXT NOT NULL,
    display_name TEXT NOT NULL,
    source_path TEXT NOT NULL,
    availability_status TEXT NOT NULL DEFAULT 'available' CHECK (availability_status IN (
        'available',
        'missing_on_disk'
    )),
    linked_profile_id TEXT REFERENCES profiles(id) ON DELETE SET NULL,
    first_detected_at TEXT NOT NULL CHECK (first_detected_at LIKE '____-__-__T__:__:__%Z'),
    last_seen_at TEXT NOT NULL CHECK (last_seen_at LIKE '____-__-__T__:__:__%Z'),
    UNIQUE(console_id, native_user_key)
) STRICT;

-- Additive columns on existing tables (preserving 100% of existing rows, constraints, and triggers)
ALTER TABLE game_versions
    ADD COLUMN availability_status TEXT NOT NULL DEFAULT 'available'
    CHECK (availability_status IN ('available', 'missing_on_disk'));

ALTER TABLE game_versions
    ADD COLUMN last_verified_at TEXT
    CHECK (last_verified_at IS NULL OR last_verified_at LIKE '____-__-__T__:__:__%Z');

ALTER TABLE game_sessions
    ADD COLUMN console_local_user_id TEXT REFERENCES console_local_users(id) ON DELETE SET NULL;

ALTER TABLE trophy_unlocks
    ADD COLUMN console_local_user_id TEXT REFERENCES console_local_users(id) ON DELETE SET NULL;

-- Additive performance indexes
CREATE INDEX IF NOT EXISTS idx_console_local_users_console
    ON console_local_users(console_id);
CREATE INDEX IF NOT EXISTS idx_game_versions_availability
    ON game_versions(game_id, availability_status);
CREATE INDEX IF NOT EXISTS idx_game_sessions_local_user
    ON game_sessions(console_local_user_id) WHERE console_local_user_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_trophy_unlocks_local_user
    ON trophy_unlocks(console_local_user_id) WHERE console_local_user_id IS NOT NULL;
