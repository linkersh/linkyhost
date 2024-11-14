-- Add up migration script here
ALTER TABLE sessions ADD COLUMN expires_at TIMESTAMP NOT NULL DEFAULT NOW() + INTERVAL '30 days';
