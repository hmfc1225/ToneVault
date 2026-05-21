-- Initial schema for ToneVault

-- Users
CREATE TABLE IF NOT EXISTS users (
    id          TEXT PRIMARY KEY,
    username    TEXT NOT NULL UNIQUE,
    display_name TEXT,
    email       TEXT,
    password_hash TEXT NOT NULL,
    role        TEXT NOT NULL DEFAULT 'user',
    is_active   INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- API Keys
CREATE TABLE IF NOT EXISTS api_keys (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    key_hash    TEXT NOT NULL UNIQUE,
    key_prefix  TEXT NOT NULL,
    last_used_at TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Refresh Tokens
CREATE TABLE IF NOT EXISTS refresh_tokens (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash  TEXT NOT NULL UNIQUE,
    expires_at  TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Libraries
CREATE TABLE IF NOT EXISTS libraries (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    root_path   TEXT NOT NULL,
    description TEXT,
    scan_enabled INTEGER NOT NULL DEFAULT 1,
    watch_enabled INTEGER NOT NULL DEFAULT 1,
    scan_status TEXT NOT NULL DEFAULT 'idle',
    last_scan_at TEXT,
    book_count  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Authors
CREATE TABLE IF NOT EXISTS authors (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    sort_name   TEXT NOT NULL,
    description TEXT,
    cover_path  TEXT,
    book_count  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Series
CREATE TABLE IF NOT EXISTS series (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT,
    book_count  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Books
CREATE TABLE IF NOT EXISTS books (
    id              TEXT PRIMARY KEY,
    library_id      TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    title           TEXT NOT NULL,
    subtitle        TEXT,
    description     TEXT,
    cover_path      TEXT,
    duration_secs   REAL NOT NULL DEFAULT 0,
    track_count     INTEGER NOT NULL DEFAULT 0,
    file_size       INTEGER NOT NULL DEFAULT 0,
    language        TEXT,
    publisher       TEXT,
    publish_year    INTEGER,
    isbn            TEXT,
    asin            TEXT,
    source_path     TEXT NOT NULL,
    metadata_source TEXT DEFAULT 'unknown',
    last_modified   TEXT NOT NULL DEFAULT (datetime('now')),
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_books_library ON books(library_id);
CREATE INDEX IF NOT EXISTS idx_books_title ON books(title);

-- Book-Author many-to-many
CREATE TABLE IF NOT EXISTS book_authors (
    book_id     TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    author_id   TEXT NOT NULL REFERENCES authors(id) ON DELETE CASCADE,
    role        TEXT NOT NULL DEFAULT 'author',
    position    INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (book_id, author_id, role)
);

-- Book-Series many-to-many
CREATE TABLE IF NOT EXISTS book_series (
    book_id     TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    series_id   TEXT NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    position    REAL,
    PRIMARY KEY (book_id, series_id)
);

-- Tracks
CREATE TABLE IF NOT EXISTS tracks (
    id              TEXT PRIMARY KEY,
    book_id         TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    title           TEXT NOT NULL,
    track_number    INTEGER NOT NULL,
    disc_number     INTEGER DEFAULT 1,
    file_path       TEXT NOT NULL,
    file_size       INTEGER NOT NULL,
    mime_type       TEXT NOT NULL,
    duration_secs   REAL NOT NULL,
    bitrate         INTEGER,
    sample_rate     INTEGER,
    channels        INTEGER,
    codec           TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_tracks_book ON tracks(book_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_tracks_book_order ON tracks(book_id, disc_number, track_number);

-- Playback Positions
CREATE TABLE IF NOT EXISTS playback_positions (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    book_id     TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    track_id    TEXT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    position_secs REAL NOT NULL DEFAULT 0,
    duration_secs REAL NOT NULL DEFAULT 0,
    percentage  REAL NOT NULL DEFAULT 0,
    is_finished INTEGER NOT NULL DEFAULT 0,
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id, book_id)
);

-- Bookmarks
CREATE TABLE IF NOT EXISTS bookmarks (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    book_id     TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    track_id    TEXT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    title       TEXT NOT NULL,
    note        TEXT,
    position_secs REAL NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_bookmarks_user_book ON bookmarks(user_id, book_id);

-- Collections
CREATE TABLE IF NOT EXISTS collections (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    description TEXT,
    cover_path  TEXT,
    book_count  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Collection-Book many-to-many
CREATE TABLE IF NOT EXISTS collection_books (
    collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    book_id       TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    position      INTEGER NOT NULL DEFAULT 0,
    added_at      TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (collection_id, book_id)
);

-- Full-text search
CREATE VIRTUAL TABLE IF NOT EXISTS books_fts USING fts5(
    title, subtitle, description, publisher,
    content=books,
    content_rowid=rowid
);

-- WAL mode for concurrent reads
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;