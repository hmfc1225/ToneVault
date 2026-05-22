export interface User {
  id: string
  username: string
  display_name?: string
  email?: string
  role: string
}

export interface AuthResponse {
  access_token: string
  refresh_token: string
  user: User
}

export interface UpsertPlaybackPosition {
  book_id: string
  track_id: string
  position_secs: number
  percentage: number
  is_finished?: boolean
}

export interface SeriesWithPosition {
  series: Series
  position: number
}

export interface CreateBookmark {
  book_id: string
  track_id: string
  title: string
  position_secs: number
  note?: string
}

export interface UpdateLibrary {
  name?: string
  root_path?: string
  description?: string
  scan_enabled?: boolean
  watch_enabled?: boolean
}

export type SourceType = 'local' | 'webdav' | 'rss'

export interface WebDavConnectRequest {
  url: string
  username: string
  password: string
}

export interface WebDavEntry {
  name: string
  path: string
  is_dir: boolean
  size?: number
}

export interface Library {
  id: string
  name: string
  root_path: string
  description?: string
  source_type: SourceType
  base_url?: string
  scan_enabled: boolean
  watch_enabled: boolean
  scan_status: string
  book_count: number
  last_scan?: string
  created_at: string
  updated_at: string
}

export interface CreateLibrary {
  name: string
  root_path: string
  description?: string
  source_type?: SourceType
  base_url?: string
  scan_enabled?: boolean
  watch_enabled?: boolean
}

export interface ScanProgress {
  library_id: string
  status: string
  total_files: number
  processed_files: number
  current_file?: string
  errors: string[]
  started_at?: string
  completed_at?: string
}

export interface Book {
  id: string
  title: string
  description?: string
  cover_path?: string
  duration_secs: number
  track_count: number
  file_size: number
  isbn?: string
  asin?: string
  source_path: string
  metadata_source?: string
  library_id: string
  authors?: Author[]
  series?: Series[]
  created_at: string
  updated_at: string
}

export interface Track {
  id: string
  book_id: string
  title: string
  track_number: number
  disc_number: number
  duration_secs: number
  file_path: string
  file_size: number
  mime_type: string
}

export interface Author {
  id: string
  name: string
  description?: string
}

export interface AuthorWithRole {
  author: Author
  role: string
}

export interface Series {
  id: string
  name: string
  description?: string
}

export interface PlaybackPosition {
  book_id: string
  track_id: string
  position_secs: number
  percentage: number
  is_finished: boolean
}

export interface Bookmark {
  id: string
  book_id: string
  track_id: string
  title: string
  position_secs: number
  note?: string
  created_at: string
}

export interface Collection {
  id: string
  user_id: string
  name: string
  description?: string
  book_count: number
  created_at: string
}

export interface CreateCollection {
  name: string
  description?: string
}

export interface PaginatedResult<T> {
  items: T[]
  total: number
  page: number
  per_page: number
  total_pages: number
}

export interface BookFilter {
  library_id?: string
  author_id?: string
  series_id?: string
  query?: string
  sort?: 'title' | 'added' | 'duration' | 'year' | 'author'
  order?: 'asc' | 'desc'
  page?: number
  per_page?: number
}
