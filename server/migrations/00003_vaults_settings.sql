-- +goose Up
-- +goose StatementBegin
alter table vaults add column flags int not null default 0;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
alter table vaults drop column flags;
-- +goose StatementEnd
