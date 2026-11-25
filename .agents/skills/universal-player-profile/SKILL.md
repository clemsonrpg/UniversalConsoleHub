---
name: universal-player-profile
description: >-
  Use when designing or implementing Universal Console Hub's unified player
  identity, multi-console game library, playtime tracking, session history,
  favorites, collections, or cross-platform progression statistics.
---

# Universal Player Profile

## Purpose & Scope

Governs the unified player identity and library domain in Universal Console Hub: local player profiles, cross-console game library management, accurate playtime and session tracking, favorites/tags/collections, and progression statistics.

**Out of scope:** SQLite pragmas/migrations (use `sqlite-engineering`), trophy provider parsing (use `universal-trophy-system`), emulator process spawning (use `emulator-integration`).

## When to Use

- Designing player profile entities (avatar, handle, active profile switching, local backup/export).
- Building the unified game library scanner, title deduplication, metadata/box-art caching, and per-game launch preferences.
- Implementing accurate playtime tracking from emulator session lifecycle events.
- Computing cross-console statistics (total playtime, most played systems, recent activity feed, completion rates, streaks).

**When NOT to use:** Visual layout/theming (use `frontend-design` / `console-ui-fidelity`) or gamepad driver work.

## Working Procedure

1. **Model Local-First Multi-Profile Identity**:
   - Support one or more local `PlayerProfile` records (`id`, `display_name`, `avatar_path`, `created_at`, `preferences_json`).
   - Scope playtime, favorites, activity history, and trophy unlocks to `profile_id` from day one—avoid hardcoding a single global user so multi-profile switching never requires a schema rewrite.
2. **Unify Game Identity Across Consoles**:
   - Model `Game` with `id` (UUID), `system_id` (PS3, PS2, Wii, GameCube, Xbox, Xbox 360, PS4, etc.), `title`, `canonical_serial` (e.g., `BLUS30443`, `SLUS-20312`, `GALE01`), `rom_or_install_path`, `preferred_adapter_id`, `boot_mode`, and cached artwork paths.
   - Deduplicate rescans by normalized path and `(system_id, canonical_serial)` so re-scanning a folder never duplicates games or wipes playtime.
3. **Track Playtime Deterministically**:
   - Record discrete `PlaySession` rows (`id`, `profile_id`, `game_id`, `adapter_id`, `started_at`, `ended_at`, `duration_seconds`, `exit_status`).
   - Persist a session start record immediately on `session:started` and update `ended_at` + `duration_seconds` on `session:exited` (plus periodic heartbeat checkpoints every 60s so a power loss or crash doesn't lose a 3-hour session).
   - Derive total game playtime and system playtime from `PlaySession` aggregates (with optional cached rollups kept consistent inside transactions).
4. **Build Fast Activity & Statistics Queries**:
   - Activity feed: unified timeline merging `PlaySession` completions and `TrophyUnlock` events sorted by UTC timestamp.
   - Statistics: playtime by console, top played games, recently played, completion percentage per console, and favorites/custom collections.
5. **Support Portable Profile Export/Backup**:
   - Provide a clean JSON/SQLite backup export of profile stats, library metadata, favorites, and session history without exporting large ROMs or proprietary files.

## Technical Best Practices

- **UTC Everywhere in Storage**: Store all timestamps in UTC (ISO-8601 or Unix epoch seconds) and format to the user's local timezone strictly at the presentation layer.
- **Heartbeat Crash Recovery**: On app startup, reconcile any orphaned `PlaySession` with `ended_at IS NULL` using its `last_heartbeat_at` timestamp so crashed sessions still preserve accurate playtime.
- **Non-Blocking Library Scans**: Run filesystem scanning and serial/header extraction on background Rust worker threads, streaming progress to the UI via Tauri channels/events.
- **Normalized Path Handling on Windows**: Handle case-insensitive Windows paths, drive letters, and Unicode characters consistently when checking for duplicate library entries.

## Common Mistakes to Avoid

- **Storing Only a Mutable `total_playtime` Counter**: Without individual `PlaySession` rows, you cannot show "Playtime this week", activity history, or recover from double-counting bugs.
- **Losing Playtime on App Crash**: Waiting until process exit to write anything to SQLite loses the entire session if the app or OS crashes.
- **Tightly Coupling Library Rows to One Emulator**: A PS2 or GameCube game may be launched with different emulator builds or configurations; keep `game_id` independent of a specific executable path.
- **Blocking the UI During Artwork Fetching**: Fetch/resize cover art asynchronously with local disk caching and graceful fallback placeholders.

## Quality Checklist

- [ ] All user-owned data (`PlaySession`, `Favorite`, `Collection`, `TrophyUnlock`) is scoped by `profile_id`.
- [ ] Library rescans are idempotent and preserve user metadata, favorites, and playtime.
- [ ] Playtime tracking uses `PlaySession` records with heartbeat crash recovery.
- [ ] All stored timestamps are UTC; local timezone formatting happens only in UI.
- [ ] Statistics queries are backed by appropriate SQLite indexes (verified with `EXPLAIN QUERY PLAN`).

## References

- SQLite schema, indexing, and WAL concurrency: see `sqlite-engineering` skill.
- Trophy & achievement aggregation: see `universal-trophy-system` skill.
- Clean Architecture domain modeling: see `clean-architecture` skill.
