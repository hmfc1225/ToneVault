-- Add source type and connection fields to libraries table
ALTER TABLE libraries ADD COLUMN source_type TEXT NOT NULL DEFAULT 'local';
ALTER TABLE libraries ADD COLUMN base_url TEXT;
ALTER TABLE libraries ADD COLUMN username TEXT;
ALTER TABLE libraries ADD COLUMN password TEXT;
