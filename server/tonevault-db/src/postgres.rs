use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::Row;
use uuid::Uuid;

use crate::repository::Repository;
use tonevault_core::models::*;

#[derive(Clone)]
pub struct PostgresRepository {
    pool: PgPool,
}

impl PostgresRepository {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<()> {
        let sql = include_str!("migrations/postgres/001_initial.sql");
        for statement in sql.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                sqlx::query(trimmed).execute(&self.pool).await?;
            }
        }
        Ok(())
    }

    fn row_to_user(row: &PgRow) -> User {
        User {
            id: row.get("id"),
            username: row.get("username"),
            display_name: row.get("display_name"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            role: match row.get::<String, _>("role").as_str() {
                "admin" => UserRole::Admin,
                "user" => UserRole::User,
                _ => UserRole::Guest,
            },
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_user_public(row: &PgRow) -> UserPublic {
        UserPublic {
            id: row.get("id"),
            username: row.get("username"),
            display_name: row.get("display_name"),
            email: row.get("email"),
            role: match row.get::<String, _>("role").as_str() {
                "admin" => UserRole::Admin,
                "user" => UserRole::User,
                _ => UserRole::Guest,
            },
        }
    }

    fn row_to_library(row: &PgRow) -> Library {
        Library {
            id: row.get("id"),
            name: row.get("name"),
            root_path: row.get("root_path"),
            description: row.get("description"),
            scan_enabled: row.get("scan_enabled"),
            watch_enabled: row.get("watch_enabled"),
            last_scan_at: row.get("last_scan_at"),
            scan_status: match row.get::<String, _>("scan_status").as_str() {
                "scanning" => ScanStatus::Scanning,
                "error" => ScanStatus::Error,
                _ => ScanStatus::Idle,
            },
            book_count: row.get("book_count"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_book(row: &PgRow) -> Book {
        Book {
            id: row.get("id"),
            library_id: row.get("library_id"),
            title: row.get("title"),
            subtitle: row.get("subtitle"),
            description: row.get("description"),
            cover_path: row.get("cover_path"),
            duration_secs: row.get("duration_secs"),
            track_count: row.get("track_count"),
            file_size: row.get("file_size"),
            language: row.get("language"),
            publisher: row.get("publisher"),
            publish_year: row.get("publish_year"),
            isbn: row.get("isbn"),
            asin: row.get("asin"),
            source_path: row.get("source_path"),
            metadata_source: row.get("metadata_source"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_track(row: &PgRow) -> Track {
        Track {
            id: row.get("id"),
            book_id: row.get("book_id"),
            title: row.get("title"),
            disc_number: row.get("disc_number"),
            track_number: row.get("track_number"),
            file_path: row.get("file_path"),
            file_size: row.get("file_size"),
            mime_type: row.get("mime_type"),
            duration_secs: row.get("duration_secs"),
            bitrate: row.get("bitrate"),
            sample_rate: row.get("sample_rate"),
            channels: row.get("channels"),
            codec: row.get("codec"),
            created_at: row.get("created_at"),
        }
    }

    fn row_to_author(row: &PgRow) -> Author {
        Author {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            cover_path: row.get("cover_path"),
            book_count: row.get("book_count"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_series(row: &PgRow) -> Series {
        Series {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            book_count: row.get("book_count"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_playback_position(row: &PgRow) -> PlaybackPosition {
        PlaybackPosition {
            id: row.get("id"),
            user_id: row.get("user_id"),
            book_id: row.get("book_id"),
            track_id: row.get("track_id"),
            position_secs: row.get("position_secs"),
            duration_secs: row.get("duration_secs"),
            percentage: row.get("percentage"),
            is_finished: row.get("is_finished"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_bookmark(row: &PgRow) -> Bookmark {
        Bookmark {
            id: row.get("id"),
            user_id: row.get("user_id"),
            book_id: row.get("book_id"),
            track_id: row.get("track_id"),
            title: row.get("title"),
            note: row.get("note"),
            position_secs: row.get("position_secs"),
            created_at: row.get("created_at"),
        }
    }

    fn row_to_collection(row: &PgRow) -> Collection {
        Collection {
            id: row.get("id"),
            user_id: row.get("user_id"),
            name: row.get("name"),
            description: row.get("description"),
            cover_path: row.get("cover_path"),
            book_count: row.get("book_count"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_api_key(row: &PgRow) -> ApiKey {
        ApiKey {
            id: row.get("id"),
            user_id: row.get("user_id"),
            name: row.get("name"),
            key_prefix: row.get("key_prefix"),
            key_hash: row.get("key_hash"),
            last_used_at: row.get("last_used_at"),
            created_at: row.get("created_at"),
        }
    }

    fn now() -> NaiveDateTime {
        chrono::Utc::now().naive_utc()
    }
}

#[async_trait]
impl Repository for PostgresRepository {
    // Users
    async fn create_user(&self, user: &CreateUser) -> Result<User> {
        let id = Uuid::new_v4();
        let now = Self::now();
        let role_str = match user.role {
            UserRole::Admin => "admin",
            UserRole::User => "user",
            UserRole::Guest => "guest",
        };
        sqlx::query(
            "INSERT INTO users (id, username, email, password_hash, display_name, role, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, true, $7, $8)"
        )
        .bind(id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password)
        .bind(&user.display_name)
        .bind(role_str)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(self.get_user_by_id(id).await?.unwrap())
    }

    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let row = sqlx::query("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.as_ref().map(Self::row_to_user))
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let row = sqlx::query("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.as_ref().map(Self::row_to_user))
    }

    async fn list_users(&self) -> Result<Vec<UserPublic>> {
        let rows = sqlx::query("SELECT id, username, display_name, email, role FROM users ORDER BY created_at")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.iter().map(Self::row_to_user_public).collect())
    }

    async fn update_user_password(&self, id: Uuid, password_hash: &str) -> Result<()> {
        let now = Self::now();
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = $2 WHERE id = $3")
            .bind(password_hash)
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete_user(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // API Keys
    async fn create_api_key(&self, user_id: Uuid, name: &str, key_hash: &str, key_prefix: &str) -> Result<ApiKey> {
        let id = Uuid::new_v4();
        let now = Self::now();
        sqlx::query(
            "INSERT INTO api_keys (id, user_id, name, key_hash, key_prefix, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(id)
        .bind(user_id)
        .bind(name)
        .bind(key_hash)
        .bind(key_prefix)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query("SELECT * FROM api_keys WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(Self::row_to_api_key(&row))
    }

    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        let row = sqlx::query("SELECT * FROM api_keys WHERE key_hash = $1")
            .bind(key_hash)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.as_ref().map(Self::row_to_api_key))
    }

    async fn list_api_keys(&self, user_id: Uuid) -> Result<Vec<ApiKey>> {
        let rows = sqlx::query("SELECT * FROM api_keys WHERE user_id = $1 ORDER BY created_at")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.iter().map(Self::row_to_api_key).collect())
    }

    async fn delete_api_key(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM api_keys WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_api_key_last_used(&self, id: Uuid) -> Result<()> {
        let now = Self::now();
        sqlx::query("UPDATE api_keys SET last_used_at = $1 WHERE id = $2")
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Libraries
    async fn create_library(&self, library: &CreateLibrary) -> Result<Library> {
        let id = Uuid::new_v4();
        let now = Self::now();
        sqlx::query(
            "INSERT INTO libraries (id, name, root_path, description, scan_enabled, watch_enabled, scan_status, book_count, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, 'idle', 0, $7, $8)"
        )
        .bind(id)
        .bind(&library.name)
        .bind(&library.root_path)
        .bind(&library.description)
        .bind(library.scan_enabled)
        .bind(library.watch_enabled)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query("SELECT * FROM libraries WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(Self::row_to_library(&row))
    }

    async fn get_library(&self, id: Uuid) -> Result<Option<Library>> {
        let row = sqlx::query("SELECT * FROM libraries WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.as_ref().map(Self::row_to_library))
    }

    async fn list_libraries(&self) -> Result<Vec<Library>> {
        let rows = sqlx::query("SELECT * FROM libraries ORDER BY name")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.iter().map(Self::row_to_library).collect())
    }

    async fn update_library(&self, id: Uuid, update: &UpdateLibrary) -> Result<Library> {
        let now = Self::now();
        let mut sets = vec!["updated_at = $1".to_string()];
        let mut param_idx = 2;

        if update.name.is_some() { sets.push(format!("name = ${}", param_idx)); param_idx += 1; }
        if update.root_path.is_some() { sets.push(format!("root_path = ${}", param_idx)); param_idx += 1; }
        if update.description.is_some() { sets.push(format!("description = ${}", param_idx)); param_idx += 1; }
        if update.scan_enabled.is_some() { sets.push(format!("scan_enabled = ${}", param_idx)); param_idx += 1; }
        if update.watch_enabled.is_some() { sets.push(format!("watch_enabled = ${}", param_idx)); param_idx += 1; }

        let sql = format!("UPDATE libraries SET {} WHERE id = ${}", sets.join(", "), param_idx);
        let mut query = sqlx::query(&sql).bind(now);

        if let Some(ref v) = update.name { query = query.bind(v); }
        if let Some(ref v) = update.root_path { query = query.bind(v); }
        if let Some(ref v) = update.description { query = query.bind(v); }
        if let Some(v) = update.scan_enabled { query = query.bind(v); }
        if let Some(v) = update.watch_enabled { query = query.bind(v); }
        query = query.bind(id);

        query.execute(&self.pool).await?;
        Ok(self.get_library(id).await?.unwrap())
    }

    async fn delete_library(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM libraries WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_library_scan_status(&self, id: Uuid, status: ScanStatus) -> Result<()> {
        let now = Self::now();
        let status_str = match status {
            ScanStatus::Idle => "idle",
            ScanStatus::Scanning => "scanning",
            ScanStatus::Error => "error",
        };
        sqlx::query("UPDATE libraries SET scan_status = $1, updated_at = $2 WHERE id = $3")
            .bind(status_str)
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_library_scan_time(&self, id: Uuid) -> Result<()> {
        let now = Self::now();
        sqlx::query("UPDATE libraries SET last_scan_at = $1, scan_status = 'idle', updated_at = $2 WHERE id = $3")
            .bind(now)
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Authors
    async fn create_author(&self, author: &CreateAuthor) -> Result<Author> {
        let id = Uuid::new_v4();
        let now = Self::now();
        let sort_name = author.name.clone();
        sqlx::query(
            "INSERT INTO authors (id, name, sort_name, description, cover_path, book_count, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 0, $6, $7)"
        )
        .bind(id)
        .bind(&author.name)
        .bind(&sort_name)
        .bind(&author.description)
        .bind(None::<String> as Option<String>)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query("SELECT * FROM authors WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(Self::row_to_author(&row))
    }

    async fn get_author(&self, id: Uuid) -> Result<Option<Author>> {
        let row = sqlx::query("SELECT * FROM authors WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.as_ref().map(Self::row_to_author))
    }

    async fn list_authors(&self) -> Result<Vec<Author>> {
        let rows = sqlx::query("SELECT * FROM authors ORDER BY sort_name")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.iter().map(Self::row_to_author).collect())
    }

    async fn find_author_by_name(&self, name: &str) -> Result<Option<Author>> {
        let row = sqlx::query("SELECT * FROM authors WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.as_ref().map(Self::row_to_author))
    }

    // Series
    async fn create_series(&self, series: &CreateSeries) -> Result<Series> {
        let id = Uuid::new_v4();
        let now = Self::now();
        sqlx::query(
            "INSERT INTO series (id, name, description, book_count, created_at, updated_at) VALUES ($1, $2, $3, 0, $4, $5)"
        )
        .bind(id)
        .bind(&series.name)
        .bind(&series.description)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query("SELECT * FROM series WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(Self::row_to_series(&row))
    }

    async fn get_series(&self, id: Uuid) -> Result<Option<Series>> {
        let row = sqlx::query("SELECT * FROM series WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.as_ref().map(Self::row_to_series))
    }

    async fn list_series(&self) -> Result<Vec<Series>> {
        let rows = sqlx::query("SELECT * FROM series ORDER BY name")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.iter().map(Self::row_to_series).collect())
    }

    async fn find_series_by_name(&self, name: &str) -> Result<Option<Series>> {
        let row = sqlx::query("SELECT * FROM series WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.as_ref().map(Self::row_to_series))
    }

    // Books
    async fn create_book(&self, book: &CreateBook) -> Result<Book> {
        let id = Uuid::new_v4();
        let now = Self::now();
        sqlx::query(
            "INSERT INTO books (id, library_id, title, subtitle, description, cover_path, duration_secs, track_count, file_size, language, publisher, publish_year, isbn, asin, source_path, metadata_source, last_modified, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, 0, 0, 0, $7, $8, $9, $10, $11, $12, 'unknown', $13, $14, $15)"
        )
        .bind(id)
        .bind(book.library_id)
        .bind(&book.title)
        .bind(&book.subtitle)
        .bind(&book.description)
        .bind(&book.cover_path)
        .bind(&book.language)
        .bind(&book.publisher)
        .bind(book.publish_year)
        .bind(&book.isbn)
        .bind(&book.asin)
        .bind(&book.source_path)
        .bind(now)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query("SELECT * FROM books WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(Self::row_to_book(&row))
    }

    async fn get_book(&self, id: Uuid) -> Result<Option<Book>> {
        let row = sqlx::query("SELECT * FROM books WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.as_ref().map(Self::row_to_book))
    }

    async fn list_books(&self, filter: &BookFilter) -> Result<PaginatedResult<Book>> {
        let mut conditions = vec!["1=1".to_string()];
        let mut param_idx = 0;

        if filter.library_id.is_some() {
            param_idx += 1;
            conditions.push(format!("library_id = ${}", param_idx));
        }
        if filter.author_id.is_some() {
            param_idx += 1;
            conditions.push(format!("id IN (SELECT book_id FROM book_authors WHERE author_id = ${})", param_idx));
        }
        if filter.series_id.is_some() {
            param_idx += 1;
            conditions.push(format!("id IN (SELECT book_id FROM book_series WHERE series_id = ${})", param_idx));
        }
        if filter.query.is_some() {
            param_idx += 1;
            conditions.push(format!("(title LIKE ${} OR subtitle LIKE ${} OR description LIKE ${})", param_idx, param_idx, param_idx));
        }

        let where_clause = conditions.join(" AND ");

        let count_sql = format!("SELECT COUNT(*) as count FROM books WHERE {}", where_clause);
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);

        if let Some(v) = filter.library_id { count_query = count_query.bind(v); }
        if let Some(v) = filter.author_id { count_query = count_query.bind(v); }
        if let Some(v) = filter.series_id { count_query = count_query.bind(v); }
        if let Some(ref v) = filter.query { count_query = count_query.bind(format!("%{}%", v)); }
        let total = count_query.fetch_one(&self.pool).await?;

        let order = match filter.sort {
            BookSort::Title => "title",
            BookSort::Added => "created_at",
            BookSort::Duration => "duration_secs",
            BookSort::Year => "publish_year",
            BookSort::Author => "title",
        };
        let direction = match filter.order {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        };

        let offset = (filter.page - 1) * filter.per_page;
        let limit_idx = param_idx + 1;
        let offset_idx = param_idx + 2;
        let data_sql = format!(
            "SELECT * FROM books WHERE {} ORDER BY {} {} LIMIT ${} OFFSET ${}",
            where_clause, order, direction, limit_idx, offset_idx
        );
        let mut data_query = sqlx::query(&data_sql);

        if let Some(v) = filter.library_id { data_query = data_query.bind(v); }
        if let Some(v) = filter.author_id { data_query = data_query.bind(v); }
        if let Some(v) = filter.series_id { data_query = data_query.bind(v); }
        if let Some(ref v) = filter.query { data_query = data_query.bind(format!("%{}%", v)); }
        data_query = data_query.bind(filter.per_page);
        data_query = data_query.bind(offset);

        let rows = data_query.fetch_all(&self.pool).await?;
        let items: Vec<Book> = rows.iter().map(Self::row_to_book).collect();
        let total_pages = (total + filter.per_page - 1) / filter.per_page;

        Ok(PaginatedResult {
            items,
            total,
            page: filter.page,
            per_page: filter.per_page,
            total_pages,
        })
    }

    async fn update_book(&self, id: Uuid, update: &UpdateBook) -> Result<Book> {
        let now = Self::now();
        let mut sets = vec!["updated_at = $1".to_string()];
        let mut param_idx = 2;

        if update.title.is_some() { sets.push(format!("title = ${}", param_idx)); param_idx += 1; }
        if update.subtitle.is_some() { sets.push(format!("subtitle = ${}", param_idx)); param_idx += 1; }
        if update.description.is_some() { sets.push(format!("description = ${}", param_idx)); param_idx += 1; }
        if update.cover_path.is_some() { sets.push(format!("cover_path = ${}", param_idx)); param_idx += 1; }
        if update.language.is_some() { sets.push(format!("language = ${}", param_idx)); param_idx += 1; }
        if update.publisher.is_some() { sets.push(format!("publisher = ${}", param_idx)); param_idx += 1; }
        if update.publish_year.is_some() { sets.push(format!("publish_year = ${}", param_idx)); param_idx += 1; }
        if update.isbn.is_some() { sets.push(format!("isbn = ${}", param_idx)); param_idx += 1; }
        if update.asin.is_some() { sets.push(format!("asin = ${}", param_idx)); param_idx += 1; }

        let sql = format!("UPDATE books SET {} WHERE id = ${}", sets.join(", "), param_idx);
        let mut query = sqlx::query(&sql).bind(now);

        if let Some(ref v) = update.title { query = query.bind(v); }
        if let Some(ref v) = update.subtitle { query = query.bind(v); }
        if let Some(ref v) = update.description { query = query.bind(v); }
        if let Some(ref v) = update.cover_path { query = query.bind(v); }
        if let Some(ref v) = update.language { query = query.bind(v); }
        if let Some(ref v) = update.publisher { query = query.bind(v); }
        if let Some(ref v) = update.publish_year { query = query.bind(*v); }
        if let Some(ref v) = update.isbn { query = query.bind(v); }
        if let Some(ref v) = update.asin { query = query.bind(v); }
        query = query.bind(id);

        query.execute(&self.pool).await?;
        Ok(self.get_book(id).await?.unwrap())
    }

    async fn delete_book(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM books WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_books_by_author(&self, author_id: Uuid) -> Result<Vec<Book>> {
        let rows = sqlx::query(
            "SELECT b.* FROM books b JOIN book_authors ba ON b.id = ba.book_id WHERE ba.author_id = $1 ORDER BY b.title"
        )
        .bind(author_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(Self::row_to_book).collect())
    }

    async fn get_books_by_series(&self, series_id: Uuid) -> Result<Vec<Book>> {
        let rows = sqlx::query(
            "SELECT b.* FROM books b JOIN book_series bs ON b.id = bs.book_id WHERE bs.series_id = $1 ORDER BY bs.position"
        )
        .bind(series_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(Self::row_to_book).collect())
    }

    async fn search_books(&self, query: &str, limit: i64) -> Result<Vec<Book>> {
        let pattern = format!("%{}%", query);
        let rows = sqlx::query(
            "SELECT * FROM books WHERE title LIKE $1 OR subtitle LIKE $1 OR description LIKE $1 OR publisher LIKE $1 ORDER BY title LIMIT $2"
        )
        .bind(&pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(Self::row_to_book).collect())
    }

    async fn find_book_by_source_path(&self, library_id: Uuid, source_path: &str) -> Result<Option<Book>> {
        let row = sqlx::query("SELECT * FROM books WHERE library_id = $1 AND source_path = $2")
            .bind(library_id)
            .bind(source_path)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.as_ref().map(Self::row_to_book))
    }

    async fn update_book_stats(&self, id: Uuid, duration_secs: f64, track_count: i32, file_size: i64) -> Result<()> {
        let now = Self::now();
        sqlx::query("UPDATE books SET duration_secs = $1, track_count = $2, file_size = $3, updated_at = $4 WHERE id = $5")
            .bind(duration_secs)
            .bind(track_count)
            .bind(file_size)
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_library_book_count(&self, id: Uuid) -> Result<()> {
        let now = Self::now();
        sqlx::query("UPDATE libraries SET book_count = (SELECT COUNT(*) FROM books WHERE library_id = $1), updated_at = $2 WHERE id = $3")
            .bind(id)
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Tracks
    async fn create_track(&self, track: &Track) -> Result<Track> {
        let now = Self::now();
        sqlx::query(
            "INSERT INTO tracks (id, book_id, title, track_number, disc_number, file_path, file_size, mime_type, duration_secs, bitrate, sample_rate, channels, codec, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)"
        )
        .bind(track.id)
        .bind(track.book_id)
        .bind(&track.title)
        .bind(track.track_number)
        .bind(track.disc_number)
        .bind(&track.file_path)
        .bind(track.file_size)
        .bind(&track.mime_type)
        .bind(track.duration_secs)
        .bind(track.bitrate)
        .bind(track.sample_rate)
        .bind(track.channels)
        .bind(&track.codec)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query("SELECT * FROM tracks WHERE id = $1")
            .bind(track.id)
            .fetch_one(&self.pool)
            .await?;
        Ok(Self::row_to_track(&row))
    }

    async fn get_tracks_by_book(&self, book_id: Uuid) -> Result<Vec<Track>> {
        let rows = sqlx::query("SELECT * FROM tracks WHERE book_id = $1 ORDER BY disc_number, track_number")
            .bind(book_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.iter().map(Self::row_to_track).collect())
    }

    async fn get_track(&self, id: Uuid) -> Result<Option<Track>> {
        let row = sqlx::query("SELECT * FROM tracks WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.as_ref().map(Self::row_to_track))
    }

    async fn delete_tracks_by_book(&self, book_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM tracks WHERE book_id = $1")
            .bind(book_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Book-Author relationships
    async fn add_book_author(&self, book_id: Uuid, author_id: Uuid, role: AuthorRole) -> Result<()> {
        let role_str = match role {
            AuthorRole::Author => "author",
            AuthorRole::Narrator => "narrator",
            AuthorRole::Translator => "translator",
        };
        sqlx::query("INSERT INTO book_authors (book_id, author_id, role) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
            .bind(book_id)
            .bind(author_id)
            .bind(role_str)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_book_authors(&self, book_id: Uuid) -> Result<Vec<(Author, AuthorRole)>> {
        let rows = sqlx::query(
            "SELECT a.*, ba.role FROM authors a JOIN book_authors ba ON a.id = ba.author_id WHERE ba.book_id = $1"
        )
        .bind(book_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(|r| {
            let author = Self::row_to_author(r);
            let role = match r.get::<String, _>("role").as_str() {
                "narrator" => AuthorRole::Narrator,
                "translator" => AuthorRole::Translator,
                _ => AuthorRole::Author,
            };
            (author, role)
        }).collect())
    }

    async fn remove_book_author(&self, book_id: Uuid, author_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM book_authors WHERE book_id = $1 AND author_id = $2")
            .bind(book_id)
            .bind(author_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Book-Series relationships
    async fn add_book_series(&self, book_id: Uuid, series_id: Uuid, position: Option<f64>) -> Result<()> {
        sqlx::query("INSERT INTO book_series (book_id, series_id, position) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
            .bind(book_id)
            .bind(series_id)
            .bind(position)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_book_series(&self, book_id: Uuid) -> Result<Vec<(Series, Option<f64>)>> {
        let rows = sqlx::query(
            "SELECT s.*, bs.position FROM series s JOIN book_series bs ON s.id = bs.series_id WHERE bs.book_id = $1"
        )
        .bind(book_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(|r| {
            let series = Self::row_to_series(r);
            let position: Option<f64> = r.get("position");
            (series, position)
        }).collect())
    }

    // Playback positions
    async fn upsert_playback_position(&self, user_id: Uuid, pos: &UpsertPlaybackPosition) -> Result<PlaybackPosition> {
        let now = Self::now();
        let percentage = if pos.duration_secs > 0.0 {
            pos.position_secs / pos.duration_secs * 100.0
        } else {
            0.0
        };
        let is_finished = percentage >= 100.0;

        sqlx::query(
            "INSERT INTO playback_positions (id, user_id, book_id, track_id, position_secs, duration_secs, percentage, is_finished, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             ON CONFLICT(user_id, book_id) DO UPDATE SET \
             track_id = EXCLUDED.track_id, position_secs = EXCLUDED.position_secs, \
             duration_secs = EXCLUDED.duration_secs, percentage = EXCLUDED.percentage, \
             is_finished = EXCLUDED.is_finished, updated_at = EXCLUDED.updated_at"
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(pos.book_id)
        .bind(pos.track_id)
        .bind(pos.position_secs)
        .bind(pos.duration_secs)
        .bind(percentage)
        .bind(is_finished)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query("SELECT * FROM playback_positions WHERE user_id = $1 AND book_id = $2")
            .bind(user_id)
            .bind(pos.book_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(Self::row_to_playback_position(&row))
    }

    async fn get_playback_position(&self, user_id: Uuid, book_id: Uuid) -> Result<Option<PlaybackPosition>> {
        let row = sqlx::query("SELECT * FROM playback_positions WHERE user_id = $1 AND book_id = $2")
            .bind(user_id)
            .bind(book_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.as_ref().map(Self::row_to_playback_position))
    }

    async fn get_user_positions(&self, user_id: Uuid) -> Result<Vec<PlaybackPosition>> {
        let rows = sqlx::query("SELECT * FROM playback_positions WHERE user_id = $1 ORDER BY updated_at DESC")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.iter().map(Self::row_to_playback_position).collect())
    }

    // Bookmarks
    async fn create_bookmark(&self, user_id: Uuid, bookmark: &CreateBookmark) -> Result<Bookmark> {
        let id = Uuid::new_v4();
        let now = Self::now();
        sqlx::query(
            "INSERT INTO bookmarks (id, user_id, book_id, track_id, title, note, position_secs, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind(id)
        .bind(user_id)
        .bind(bookmark.book_id)
        .bind(bookmark.track_id)
        .bind(&bookmark.title)
        .bind(&bookmark.note)
        .bind(bookmark.position_secs)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query("SELECT * FROM bookmarks WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(Self::row_to_bookmark(&row))
    }

    async fn get_bookmark(&self, id: Uuid) -> Result<Option<Bookmark>> {
        let row = sqlx::query("SELECT * FROM bookmarks WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.as_ref().map(Self::row_to_bookmark))
    }

    async fn list_bookmarks(&self, user_id: Uuid, book_id: Uuid) -> Result<Vec<Bookmark>> {
        let rows = sqlx::query("SELECT * FROM bookmarks WHERE user_id = $1 AND book_id = $2 ORDER BY position_secs")
            .bind(user_id)
            .bind(book_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.iter().map(Self::row_to_bookmark).collect())
    }

    async fn list_all_user_bookmarks(&self, user_id: Uuid) -> Result<Vec<Bookmark>> {
        let rows = sqlx::query("SELECT * FROM bookmarks WHERE user_id = $1 ORDER BY created_at DESC")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.iter().map(Self::row_to_bookmark).collect())
    }

    async fn update_bookmark(&self, id: Uuid, update: &UpdateBookmark) -> Result<Bookmark> {
        let mut sets = Vec::new();
        let mut param_idx = 1;

        if update.title.is_some() { sets.push(format!("title = ${}", param_idx)); param_idx += 1; }
        if update.note.is_some() { sets.push(format!("note = ${}", param_idx)); param_idx += 1; }

        if sets.is_empty() {
            return Ok(self.get_bookmark(id).await?.unwrap());
        }

        let sql = format!("UPDATE bookmarks SET {} WHERE id = ${}", sets.join(", "), param_idx);
        let mut query = sqlx::query(&sql);

        if let Some(ref v) = update.title { query = query.bind(v); }
        if let Some(ref v) = update.note { query = query.bind(v); }
        query = query.bind(id);

        query.execute(&self.pool).await?;
        Ok(self.get_bookmark(id).await?.unwrap())
    }

    async fn delete_bookmark(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM bookmarks WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Collections
    async fn create_collection(&self, user_id: Uuid, collection: &CreateCollection) -> Result<Collection> {
        let id = Uuid::new_v4();
        let now = Self::now();
        sqlx::query(
            "INSERT INTO collections (id, user_id, name, description, cover_path, book_count, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 0, $6, $7)"
        )
        .bind(id)
        .bind(user_id)
        .bind(&collection.name)
        .bind(&collection.description)
        .bind(None::<String> as Option<String>)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query("SELECT * FROM collections WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(Self::row_to_collection(&row))
    }

    async fn get_collection(&self, id: Uuid) -> Result<Option<Collection>> {
        let row = sqlx::query("SELECT * FROM collections WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.as_ref().map(Self::row_to_collection))
    }

    async fn list_collections(&self, user_id: Uuid) -> Result<Vec<Collection>> {
        let rows = sqlx::query("SELECT * FROM collections WHERE user_id = $1 ORDER BY name")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.iter().map(Self::row_to_collection).collect())
    }

    async fn update_collection(&self, id: Uuid, update: &UpdateCollection) -> Result<Collection> {
        let now = Self::now();
        let mut sets = vec!["updated_at = $1".to_string()];
        let mut param_idx = 2;

        if update.name.is_some() { sets.push(format!("name = ${}", param_idx)); param_idx += 1; }
        if update.description.is_some() { sets.push(format!("description = ${}", param_idx)); param_idx += 1; }

        let sql = format!("UPDATE collections SET {} WHERE id = ${}", sets.join(", "), param_idx);
        let mut query = sqlx::query(&sql).bind(now);

        if let Some(ref v) = update.name { query = query.bind(v); }
        if let Some(ref v) = update.description { query = query.bind(v); }
        query = query.bind(id);

        query.execute(&self.pool).await?;
        Ok(self.get_collection(id).await?.unwrap())
    }

    async fn delete_collection(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM collections WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn add_book_to_collection(&self, collection_id: Uuid, book_id: Uuid) -> Result<()> {
        sqlx::query("INSERT INTO collection_books (collection_id, book_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(collection_id)
            .bind(book_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn remove_book_from_collection(&self, collection_id: Uuid, book_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM collection_books WHERE collection_id = $1 AND book_id = $2")
            .bind(collection_id)
            .bind(book_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_collection_books(&self, collection_id: Uuid) -> Result<Vec<Book>> {
        let rows = sqlx::query(
            "SELECT b.* FROM books b JOIN collection_books cb ON b.id = cb.book_id WHERE cb.collection_id = $1 ORDER BY cb.position"
        )
        .bind(collection_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(Self::row_to_book).collect())
    }
}
