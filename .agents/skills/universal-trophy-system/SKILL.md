---
name: universal-trophy-system
description: >-
  Use when designing or implementing Universal Console Hub's unified achievement
  and trophy architecture, local trophy parsing, RetroAchievements integration,
  rarity calculation, unlock history, or synchronization pipelines.
---

# Universal Trophy System

## Purpose & Scope

Guides the design and implementation of Universal Console Hub's unified achievement/trophy subsystem: canonical data models, provider adapters, local emulator trophy reading, honest capability boundaries, unlock notifications, and SQLite persistence.

**Out of scope:** SQLite low-level pragma/migration mechanics (use `sqlite-engineering`), player profile aggregation and playtime (use `universal-player-profile`).

## When to Use

- Designing canonical trophy/achievement schemas (Bronze/Silver/Gold/Platinum or points-based equivalents).
- Implementing achievement providers (e.g., RPCS3 local `TROPUSR.DAT`/`TROPCONF.SFM` reader, RetroAchievements API adapter for PCSX2/Dolphin/etc.).
- Building unlock event watchers, deduplication, timestamp reconciliation, and toast notifications.
- Calculating completion percentages, rarity tiers, and unlock history timelines.

**When NOT to use:** Emulator launching or controller input handling.

## Provider Honesty & Support Matrix

Never fabricate trophy or achievement support for emulators that do not expose it:

| Source / Emulator | Real Integration Path | Honest Constraints |
|---|---|---|
| **RPCS3 (PS3)** | Local trophy directory (`dev_hdd0/home/<user>/trophy/<NPCOMMID>/` — `TROPCONF.SFM` + `TROPUSR.DAT` + PNG icons). | Offline local unlock tracking only; no official PSN cloud sync. |
| **PCSX2 (PS2) / Dolphin (GC/Wii)** | **RetroAchievements** (when user configures RA credentials in the emulator and provides an RA Web API key to the hub). | Requires user's own RetroAchievements account/API key; tracks RA-supported hashes only. |
| **xemu (Xbox) / Xenia Canary (360) / shadPS4 (PS4)** | Only implement providers that read verified local formats (e.g., Xenia/shadPS4 local trophy/achievement files *if and only if* the user's emulator version writes documented, parseable state). | Otherwise mark provider as `Unavailable` / `Planned`—never mock unlocks as real. |

## Working Procedure

1. **Normalize Into a Canonical Domain Model**:
   - `TrophySet`: `id`, `game_id`, `provider_id`, `external_set_id`, `title`, `total_count`, `supports_platinum`.
   - `TrophyDefinition`: `id`, `trophy_set_id`, `external_id`, `name`, `description`, `is_hidden`, `grade` (`Bronze | Silver | Gold | Platinum | Standard`), `points`, `icon_path`.
   - `TrophyUnlock`: `profile_id`, `trophy_id`, `unlocked_at` (UTC ISO-8601 or `null` if timestamp unavailable), `source_provider`, `is_hardcore` (for RetroAchievements), `synced_at`.
2. **Implement Read-Only Provider Adapters**:
   - Define a `TrophyProvider` trait with `provider_id()`, `capabilities()`, `sync_game(...)`, and `sync_all(...)`.
   - Treat emulator trophy files (`TROPUSR.DAT`, etc.) as **strictly read-only**—never write to or lock files an active emulator owns.
3. **Deduplicate & Preserve Earliest Verified Unlock**:
   - Key unlocks by `(profile_id, trophy_id)` with a unique constraint in SQLite.
   - On re-sync, never regress an already-unlocked trophy back to locked unless the user explicitly triggers a full provider reset.
   - Preserve the earliest valid `unlocked_at` timestamp when merging sources.
4. **Watch & Sync Safely**:
   - Trigger sync on `session:exited` (primary, guaranteed clean file state) and optionally via debounced file-watcher or rate-limited API polling during active play.
   - Respect external API rate limits (e.g., RetroAchievements API throttling, exponential backoff, local caching of static game/trophy metadata and icons).
5. **Rarity & Secret Trophy Handling**:
   - Store provider-reported rarity percentages when available (e.g., RetroAchievements unlock rate); mark local-only offline trophies as `rarity: None` ("Local Only") rather than inventing fake global percentages.
   - Respect `is_hidden` flags in the UI until unlocked (with an explicit user toggle to reveal spoilers).

## Technical Best Practices

- **Cache Media Locally**: Copy/cache trophy icons into the app's local cache directory with content hashes so the UI remains fast and offline-capable.
- **Idempotent Sync Jobs**: Every sync operation must be safely repeatable without creating duplicate unlock events or duplicate toast notifications.
- **Clear Provenance**: Always display *where* a trophy/achievement came from (e.g., `RPCS3 Local`, `RetroAchievements`) so users trust the data.

## Common Mistakes to Avoid

- **Writing to Active Emulator Files**: Opening `TROPUSR.DAT` with write locks while RPCS3 is running can corrupt save/trophy data. Always open read-only (or copy to temp before parsing).
- **Inventing Fake Rarity Percentages**: Displaying made-up "Ultra Rare 1.2%" stats for offline local trophies destroys user trust.
- **Unthrottled API Polling**: Hammering the RetroAchievements API on every UI render risks account/key throttling. Cache metadata in SQLite with TTLs.
- **Hardcoding PSN/Xbox Live Scraping**: Do not rely on fragile or TOS-violating scraping of private console networks.

## Quality Checklist

- [ ] Canonical schema supports both PlayStation-style grades and points-based achievements cleanly.
- [ ] Emulator trophy files are accessed strictly read-only.
- [ ] Sync is idempotent and preserves earliest valid unlock timestamps.
- [ ] Hidden/secret trophy descriptions are masked until unlocked (with spoiler reveal option).
- [ ] Offline-only trophies accurately report no global rarity instead of fabricated numbers.
- [ ] External API calls enforce rate limiting, timeouts, and local SQLite caching.

## References

- RetroAchievements Web API Documentation: https://api-docs.retroachievements.org/
- RPCS3 Trophy System Documentation & Wiki: https://wiki.rpcs3.net/
- Clean Architecture Ports & Adapters: see `clean-architecture` skill.
