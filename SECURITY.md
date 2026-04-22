# Security & Cookie Islands

Aura implements a strict "Cookie Island" architecture. Each registrable domain (e.g., `github.com`) receives its own isolated SQLite database.

## Silo Schema

Silos are stored at `~/.aura/silos/{sha256(registrable_domain)}.silo.db`.

```sql
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS cookies (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    host         TEXT NOT NULL,
    name         TEXT NOT NULL,
    value        BLOB NOT NULL,       -- AES-256-GCM encrypted
    path         TEXT NOT NULL DEFAULT '/',
    secure       INTEGER NOT NULL DEFAULT 1,
    http_only    INTEGER NOT NULL DEFAULT 1,
    same_site    TEXT CHECK(same_site IN ('Strict','Lax','None')) DEFAULT 'Lax',
    expiry_utc   INTEGER,
    created_utc  INTEGER NOT NULL DEFAULT (unixepoch()),
    last_access  INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(host, name, path)
);

CREATE TABLE IF NOT EXISTS local_storage (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    origin       TEXT NOT NULL,
    key          TEXT NOT NULL,
    value        BLOB NOT NULL,       -- AES-256-GCM encrypted
    updated_utc  INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(origin, key)
);

CREATE TABLE IF NOT EXISTS silo_meta (
    key          TEXT PRIMARY KEY,
    value        TEXT
);
```

Cookie values and local storage data are encrypted using `AES-256-GCM`. The key is derived per-silo from a master key stored in the OS keychain.

## Network Interceptor

All requests pass through a network interceptor (`aura-net`) powered by the `adblock` crate.
It checks EasyList/EasyPrivacy rules *before* hitting the Cookie Silo.
Requests to HTTP are forcefully upgraded to HTTPS where possible.
