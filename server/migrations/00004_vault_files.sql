-- +goose Up
-- +goose StatementBegin
CREATE TABLE
    vault_files (
        id serial PRIMARY KEY not null,
        file_name TEXT NOT NULL,
        size INTEGER NOT NULL,
        uploaded_at TIMESTAMP NOT NULL default NOW (),
        created_at TIMESTAMP NOT NULL,
        vault_id INTEGER NOT NULL,
        user_id INTEGER NOT NULL,
        FOREIGN KEY (vault_id) REFERENCES vaults (id),
        FOREIGN KEY (user_id) REFERENCES users (id)
    );

-- +goose StatementEnd
-- +goose Down
-- +goose StatementBegin
DROP TABLE vault_files;

-- +goose StatementEnd