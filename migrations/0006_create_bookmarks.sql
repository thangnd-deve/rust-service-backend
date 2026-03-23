-- migrations/0005_create_bookmarks.sql
CREATE TABLE bookmarks
(
    id         UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    user_id    UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    chapter_id UUID        NOT NULL REFERENCES chapters (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, chapter_id)
);
CREATE INDEX idx_bookmarks_user ON bookmarks (user_id);