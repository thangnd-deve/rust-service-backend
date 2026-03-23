-- migrations/0004_create_audio_files.sql
CREATE TABLE audio_files
(
    id               UUID PRIMARY KEY       DEFAULT gen_random_uuid(),
    chapter_id       UUID UNIQUE   NOT NULL REFERENCES chapters (id) ON DELETE CASCADE,
    file_path        VARCHAR(1000) NOT NULL,
    file_size        BIGINT,
    duration_seconds INT,
    status           VARCHAR(20)   NOT NULL DEFAULT 'pending', -- pending, processing, ready, failed
    error_message    TEXT,
    created_at       TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ   NOT NULL DEFAULT NOW()
);