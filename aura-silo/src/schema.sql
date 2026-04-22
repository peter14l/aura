-- Applied on every new silo file creation via rusqlite migrations

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS cookies (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    host         TEXT NOT NULL,       -- e.g. "api.github.com"
    name         TEXT NOT NULL,
    value        BLOB NOT NULL,       -- AES-256-GCM encrypted
    path         TEXT NOT NULL DEFAULT '/',
    secure       INTEGER NOT NULL DEFAULT 1,  -- bool: 1 = Secure-only
    http_only    INTEGER NOT NULL DEFAULT 1,  -- bool: 1 = HttpOnly
    same_site    TEXT CHECK(same_site IN ('Strict','Lax','None')) DEFAULT 'Lax',
    expiry_utc   INTEGER,             -- Unix epoch; NULL = session cookie
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

-- Seed meta
INSERT OR IGNORE INTO silo_meta VALUES ('pinned', '0');    -- 0 = cleared on session end
INSERT OR IGNORE INTO silo_meta VALUES ('created', unixepoch());
