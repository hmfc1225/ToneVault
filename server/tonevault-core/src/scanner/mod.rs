use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::metadata::MetadataParser;
use crate::models::library::Library;

pub struct Scanner;

#[derive(Debug, Clone)]
pub enum ScannerEvent {
    Started { library_id: Uuid },
    Progress { library_id: Uuid, processed: i64, total: i64 },
    BookParsed { library_id: Uuid, dir: String, metadata: crate::metadata::ParsedMetadata },
    Completed { library_id: Uuid, new_books: i64, updated_books: i64 },
    Error { library_id: Uuid, message: String },
}

impl Scanner {
    pub fn new() -> Self {
        Self
    }

    pub async fn scan_library(
        &self,
        library: &Library,
        event_tx: mpsc::Sender<ScannerEvent>,
    ) -> Result<()> {
        let root = Path::new(&library.root_path);
        if !root.exists() {
            anyhow::bail!("Library root path does not exist: {}", library.root_path);
        }

        info!("Starting scan for library: {} ({})", library.name, library.root_path);

        let _ = event_tx.send(ScannerEvent::Started { library_id: library.id }).await;

        let mut book_dirs = Vec::new();
        self.collect_book_dirs(root, &mut book_dirs)?;

        let total = book_dirs.len() as i64;
        let mut new_books = 0i64;

        for (i, dir) in book_dirs.iter().enumerate() {
            match MetadataParser::parse_directory(dir) {
                Ok(metadata) => {
                    new_books += 1;
                    let _ = event_tx.send(ScannerEvent::BookParsed {
                        library_id: library.id,
                        dir: dir.to_string_lossy().to_string(),
                        metadata,
                    }).await;
                }
                Err(e) => {
                    warn!("Failed to parse {}: {}", dir.display(), e);
                }
            }

            if i % 10 == 0 {
                let _ = event_tx.send(ScannerEvent::Progress {
                    library_id: library.id,
                    processed: i as i64 + 1,
                    total,
                }).await;
            }
        }

        let _ = event_tx.send(ScannerEvent::Completed {
            library_id: library.id,
            new_books,
            updated_books: 0,
        }).await;

        Ok(())
    }

    fn collect_book_dirs(&self, root: &Path, dirs: &mut Vec<PathBuf>) -> Result<()> {
        for entry in walkdir::WalkDir::new(root)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let has_audio = std::fs::read_dir(path)
                .ok()
                .map(|mut entries| {
                    entries.any(|e| {
                        e.ok()
                            .map(|e| MetadataParser::is_audio_file(&e.path()))
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);

            if has_audio {
                dirs.push(path.to_path_buf());
            }
        }
        Ok(())
    }
}