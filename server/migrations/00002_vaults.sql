-- +goose Up
-- +goose StatementBegin
create table vaults (
    id bigserial primary key,
    name varchar not null,
    user_id int not null references users(id),
    kind int not null,
    created_at timestamp not null default now()
);
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
drop table vaults;
-- +goose StatementEnd
