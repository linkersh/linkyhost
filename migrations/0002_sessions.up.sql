-- Add up migration script here
CREATE TABLE sessions (

    user_id UUID NOT NULL REFERENCES users(id),
    session_id VARCHAR(256) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()

);