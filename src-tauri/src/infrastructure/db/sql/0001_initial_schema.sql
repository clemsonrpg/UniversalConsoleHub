-- Universal Console Hub 3.0 — Initial Schema (Migration 0001)
-- 19 STRICT tables, UTC RFC3339 checks, natural UNIQUE keys, and immutable trophy origin triggers.

CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL CHECK (created_at LIKE '____-__-__T__:__:__%Z'),
    updated_at TEXT NOT NULL CHECK (updated_at LIKE '____-__-__T__:__:__%Z')
) STRICT;

CREATE TABLE profiles (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    handle TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    avatar_path TEXT,
    banner_path TEXT,
    bio TEXT NOT NULL DEFAULT '',
    privacy_level TEXT NOT NULL CHECK (privacy_level IN ('private', 'friends', 'public')),
    preferences_json TEXT NOT NULL DEFAULT '{}',
    is_active INTEGER NOT NULL DEFAULT 0 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL CHECK (created_at LIKE '____-__-__T__:__:__%Z'),
    updated_at TEXT NOT NULL CHECK (updated_at LIKE '____-__-__T__:__:__%Z')
) STRICT;

CREATE TABLE consoles (
    id TEXT PRIMARY KEY NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    manufacturer TEXT NOT NULL,
    generation INTEGER NOT NULL CHECK (generation >= 1),
    release_year INTEGER NOT NULL CHECK (release_year >= 1972),
    phase_status TEXT NOT NULL,
    capabilities_json TEXT NOT NULL,
    notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL CHECK (created_at LIKE '____-__-__T__:__:__%Z')
) STRICT;

CREATE TABLE emulators (
    id TEXT PRIMARY KEY NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    console_id TEXT NOT NULL REFERENCES consoles(id) ON DELETE RESTRICT,
    executable_path TEXT,
    config_path TEXT,
    detected_version TEXT,
    integration_phase TEXT NOT NULL,
    capabilities_json TEXT NOT NULL,
    updated_at TEXT NOT NULL CHECK (updated_at LIKE '____-__-__T__:__:__%Z')
) STRICT;

CREATE TABLE emulator_versions (
    id TEXT PRIMARY KEY NOT NULL,
    emulator_id TEXT NOT NULL REFERENCES emulators(id) ON DELETE CASCADE,
    version TEXT NOT NULL,
    executable_sha256 TEXT,
    detected_at TEXT NOT NULL CHECK (detected_at LIKE '____-__-__T__:__:__%Z'),
    UNIQUE(emulator_id, version)
) STRICT;

CREATE TABLE console_firmware (
    id TEXT PRIMARY KEY NOT NULL,
    console_id TEXT NOT NULL REFERENCES consoles(id) ON DELETE CASCADE,
    emulator_id TEXT NOT NULL REFERENCES emulators(id) ON DELETE CASCADE,
    file_role TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size_bytes INTEGER NOT NULL CHECK (file_size_bytes >= 0),
    sha256 TEXT NOT NULL,
    validation_status TEXT NOT NULL,
    notes TEXT NOT NULL DEFAULT '',
    validated_at TEXT NOT NULL CHECK (validated_at LIKE '____-__-__T__:__:__%Z'),
    UNIQUE(console_id, file_role, file_path)
) STRICT;

CREATE TABLE games (
    id TEXT PRIMARY KEY NOT NULL,
    console_id TEXT NOT NULL REFERENCES consoles(id) ON DELETE RESTRICT,
    canonical_key TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    developer TEXT,
    publisher TEXT,
    release_year INTEGER,
    genre TEXT,
    cover_path TEXT,
    metadata_source TEXT NOT NULL,
    install_state TEXT NOT NULL CHECK (install_state IN ('registered', 'installed', 'missing_files')),
    associated_emulator_id TEXT REFERENCES emulators(id) ON DELETE SET NULL,
    completion_percent INTEGER NOT NULL DEFAULT 0 CHECK (completion_percent BETWEEN 0 AND 100),
    created_at TEXT NOT NULL CHECK (created_at LIKE '____-__-__T__:__:__%Z'),
    updated_at TEXT NOT NULL CHECK (updated_at LIKE '____-__-__T__:__:__%Z'),
    UNIQUE(console_id, canonical_key)
) STRICT;

