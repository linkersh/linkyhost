-- +goose Up
-- +goose StatementBegin
create table users (
    id serial primary key,
    username varchar not null,
    password varchar not null,
    created_at timestamp not null default now()
);
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
drop table users
-- +goose StatementEnd
