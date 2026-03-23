-- migrations/0003_create_chapters.sql
CREATE TABLE chapters
(
    id            UUID PRIMARY KEY      DEFAULT gen_random_uuid(),
    story_id      UUID         NOT NULL REFERENCES stories (id) ON DELETE CASCADE,
    title         VARCHAR(500) NOT NULL,
    content       TEXT         NOT NULL,
    chapter_order INT          NOT NULL,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    UNIQUE (story_id, chapter_order)
);
CREATE INDEX idx_chapters_story ON chapters (story_id);