CREATE TABLE game_versions (
    id TEXT PRIMARY KEY NOT NULL,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    version_label TEXT NOT NULL,
    region TEXT NOT NULL,
    serial_code TEXT,
    file_path TEXT NOT NULL,
    install_state TEXT NOT NULL CHECK (install_state IN ('registered', 'installed', 'missing_files')),
    created_at TEXT NOT NULL CHECK (created_at LIKE '____-__-__T__:__:__%Z'),
    UNIQUE(game_id, region, version_label, file_path)
) STRICT;

CREATE TABLE game_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    game_version_id TEXT REFERENCES game_versions(id) ON DELETE SET NULL,
    emulator_id TEXT NOT NULL REFERENCES emulators(id) ON DELETE RESTRICT,
    started_at TEXT NOT NULL CHECK (started_at LIKE '____-__-__T__:__:__%Z'),
    last_heartbeat_at TEXT NOT NULL CHECK (last_heartbeat_at LIKE '____-__-__T__:__:__%Z'),
    ended_at TEXT CHECK (ended_at IS NULL OR ended_at LIKE '____-__-__T__:__:__%Z'),
    duration_seconds INTEGER NOT NULL DEFAULT 0 CHECK (duration_seconds >= 0),
    exit_status TEXT NOT NULL
) STRICT;

CREATE TABLE achievement_providers (
    id TEXT PRIMARY KEY NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    origin TEXT NOT NULL CHECK (origin IN ('official', 'emulator', 'local')),
    availability_status TEXT NOT NULL,
    supports_auth INTEGER NOT NULL DEFAULT 0 CHECK (supports_auth IN (0, 1)),
    supports_sync INTEGER NOT NULL DEFAULT 0 CHECK (supports_sync IN (0, 1)),
    notes TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL CHECK (updated_at LIKE '____-__-__T__:__:__%Z')
) STRICT;

CREATE TABLE trophies (
    id TEXT PRIMARY KEY NOT NULL,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    console_id TEXT NOT NULL REFERENCES consoles(id) ON DELETE RESTRICT,
    provider_id TEXT NOT NULL REFERENCES achievement_providers(id) ON DELETE RESTRICT,
    external_trophy_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    icon_path TEXT,
    category TEXT NOT NULL CHECK (category IN (
        'bronze', 'silver', 'gold', 'platinum',
        'xbox_achievement', 'steam_achievement',
        'emulator_achievement', 'hub_local'
    )),
    rarity_percent INTEGER CHECK (rarity_percent IS NULL OR (rarity_percent BETWEEN 0 AND 100)),
    origin TEXT NOT NULL CHECK (origin IN ('official', 'emulator', 'local')),
    created_at TEXT NOT NULL CHECK (created_at LIKE '____-__-__T__:__:__%Z'),
    UNIQUE(game_id, provider_id, external_trophy_id)
) STRICT;

CREATE TABLE trophy_unlocks (
    id TEXT PRIMARY KEY NOT NULL,
    profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    trophy_id TEXT NOT NULL REFERENCES trophies(id) ON DELETE CASCADE,
    origin TEXT NOT NULL CHECK (origin IN ('official', 'emulator', 'local')),
    unlocked_at TEXT NOT NULL CHECK (unlocked_at LIKE '____-__-__T__:__:__%Z'),
    synced_at TEXT NOT NULL CHECK (synced_at LIKE '____-__-__T__:__:__%Z'),
    UNIQUE(profile_id, trophy_id)
) STRICT;

CREATE TABLE platform_accounts (
    id TEXT PRIMARY KEY NOT NULL,
    profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    provider_id TEXT NOT NULL REFERENCES achievement_providers(id) ON DELETE CASCADE,
    external_account_id TEXT NOT NULL,
    display_handle TEXT NOT NULL,
    connection_status TEXT NOT NULL,
    connected_at TEXT NOT NULL CHECK (connected_at LIKE '____-__-__T__:__:__%Z'),
    UNIQUE(profile_id, provider_id, external_account_id)
) STRICT;

