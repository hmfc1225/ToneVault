CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users
CREATE TABLE IF NOT EXISTS users (
    id          UUID PRIMARY KEY,
    username    VARCHAR(255) NOT NULL UNIQUE,
    display_name VARCHAR(255),
    email       VARCHAR(255),
    password_hash VARCHAR(255) NOT NULL,
    role        VARCHAR(50) NOT NULL DEFAULT 'user',
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- API Keys
CREATE TABLE IF NOT EXISTS api_keys (
    id          UUID PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name        VARCHAR(255) NOT NULL,
    key_hash    VARCHAR(255) NOT NULL UNIQUE,
    key_prefix  VARCHAR(50) NOT NULL,
    last_used_at TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Refresh Tokens
CREATE TABLE IF NOT EXISTS refresh_tokens (
    id          UUID PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash  VARCHAR(255) NOT NULL UNIQUE,
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Libraries
CREATE TABLE IF NOT EXISTS libraries (
    id          UUID PRIMARY KEY,
    name        VARCHAR(255) NOT NULL,
    root_path   TEXT NOT NULL,
    description TEXT,
    scan_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    watch_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    scan_status VARCHAR(50) NOT NULL DEFAULT 'idle',
    last_scan_at TIMESTAMPTZ,
    book_count  INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Authors
CREATE TABLE IF NOT EXISTS authors (
    id          UUID PRIMARY KEY,
    name        VARCHAR(255) NOT NULL,
    sort_name   VARCHAR(255) NOT NULL,
    description TEXT,
    cover_path  TEXT,
    book_count  INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Series
CREATE TABLE IF NOT EXISTS series (
    id          UUID PRIMARY KEY,
    name        VARCHAR(255) NOT NULL,
    description TEXT,
    book_count  INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Books
CREATE TABLE IF NOT EXISTS books (
    id              UUID PRIMARY KEY,
    library_id      UUID NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    title           VARCHAR(255) NOT NULL,
    subtitle        TEXT,
    description     TEXT,
    cover_path      TEXT,
    duration_secs   DOUBLE PRECISION NOT NULL DEFAULT 0,
    track_count     INTEGER NOT NULL DEFAULT 0,
    file_size       BIGINT NOT NULL DEFAULT 0,
    language        VARCHAR(50),
    publisher       VARCHAR(255),
    publish_year    INTEGER,
    isbn            VARCHAR(50),
    asin            VARCHAR(50),
    source_path     TEXT NOT NULL DEFAULT '',
    metadata_source VARCHAR(50) DEFAULT 'unknown',
    last_modified   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_books_library ON books(library_id);
CREATE INDEX IF NOT EXISTS idx_books_title ON books(title);

-- Book-Author many-to-many
CREATE TABLE IF NOT EXISTS book_authors (
    book_id     UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    author_id   UUID NOT NULL REFERENCES authors(id) ON DELETE CASCADE,
    role        VARCHAR(50) NOT NULL DEFAULT 'author',
    position    INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (book_id, author_id, role)
);

-- Book-Series many-to-many
CREATE TABLE IF NOT EXISTS book_series (
    book_id     UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    series_id   UUID NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    position    DOUBLE PRECISION,
    PRIMARY KEY (book_id, series_id)
);

-- Tracks
CREATE TABLE IF NOT EXISTS tracks (
    id              UUID PRIMARY KEY,
    book_id         UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    title           VARCHAR(255) NOT NULL,
    track_number    INTEGER NOT NULL,
    disc_number     INTEGER DEFAULT 1,
    file_path       TEXT NOT NULL,
    file_size       BIGINT NOT NULL,
    mime_type       VARCHAR(100) NOT NULL,
    duration_secs   DOUBLE PRECISION NOT NULL,
    bitrate         INTEGER,
    sample_rate     INTEGER,
    channels        INTEGER,
    codec           VARCHAR(50),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_tracks_book ON tracks(book_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_tracks_book_order ON tracks(book_id, disc_number, track_number);

-- Playback Positions
CREATE TABLE IF NOT EXISTS playback_positions (
    id          UUID PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    book_id     UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    track_id    UUID NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    position_secs DOUBLE PRECISION NOT NULL DEFAULT 0,
    duration_secs DOUBLE PRECISION NOT NULL DEFAULT 0,
    percentage  DOUBLE PRECISION NOT NULL DEFAULT 0,
    is_finished BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, book_id)
);

-- Bookmarks
CREATE TABLE IF NOT EXISTS bookmarks (
    id          UUID PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    book_id     UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    track_id    UUID NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    title       VARCHAR(255) NOT NULL,
    note        TEXT,
    position_secs DOUBLE PRECISION NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_bookmarks_user_book ON bookmarks(user_id, book_id);

-- Collections
CREATE TABLE IF NOT EXISTS collections (
    id          UUID PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name        VARCHAR(255) NOT NULL,
    description TEXT,
    cover_path  TEXT,
    book_count  INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Collection-Book many-to-many
CREATE TABLE IF NOT EXISTS collection_books (
    collection_id UUID NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    book_id       UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    position      INTEGER NOT NULL DEFAULT 0,
    added_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (collection_id, book_id)
);

-- Full-text search using pg_trgm
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX IF NOT EXISTS idx_books_title_trgm ON books USING gin (title gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_books_subtitle_trgm ON books USING gin (subtitle gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_books_description_trgm ON books USING gin (description gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_authors_name_trgm ON authors USING gin (name gin_trgm_ops);
