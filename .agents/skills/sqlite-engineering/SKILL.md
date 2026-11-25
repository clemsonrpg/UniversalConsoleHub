---
name: sqlite-engineering
description: >-
  Use when designing SQLite schemas, writing versioned migrations, configuring
  WAL concurrency, tuning pragmas, optimizing queries, or implementing local
  backup and integrity checks in Universal Console Hub's Rust/Tauri backend.
---

# SQLite Engineering

## Purpose & Scope

Provides production-grade SQLite 3.45+ engineering practices for local-first Rust/Tauri desktop applications (`rusqlite` / `sqlx`): connection pragmas, WAL concurrency, strict schema design, atomic forward migrations, query optimization, and online backup/recovery.

**Out of scope:** High-level domain modeling for trophies (use `universal-trophy-system`) or player profiles (use `universal-player-profile`).

## When to Use

- Initializing SQLite connections or pools in the Rust backend.
- Designing tables, foreign keys, composite indexes, or `STRICT` constraints.
- Writing or reviewing forward schema migrations (including SQLite table-rebuild migrations).
- Diagnosing `SQLITE_BUSY` / locking contention, slow queries (`EXPLAIN QUERY PLAN`), or database corruption recovery.
- Implementing automated local backups before risky migrations or user-triggered exports.

**When NOT to use:** Client-side React state or non-database configuration files.

## Required Connection Initialization

Apply these pragmas on **every** opened SQLite connection (not just once per process, because `foreign_keys` and `busy_timeout` are per-connection):

```sql
PRAGMA journal_mode = WAL;          -- Concurrent readers while a writer commits
PRAGMA synchronous = NORMAL;        -- Safe & fast with WAL mode
PRAGMA foreign_keys = ON;           -- Disabled by default in SQLite; MUST enable per connection
PRAGMA busy_timeout = 5000;         -- Wait up to 5s before returning SQLITE_BUSY
PRAGMA cache_size = -64000;         -- Negative = KiB (~64 MB page cache); NEVER positive 1000000000
PRAGMA temp_store = MEMORY;         -- Keep temp B-trees in memory
```

## Working Procedure

1. **Design Strict, Normalized Schemas**:
   - Use `STRICT` tables (SQLite 3.37+) to enforce column types (`INTEGER`, `TEXT`, `REAL`, `BLOB`) instead of dynamic affinity surprises.
   - Define explicit `PRIMARY KEY`, `NOT NULL`, `CHECK`, `UNIQUE`, and `FOREIGN KEY ... ON DELETE CASCADE/RESTRICT` constraints.
   - Store timestamps consistently as UTC ISO-8601 `TEXT` (`2026-11-25T21:00:00Z`) or UTC Unix epoch `INTEGER` seconds—pick one convention project-wide and never mix them.
2. **Manage Migrations Atomically & Deterministically**:
   - Track schema versions using an embedded migration runner (e.g., `rusqlite_migration` or `sqlx::migrate!`) backed by `PRAGMA user_version` or a `_migrations` table.
   - Never edit an already-released migration file; always add a new numbered forward migration.
   - For structural changes SQLite's `ALTER TABLE` cannot do directly (e.g., changing column constraints or foreign keys), follow SQLite's official 12-step shadow-table rebuild inside a transaction, and run `PRAGMA foreign_key_check;` before committing.
   - Create an automatic timestamped backup copy before applying pending migrations on startup.
3. **Prevent Locking & Concurrency Contention**:
   - Use `BEGIN IMMEDIATE` for write transactions so lock acquisition fails fast or waits cleanly under `busy_timeout` instead of deadlocking on deferred read-to-write upgrades.
   - Keep transactions short—never perform filesystem I/O, network requests, or emulator launches while holding an open write transaction.
   - Use a single writer / serialized write queue (or dedicated connection pool with small size) in Rust so concurrent async tasks do not thrash write locks.
4. **Optimize & Verify Queries**:
   - Run `EXPLAIN QUERY PLAN` on all library, activity-feed, and trophy-aggregation queries to verify index usage (`SEARCH ... USING INDEX` / `COVERING INDEX`, avoiding `SCAN` on large tables).
   - Run `PRAGMA optimize;` periodically (e.g., on clean app shutdown) to keep query planner statistics fresh.
5. **Implement Safe Online Backups & Integrity Verification**:
   - **Never** back up a live WAL-mode SQLite file with naive `std::fs::copy` while connections are open (risks copying an inconsistent state missing `-wal` frames).
   - Use SQLite's Online Backup API (`rusqlite::backup::Backup`) or `VACUUM INTO 'path/to/backup.db'` for atomic, consistent snapshots.
   - Run `PRAGMA quick_check;` on startup diagnostics and `PRAGMA integrity_check;` on user-requested database repair/verification.

## Common Mistakes to Avoid

- **Forgetting `PRAGMA foreign_keys = ON` Per Connection**: Causes silent orphaned rows when parent records are deleted.
- **Positive Huge `cache_size` Values**: `PRAGMA cache_size = 1000000000` asks for 1 billion *pages* (~4 TB), not 1 GB! Always use negative KiB values (e.g., `-64000` for 64 MB).
- **Copying `.db` Files While Open in WAL Mode**: Drops uncommitted WAL frames and can corrupt the backup; always use `VACUUM INTO` or the Online Backup API.
- **String-Interpolated SQL**: Always bind parameters (`params![...]`)—even for local desktop apps, game titles frequently contain apostrophes (`Assassin's Creed`, `Baldur's Gate`).
- **Holding Write Transactions Across Async `.await`s**: Blocks all other writers and risks `SQLITE_BUSY` timeouts.

## Quality Checklist

- [ ] Every connection executes the standard pragma block (`WAL`, `foreign_keys = ON`, `busy_timeout`, negative `cache_size`).
- [ ] All tables use `STRICT` and explicit foreign-key/unique constraints.
- [ ] Migrations are versioned, atomic, idempotent, and tested on both fresh and prior-version DB fixtures.
- [ ] All SQL statements use bound parameters (zero string interpolation).
- [ ] Write transactions use `BEGIN IMMEDIATE` and perform zero I/O or network calls inside the lock.
- [ ] Backups use `VACUUM INTO` or SQLite's Online Backup API (never raw file copy on open WAL DBs).

## References

- SQLite WAL Mode: https://www.sqlite.org/wal.html
- SQLite STRICT Tables: https://www.sqlite.org/stricttables.html
- SQLite Official `ALTER TABLE` 12-Step Procedure: https://www.sqlite.org/lang_altertable.html#otheralter
- SQLite Online Backup API & `VACUUM INTO`: https://www.sqlite.org/backup.html
- `rusqlite` Documentation: https://docs.rs/rusqlite/
