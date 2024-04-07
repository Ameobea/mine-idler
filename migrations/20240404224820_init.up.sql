create table if not exists users (
  id serial primary key,
  username text not null unique,
  hashed_password text not null,
  last_login timestamp not null default now()
);

create table if not exists sessions (
  id serial primary key,
  user_id integer not null references users(id),
  token text not null unique,
  created_at timestamp not null default now()
);
create index if not exists token_index on sessions(token);

create table if not exists items (
  id serial primary key,
  name text not null,
  description text not null,
  rarity_tier smallint not null
);

create table if not exists inventory (
  id uuid primary key default gen_random_uuid(),
  user_id integer not null references users(id),
  item_id integer not null references items(id),
  quality float4 not null,
  value float4 not null,
  modifiers jsonb,
  created_at timestamp not null default now()
);
