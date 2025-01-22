-- +goose Up
-- +goose StatementBegin
delete from vault_files;
alter table vault_files add column content_type varchar not null;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
alter table vault_files drop column content_type;
-- +goose StatementEnd
