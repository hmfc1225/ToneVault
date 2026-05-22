use std::sync::Arc;

use tokio::sync::mpsc;
use tracing::{error, info};

use tonevault_core::scanner::{Scanner, ScannerEvent};
use tonevault_db::Database;

pub struct ScanManager {
    db: Arc<Database>,
    scanner: Scanner,
}

impl ScanManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            scanner: Scanner::new(),
        }
    }

    pub async fn start_scan(&self, library_id: i64) -> anyhow::Result<()> {
        let library = self.db.get_library(library_id).await?
            .ok_or_else(|| anyhow::anyhow!("Library not found: {}", library_id))?;

        let (tx, mut rx) = mpsc::channel::<ScannerEvent>(100);

        let db = self.db.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    ScannerEvent::Started { library_id } => {
                        info!(library_id, "Scan started");
                    }
                    ScannerEvent::Progress { library_id, processed, total } => {
                        info!(library_id, processed, total, "Scan progress");
                    }
                    ScannerEvent::BookParsed { library_id, dir, metadata } => {
                        info!(library_id, dir = %dir, "Book parsed");
                    }
                    ScannerEvent::Completed { library_id, new_books, updated_books } => {
                        info!(library_id, new_books, updated_books, "Scan completed");
                        if let Err(e) = db.update_last_scan(library_id).await {
                            error!(library_id, error = %e, "Failed to update scan time");
                        }
                    }
                    ScannerEvent::Error { library_id, message } => {
                        error!(library_id, message = %message, "Scan error");
                    }
                }
            }
        });

        self.scanner.scan_library(&library, tx).await?;
        Ok(())
    }
}