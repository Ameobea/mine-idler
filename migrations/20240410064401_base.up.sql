create table bases (
  user_id integer not null references users(id),
  storage_level integer not null default 0
);
