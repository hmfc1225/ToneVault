use anyhow::Result;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::Accessor;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ParsedMetadata {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub narrator: Option<String>,
    pub series: Option<String>,
    pub series_position: Option<f64>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub publish_year: Option<i32>,
    pub language: Option<String>,
    pub cover_data: Option<Vec<u8>>,
    pub tracks: Vec<ParsedTrack>,
    pub source: MetadataSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataSource {
    EmbeddedTags,
    FilenamePattern,
    DirectoryStructure,
    Manual,
}

#[derive(Debug, Clone)]
pub struct ParsedTrack {
    pub title: String,
    pub track_number: i32,
    pub disc_number: Option<i32>,
    pub file_path: String,
    pub duration_secs: f64,
    pub bitrate: Option<i32>,
    pub sample_rate: Option<i32>,
    pub channels: Option<i32>,
    pub codec: Option<String>,
    pub mime_type: String,
}

pub struct MetadataParser;

impl MetadataParser {
    pub fn parse_directory(dir: &Path) -> Result<ParsedMetadata> {
        let mut metadata = Self::from_directory_structure(dir);

        if let Ok(tag_meta) = Self::from_embedded_tags(dir) {
            metadata.merge(tag_meta);
        }

        if let Some(name_meta) = Self::from_filename_patterns(dir) {
            metadata.merge(name_meta);
        }

        Ok(metadata)
    }

    fn from_directory_structure(dir: &Path) -> ParsedMetadata {
        let parent = dir.parent();
        let grandparent = parent.and_then(|p| p.parent());

        let (author, title) = match (parent, grandparent) {
            (Some(p), Some(_gp)) if Self::looks_like_author_dir(p) && Self::looks_like_book_dir(dir) => {
                (Some(p.file_name().unwrap().to_string_lossy().to_string()),
                 Some(dir.file_name().unwrap().to_string_lossy().to_string()))
            }
            (Some(p), _) => {
                (None, Some(p.file_name().unwrap().to_string_lossy().to_string()))
            }
            _ => (None, Some(dir.file_name().unwrap().to_string_lossy().to_string())),
        };

        ParsedMetadata {
            title,
            subtitle: None,
            author,
            narrator: None,
            series: None,
            series_position: None,
            description: None,
            publisher: None,
            publish_year: None,
            language: None,
            cover_data: None,
            tracks: Vec::new(),
            source: MetadataSource::DirectoryStructure,
        }
    }

    fn from_embedded_tags(dir: &Path) -> Result<ParsedMetadata> {
        let mut tracks = Vec::new();
        let mut book_title = None;
        let mut author = None;

        for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !Self::is_audio_file(path) {
                continue;
            }

            if let Ok(tagged_file) = lofty::read_from_path(path) {
                let tags = tagged_file.primary_tag();
                if let Some(tag) = tags {
                    if book_title.is_none() {
                        book_title = tag.title().map(|t| t.to_string());
                    }
                    if author.is_none() {
                        author = tag.artist().map(|t| t.to_string());
                    }

                    let track_num = tag.track()
                        .map(|n| n as i32)
                        .unwrap_or(tracks.len() as i32 + 1);

                    let properties = tagged_file.properties();
                    tracks.push(ParsedTrack {
                        title: tag.title()
                            .map(|t| t.to_string())
                            .unwrap_or_else(|| format!("Track {}", track_num)),
                        track_number: track_num,
                        disc_number: tag.disk().map(|n| n as i32),
                        file_path: path.to_string_lossy().to_string(),
                        duration_secs: properties.duration().as_secs_f64(),
                        bitrate: properties.audio_bitrate().map(|b| b as i32),
                        sample_rate: properties.sample_rate().map(|r| r as i32),
                        channels: properties.channels().map(|c| c as i32),
                        codec: None,
                        mime_type: Self::mime_from_path(path),
                    });
                } else {
                    let properties = tagged_file.properties();
                    let track_num = tracks.len() as i32 + 1;
                    tracks.push(ParsedTrack {
                        title: format!("Track {}", track_num),
                        track_number: track_num,
                        disc_number: None,
                        file_path: path.to_string_lossy().to_string(),
                        duration_secs: properties.duration().as_secs_f64(),
                        bitrate: properties.audio_bitrate().map(|b| b as i32),
                        sample_rate: properties.sample_rate().map(|r| r as i32),
                        channels: properties.channels().map(|c| c as i32),
                        codec: None,
                        mime_type: Self::mime_from_path(path),
                    });
                }
            }
        }

        tracks.sort_by_key(|t| (t.disc_number.unwrap_or(1), t.track_number));

        Ok(ParsedMetadata {
            title: book_title,
            subtitle: None,
            author,
            narrator: None,
            series: None,
            series_position: None,
            description: None,
            publisher: None,
            publish_year: None,
            language: None,
            cover_data: None,
            tracks,
            source: MetadataSource::EmbeddedTags,
        })
    }

    fn from_filename_patterns(_dir: &Path) -> Option<ParsedMetadata> {
        // TODO: implement filename pattern parsing
        None
    }

    pub fn find_cover(dir: &Path) -> Option<std::path::PathBuf> {
        let cover_names = ["cover.jpg", "cover.png", "folder.jpg", "folder.png"];
        for name in &cover_names {
            let path = dir.join(name);
            if path.exists() {
                return Some(path);
            }
        }
        for entry in std::fs::read_dir(dir).ok()? {
            let entry = entry.ok()?;
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.ends_with(".jpg") || name.ends_with(".png") {
                return Some(entry.path());
            }
        }
        None
    }

    fn looks_like_author_dir(path: &Path) -> bool {
        path.is_dir() && path.parent().is_some()
    }

    fn looks_like_book_dir(path: &Path) -> bool {
        path.is_dir()
    }

    pub fn is_audio_file(path: &Path) -> bool {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        matches!(
            ext.to_lowercase().as_str(),
            "mp3" | "m4a" | "m4b" | "flac" | "ogg" | "wav" | "aac"
        )
    }

    fn mime_from_path(path: &Path) -> String {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext.to_lowercase().as_str() {
            "mp3" => "audio/mpeg".into(),
            "m4a" | "m4b" => "audio/mp4".into(),
            "flac" => "audio/flac".into(),
            "ogg" => "audio/ogg".into(),
            "wav" => "audio/wav".into(),
            "aac" => "audio/aac".into(),
            _ => "application/octet-stream".into(),
        }
    }
}

impl ParsedMetadata {
    fn merge(&mut self, other: ParsedMetadata) {
        if self.title.is_none() && other.title.is_some() {
            self.title = other.title;
        }
        if self.author.is_none() && other.author.is_some() {
            self.author = other.author;
        }
        if self.narrator.is_none() && other.narrator.is_some() {
            self.narrator = other.narrator;
        }
        if self.series.is_none() && other.series.is_some() {
            self.series = other.series;
        }
        if self.description.is_none() && other.description.is_some() {
            self.description = other.description;
        }
        if self.publisher.is_none() && other.publisher.is_some() {
            self.publisher = other.publisher;
        }
        if self.publish_year.is_none() && other.publish_year.is_some() {
            self.publish_year = other.publish_year;
        }
        if self.language.is_none() && other.language.is_some() {
            self.language = other.language;
        }
        if self.cover_data.is_none() && other.cover_data.is_some() {
            self.cover_data = other.cover_data;
        }
        if !other.tracks.is_empty() && self.tracks.is_empty() {
            self.tracks = other.tracks;
        }
    }
}
