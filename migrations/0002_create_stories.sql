-- migrations/0002_create_stories.sql
CREATE TABLE stories
(
    id              UUID PRIMARY KEY      DEFAULT gen_random_uuid(),
    title           VARCHAR(500) NOT NULL,
    description     TEXT,
    cover_image_url VARCHAR(1000),
    author_id       UUID         NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    status          VARCHAR(20)  NOT NULL DEFAULT 'draft', -- draft, published, archived
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_stories_author ON stories (author_id);
CREATE INDEX idx_stories_status ON stories (status);
