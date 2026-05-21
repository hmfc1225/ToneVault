use crate::{Database, Result};
use tonevault_core::models::library::{CreateLibrary, Library, SourceType, UpdateLibrary};

impl Database {
    pub async fn list_libraries(&self) -> Result<Vec<Library>> {
        let rows = sqlx::query_as::<_, LibraryRow>(
            "SELECT id, name, root_path, source_type, base_url, username, password, scan_interval, last_scan, created_at, updated_at FROM libraries ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_library(&self, id: i64) -> Result<Option<Library>> {
        let row = sqlx::query_as::<_, LibraryRow>(
            "SELECT id, name, root_path, source_type, base_url, username, password, scan_interval, last_scan, created_at, updated_at FROM libraries WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn create_library(&self, input: &CreateLibrary) -> Result<Library> {
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
        .fetch_one(&self.pool)
        .await?;
        Ok(result.into())
    }

    pub async fn update_library(&self, id: i64, input: &UpdateLibrary) -> Result<Library> {
        let current = self
            .get_library(id)
            .await?
            .ok_or(crate::Error::NotFound)?;

        let name = input.name.as_deref().unwrap_or(&current.name);
        let root_path = input.root_path.as_deref().unwrap_or(&current.root_path);
        let source_type = input.source_type.unwrap_or(current.source_type);
        let base_url = input
            .base_url
            .as_deref()
            .or(current.base_url.as_deref());
        let username = input
            .username
            .as_deref()
            .or(current.username.as_deref());
        let password = input
            .password
            .as_deref()
            .or(current.password.as_deref());
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
        .fetch_one(&self.pool)
        .await?;
        Ok(result.into())
    }

    pub async fn delete_library(&self, id: i64) -> Result<()> {
        let result = sqlx::query("DELETE FROM libraries WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(crate::Error::NotFound);
        }
        Ok(())
    }

    pub async fn update_last_scan(&self, id: i64) -> Result<()> {
        sqlx::query("UPDATE libraries SET last_scan = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

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
    last_scan: Option<time::OffsetDateTime>,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
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
