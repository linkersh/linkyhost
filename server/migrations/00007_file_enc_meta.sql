-- +goose Up
-- +goose StatementBegin
alter table vault_files add column fixed_iv         bytea   not null default '{}';
alter table vault_files add column chunk_size       int     not null default 0;
alter table vault_files add column is_encrypted     boolean not null default false;
alter table vault_files add column is_hidden        boolean not null default false;
alter table vault_files add column password_salt    bytea   not null default '{}';

alter table vaults      add column is_encrypted     boolean not null default false;
alter table vaults      drop column flags;
alter table vaults      drop column kind;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
alter table vault_files drop column fixed_iv; 
alter table vault_files drop column chunk_size;
alter table vault_files drop column is_encrypted;
alter table vault_files drop column is_hidden;

alter table vaults      drop column is_encrypted;
alter table vaults      drop column salt;
alter table vaults      add column flags int default 0;
alter table vaults      add column kind int default 0;
-- +goose StatementEnd
