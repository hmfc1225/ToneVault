use chrono::{DateTime, NaiveDateTime, Utc};
use uuid::Uuid;

use crate::{Database, DbError, Result};
use tonevault_core::models::library::{CreateLibrary, Library, SourceType, UpdateLibrary};
use tonevault_core::models::*;

// --- SqliteRepository ---

pub struct SqliteRepository {
    db: Database,
}

impl SqliteRepository {
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        let pool = sqlx::SqlitePool::connect(url).await?;
        Ok(Self { db: Database::new(pool) })
    }

    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./migrations")
            .run(self.db.pool())
            .await?;
        Ok(())
    }

    pub fn database(&self) -> &Database {
        &self.db
    }
}

impl std::ops::Deref for SqliteRepository {
    type Target = Database;
    fn deref(&self) -> &Database {
        &self.db
    }
}

// --- Library functions ---

pub async fn list_libraries(pool: &sqlx::SqlitePool) -> Result<Vec<Library>> {
    let rows = sqlx::query_as::<_, LibraryRow>(
        "SELECT id, name, root_path, source_type, base_url, username, password, scan_interval, last_scan, created_at, updated_at FROM libraries ORDER BY name",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
}

pub async fn get_library(pool: &sqlx::SqlitePool, id: i64) -> Result<Option<Library>> {
    let row = sqlx::query_as::<_, LibraryRow>(
        "SELECT id, name, root_path, source_type, base_url, username, password, scan_interval, last_scan, created_at, updated_at FROM libraries WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.into()))
}

pub async fn create_library(pool: &sqlx::SqlitePool, input: &CreateLibrary) -> Result<Library> {
    let source_type_str = input.source_type.as_str();
    let result = sqlx::query_as::<_, LibraryRow>(
        "INSERT INTO libraries (name, root_path, source_type, base_url, username, password, scan_interval) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id, name, root_path, source_type, base_url, username, password, scan_interval, last_scan, created_at, updated_at",
    )
    .bind(&input.name)
    .bind(&input.root_path)
    .bind(source_type_str)
    .bind(&input.base_url)
    .bind(&input.username)
    .bind(&input.password)
    .bind(input.scan_interval)
    .fetch_one(pool)
    .await?;
    Ok(result.into())
}

pub async fn update_library(pool: &sqlx::SqlitePool, id: i64, input: &UpdateLibrary) -> Result<Library> {
    let current = get_library(pool, id).await?
        .ok_or(DbError::NotFound)?;

    let name = input.name.as_deref().unwrap_or(&current.name);
    let root_path = input.root_path.as_deref().unwrap_or(&current.root_path);
    let source_type = input.source_type.unwrap_or(current.source_type);
    let base_url = input.base_url.as_deref().or(current.base_url.as_deref());
    let username = input.username.as_deref().or(current.username.as_deref());
    let password = input.password.as_deref().or(current.password.as_deref());
    let scan_interval = input.scan_interval.or(current.scan_interval);

    let source_type_str = source_type.as_str();
    let result = sqlx::query_as::<_, LibraryRow>(
        "UPDATE libraries SET name = ?, root_path = ?, source_type = ?, base_url = ?, username = ?, password = ?, scan_interval = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? RETURNING id, name, root_path, source_type, base_url, username, password, scan_interval, last_scan, created_at, updated_at",
    )
    .bind(name)
    .bind(root_path)
    .bind(source_type_str)
    .bind(base_url)
    .bind(username)
    .bind(password)
    .bind(scan_interval)
    .bind(id)
    .fetch_one(pool)
    .await?;
    Ok(result.into())
}

pub async fn delete_library(pool: &sqlx::SqlitePool, id: i64) -> Result<()> {
    let result = sqlx::query("DELETE FROM libraries WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

pub async fn update_last_scan(pool: &sqlx::SqlitePool, id: i64) -> Result<()> {
    sqlx::query("UPDATE libraries SET last_scan = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// --- Book functions ---

pub async fn list_books(pool: &sqlx::SqlitePool, filter: &BookFilter) -> Result<PaginatedResult<Book>> {
    let mut where_clauses = Vec::new();
    let mut count_where = Vec::new();

    if filter.library_id.is_some() {
        where_clauses.push("library_id = ?".to_string());
        count_where.push("library_id = ?".to_string());
    }
    if let Some(ref q) = filter.query {
        where_clauses.push("(title LIKE ? OR author LIKE ?)".to_string());
        count_where.push("(title LIKE ? OR author LIKE ?)".to_string());
    }

    let where_str = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };
    let count_where_str = if count_where.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", count_where.join(" AND "))
    };

    let order = match filter.sort {
        BookSort::Title => "title",
        BookSort::Added => "created_at",
        BookSort::Duration => "duration_secs",
        BookSort::Year => "publish_year",
        BookSort::Author => "title",
    };
    let dir = match filter.order {
        SortOrder::Asc => "ASC",
        SortOrder::Desc => "DESC",
    };

    let count_sql = format!("SELECT COUNT(*) as count FROM books {}", count_where_str);
    let query_sql = format!(
        "SELECT id, library_id, title, subtitle, description, cover_path, duration_secs, track_count, file_size, language, publisher, publish_year, isbn, asin, source_path, metadata_source, created_at, updated_at FROM books {} ORDER BY {} {} LIMIT ? OFFSET ?",
        where_str, order, dir
    );

    // Count
    let pattern = filter.query.as_ref().map(|q| format!("%{}%", q));
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    if let Some(lid) = filter.library_id {
        count_query = count_query.bind(lid.to_string());
    }
    if let Some(ref p) = pattern {
        count_query = count_query.bind(p).bind(p);
    }
    let total = count_query.fetch_one(pool).await?;

    // Query
    let mut q = sqlx::query_as::<_, BookRow>(&query_sql);
    if let Some(lid) = filter.library_id {
        q = q.bind(lid.to_string());
    }
    if let Some(ref p) = pattern {
        q = q.bind(p).bind(p);
    }
    let offset = (filter.page - 1) * filter.per_page;
    q = q.bind(filter.per_page).bind(offset);

    let rows = q.fetch_all(pool).await?;
    let items: Vec<Book> = rows.into_iter().map(|r| r.into()).collect();
    let total_pages = (total as f64 / filter.per_page as f64).ceil() as i64;

    Ok(PaginatedResult {
        items,
        total,
        page: filter.page,
        per_page: filter.per_page,
        total_pages,
    })
}

pub async fn get_book(pool: &sqlx::SqlitePool, id: Uuid) -> Result<Option<Book>> {
    let row = sqlx::query_as::<_, BookRow>(
        "SELECT id, library_id, title, subtitle, description, cover_path, duration_secs, track_count, file_size, language, publisher, publish_year, isbn, asin, source_path, metadata_source, created_at, updated_at FROM books WHERE id = ?",
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.into()))
}

pub async fn update_book(pool: &sqlx::SqlitePool, id: Uuid, input: &UpdateBook) -> Result<Book> {
    let current = get_book(pool, id).await?.ok_or(DbError::NotFound)?;

    let title = input.title.as_deref().unwrap_or(&current.title);
    let subtitle = input.subtitle.as_deref().or(current.subtitle.as_deref());
    let description = input.description.as_deref().or(current.description.as_deref());
    let cover_path = input.cover_path.as_deref().or(current.cover_path.as_deref());
    let language = input.language.as_deref().or(current.language.as_deref());
    let publisher = input.publisher.as_deref().or(current.publisher.as_deref());
    let publish_year = input.publish_year.or(current.publish_year);
    let isbn = input.isbn.as_deref().or(current.isbn.as_deref());
    let asin = input.asin.as_deref().or(current.asin.as_deref());

    let row = sqlx::query_as::<_, BookRow>(
        "UPDATE books SET title = ?, subtitle = ?, description = ?, cover_path = ?, language = ?, publisher = ?, publish_year = ?, isbn = ?, asin = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? RETURNING id, library_id, title, subtitle, description, cover_path, duration_secs, track_count, file_size, language, publisher, publish_year, isbn, asin, source_path, metadata_source, created_at, updated_at",
    )
    .bind(title)
    .bind(subtitle)
    .bind(description)
    .bind(cover_path)
    .bind(language)
    .bind(publisher)
    .bind(publish_year)
    .bind(isbn)
    .bind(asin)
    .bind(id.to_string())
    .fetch_one(pool)
    .await?;
    Ok(row.into())
}

pub async fn delete_book(pool: &sqlx::SqlitePool, id: Uuid) -> Result<()> {
    let result = sqlx::query("DELETE FROM books WHERE id = ?")
        .bind(id.to_string())
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

pub async fn search_books(pool: &sqlx::SqlitePool, query: &str, limit: i64) -> Result<Vec<Book>> {
    let pattern = format!("%{}%", query);
    let rows = sqlx::query_as::<_, BookRow>(
        "SELECT id, library_id, title, subtitle, description, cover_path, duration_secs, track_count, file_size, language, publisher, publish_year, isbn, asin, source_path, metadata_source, created_at, updated_at FROM books WHERE title LIKE ? OR description LIKE ? ORDER BY title LIMIT ?",
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
}

pub async fn get_book_authors(pool: &sqlx::SqlitePool, book_id: Uuid) -> Result<Vec<(Author, AuthorRole)>> {
    let rows = sqlx::query_as::<_, BookAuthorRow>(
        "SELECT ba.book_id, ba.author_id, ba.role, a.id as author_id2, a.name, a.description, a.cover_path, a.created_at, a.updated_at FROM book_authors ba JOIN authors a ON ba.author_id = a.id WHERE ba.book_id = ?",
    )
    .bind(book_id.to_string())
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    for row in rows {
        let role = match row.role.as_str() {
            "narrator" => AuthorRole::Narrator,
            "translator" => AuthorRole::Translator,
            _ => AuthorRole::Author,
        };
        let author = Author {
            id: Uuid::parse_str(&row.author_id2).unwrap_or_default(),
            name: row.name,
            description: row.author_description,
            cover_path: row.author_cover_path,
            book_count: 0,
            created_at: row.author_created_at,
            updated_at: row.author_updated_at,
        };
        result.push((author, role));
    }
    Ok(result)
}

pub async fn get_book_series(pool: &sqlx::SqlitePool, book_id: Uuid) -> Result<Vec<(Series, Option<f64>)>> {
    let rows = sqlx::query_as::<_, BookSeriesRow>(
        "SELECT bs.book_id, bs.series_id, bs.position, s.id as series_id2, s.name, s.description, s.created_at, s.updated_at FROM book_series bs JOIN series s ON bs.series_id = s.id WHERE bs.book_id = ?",
    )
    .bind(book_id.to_string())
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    for row in rows {
        let series = Series {
            id: Uuid::parse_str(&row.series_id2).unwrap_or_default(),
            name: row.name,
            description: row.series_description,
            book_count: 0,
            created_at: row.series_created_at,
            updated_at: row.series_updated_at,
        };
        result.push((series, row.position));
    }
    Ok(result)
}

// --- Track functions ---

pub async fn get_track(pool: &sqlx::SqlitePool, id: Uuid) -> Result<Option<Track>> {
    let row = sqlx::query_as::<_, TrackRow>(
        "SELECT id, book_id, title, disc_number, track_number, file_path, file_size, mime_type, duration_secs, bitrate, sample_rate, channels, codec, created_at FROM tracks WHERE id = ?",
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.into()))
}

pub async fn get_tracks_by_book(pool: &sqlx::SqlitePool, book_id: Uuid) -> Result<Vec<Track>> {
    let rows = sqlx::query_as::<_, TrackRow>(
        "SELECT id, book_id, title, disc_number, track_number, file_path, file_size, mime_type, duration_secs, bitrate, sample_rate, channels, codec, created_at FROM tracks WHERE book_id = ? ORDER BY track_number",
    )
    .bind(book_id.to_string())
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
}

// --- Author functions ---

pub async fn list_authors(pool: &sqlx::SqlitePool) -> Result<Vec<Author>> {
    let rows = sqlx::query_as::<_, AuthorRow>(
        "SELECT a.id, a.name, a.description, a.cover_path, a.created_at, a.updated_at, COUNT(ba.book_id) as book_count FROM authors a LEFT JOIN book_authors ba ON a.id = ba.author_id GROUP BY a.id ORDER BY a.name",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
}

pub async fn get_books_by_author(pool: &sqlx::SqlitePool, author_id: Uuid) -> Result<Vec<Book>> {
    let rows = sqlx::query_as::<_, BookRow>(
        "SELECT b.id, b.library_id, b.title, b.subtitle, b.description, b.cover_path, b.duration_secs, b.track_count, b.file_size, b.language, b.publisher, b.publish_year, b.isbn, b.asin, b.source_path, b.metadata_source, b.created_at, b.updated_at FROM books b JOIN book_authors ba ON b.id = ba.book_id WHERE ba.author_id = ? ORDER BY b.title",
    )
    .bind(author_id.to_string())
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
}

// --- Series functions ---

pub async fn list_series(pool: &sqlx::SqlitePool) -> Result<Vec<Series>> {
    let rows = sqlx::query_as::<_, SeriesListRow>(
        "SELECT s.id, s.name, s.description, s.created_at, s.updated_at, COUNT(bs.book_id) as book_count FROM series s LEFT JOIN book_series bs ON s.id = bs.series_id GROUP BY s.id ORDER BY s.name",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
}

pub async fn get_books_by_series(pool: &sqlx::SqlitePool, series_id: Uuid) -> Result<Vec<Book>> {
    let rows = sqlx::query_as::<_, BookRow>(
        "SELECT b.id, b.library_id, b.title, b.subtitle, b.description, b.cover_path, b.duration_secs, b.track_count, b.file_size, b.language, b.publisher, b.publish_year, b.isbn, b.asin, b.source_path, b.metadata_source, b.created_at, b.updated_at FROM books b JOIN book_series bs ON b.id = bs.book_id WHERE bs.series_id = ? ORDER BY bs.position",
    )
    .bind(series_id.to_string())
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
}

// --- Playback position functions ---

pub async fn upsert_playback_position(pool: &sqlx::SqlitePool, user_id: Uuid, pos: &UpsertPlaybackPosition) -> Result<PlaybackPosition> {
    let percentage = if pos.duration_secs > 0.0 { pos.position_secs / pos.duration_secs } else { 0.0 };
    let is_finished = percentage >= 0.95;

    let row = sqlx::query_as::<_, PlaybackPositionRow>(
        "INSERT INTO playback_positions (user_id, book_id, track_id, position_secs, duration_secs, percentage, is_finished, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP) ON CONFLICT(user_id, book_id) DO UPDATE SET track_id = excluded.track_id, position_secs = excluded.position_secs, duration_secs = excluded.duration_secs, percentage = excluded.percentage, is_finished = excluded.is_finished, updated_at = CURRENT_TIMESTAMP RETURNING id, user_id, book_id, track_id, position_secs, duration_secs, percentage, is_finished, updated_at",
    )
    .bind(user_id.to_string())
    .bind(pos.book_id.to_string())
    .bind(pos.track_id.to_string())
    .bind(pos.position_secs)
    .bind(pos.duration_secs)
    .bind(percentage)
    .bind(is_finished)
    .fetch_one(pool)
    .await?;
    Ok(row.into())
}

pub async fn get_playback_position(pool: &sqlx::SqlitePool, user_id: Uuid, book_id: Uuid) -> Result<Option<PlaybackPosition>> {
    let row = sqlx::query_as::<_, PlaybackPositionRow>(
        "SELECT id, user_id, book_id, track_id, position_secs, duration_secs, percentage, is_finished, updated_at FROM playback_positions WHERE user_id = ? AND book_id = ?",
    )
    .bind(user_id.to_string())
    .bind(book_id.to_string())
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.into()))
}

pub async fn get_user_positions(pool: &sqlx::SqlitePool, user_id: Uuid) -> Result<Vec<PlaybackPosition>> {
    let rows = sqlx::query_as::<_, PlaybackPositionRow>(
        "SELECT id, user_id, book_id, track_id, position_secs, duration_secs, percentage, is_finished, updated_at FROM playback_positions WHERE user_id = ? ORDER BY updated_at DESC",
    )
    .bind(user_id.to_string())
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
}

// --- Bookmark functions ---

pub async fn create_bookmark(pool: &sqlx::SqlitePool, user_id: Uuid, bookmark: &CreateBookmark) -> Result<Bookmark> {
    let row = sqlx::query_as::<_, BookmarkRow>(
        "INSERT INTO bookmarks (user_id, book_id, track_id, title, note, position_secs, created_at) VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP) RETURNING id, user_id, book_id, track_id, title, note, position_secs, created_at",
    )
    .bind(user_id.to_string())
    .bind(bookmark.book_id.to_string())
    .bind(bookmark.track_id.to_string())
    .bind(&bookmark.title)
    .bind(&bookmark.note)
    .bind(bookmark.position_secs)
    .fetch_one(pool)
    .await?;
    Ok(row.into())
}

pub async fn list_bookmarks(pool: &sqlx::SqlitePool, user_id: Uuid, book_id: Uuid) -> Result<Vec<Bookmark>> {
    let rows = sqlx::query_as::<_, BookmarkRow>(
        "SELECT id, user_id, book_id, track_id, title, note, position_secs, created_at FROM bookmarks WHERE user_id = ? AND book_id = ? ORDER BY position_secs",
    )
    .bind(user_id.to_string())
    .bind(book_id.to_string())
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
}

pub async fn list_all_user_bookmarks(pool: &sqlx::SqlitePool, user_id: Uuid) -> Result<Vec<Bookmark>> {
    let rows = sqlx::query_as::<_, BookmarkRow>(
        "SELECT id, user_id, book_id, track_id, title, note, position_secs, created_at FROM bookmarks WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(user_id.to_string())
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
}

pub async fn delete_bookmark(pool: &sqlx::SqlitePool, id: Uuid) -> Result<()> {
    let result = sqlx::query("DELETE FROM bookmarks WHERE id = ?")
        .bind(id.to_string())
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

// --- Collection functions ---

pub async fn create_collection(pool: &sqlx::SqlitePool, user_id: Uuid, collection: &CreateCollection) -> Result<Collection> {
    let row = sqlx::query_as::<_, CollectionRow>(
        "INSERT INTO collections (user_id, name, description, created_at, updated_at) VALUES (?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) RETURNING id, user_id, name, description, cover_path, created_at, updated_at",
    )
    .bind(user_id.to_string())
    .bind(&collection.name)
    .bind(&collection.description)
    .fetch_one(pool)
    .await?;

    let mut col: Collection = row.into();
    col.book_count = 0;
    Ok(col)
}

pub async fn list_collections(pool: &sqlx::SqlitePool, user_id: Uuid) -> Result<Vec<Collection>> {
    let rows = sqlx::query_as::<_, CollectionWithCountRow>(
        "SELECT c.id, c.user_id, c.name, c.description, c.cover_path, c.created_at, c.updated_at, COUNT(cb.book_id) as book_count FROM collections c LEFT JOIN collection_books cb ON c.id = cb.collection_id WHERE c.user_id = ? GROUP BY c.id ORDER BY c.name",
    )
    .bind(user_id.to_string())
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
}

pub async fn delete_collection(pool: &sqlx::SqlitePool, id: Uuid) -> Result<()> {
    let result = sqlx::query("DELETE FROM collections WHERE id = ?")
        .bind(id.to_string())
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

pub async fn add_book_to_collection(pool: &sqlx::SqlitePool, collection_id: Uuid, book_id: Uuid) -> Result<()> {
    sqlx::query("INSERT OR IGNORE INTO collection_books (collection_id, book_id, position, added_at) VALUES (?, ?, 0, CURRENT_TIMESTAMP)")
        .bind(collection_id.to_string())
        .bind(book_id.to_string())
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn remove_book_from_collection(pool: &sqlx::SqlitePool, collection_id: Uuid, book_id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM collection_books WHERE collection_id = ? AND book_id = ?")
        .bind(collection_id.to_string())
        .bind(book_id.to_string())
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_collection_books(pool: &sqlx::SqlitePool, collection_id: Uuid) -> Result<Vec<Book>> {
    let rows = sqlx::query_as::<_, BookRow>(
        "SELECT b.id, b.library_id, b.title, b.subtitle, b.description, b.cover_path, b.duration_secs, b.track_count, b.file_size, b.language, b.publisher, b.publish_year, b.isbn, b.asin, b.source_path, b.metadata_source, b.created_at, b.updated_at FROM books b JOIN collection_books cb ON b.id = cb.book_id WHERE cb.collection_id = ? ORDER BY cb.position",
    )
    .bind(collection_id.to_string())
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
}

// --- User functions ---

pub async fn list_users(pool: &sqlx::SqlitePool) -> Result<Vec<User>> {
    let rows = sqlx::query_as::<_, UserRow>(
        "SELECT id, username, display_name, email, password_hash, role, is_active, created_at, updated_at FROM users ORDER BY username",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
}

pub async fn get_user_by_id(pool: &sqlx::SqlitePool, id: Uuid) -> Result<Option<User>> {
    let row = sqlx::query_as::<_, UserRow>(
        "SELECT id, username, display_name, email, password_hash, role, is_active, created_at, updated_at FROM users WHERE id = ?",
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.into()))
}

pub async fn get_user_by_username(pool: &sqlx::SqlitePool, username: &str) -> Result<Option<User>> {
    let row = sqlx::query_as::<_, UserRow>(
        "SELECT id, username, display_name, email, password_hash, role, is_active, created_at, updated_at FROM users WHERE username = ?",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.into()))
}

pub async fn create_user(pool: &sqlx::SqlitePool, input: &CreateUser) -> Result<User> {
    let role_str = match input.role {
        UserRole::Admin => "admin",
        UserRole::User => "user",
        UserRole::Guest => "guest",
    };
    let row = sqlx::query_as::<_, UserRow>(
        "INSERT INTO users (username, display_name, email, password_hash, role, is_active, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) RETURNING id, username, display_name, email, password_hash, role, is_active, created_at, updated_at",
    )
    .bind(&input.username)
    .bind(&input.display_name)
    .bind(&input.email)
    .bind(&input.password)
    .bind(role_str)
    .fetch_one(pool)
    .await?;
    Ok(row.into())
}

// --- Row types ---

#[derive(sqlx::FromRow)]
struct LibraryRow {
    id: i64,
    name: String,
    root_path: String,
    source_type: String,
    base_url: Option<String>,
    username: Option<String>,
    password: Option<String>,
    scan_interval: Option<i64>,
    last_scan: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<LibraryRow> for Library {
    fn from(row: LibraryRow) -> Self {
        Library {
            id: row.id,
            name: row.name,
            root_path: row.root_path,
            source_type: SourceType::from_str_opt(&row.source_type).unwrap_or_default(),
            base_url: row.base_url,
            username: row.username,
            password: row.password,
            scan_interval: row.scan_interval,
            last_scan: row.last_scan,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct BookRow {
    id: String,
    library_id: String,
    title: String,
    subtitle: Option<String>,
    description: Option<String>,
    cover_path: Option<String>,
    duration_secs: f64,
    track_count: i32,
    file_size: i64,
    language: Option<String>,
    publisher: Option<String>,
    publish_year: Option<i32>,
    isbn: Option<String>,
    asin: Option<String>,
    source_path: String,
    metadata_source: Option<String>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl From<BookRow> for Book {
    fn from(row: BookRow) -> Self {
        Book {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            library_id: Uuid::parse_str(&row.library_id).unwrap_or_default(),
            title: row.title,
            subtitle: row.subtitle,
            description: row.description,
            cover_path: row.cover_path,
            duration_secs: row.duration_secs,
            track_count: row.track_count,
            file_size: row.file_size,
            language: row.language,
            publisher: row.publisher,
            publish_year: row.publish_year,
            isbn: row.isbn,
            asin: row.asin,
            source_path: row.source_path,
            metadata_source: row.metadata_source,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TrackRow {
    id: String,
    book_id: String,
    title: String,
    disc_number: Option<i32>,
    track_number: i32,
    file_path: String,
    file_size: i64,
    mime_type: String,
    duration_secs: f64,
    bitrate: Option<i32>,
    sample_rate: Option<i32>,
    channels: Option<i32>,
    codec: Option<String>,
    created_at: NaiveDateTime,
}

impl From<TrackRow> for Track {
    fn from(row: TrackRow) -> Self {
        Track {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            book_id: Uuid::parse_str(&row.book_id).unwrap_or_default(),
            title: row.title,
            disc_number: row.disc_number,
            track_number: row.track_number,
            file_path: row.file_path,
            file_size: row.file_size,
            mime_type: row.mime_type,
            duration_secs: row.duration_secs,
            bitrate: row.bitrate,
            sample_rate: row.sample_rate,
            channels: row.channels,
            codec: row.codec,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct AuthorRow {
    id: String,
    name: String,
    description: Option<String>,
    cover_path: Option<String>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    book_count: i64,
}

impl From<AuthorRow> for Author {
    fn from(row: AuthorRow) -> Self {
        Author {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            name: row.name,
            description: row.description,
            cover_path: row.cover_path,
            book_count: row.book_count,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct SeriesListRow {
    id: String,
    name: String,
    description: Option<String>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    book_count: i64,
}

impl From<SeriesListRow> for Series {
    fn from(row: SeriesListRow) -> Self {
        Series {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            name: row.name,
            description: row.description,
            book_count: row.book_count,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct BookAuthorRow {
    book_id: String,
    author_id: String,
    role: String,
    author_id2: String,
    name: String,
    author_description: Option<String>,
    author_cover_path: Option<String>,
    author_created_at: NaiveDateTime,
    author_updated_at: NaiveDateTime,
}

#[derive(sqlx::FromRow)]
struct BookSeriesRow {
    book_id: String,
    series_id: String,
    position: Option<f64>,
    series_id2: String,
    name: String,
    series_description: Option<String>,
    series_created_at: NaiveDateTime,
    series_updated_at: NaiveDateTime,
}

#[derive(sqlx::FromRow)]
struct PlaybackPositionRow {
    id: String,
    user_id: String,
    book_id: String,
    track_id: String,
    position_secs: f64,
    duration_secs: f64,
    percentage: f64,
    is_finished: bool,
    updated_at: NaiveDateTime,
}

impl From<PlaybackPositionRow> for PlaybackPosition {
    fn from(row: PlaybackPositionRow) -> Self {
        PlaybackPosition {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            user_id: Uuid::parse_str(&row.user_id).unwrap_or_default(),
            book_id: Uuid::parse_str(&row.book_id).unwrap_or_default(),
            track_id: Uuid::parse_str(&row.track_id).unwrap_or_default(),
            position_secs: row.position_secs,
            duration_secs: row.duration_secs,
            percentage: row.percentage,
            is_finished: row.is_finished,
            updated_at: row.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct BookmarkRow {
    id: String,
    user_id: String,
    book_id: String,
    track_id: String,
    title: String,
    note: Option<String>,
    position_secs: f64,
    created_at: NaiveDateTime,
}

impl From<BookmarkRow> for Bookmark {
    fn from(row: BookmarkRow) -> Self {
        Bookmark {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            user_id: Uuid::parse_str(&row.user_id).unwrap_or_default(),
            book_id: Uuid::parse_str(&row.book_id).unwrap_or_default(),
            track_id: Uuid::parse_str(&row.track_id).unwrap_or_default(),
            title: row.title,
            note: row.note,
            position_secs: row.position_secs,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct CollectionRow {
    id: String,
    user_id: String,
    name: String,
    description: Option<String>,
    cover_path: Option<String>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl From<CollectionRow> for Collection {
    fn from(row: CollectionRow) -> Self {
        Collection {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            user_id: Uuid::parse_str(&row.user_id).unwrap_or_default(),
            name: row.name,
            description: row.description,
            cover_path: row.cover_path,
            book_count: 0,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct CollectionWithCountRow {
    id: String,
    user_id: String,
    name: String,
    description: Option<String>,
    cover_path: Option<String>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    book_count: i64,
}

impl From<CollectionWithCountRow> for Collection {
    fn from(row: CollectionWithCountRow) -> Self {
        Collection {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            user_id: Uuid::parse_str(&row.user_id).unwrap_or_default(),
            name: row.name,
            description: row.description,
            cover_path: row.cover_path,
            book_count: row.book_count,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: String,
    username: String,
    display_name: Option<String>,
    email: Option<String>,
    password_hash: String,
    role: String,
    is_active: bool,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        User {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            username: row.username,
            display_name: row.display_name,
            email: row.email,
            password_hash: row.password_hash,
            role: match row.role.as_str() {
                "admin" => UserRole::Admin,
                "guest" => UserRole::Guest,
                _ => UserRole::User,
            },
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
