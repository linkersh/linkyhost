-- +goose Up
-- +goose StatementBegin
alter table vault_files alter column size type bigint;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
alter table vault_files alter column size type int;
-- +goose StatementEnd