CREATE TABLE statistics (
    id TEXT PRIMARY KEY NOT NULL,
    profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    scope_type TEXT NOT NULL CHECK (scope_type IN ('global', 'console', 'game')),
    scope_id TEXT NOT NULL,
    total_playtime_seconds INTEGER NOT NULL DEFAULT 0 CHECK (total_playtime_seconds >= 0),
    sessions_count INTEGER NOT NULL DEFAULT 0 CHECK (sessions_count >= 0),
    games_started INTEGER NOT NULL DEFAULT 0 CHECK (games_started >= 0),
    games_completed INTEGER NOT NULL DEFAULT 0 CHECK (games_completed >= 0),
    unlocked_trophies INTEGER NOT NULL DEFAULT 0 CHECK (unlocked_trophies >= 0),
    updated_at TEXT NOT NULL CHECK (updated_at LIKE '____-__-__T__:__:__%Z'),
    UNIQUE(profile_id, scope_type, scope_id)
) STRICT;

CREATE TABLE favorites (
    id TEXT PRIMARY KEY NOT NULL,
    profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL CHECK (created_at LIKE '____-__-__T__:__:__%Z'),
    UNIQUE(profile_id, game_id)
) STRICT;

CREATE TABLE activity_history (
    id TEXT PRIMARY KEY NOT NULL,
    profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    summary TEXT NOT NULL,
    occurred_at TEXT NOT NULL CHECK (occurred_at LIKE '____-__-__T__:__:__%Z')
) STRICT;

CREATE TABLE synchronization_history (
    id TEXT PRIMARY KEY NOT NULL,
    provider_id TEXT NOT NULL REFERENCES achievement_providers(id) ON DELETE CASCADE,
    profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    items_synced INTEGER NOT NULL DEFAULT 0 CHECK (items_synced >= 0),
    error_message TEXT,
    started_at TEXT NOT NULL CHECK (started_at LIKE '____-__-__T__:__:__%Z'),
    finished_at TEXT CHECK (finished_at IS NULL OR finished_at LIKE '____-__-__T__:__:__%Z')
) STRICT;

CREATE TABLE controller_profiles (
    id TEXT PRIMARY KEY NOT NULL,
    profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    device_signature TEXT NOT NULL,
    profile_name TEXT NOT NULL,
    glyph_family TEXT NOT NULL CHECK (glyph_family IN ('xbox', 'playstation', 'nintendo', 'generic')),
    deadzone_left REAL NOT NULL DEFAULT 0.18 CHECK (deadzone_left >= 0.0 AND deadzone_left <= 0.8),
    deadzone_right REAL NOT NULL DEFAULT 0.18 CHECK (deadzone_right >= 0.0 AND deadzone_right <= 0.8),
    trigger_threshold REAL NOT NULL DEFAULT 0.15 CHECK (trigger_threshold >= 0.0 AND trigger_threshold <= 0.9),
    mapping_json TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL CHECK (updated_at LIKE '____-__-__T__:__:__%Z'),
    UNIQUE(profile_id, device_signature, profile_name)
) STRICT;

CREATE TABLE application_settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL CHECK (updated_at LIKE '____-__-__T__:__:__%Z')
) STRICT;

-- Performance indexes
CREATE INDEX idx_games_console ON games(console_id);
CREATE INDEX idx_game_versions_game ON game_versions(game_id);
CREATE INDEX idx_game_sessions_profile_started ON game_sessions(profile_id, started_at DESC);
CREATE INDEX idx_trophies_game ON trophies(game_id);
CREATE INDEX idx_trophy_unlocks_profile ON trophy_unlocks(profile_id, unlocked_at DESC);
CREATE INDEX idx_activity_history_profile_occurred ON activity_history(profile_id, occurred_at DESC);

-- Immutable trophy origin guards (Adjustment #3)
CREATE TRIGGER trg_trophies_immutable_origin
BEFORE UPDATE OF origin ON trophies
WHEN NEW.origin != OLD.origin
BEGIN
    SELECT RAISE(ABORT, 'trophy origin is immutable and cannot be modified');
END;

CREATE TRIGGER trg_trophy_unlocks_immutable_origin
BEFORE UPDATE OF origin ON trophy_unlocks
WHEN NEW.origin != OLD.origin
BEGIN
    SELECT RAISE(ABORT, 'trophy unlock origin is immutable and cannot be modified');
END;
